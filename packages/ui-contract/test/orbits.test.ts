// U11 part 2: building the kernel's orbits_at request from a scene, and laying its phases back on.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { AU_M, M_SUN, meanFromTrue, orbitBodies, phasesFrom, withPhases } from "../src/orbits.ts";
import type { SystemScene } from "../src/scene.ts";

const kestrel = JSON.parse(readFileSync(new URL("../../shell/src/documents/kestrel.scene.json", import.meta.url), "utf8")) as SystemScene;

test("true to mean anomaly: circular orbits are the same angle; periapsis and apoapsis hold", () => {
  assert.ok(Math.abs(meanFromTrue(90, 0) - Math.PI / 2) < 1e-12);
  assert.ok(Math.abs(meanFromTrue(0, 0.5)) < 1e-12);
  assert.ok(Math.abs(meanFromTrue(180, 0.5) - Math.PI) < 1e-12);
  assert.ok(meanFromTrue(90, 0.3) < Math.PI / 2, "fast near periapsis: less mean anomaly than true");
});

test("a fixture sends its own elements, its drawn phases as places at the epoch, and its star's mass", () => {
  const bodies = orbitBodies(kestrel, 1000n)!;
  const star = bodies.find((b) => b.address === "u:kestrel/sys:1/body:0")!;
  assert.ok(Math.abs(Number(star.facts["body.mass"]) - 0.82 * M_SUN) < 1e20);
  const ember = bodies.find((b) => b.address === "u:kestrel/sys:1/body:1")!;
  assert.equal(ember.facts["orbit.host"], "u:kestrel/sys:1/body:0");
  assert.ok(Math.abs(Number(ember.facts["orbit.semi_major_axis"]) - 0.21 * AU_M) < 1);
  assert.equal(ember.facts["orbit.epoch_s"], 1000n);
  assert.equal(orbitBodies({ ...kestrel, fixture: false }, 0n), undefined, "a system the kernel holds sends nothing");
});

test("phases go onto the scene; unplaced bodies keep theirs", () => {
  const p = phasesFrom([
    { address: "u:kestrel/sys:1/body:1", placed: true, at: [{ at_s: 0, r_m: [0, 0, 0], phase_deg: 10 }] },
    { address: "u:kestrel/sys:1/body:2", placed: false, missing: ["body.mass"] },
  ]);
  const s = withPhases(kestrel, p);
  assert.equal(s.subjects.find((x) => x.address.endsWith("body:1"))!.orbit!.phase_deg, 10);
  assert.equal(s.subjects.find((x) => x.address.endsWith("body:2"))!.orbit!.phase_deg, 212);
  assert.equal(withPhases(kestrel, null), kestrel);
});
