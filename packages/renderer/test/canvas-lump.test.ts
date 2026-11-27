// RW16a: the Body globe's canvas tier draws a small body's lump, the same one the GPU tiers draw, not a disc.
import { test } from "node:test";
import assert from "node:assert/strict";
import { drawCanvas } from "../src/canvas.ts";
import { prepare } from "../src/surface.ts";
import type { Plan } from "../src/recipe.ts";

/** A 2D canvas stand-in: keeps the image the tier puts. */
function fakeCanvas(n: number) {
  let put: { width: number; height: number; data: Uint8ClampedArray } | null = null;
  const ctx = {
    createImageData: (w: number, h: number) => ({ width: w, height: h, data: new Uint8ClampedArray(w * h * 4) }),
    putImageData: (img: typeof put) => void (put = img),
    drawImage: () => {},
    clearRect: () => {},
  };
  const c = { width: n, height: n, getContext: () => ctx };
  return { c: c as unknown as HTMLCanvasElement, image: () => put! };
}

/** Covered pixels and the silhouette's width and height. */
function silhouette(img: { width: number; height: number; data: Uint8ClampedArray }) {
  let count = 0, x0 = img.width, x1 = -1, y0 = img.height, y1 = -1;
  for (let y = 0; y < img.height; y++)
    for (let x = 0; x < img.width; x++)
      if (img.data[(y * img.width + x) * 4 + 3]! > 128) {
        count++;
        x0 = Math.min(x0, x), x1 = Math.max(x1, x), y0 = Math.min(y0, y), y1 = Math.max(y1, y);
      }
  return { count, w: x1 - x0 + 1, h: y1 - y0 + 1 };
}

const plan = { render_scale: 1, passes: [{ pass: "relief-globe", params: {} }] } as unknown as Plan;
const n = 96;

test("without a shape the canvas tier draws the globe's disc", () => {
  const f = fakeCanvas(n);
  drawCanvas(f.c, prepare([], "survey", 1), plan, 0, 0, n, {});
  const s = silhouette(f.image());
  assert.ok(Math.abs(s.w - s.h) <= 1, `${s.w} by ${s.h}`);
  assert.ok(Math.abs(s.count - (Math.PI * n * n) / 4) < 0.03 * n * n, `${s.count} px`);
});

test("with a lump it draws the lump: an ellipsoid twice as long as it is wide, seen side on", () => {
  // Long axis on the view's horizontal: at longitude 0 the view looks along x, so put it on y.
  const shape = { key: "test-ellipsoid", at: (d: [number, number, number]) => 1 / Math.sqrt(d[0] * d[0] * 4 + d[1] * d[1] + d[2] * d[2] * 4) };
  const f = fakeCanvas(n);
  drawCanvas(f.c, prepare([], "survey", 1), plan, 0, 0, n, {}, null, shape);
  const s = silhouette(f.image());
  const long = Math.max(s.w, s.h), short = Math.min(s.w, s.h);
  assert.ok(long > 0.9 * n, `the long axis spans the square: ${s.w} by ${s.h}`);
  assert.ok(short < 0.6 * n && short > 0.4 * n, `the short axis is half of it: ${s.w} by ${s.h}`);
  assert.ok(s.count < 0.6 * ((Math.PI * n * n) / 4), `fewer pixels than the disc: ${s.count}`);
});
