import { test } from "node:test";
import assert from "node:assert/strict";
import { cutRings, ringMarks, type RingMoon } from "../src/ring-structure.ts";

// Saturn and moons (JPL: masses kg, semi-major axes km); rings' edges from Cassini (km).
const SATURN = 5.683e26;
const km = 1000;
const MOONS: RingMoon[] = [
  { name: "Pan", a_m: 133584 * km, mass_kg: 4.95e15 },
  { name: "Daphnis", a_m: 136505 * km, mass_kg: 7.7e13 },
  { name: "Prometheus", a_m: 139380 * km, mass_kg: 1.595e17 },
  { name: "Janus", a_m: 151460 * km, mass_kg: 1.898e18 },
  { name: "Mimas", a_m: 185539 * km, mass_kg: 3.75e19 },
];
const marks = ringMarks(MOONS, SATURN, 92000 * km, 136775 * km);

test("Mimas 2:1 falls at the B ring's outer edge and is strong enough to hold it", () => {
  const k = marks.find((x) => x.moon === "Mimas" && x.m === 2)!;
  assert.ok(Math.abs(k.r_m / (117580 * km) - 1) < 0.01, `${k.r_m / km} km`);
  assert.equal(k.kind, "edge");
  // Tajeddine et al. 2017: more than ten times the B ring's viscous torque.
  assert.ok(k.ratio > 10, `${k.ratio}`);
});

test("Janus 7:6 falls at the A ring's outer edge, near the balance with the viscosity", () => {
  const k = marks.find((x) => x.moon === "Janus" && x.m === 7)!;
  assert.ok(Math.abs(k.r_m / (136770 * km) - 1) < 0.002, `${k.r_m / km} km`);
  assert.ok(k.ratio > 0.2 && k.ratio < 5, `${k.ratio}`);
});

test("Pan clears the Encke gap and Daphnis the Keeler gap at about their real widths", () => {
  const pan = marks.find((x) => x.moon === "Pan")!;
  const daphnis = marks.find((x) => x.moon === "Daphnis")!;
  assert.equal(pan.kind, "gap");
  assert.ok(Math.abs((2 * pan.half_width_m) / (325 * km) - 1) < 0.1, `${(2 * pan.half_width_m) / km} km`);
  assert.ok(Math.abs((2 * daphnis.half_width_m) / (42 * km) - 1) < 0.1, `${(2 * daphnis.half_width_m) / km} km`);
});

test("cutting: a gap splits a ring, an edge well inside a ring truncates it, a recorded edge near it stays", () => {
  const rings = [
    { name: "B", inner_m: 92000 * km, outer_m: 117580 * km, tau: 2, albedo: 0.5 },
    { name: "A", inner_m: 122170 * km, outer_m: 136775 * km, tau: 0.5, albedo: 0.5 },
  ];
  const cut = cutRings(rings, marks);
  // The B ring keeps its recorded edge (within the resonance's slack).
  assert.ok(cut.some((r) => r.name === "B" && r.outer_m === 117580 * km));
  // The A ring is split by the Encke gap.
  const a = cut.filter((r) => r.name === "A");
  assert.ok(a.some((r) => r.outer_m < 133584 * km) && a.some((r) => r.inner_m > 133584 * km));
  // A made ring running far past Mimas 2:1 is cut there.
  const wide = cutRings([{ name: "made", inner_m: 100000 * km, outer_m: 130000 * km, tau: 1, albedo: 0.5 }], ringMarks([MOONS[4]!], SATURN, 100000 * km, 130000 * km));
  assert.ok(Math.abs(wide[0]!.outer_m / (116900 * km) - 1) < 0.01, `${wide[0]!.outer_m / km}`);
});

test("no moons, no marks; a world with no mass on record is left alone", () => {
  assert.deepEqual(ringMarks([], SATURN, 1, 2), []);
  assert.deepEqual(ringMarks(MOONS, 0, 92000 * km, 136775 * km), []);
});

test("trueLook cuts a ringed world's rings when it is given its mass and moons", async () => {
  const { trueLook } = await import("../src/true-body.ts");
  const base = { key: "saturn", star: false, colour: "#ccc", radius_m: 58232 * km, rings: [{ name: "A", inner_m: 122170 * km, outer_m: 136775 * km, tau: 0.5, albedo: 0.5 }], pole: [0, 0, 1] as [number, number, number] };
  const plain = trueLook(base).rings!;
  const cut = trueLook({ ...base, mass_kg: SATURN, moons: MOONS }).rings!;
  assert.equal(plain.width, 512);
  assert.equal(cut.width, 2048);
  // A clear pixel where Pan is.
  const u = (133584 / 58232 - cut.inner) / (cut.outer - cut.inner);
  assert.equal(cut.data[Math.floor(u * cut.width) * 4 + 3], 0);
});
