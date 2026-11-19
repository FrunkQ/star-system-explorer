// V13 part 2 in Node: adopting writes a preview into the open universe as
// history (the state hash moves; adopting the same system twice is refused);
// the chart then reads the system back from the universe (resolved_system)
// and draws the kernel's facts. A look-ahead from the resolved spec, kept
// with write-back, changes the star the chart reads back.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { sceneFromResolved, sceneFromFormation } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];

test("a grown system adopts as one event and reads back as the same worlds", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const h0 = await s.stateHash();
    const req = { seed: "kestrel-40", system: "u:kestrel/sys:40" };
    const ran = await s.runFormation(req);
    assert.ok(ran.ok);
    const preview = sceneFromFormation(ran.result, req.seed);
    const a = await s.adopt({ formation: req });
    assert.ok(a.ok, JSON.stringify(a));
    assert.equal(a.result.events.length, 1);
    assert.deepEqual(a.result.systems, ["u:kestrel/sys:40"]);
    assert.notEqual(await s.stateHash(), h0, "adopting writes history");
    assert.equal(a.result.state_hash, await s.stateHash());
    const again = await s.adopt({ formation: req });
    assert.ok(!again.ok && again.code === "already_adopted");

    const r = await s.resolvedSystem("u:kestrel/sys:40");
    assert.ok(r.ok, JSON.stringify(r));
    assert.equal(r.result.source, "formation");
    const scene = sceneFromResolved(r.result, `Seed "${req.seed}"`);
    const worlds = (x: typeof scene) => x.subjects.filter((b) => b.kind === "planet");
    assert.equal(worlds(scene).length, worlds(preview).length, "the same worlds as the preview");
    const orbits = (x: typeof scene) => worlds(x).map((w) => w.orbit!.a_au.toPrecision(9)).sort();
    assert.deepEqual(orbits(scene), orbits(preview));
    assert.ok(Math.abs(scene.now_yr - preview.now_yr) / preview.now_yr < 1e-3);
    assert.ok(scene.subjects.every((b) => b.facts.every((x) => !x.editable)));
  } finally {
    k.close();
  }
});

test("a V3 save adopts by evidence class; Sol reads back with its names", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const a = await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") });
    assert.ok(a.ok, JSON.stringify(a));
    assert.ok(a.result.events.length >= 1 && a.result.events.length <= 3);
    const r = await s.resolvedSystem(a.result.systems[0]!);
    assert.ok(r.ok);
    const scene = sceneFromResolved(r.result);
    assert.equal(scene.title, "Sol");
    assert.deepEqual(scene.subjects.map((x) => x.label), ["Sun", "Earth", "Moon", "Station"]);
    assert.equal(scene.subjects.find((x) => x.label === "Earth")!.orbit!.a_au, 1);
    // Players never adopt.
    const p = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs).catch(() => null);
    if (p) assert.equal((await p.adopt({ import_v3: "{}" })).ok, false);
  } finally {
    k.close();
  }
});

test("look ahead from the resolved spec, keep with write-back: the star read back has aged", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    assert.ok((await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") })).ok);
    const sys = (await s.resolvedSystem("u:kestrel/sys:@1"));
    assert.ok(sys.ok && sys.result.spec, JSON.stringify(sys).slice(0, 300));
    const before = sceneFromResolved(sys.result);
    const lum = (x: typeof before) => Number(x.subjects.find((b) => b.kind === "star")!.facts.find((f) => f.field === "luminosity_sun")?.value ?? 1);
    const la = await s.lookAhead(sys.result.spec!, 9000);
    assert.ok(la.ok, JSON.stringify(la));
    const snap = la.result.snapshots.find((x) => x.age_myr >= 8000) ?? la.result.snapshots.at(-1)!;
    const kept = await s.commitLookAhead(la.result.branch, snap.age_myr, true);
    assert.ok(kept.ok && kept.aged_event, JSON.stringify(kept));
    const after = await s.resolvedSystem("u:kestrel/sys:@1");
    assert.ok(after.ok);
    const aged = sceneFromResolved(after.result);
    assert.ok(lum(aged) > lum(before) * 1.2, `${lum(before)} -> ${lum(aged)}`);
    // The aged star's values are the ageing model's, not the save's.
    const sun = aged.subjects.find((b) => b.kind === "star")!;
    assert.ok(sun.facts.filter((x) => x.field !== "name").every((x) => x.source === "derived"));
    assert.ok(!sun.stamps.includes("authored"));
    assert.equal(aged.title, "Sol");
  } finally {
    k.close();
  }
});
