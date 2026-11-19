// V8 in Node: the astrogation console's pure parts against the real kernel's
// plan_route. Kestrel's chart becomes the system NAV reads (planets only, the
// star's pull and light), Tarn to Vault plots by torch and by coasting ship,
// every course starts at Tarn and ends at Vault on the kernel's own track,
// the shortlist keeps one course of each kind, and the chart frame holds it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { navFromScene, shortlist, safestFirst, routeFrame, trackPoint, fuelLeft, difficultyPips, daysWords, fromNowWords, trackIndexAt } from "../src/route.ts";
import type { SystemScene } from "../src/scene.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
const scene: SystemScene = JSON.parse(readFileSync(new URL("packages/shell/src/documents/kestrel.scene.json", root), "utf8"));
const universe = { id: "kestrel", seed: "kestrel reach campaign" };
const AU = 149_597_870_700;

test("Kestrel's chart as NAV reads it: planets around the star, moons left out and said so", () => {
  // The derived luminosity and radius ride on the star once the base-physics flows have run; stand them in here.
  const lit = { ...scene, subjects: scene.subjects.map((s) => (s.kind === "star" ? { ...s, facts: [...s.facts, { field: "luminosity_sun", label: "L", value: 0.42, display: "0.42", editable: false, source: "derived" as const }] } : s)) } as SystemScene;
  const r = navFromScene(lit);
  assert.ok(r.nav);
  assert.deepEqual(r.bodies.map((b) => b.label), ["Ember", "Sable", "Tarn", "Vault", "Hollow"]);
  assert.equal(r.nav.star_mass_msun, 0.82);
  assert.equal(r.nav.star_luminosity_lsun, 0.42);
  assert.deepEqual(r.nav.belts, [{ name: "Grit belt", a_in_au: 1.4, a_out_au: 1.9 }]);
  assert.ok(r.notes.some((n) => n.includes("4 moons")));
  const bare = navFromScene(scene);
  assert.ok(bare.notes.some((n) => n.includes("luminosity")), "a missing luminosity is said, not hidden");
  assert.equal(navFromScene({ ...scene, subjects: scene.subjects.filter((s) => s.kind !== "star") }).nav, null);
});

test("Tarn to Vault through plan_route: courses start and end on the kernel's track, laid out in the star's frame", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), universe, [base], k.scratch);
    const { nav } = navFromScene(scene);
    assert.ok(nav);
    const tarn = nav.bodies.find((b) => b.name === "Tarn")!;
    const vault = nav.bodies.find((b) => b.name === "Vault")!;
    const sable = nav.bodies.find((b) => b.name === "Sable")!;
    // Even to Sable next door a chemical ship falls just short: refused, with what it would need.
    const weak = await s.planRoute({ nav, ship: "chemical-transfer", from: tarn.address, to: sable.address, window_days: 365 });
    assert.ok(weak.ok);
    assert.equal(weak.plan.candidates.length, 0);
    assert.ok(weak.plan.refused.length > 0 && weak.plan.refused.every((x) => /needs .* km\/s/.test(x.why)), JSON.stringify(weak.plan.refused));
    for (const [ship, dest] of [["fusion-torch", vault], ["nuclear-thermal", sable]] as const) {
      const r = await s.planRoute({ nav, ship, from: tarn.address, to: dest.address, prefer: "balanced", window_days: 365 });
      assert.ok(r.ok, `${ship}: ${JSON.stringify(r)}`);
      const p = r.plan;
      assert.equal(p.from, tarn.address);
      assert.equal(p.to, dest.address);
      assert.ok(p.candidates.length >= 1, `${ship}: some course`);
      for (const c of p.candidates) {
        assert.ok(c.days > 0 && c.arrive_s > c.depart_s);
        assert.ok(c.track.length >= 2);
        const r0 = Math.hypot(c.track[0]![0], c.track[0]![1]) / AU;
        const r1 = Math.hypot(c.track.at(-1)![0], c.track.at(-1)![1]) / AU;
        assert.ok(Math.abs(r0 - tarn.a_au) < 0.1, `${c.id} starts at Tarn's distance (${r0})`);
        assert.ok(Math.abs(r1 - dest.a_au) < 0.3, `${c.id} ends at ${dest.name}'s distance (${r1})`);
        const fl = fuelLeft(c, p.ship);
        assert.ok(fl.left >= 0 && fl.left <= 1);
        assert.ok(difficultyPips(c.difficulty.label) >= 0);
      }
      const short = shortlist(p.candidates, 3);
      assert.ok(short.length <= 3 && short.length >= 1);
      assert.equal(new Set(short.slice(0, Math.min(short.length, new Set(p.candidates.map((c) => c.kind)).size)).map((c) => c.kind)).size, Math.min(short.length, new Set(p.candidates.map((c) => c.kind)).size), "one of each kind first");
      const safe = safestFirst(p.candidates);
      for (let i = 1; i < safe.length; i++) assert.ok(safe[i - 1]!.difficulty.score <= safe[i]!.difficulty.score);
      const f = routeFrame(640, 420, nav, p, p.candidates[0]!);
      for (const pt of p.candidates[0]!.track) {
        const q = trackPoint(pt, f);
        assert.ok(q.x >= 0 && q.x <= 640 && q.y >= 0 && q.y <= 420, "the whole course is on the chart");
      }
    }
    {
      const a = await s.planRoute({ nav, ship: "fusion-torch", from: "Tarn", to: "Vault" });
      const b = await s.planRoute({ nav, ship: "fusion-torch", from: "Tarn", to: "Vault" });
      assert.deepEqual(a, b, "the same question gives the same plan");
    }
    // A player's seat may plan from what it knows (an inline system), never from the GM's record.
    const p = await Session.open(k.client, tableSeat(), universe, [base], k.scratch);
    assert.equal((await p.planRoute({ system: "u:kestrel/sys:1", ship: "fusion-torch", from: "Tarn", to: "Vault" })).ok, false);
  } finally {
    await k.close();
  }
});

test("words", () => {
  assert.equal(daysWords(0.5), "12 h");
  assert.equal(daysWords(9.2), "9.2 days");
  assert.equal(daysWords(800), "2.2 years");
  assert.equal(fromNowWords(86_400 * 3, 0), "in 3.0 days");
});

test("burns are timed from departure and land on the track in order", () => {
  const c = { depart_s: 1000, arrive_s: 1000 + 100 * 86400, track: Array.from({ length: 64 }, (_, i) => [i, 0, 0] as [number, number, number]) } as never;
  assert.equal(trackIndexAt(c, 0), 0);
  assert.equal(trackIndexAt(c, 100 * 86400), 63);
  assert.equal(trackIndexAt(c, 50 * 86400), 32);
});
