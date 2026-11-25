import { test } from "node:test";
import assert from "node:assert/strict";
import { faceSample, withFace } from "../src/globe-face.ts";
import { cubeDirection } from "../../ui-contract/src/index.ts";

// A face laid out as true-look.ts `equirect` lays it, each pixel's colour naming its column and row.
const w = 32;
const h = 16;
const data = new Uint8Array(w * h * 4);
for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) data.set([i * 8, j * 16, 77, 255], (j * w + i) * 4);
const tex = { width: w, height: h, data };
const dirOf = (i: number, j: number) => {
  const u = (i + 0.5) / w;
  const th = Math.PI * (1 - (j + 0.5) / h);
  return [-Math.cos(2 * Math.PI * u) * Math.sin(th), -Math.sin(2 * Math.PI * u) * Math.sin(th), Math.cos(th)];
};

test("a face read along a pixel's own direction gives that pixel, in equirect's layout", () => {
  for (const [i, j] of [[0, 0], [5, 3], [31, 15], [16, 8]] as const) {
    const c = faceSample(tex, dirOf(i, j));
    assert.ok(Math.abs(c[0] - i * 8) < 1e-6 && Math.abs(c[1] - j * 16) < 1e-6, `${i},${j}: ${c}`);
  }
  // Longitude wraps between the last column and the first.
  const mid = faceSample(tex, dirOf(31.5, 8));
  assert.ok(Math.abs(mid[0] - (31 * 8) / 2) < 1e-6);
});

test("the globe's samples wear the face: its colours, flat normals, albedo from the colour", () => {
  const s = 4;
  const f = { s, rgb: new Uint8ClampedArray(s * s * 3), normal: new Float32Array(s * s * 3), albedo: new Float32Array(s * s), tag: "kept" };
  const p = { faces: [f, null, null, null, null, null], other: 1 };
  const out = withFace(p, tex, cubeDirection);
  assert.equal(out.other, 1);
  assert.equal(out.faces[1], null);
  const g = out.faces[0]!;
  assert.equal(g.tag, "kept");
  const d = cubeDirection(0, -1 + 0.5 * (2 / s), -1 + 0.5 * (2 / s));
  const want = faceSample(tex, d);
  assert.deepEqual([g.rgb[0], g.rgb[1], g.rgb[2]], want.map((x) => Math.round(Math.min(255, x))));
  const l = Math.hypot(...d);
  assert.ok(Math.abs(g.normal[0]! - d[0] / l) < 1e-6);
  assert.ok(g.albedo[0]! > 0 && g.albedo[0]! <= 1);
});
