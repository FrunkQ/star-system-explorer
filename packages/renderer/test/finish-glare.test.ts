// The shared finish's glare (VP1): the eye's scatter by the CIE equation, split into doubling blurs.
import { test } from "node:test";
import assert from "node:assert/strict";
import { GLARE_FLOOR_DEG, glareLadder, glareShare, glareTotal, rungScale } from "../src/finish-glare.ts";
import { GLOW_REACH_DEG } from "../src/star-glow.ts";


test("the ladder carries exactly the share the CIE equation scatters from its floor to the reach", () => {
  for (const px of [0.01, 0.05, 0.2]) {
    const l = glareLadder(px, 6);
    const from = Math.max(GLARE_FLOOR_DEG, px);
    const want = glareShare(from, GLOW_REACH_DEG);
    assert.ok(Math.abs(glareTotal(l) - want) < 1e-3 * want, `${px}: ${glareTotal(l)} against ${want}`);
  }
});

test("about a quarter of a point's light is scattered between 0.1° and 20° (young eye, brown iris)", () => {
  const s = glareShare(0.1, 20);
  assert.ok(s > 0.2 && s < 0.3, `${s}`);
});

test("the rungs double in width and never start inside the equation's floor", () => {
  const px = 0.02;
  const l = glareLadder(px, 6);
  for (let i = 1; i < l.length; i++) assert.equal(l[i]!.sigma_px, 2 * l[i - 1]!.sigma_px);
  assert.ok(2 * l[0]!.sigma_px * px >= GLARE_FLOOR_DEG);
});

test("a wide rung is blurred at low resolution with a few samples' spacing", () => {
  const r = rungScale(64);
  assert.ok(r.scale < 1 && Math.abs(64 * r.scale - 3) < 1e-9);
  assert.equal(rungScale(1).scale, 1);
});
