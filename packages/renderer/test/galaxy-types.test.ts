import { test } from "node:test";
import assert from "node:assert/strict";
import { galaxyTypePicture, sersicB, typeLight, youngShare, bvTemperature, PICTURE_REACH_RE, type GalaxyTypeFacts } from "../src/galaxy-types.ts";

const base: GalaxyTypeFacts = { address: "u:k/cell:0.0.0/halo:3", morphology: "disc", half_light_kpc: 4, axis_ratio: 1, position_angle_deg: 0, b_v: 0.6, stellar_mass_msun: 3e10, sfr_msun_yr: 1.5 };

test("Sersic b_n: R_e holds half the light (absolute values from Ciotti and Bertin 1999)", () => {
  assert.ok(Math.abs(sersicB(1) - 1.678) < 0.001, String(sersicB(1)));
  assert.ok(Math.abs(sersicB(4) - 7.669) < 0.001, String(sersicB(4)));
});

test("a face-on elliptical's light is half inside its half-light radius", () => {
  const e = { ...base, morphology: "elliptical", sfr_msun_yr: 0 };
  // Integrate in rings out to 60 R_e (a de Vaucouleurs law's wings are long).
  let inner = 0;
  let all = 0;
  const dr = 0.002;
  for (let r = dr / 2; r < 60 * e.half_light_kpc; r += r < 8 ? dr : 0.05) {
    const step = r < 8 ? dr : 0.05;
    const l = typeLight(e, r, 0).old * 2 * Math.PI * r * step;
    all += l;
    if (r < e.half_light_kpc) inner += l;
  }
  assert.ok(Math.abs(inner / all - 0.5) < 0.02, String(inner / all));
});

test("tilted and turned: the light is spread along the major axis by the axis ratio, turned by the position angle", () => {
  const flat = { ...base, morphology: "lenticular", axis_ratio: 0.3, sfr_msun_yr: 0 };
  // Position angle 0: major axis up the page.
  assert.ok(typeLight(flat, 0, 4).old > 5 * typeLight(flat, 4, 0).old);
  const turned = { ...flat, position_angle_deg: 90 };
  assert.ok(Math.abs(typeLight(turned, -4, 0).old - typeLight(flat, 0, 4).old) < 1e-12 * typeLight(flat, 0, 4).old + 1e-15);
});

test("young stars: knots only where stars form, never in an elliptical; the share is the True galaxy's", () => {
  assert.equal(youngShare(1e10, 0), 0);
  assert.ok(youngShare(1e10, 1) > youngShare(1e10, 0.1));
  const knots = (f: GalaxyTypeFacts) => {
    let y = 0;
    for (let i = -40; i <= 40; i++) for (let j = -40; j <= 40; j++) y += typeLight(f, i * 0.2, j * 0.2).young;
    return y;
  };
  assert.ok(knots(base) > 0);
  assert.equal(knots({ ...base, sfr_msun_yr: 0 }), 0);
  assert.equal(knots({ ...base, morphology: "elliptical" }), 0);
});

test("two discs differ by their keyed pattern; one disc is the same every time", () => {
  const a = galaxyTypePicture(base, 32);
  const again = galaxyTypePicture(base, 32);
  const b = galaxyTypePicture({ ...base, address: "u:k/cell:0.0.0/halo:4" }, 32);
  assert.deepEqual(a.rgba, again.rgba);
  assert.notDeepEqual(a.rgba, b.rgba);
  assert.equal(a.extent_kpc, PICTURE_REACH_RE * 4);
  // Brightest at the middle, dark at the corner.
  const at = (i: number, j: number) => a.rgba[(j * 32 + i) * 4 + 3]!;
  assert.ok(at(16, 16) > 200 && at(0, 0) < at(16, 16) / 3);
});

test("colour from B-V: a red galaxy is cooler than a blue one", () => {
  assert.ok(bvTemperature(1.0) < bvTemperature(0.3));
  assert.ok(Math.abs(bvTemperature(0.65) - 5800) < 300, String(bvTemperature(0.65)));
});
