// The system view asks the kernel for a ringed world's ring whole (VP-R3, track SB): its bands, the world's mass and
// radius and its moons as sculptors (system-3d ring-read.ts), and True draws what comes back (renderer rings.ts).
import { test } from "node:test";
import assert from "node:assert/strict";
import { fromSystemScene } from "../../packages/views/system-3d/src/bodies.ts";
import { ringAskFor, sculptorsOf } from "../../packages/views/system-3d/src/ring-read.ts";
import { constant, type SceneSubject, type SystemScene } from "../../packages/ui-contract/src/index.ts";
import { trueLook, type KernelRing } from "../../packages/renderer/src/index.ts";

const fact = (field: string, value: number) => ({ field, label: field, value, display: String(value), editable: false, source: "catalogue" }) as unknown as SceneSubject["facts"][number];
const subject = (address: string, kind: string, extra: Partial<SceneSubject>): SceneSubject => ({ address, kind, label: address.split("/").pop()!, facts: [], ...extra }) as SceneSubject;
const AU = constant("au");

const scene = {
  subjects: [
    subject("u/sun", "star", { facts: [fact("mass_sun", 1)] }),
    subject("u/saturn", "planet", { orbit: { a_au: 9.58, phase_deg: 0 }, facts: [fact("mass_earth", 95.16), fact("radius_earth", 9.14), fact("j2", 0.016298)] }),
    subject("u/saturn/b", "ring", { parent: "u/saturn", ring: { inner_m: 9.1975e7, outer_m: 1.17507e8, tau: 1.45, albedo: 0.5, tau_range: [0.4, 2.5] } }),
    subject("u/saturn/a", "ring", { parent: "u/saturn", ring: { inner_m: 1.2234e8, outer_m: 1.3678e8, tau: 0.7, albedo: 0.5 } }),
    subject("u/pan", "moon", { parent: "u/saturn", orbit: { a_au: 1.33584e8 / AU, phase_deg: 0 }, facts: [fact("mass_earth", 4.95e15 / constant("M_earth"))] }),
    subject("u/janus", "moon", { parent: "u/saturn", orbit: { a_au: 1.5146e8 / AU, phase_deg: 0 }, facts: [] }),
    subject("u:g/sys:1/body:3", "planet", { orbit: { a_au: 5, phase_deg: 0 }, facts: [fact("mass_earth", 300), fact("radius_earth", 11)] }),
    subject("u:g/sys:1/cloud:2", "belt", { parent: "u:g/sys:1/body:3", ring: { inner_m: 8e7, outer_m: 1.6e8, tau: null, albedo: null, cloud: "u:g/sys:1/cloud:2" } }),
  ],
} as unknown as SystemScene;

test("a catalogued ring is asked for whole, with the world and its moons that have a mass", () => {
  const bodies = fromSystemScene(scene, () => null as never);
  const saturn = bodies.find((b) => b.id === "u/saturn")!;
  assert.ok(Math.abs(saturn.mass_kg! / 5.683e26 - 1) < 0.01, `${saturn.mass_kg}`);
  // Janus has no mass on record, so only Pan shapes the ring.
  assert.deepEqual(sculptorsOf(bodies, "u/saturn").map((m) => m.name), ["pan"]);
  const ask = ringAskFor(bodies, "u/saturn") as { ask: { host: { mass_kg: number; j2?: number }; sculptors: unknown[]; ring: { zones: { tau_range?: number[]; source: string }[] } } };
  assert.equal(ask.ask.ring.zones.length, 2);
  assert.deepEqual(ask.ask.ring.zones[0]!.tau_range, [0.4, 2.5]);
  assert.equal(ask.ask.ring.zones[0]!.source, "catalogued");
  assert.equal(ask.ask.sculptors.length, 1);
  assert.equal(ask.ask.host.j2, 0.016298, "the world's J2 moves its resonances");
});

test("a ring formation made is asked for by its cloud", () => {
  const bodies = fromSystemScene(scene, () => null as never);
  assert.deepEqual(ringAskFor(bodies, "u:g/sys:1/body:3"), { system: "u:g/sys:1", cloud: "u:g/sys:1/cloud:2", samples: 8192 });
});

test("True draws the kernel's ring: its gap is clear and its grains keep their colour", () => {
  const bodies = fromSystemScene(scene, () => null as never);
  const saturn = bodies.find((b) => b.id === "u/saturn")!;
  // A kernel answer by hand: one A ring zone with Pan's gap (zone -1) in the middle sample.
  const ring: KernelRing = {
    inner_m: 1.2234e8,
    outer_m: 1.3678e8,
    zones: [{ name: "A", inner_m: 1.2234e8, outer_m: 1.3678e8, tau: 0.7, albedo: 0.5, rgb: [0.58, 0.5, 0.45], dust_share: 0.01, assumed: false }],
    marks: [],
    phase: { back_g: -0.3, forward_g: 0.7, surge_b0: 0.33, surge_width_deg: 0.5 },
    profile: { lo_m: 1.2234e8, hi_m: 1.3678e8, tau: [0.7, 0.7, 0, 0.7, 0.7], zone: [0, 0, -1, 0, 0] },
    why: [],
  };
  const look = trueLook({ key: "saturn", star: false, colour: "#ccc", radius_m: saturn.radius_m!, rings: saturn.rings!, ring_kernel: ring, pole: [0, 0, 1] });
  const r = look.rings!;
  assert.equal(r.width, 5);
  assert.equal(r.data[2 * 4 + 3], 0);
  assert.ok(r.data[3]! > 100);
  // Tan, not grey: red over blue.
  assert.ok(r.data[0]! > r.data[2]!);
  assert.ok(r.look);
});
