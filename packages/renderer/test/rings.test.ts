// V10f: rings and belts in True.
import { test } from "node:test";
import assert from "node:assert/strict";
import { beltHaze, poleOf, RING_FALLBACK, ringProfile } from "../src/rings.ts";

test("a ring's cover follows its optical depth; gaps are clear; the narrowest ring speaks inside a broader one", () => {
  const p = ringProfile(
    [
      { name: "thick", inner_m: 100, outer_m: 200, tau: 2, albedo: 0.5 },
      { name: "gap", inner_m: 140, outer_m: 150, tau: 0.1, albedo: 0.5 },
      { name: "thin", inner_m: 300, outer_m: 400, tau: 0.05, albedo: 0.5 },
    ],
    [1, 1, 1],
    300,
  )!;
  assert.equal(p.inner_m, 100);
  assert.equal(p.outer_m, 400);
  assert.equal(p.assumed, false);
  const alpha = (r: number) => p.data[Math.floor(((r - 100) / 300) * 300) * 4 + 3]!;
  assert.equal(alpha(120), Math.round(255 * (1 - Math.exp(-2))));
  assert.equal(alpha(145), Math.round(255 * (1 - Math.exp(-0.1))), "the gap inside the broad ring");
  assert.equal(alpha(250), 0, "clear between rings");
  assert.ok(alpha(350) > 0 && alpha(350) < alpha(120));
});

test("a ring with nothing on record is drawn at the fallbacks and says so; no edges, no picture", () => {
  const p = ringProfile([{ name: "?", inner_m: 1, outer_m: 2, tau: null, albedo: null }], [0.5, 0.5, 0.5], 10)!;
  assert.equal(p.assumed, true);
  assert.equal(p.data[3], Math.round(255 * (1 - Math.exp(-RING_FALLBACK.tau))));
  assert.equal(ringProfile([{ name: "bad", inner_m: 2, outer_m: 1, tau: 1, albedo: 1 }], [1, 1, 1]), null);
});

test("the pole leans by the tilt; a belt's haze stays between its edges, thin, and the same every time", () => {
  const z = poleOf(0);
  assert.deepEqual(z, [0, 0, 1]);
  const t = poleOf(Math.PI / 2);
  assert.ok(Math.abs(t[0] - 1) < 1e-12 && Math.abs(t[2]) < 1e-12);
  const h = beltHaze(2, 3, 0.05, 500);
  assert.equal(h.length, 500);
  for (const [x, y, zz] of h) {
    const r = Math.hypot(x, y);
    assert.ok(r >= 2 - 1e-9 && r <= 3 + 1e-9);
    assert.ok(Math.abs(zz) < 0.5 * r, "a disc, not a shell");
  }
  assert.deepEqual(beltHaze(2, 3, 0.05, 500), h);
});
