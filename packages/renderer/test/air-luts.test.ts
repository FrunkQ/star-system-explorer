// The air's look-up tables (VP-A3, Hillaire 2020): straight up the air lets through e^-τ of the zenith depth, a ray
// into the ground lets nothing through, and light scattered many times is bluer than red and grows with the air.
import { test } from "node:test";
import assert from "node:assert/strict";
import { airLuts, airShapeOf, readMultiple, readTransmittance, TOP_H } from "../src/air-luts.ts";

// Earth-like: Rayleigh at the ground per metre at 680, 550 and 440 nm (Bucholtz 1995 order), 8.5 km scale height.
const EARTH = airShapeOf({ rayleigh_m: [5.8e-6, 13.5e-6, 33.1e-6], scale_height_m: 8500 }, 6.371e6);
const out = [0, 0, 0];

test("straight up, the air lets through e^-(β H) of the light; into the ground, none", () => {
  const l = airLuts(EARTH, 16, 8);
  assert.ok(Math.abs(l.top - (1 + TOP_H * EARTH.scale_height)) < 1e-12);
  readTransmittance(l, 0, 1, out);
  for (let c = 0; c < 3; c++) {
    const want = Math.exp(-EARTH.beta[c]! * EARTH.scale_height * (1 - Math.exp(-TOP_H)));
    assert.ok(Math.abs(out[c]! - want) < 0.003, `channel ${c}: ${out[c]} vs ${want}`);
  }
  // Blue is dimmed most.
  assert.ok(out[2]! < out[1]! && out[1]! < out[0]!);
  readTransmittance(l, 0, -0.5, out);
  assert.deepEqual(out.map((x) => Math.round(x * 1e6)), [0, 0, 0]);
  // At the top looking up, nothing in the way.
  readTransmittance(l, l.top - 1, 1, out);
  assert.ok(out.every((x) => x > 0.999), out.join(" "));
});

test("light scattered many times is blue, grows with thicker air, and is none in the shadow's depth", () => {
  const thin = airLuts({ beta: EARTH.beta.map((b) => b / 10) as [number, number, number], scale_height: EARTH.scale_height }, 32, 12);
  const thick = airLuts(EARTH, 32, 12);
  readMultiple(thick, 0, 1, out);
  const [r, , b] = out;
  assert.ok(b! > r! * 1.5, `blue ${b} red ${r}`);
  // Hillaire 2020 fig. 6: a few per cent of the sun's light at the ground in blue.
  assert.ok(b! > 0.005 && b! < 0.1, `blue ψ_ms ${b}`);
  const earth = b!;
  readMultiple(thin, 0, 1, out);
  assert.ok(out[2]! < earth / 2, `thin ${out[2]} thick ${earth}`);
  // The sun well below the horizon at the ground: almost nothing.
  readMultiple(thick, 0, -0.9, out);
  assert.ok(out[2]! < earth / 50, `night ${out[2]}`);
});
