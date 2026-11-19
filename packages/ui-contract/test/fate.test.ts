// U4 part 2 in Node: the chart's scene becomes the ageing model's system;
// looking ahead runs on a branch, so the main line's state hash holds until
// the branch is kept; each snapshot draws as a scene (engulfed planets off
// the chart, their moons with them; the star's new state on its card); ages
// past the horizon are marked one plausible future; keeping with "move now"
// changes the main line.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { deriveScene } from "../src/derivation.ts";
import type { SystemScene } from "../src/scene.ts";
import { fateEvents, sceneAt, specFromScene, Z_SUN } from "../src/fate.ts";
import { nodeKernel } from "./node-kernel.ts";

const json = (p: string) => JSON.parse(readFileSync(new URL(`../../../${p}`, import.meta.url), "utf8"));
const kestrel: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");

test("the chart's system as the ageing model reads it", () => {
  const r = specFromScene(kestrel);
  assert.ok("spec" in r);
  const { spec, assumed } = r;
  assert.equal(spec.star_mass, 0.82);
  assert.ok(Math.abs(spec.z - Z_SUN * 10 ** -0.08) < 1e-12);
  assert.equal(spec.age_myr, 4610);
  assert.deepEqual(spec.planets.map((p) => p.name), ["Ember", "Sable", "Tarn", "Vault", "Hollow"], "planets only; moons ride with them");
  assert.equal(spec.planets.find((p) => p.name === "Tarn")!.oceans, true);
  assert.ok(assumed.some((a) => /coplanar/.test(a)));
  assert.deepEqual(specFromScene({ ...kestrel, subjects: [] }), { error: "no star in this system" });
});

test("Kestrel aged to 40 Gyr on a branch: events, snapshots, plausible futures, keep", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), kestrel.universe, [json("packs/base-physics/pack.json"), json("packs/base-time/pack.json")]);
    const scene = await deriveScene(s, kestrel, json("packages/shell/src/documents/derivations.base.json"));
    const h0 = await s.stateHash();
    const prepared = specFromScene(scene);
    assert.ok("spec" in prepared);
    const r = await s.lookAhead(prepared.spec, 40_000);
    assert.ok(r.ok, JSON.stringify(r));
    const a = r.result;
    assert.equal(await s.stateHash(), h0, "the main line does not move while looking");
    const kinds = a.events.map((e) => e.kind);
    for (const k of ["leaves_main_sequence", "red_giant", "engulfed", "planetary_nebula"]) assert.ok(kinds.includes(k), k);
    // The last snapshot: a white dwarf; the inner planets gone, the outer ones still there.
    const last = a.snapshots.at(-1)!;
    assert.equal(last.star.phase, "white dwarf");
    const end = sceneAt(scene, a, last);
    const names = end.subjects.map((x) => x.label);
    assert.ok(!names.includes("Tarn") && !names.includes("Tarn a"), "an engulfed planet and its moon leave the chart");
    assert.ok(names.includes("Vault") && names.includes("Hasp"), "outer planets and their moons stay");
    assert.ok(end.subjects[0]!.stamps.includes("plausible-future"));
    assert.match(end.subjects[0]!.subtitle!, /white dwarf/);
    assert.ok(end.subjects.every((x) => x.facts.every((f) => !f.editable)), "a look-ahead scene cannot be edited");
    assert.ok(end.span_yr[1] >= 40e9);
    const marks = fateEvents(a, scene);
    assert.ok(marks.length < a.events.length, "cooling steps and simultaneous engulfings share marks on the ruler");
    assert.ok(marks.some((m) => /^Engulfed: Ember, Sable, Tarn$/.test(m.label)), marks.map((m) => m.label).join("; "));
    // The first snapshot is within the horizon: no plausible-future stamp.
    assert.ok(!sceneAt(scene, a, a.snapshots[0]!).subjects[0]!.stamps.includes("plausible-future"));
    // Keep it and move now: the main line changes.
    const kept = await s.commitLookAhead(a.branch, 9_000);
    assert.ok(kept.ok, JSON.stringify(kept));
    assert.notEqual(await s.stateHash(), h0);
    const again = await s.commitLookAhead(a.branch);
    assert.ok(!again.ok && again.code === "unknown_branch");
  } finally {
    await k.close();
  }
});
