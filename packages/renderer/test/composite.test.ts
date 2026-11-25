import { test } from "node:test";
import assert from "node:assert/strict";
import { composite, tintMatrix } from "../src/composite.ts";

test("each band is scaled to its own brightest before adding, so a faint layer is not drowned", () => {
  // Two pixels. Visible is bright, radio is a millionth of it but still reaches full strength in its colour.
  const out = composite(
    [
      { band: "visible", colour: "natural", rgb: [1000, 1000, 1000, 0, 0, 0] },
      { band: "radio", colour: "#ff9a1f", rgb: [0, 0, 0, 1e-3, 1e-3, 1e-3] },
    ],
    2,
  );
  assert.deepEqual([...out.slice(0, 3)], [1, 1, 1]);
  assert.ok(Math.abs(out[3]! - 1) < 1e-6 && Math.abs(out[4]! - 0x9a / 255) < 1e-6 && out[5]! < 0.13);
});

test("the tint matrix lays light in its colour on a dark ground and ink in its colour on a light one", () => {
  const d = tintMatrix("#ff0000", true);
  // white in: red out
  const apply = (m: number[], c: number[]) => [0, 1, 2].map((k) => m[5 * k]! * c[0]! + m[5 * k + 1]! * c[1]! + m[5 * k + 2]! * c[2]! + m[5 * k + 4]!);
  assert.deepEqual(apply(d, [1, 1, 1]).map((x) => +x.toFixed(6)), [1, 0, 0]);
  assert.deepEqual(apply(d, [0, 0, 0]), [0, 0, 0]);
  const l = tintMatrix("#ff0000", false);
  assert.deepEqual(apply(l, [1, 1, 1]).map((x) => +x.toFixed(6)), [1, 1, 1], "white ground stays white");
  assert.deepEqual(apply(l, [0, 0, 0]), [1, 0, 0], "black ink becomes red");
});
