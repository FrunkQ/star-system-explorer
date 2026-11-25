// The system view hands a ringed world's mass and its moons to True, so the rings get gaps and edges at the moons'
// resonances (renderer ring-structure.ts, track SB VP-R3).
import { test } from "node:test";
import assert from "node:assert/strict";
import { fromSystemScene, moonsOf } from "../../packages/views/system-3d/src/bodies.ts";
import { constant, type SceneSubject, type SystemScene } from "../../packages/ui-contract/src/index.ts";
import { trueLook } from "../../packages/renderer/src/index.ts";

const fact = (field: string, value: number) => ({ field, label: field, value, display: String(value), editable: false, source: "catalogue" }) as unknown as SceneSubject["facts"][number];
const subject = (address: string, kind: string, extra: Partial<SceneSubject>): SceneSubject => ({ address, kind, label: address.split("/").pop()!, facts: [], ...extra }) as SceneSubject;
const AU = constant("au");

const scene = {
  subjects: [
    subject("u/sun", "star", { facts: [fact("mass_sun", 1)] }),
    subject("u/saturn", "planet", { orbit: { a_au: 9.58, phase_deg: 0 }, facts: [fact("mass_earth", 95.16), fact("radius_earth", 9.14)] }),
    subject("u/saturn/rings", "ring", { parent: "u/saturn", ring: { inner_m: 1.2217e8, outer_m: 1.36775e8, tau: 0.5, albedo: 0.5 } }),
    subject("u/pan", "moon", { parent: "u/saturn", orbit: { a_au: 1.33584e8 / AU, phase_deg: 0 }, facts: [fact("mass_earth", 4.95e15 / constant("M_earth"))] }),
    subject("u/janus", "moon", { parent: "u/saturn", orbit: { a_au: 1.5146e8 / AU, phase_deg: 0 }, facts: [] }),
  ],
} as unknown as SystemScene;

test("a world's mass comes from its facts and its moons with masses are listed for its rings", () => {
  const bodies = fromSystemScene(scene, () => null as never);
  const saturn = bodies.find((b) => b.id === "u/saturn")!;
  assert.ok(Math.abs(saturn.mass_kg! / 5.683e26 - 1) < 0.01, `${saturn.mass_kg}`);
  const moons = moonsOf(bodies, "u/saturn");
  // Janus has no mass on record, so only Pan is listed.
  assert.deepEqual(moons.map((m) => m.name), ["pan"]);
  assert.ok(Math.abs(moons[0]!.a_m / 1.33584e8 - 1) < 1e-6);
  // True then clears the Encke gap where Pan is.
  const look = trueLook({ key: "saturn", star: false, colour: "#ccc", radius_m: saturn.radius_m!, rings: saturn.rings!, pole: [0, 0, 1], mass_kg: saturn.mass_kg!, moons });
  const r = look.rings!;
  const u = (1.33584e8 / saturn.radius_m! - r.inner) / (r.outer - r.inner);
  assert.equal(r.data[Math.floor(u * r.width) * 4 + 3], 0);
});
