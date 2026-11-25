// Hot things glow in Visible (third audit R11, R12): Planck seen by the eye, against the world's own daylight.
import { test } from "node:test";
import assert from "node:assert/strict";
import { drawnGlow, hotGlow } from "../src/hot-glow.ts";

const R_SUN = 6.957e8;
const AU = 1.495978707e11;
const sun = (d_au: number) => ({ t_k: 5772, r_m: R_SUN, d_m: d_au * AU });

test("below the Draper point nothing glows, and an unknown temperature draws nothing", () => {
  assert.equal(hotGlow(288, sun(1)), null);
  assert.equal(hotGlow(700, sun(0.05)), null);
  assert.equal(hotGlow(null, sun(1)), null);
});

test("a hot Jupiter's night side is a faint red ember against its daylight", () => {
  const g = hotGlow(1500, sun(0.05))!;
  assert.ok(g.ratio > 1e-5 && g.ratio < 1e-2, `ratio ${g.ratio}`);
  // Red before green before blue.
  assert.ok(g.rgb[0] > g.rgb[1] && g.rgb[1] >= g.rgb[2], `${g.rgb}`);
  assert.match(g.why, /1500 K/);
});

test("an ultra-hot Jupiter glows more strongly; hotter is brighter and whiter", () => {
  const a = hotGlow(1500, sun(0.02))!;
  const b = hotGlow(2500, sun(0.02))!;
  assert.ok(b.ratio > 50 * a.ratio, `${a.ratio} ${b.ratio}`);
  assert.ok(b.rgb[2] / b.rgb[0] > a.rgb[2] / Math.max(a.rgb[0], 1e-30));
});

test("a brown dwarf far from its star outshines its own daylight, clipped to its colour", () => {
  const g = hotGlow(1300, sun(50))!;
  assert.ok(g.ratio > 1, `ratio ${g.ratio}`);
  assert.equal(Math.max(...g.rgb), 1);
  assert.match(g.why, /brighter than its own daylight/);
});

test("with no star the world's glow is its only light, at full colour", () => {
  const g = hotGlow(1000, null)!;
  assert.equal(Math.max(...g.rgb), 1);
  assert.equal(g.ratio, Infinity);
});

test("a glow under a screen's bottom step is not drawn", () => {
  assert.equal(drawnGlow(hotGlow(900, sun(0.05))), null);
  assert.ok(drawnGlow(hotGlow(1300, sun(50))));
});
