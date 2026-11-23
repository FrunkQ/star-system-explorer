import { test } from "node:test";
import assert from "node:assert/strict";
import { magExtent, magShape, placeMag } from "../src/mag-true.ts";
import type { FieldReadJson } from "../src/field.ts";

// Earth-like test data in read_field's shape (not physics): a 30 µT dipole tilted about 10°, a 9.3-radius stand-off.
const earth: FieldReadJson = {
  address: "test/earth",
  name: "Earth",
  obstacle: { kind: "field", standoff_radii: 9.3 },
  flaring: 0.58,
  field: { reference_radius_m: 6.3712e6, g10_t: -2.94e-5, g11_t: -1.5e-6, h11_t: 4.8e-6 },
  wind_field: { radial_t: 3.5e-9, azimuthal_t: 3.5e-9 },
};

test("a world's magnetosphere: lines in both inks, all inside the drawn reach, the bubble's nose toward the star", () => {
  const s = magShape(earth)!;
  assert.ok(s, "a shape");
  assert.equal(s.standoff, 9.3);
  assert.equal(s.extent, magExtent(9.3));
  assert.ok(s.runs.some((r) => r.out) && s.runs.some((r) => !r.out), "both inks");
  assert.ok(s.runs.some((r) => r.open), "open lines from the caps");
  for (const r of s.runs) for (const p of r.points) assert.ok(Math.hypot(...p) <= s.extent + 1e-9);
  // The bubble's farthest point sunward is its nose, at the stand-off on +x.
  const nose = Math.max(...s.bubble.flat().map((p) => p[0]));
  assert.ok(Math.abs(nose - 9.3) < 0.05, `nose ${nose}`);
});

test("no field facts, no shape; a world whose air holds the wind off has no bubble", () => {
  assert.equal(magShape({ ...earth, field: undefined }), null);
  const venus = magShape({ ...earth, obstacle: { kind: "ionosphere", standoff_radii: 1.1 } })!;
  assert.equal(venus.standoff, null);
  assert.equal(venus.bubble.length, 0);
});

test("placed in the view: body x turns to the star, sizes scale with the drawn world, and the reach cuts it short", () => {
  const s = magShape(earth)!;
  const centre: [number, number, number] = [10, 0, 0];
  // The star is at the origin: sunward is −x in the view.
  const p = placeMag(s, centre, 0.1, [-1, 0]);
  const xs = p.bubble.flat().map((q) => q[0]);
  assert.ok(Math.abs(Math.min(...xs) - (10 - 0.93)) < 0.01, "nose toward the star");
  const cut = placeMag(s, centre, 0.1, [-1, 0], 0.3);
  for (const r of cut.runs) for (const q of r.points) assert.ok(Math.hypot(q[0] - 10, q[1], q[2]) <= 0.3 + 1e-9);
  assert.ok(cut.runs.length > 0);
});
