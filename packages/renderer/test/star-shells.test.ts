import { test } from "node:test";
import assert from "node:assert/strict";
import { drawShells, nebulaColour, wavelengthRgb, byDensitySquared, type NebulaShape } from "../src/star-shells.ts";

const AU = 1.495978707e11;
const pic = (s: NebulaShape, half_m: number, axis: [number, number, number] = [0, 1, 0]) => {
  const o = { width: 48, data: new Uint8ClampedArray(48 * 48 * 4) };
  drawShells(o, s, { half_m, axis });
  return (x: number, y: number) => o.data[(y * 48 + x) * 4 + 3]!;
};
const pn: NebulaShape = { address: "a", kind: "planetary-nebula", shells: [{ radius_m: 1000 * AU, thickness_m: 150 * AU, mass_kg: 1e29 }], light: { lines: [{ line: "ha", power_w: 1e27 }, { line: "oiii", power_w: 3e26 }] } };

test("a shed shell is brightest at its rim (the line of sight runs along it) and dark beyond it", () => {
  const at = pic(pn, 1300 * AU);
  const rim = at(24 + Math.round((950 / 1300) * 24), 24);
  assert.ok(rim > at(24, 24), `rim ${rim} over middle ${at(24, 24)}`);
  assert.equal(at(47, 24), 0, "nothing beyond the shell");
});

test("two lobes stretch along the axis; a jet lies only along it, its knots brightest", () => {
  const lobes = pic({ ...pn, lobes: { axis: [0, 1, 0], ratio: 3 } }, 3500 * AU);
  assert.ok(lobes(24, 3) > lobes(3, 24), "longer along the axis than across");
  const jet: NebulaShape = { address: "j", kind: "jet", shells: [], jet: { length_m: 2000 * AU, speed_m_s: 2e5, opening_rad: 0.05, mass_rate_kg_s: 1e15, knots: [{ distance_m: 1200 * AU }] } };
  const j = pic(jet, 2200 * AU, [1, 0, 0]);
  assert.ok(j(40, 24) > 0 && j(8, 24) > 0, "both sides along x");
  assert.equal(j(24, 4), 0, "nothing across the axis");
  const knot = 24 + Math.round((1200 / 2200) * 24);
  assert.ok(j(knot, 24) >= j(knot - 4, 24), "the knot stands out");
});

test("glowing gas is drawn by density squared, dust by density; colours come from the descriptor's own light", () => {
  assert.equal(byDensitySquared(pn), true);
  assert.equal(byDensitySquared({ ...pn, kind: "dust-shell" }), false);
  const c = nebulaColour({ ...pn, light: { lines: [{ line: "ha", power_w: 1 }] } }, "visible");
  assert.ok(c[0] === 1 && c[2] < 0.05, "H-alpha is red");
  const g = wavelengthRgb(500.7e-9);
  assert.ok(g[1] > g[0] && g[1] > g[2], "[O III] is green-blue, greenest");
  const dust = nebulaColour({ address: "d", kind: "dust-shell", shells: [], light: { parts: [{ kind: "thermal", t_k: 1000 }] } }, "visible", 3500);
  assert.ok(dust[0] >= dust[2], "cool dust shows the red giant's reflected light");
});
