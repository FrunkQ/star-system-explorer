// The atmosphere pass's numbers come from physics: Earth's air from its
// pressure, temperature, gravity and mix must give Bucholtz's (1995) sea-level
// Rayleigh coefficients (about 11.5 per million metres at 550 nm, falling as
// lambda^-4.08: 4.84 at 680 nm and 28.6 at 440 nm) and a scale height near
// 8.4 km; Mars and Titan follow from their own facts. (Graphics papers often
// use 5.8, 13.5 and 33.1, about 15% above Bucholtz; we follow the physics.)
import { test } from "node:test";
import assert from "node:assert/strict";
import { air, airFactsOf } from "../src/atmosphere.ts";

const near = (a: number, b: number, tol: number) => Math.abs(a - b) / b <= tol;

test("Earth's air gives the textbook Rayleigh coefficients and scale height", () => {
  const a = air({ pressure_pa: 101325, temperature_k: 288, gravity_m_s2: 9.81, radius_m: 6.371e6, composition: { N2: 0.78, O2: 0.21, Ar: 0.0093 } })!;
  const want = [4.84e-6, 11.49e-6, 28.6e-6];
  a.rayleigh_m.forEach((b, i) => assert.ok(near(b, want[i]!, 0.04), `band ${i}: ${b} vs ${want[i]}`));
  assert.ok(near(a.scale_height_m, 8430, 0.03), `H = ${a.scale_height_m}`);
  assert.ok(near(a.mean_molar_mass, 28.96, 0.01));
  // Blue scatters (440/680)^4 ≈ 5.7 times more than red.
  assert.ok(near(a.rayleigh_m[2] / a.rayleigh_m[0], (680 / 440) ** 4, 0.01));
});

test("Mars: thin CO2 air, a scale height near 11 km, far less scattering", () => {
  const m = air({ pressure_pa: 610, temperature_k: 210, gravity_m_s2: 3.71, radius_m: 3.39e6, composition: { CO2: 0.95, N2: 0.028, Ar: 0.02 } })!;
  assert.ok(near(m.scale_height_m, 10.8e3, 0.08), `H = ${m.scale_height_m}`);
  assert.ok(m.zenith_depth[2] < 0.02, `Mars's blue zenith depth is small: ${m.zenith_depth[2]}`);
});

test("Titan: thick nitrogen, a deep zenith column", () => {
  const t = air({ pressure_pa: 146700, temperature_k: 94, gravity_m_s2: 1.352, radius_m: 2.575e6, composition: { N2: 0.95, CH4: 0.05 } })!;
  assert.ok(t.scale_height_m > 15e3 && t.scale_height_m < 25e3, `H = ${t.scale_height_m}`);
  assert.ok(t.zenith_depth[2] > 1, `Titan's blue zenith column is optically thick: ${t.zenith_depth[2]}`);
});

test("no air, unknown gases and missing facts are said, not invented", () => {
  assert.equal(air({ pressure_pa: 0, temperature_k: 300, gravity_m_s2: 3, radius_m: 1e6, composition: { N2: 1 } }), null);
  const x = air({ pressure_pa: 1e5, temperature_k: 300, gravity_m_s2: 9, radius_m: 6e6, composition: { N2: 0.9, Xq: 0.1 } })!;
  assert.deepEqual(x.unknown_gases, ["Xq"]);
  assert.ok(near(x.unknown_share, 0.1, 1e-9));
  assert.equal(airFactsOf({ "body.surface_pressure": 101300 }, { gravity_m_s2: 9.8, t_surface_now_k: 285, reference_radius_m: 6.4e6 }), null, "no gas mix, no air");
  assert.ok(airFactsOf({ "body.surface_pressure": 101300, "v3.atmosphere": { composition: { N2: 0.78, O2: 0.21 } } }, { gravity_m_s2: 9.8, t_surface_now_k: 285, reference_radius_m: 6.4e6 }));
});
