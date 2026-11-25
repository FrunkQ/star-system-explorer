// VP5: the enlarging and sharpening maths, in plain numbers (the shaders use the same formulas; the browser check is
// tests/ui/enlarge.e2e.mjs).
import { test } from "node:test";
import assert from "node:assert/strict";
import { catmullRom, RCAS_LIMIT, rcasLobe, rcasValue } from "../src/enlarge.ts";

test("Catmull-Rom weights sum to one and pass through the texels", () => {
  for (const t of [0, 0.25, 0.5, 0.75, 1]) assert.ok(Math.abs(catmullRom(t).reduce((a, b) => a + b, 0) - 1) < 1e-12);
  assert.deepEqual(catmullRom(0), [0, 1, 0, 0]);
});

test("the sharpener does nothing on flat ground and never passes its limit", () => {
  assert.equal(rcasValue(0.5, 0.5, 0.5, 0.5, 0.5, 0), 0.5);
  for (const [mn, mx] of [[0, 1], [0.2, 0.8], [0.45, 0.55], [0, 0.1], [0.9, 1]]) {
    const l = rcasLobe(mn!, mx!, 0);
    assert.ok(l <= 0 && l >= -RCAS_LIMIT, `${mn} ${mx}: ${l}`);
  }
  // Each stop halves it.
  assert.ok(Math.abs(rcasLobe(0.4, 0.6, 1) - rcasLobe(0.4, 0.6, 0) / 2) < 1e-12);
});

test("an edge is steepened but stays within its neighbours' range (no ringing)", () => {
  // A pixel at 0.4 beside neighbours 0.2 and 0.8: pushed away from the mean, never past 0..1.
  // Across an edge, a pixel on the bright side is pushed brighter (steeper), within range.
  const v = rcasValue(0.7, 0.7, 0.2, 0.9, 0.7, 0);
  assert.ok(v > 0.7 && v <= 0.9, String(v));
  for (let i = 0; i < 200; i++) {
    const r = () => ((i * 7919 + 13) % 101) / 100;
    const xs = [r(), ((i * 31) % 97) / 96, ((i * 17) % 89) / 88, ((i * 13) % 83) / 82, ((i * 11) % 79) / 78];
    const out = rcasValue(xs[0]!, xs[1]!, xs[2]!, xs[3]!, xs[4]!, 0);
    assert.ok(out >= Math.min(...xs) - 1e-9 && out <= Math.max(...xs) + 1e-9, `${xs}: ${out}`);
  }
});
