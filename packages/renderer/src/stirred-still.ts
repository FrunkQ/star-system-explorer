// A giant's stirred still (track WS, package WS5; design SSE4-stirred-clouds-design.md 3.6, 6 and 11A).
//
// Every pixel of today's face is carried back along the kernel's winds (stirred-gpu.ts, or its processor twin
// stirred-carry.ts) to where each of its two generations of cloud was made, and takes the unstirred face's colour
// there, the two weighed as the kernel weighs them. Where the backward map has drawn the cloud finer than a pixel,
// the colour is read from a coarser level of the face, by how far the map stretches there: cloud stirred finer than
// the picture shows comes out as its local mean, as the eye would see it. The colours are the face's own; stirring
// only moves them.
//
// The face is equirectangular, row 0 at the south pole, lat = (j + ½)/h · 180 − 90, lon = (i + ½)/w · 360 − 180.

import type { TrueTexture } from "./true-look.ts";
import { generations, wrap, type CarryTable } from "./stirred-carry.ts";

/** The places of a `width` × `height` face's pixel centres, lat and lon in degrees, row by row from the south. */
export function placesFor(width: number, height: number): Float32Array {
  const out = new Float32Array(width * height * 2);
  for (let j = 0; j < height; j++)
    for (let i = 0; i < width; i++) out.set([((j + 0.5) / height) * 180 - 90, ((i + 0.5) / width) * 360 - 180], (j * width + i) * 2);
  return out;
}

/** The face as float levels, each half the last (wrapped east-west), down to four pixels wide: RGBA 0..255. */
export function levelsOf(face: TrueTexture): { width: number; height: number; data: Float32Array }[] {
  const out = [{ width: face.width, height: face.height, data: Float32Array.from(face.data) }];
  for (;;) {
    const a = out[out.length - 1]!;
    if (a.width < 8 || a.height < 2) break;
    const w = a.width >> 1;
    const h = a.height >> 1;
    const d = new Float32Array(w * h * 4);
    for (let j = 0; j < h; j++)
      for (let i = 0; i < w; i++)
        for (let c = 0; c < 4; c++) {
          const at = (x: number, y: number) => a.data[(y * a.width + x) * 4 + c]!;
          d[(j * w + i) * 4 + c] = (at(2 * i, 2 * j) + at(2 * i + 1, 2 * j) + at(2 * i, 2 * j + 1) + at(2 * i + 1, 2 * j + 1)) / 4;
        }
    out.push({ width: w, height: h, data: d });
  }
  return out;
}

/** A level's colour at a place, bilinear, wrapped east-west and held at the poles. */
function sample(l: { width: number; height: number; data: Float32Array }, lat: number, lon: number, out: Float32Array, k: number, wgt: number) {
  const fx = ((wrap(lon) + 180) / 360) * l.width - 0.5;
  const fy = Math.min(Math.max(((lat + 90) / 180) * l.height - 0.5, 0), l.height - 1);
  const x0 = Math.floor(fx);
  const y0 = Math.min(Math.floor(fy), l.height - 2 < 0 ? 0 : l.height - 2);
  const tx = fx - x0;
  const ty = l.height > 1 ? fy - y0 : 0;
  const xa = ((x0 % l.width) + l.width) % l.width;
  const xb = (xa + 1) % l.width;
  const ya = y0;
  const yb = Math.min(y0 + 1, l.height - 1);
  for (let c = 0; c < 4; c++) {
    const v = (x: number, y: number) => l.data[(y * l.width + x) * 4 + c]!;
    out[k + c]! += wgt * ((v(xa, ya) * (1 - tx) + v(xb, ya) * tx) * (1 - ty) + (v(xa, yb) * (1 - tx) + v(xb, yb) * tx) * ty);
  }
}

/**
 * How far the backward map stretches cloud at pixel (i, j) for one generation, in the face's pixels per picture pixel:
 * the larger spread of the map's slopes, from the read places of the pixels either side (one side at a pole row).
 */
export function stretchAt(read: Float32Array, width: number, height: number, i: number, j: number, gen: 0 | 1): number {
  const at = (x: number, y: number): [number, number] => {
    const k = (y * width + ((x + width) % width)) * 4 + gen * 2;
    return [read[k]!, read[k + 1]!];
  };
  const d = (a: [number, number], b: [number, number], n: number): [number, number] => [
    ((wrap(b[1] - a[1]) / 360) * width) / n,
    (((b[0] - a[0]) / 180) * height) / n,
  ];
  const ex = d(at(i - 1, j), at(i + 1, j), 2);
  const ny = d(at(i, Math.max(j - 1, 0)), at(i, Math.min(j + 1, height - 1)), j === 0 || j === height - 1 ? 1 : 2);
  // The larger singular value of [[ex₀, ny₀], [ex₁, ny₁]].
  const a = ex[0] * ex[0] + ex[1] * ex[1];
  const b = ex[0] * ny[0] + ex[1] * ny[1];
  const c = ny[0] * ny[0] + ny[1] * ny[1];
  return Math.sqrt((a + c) / 2 + Math.sqrt(((a - c) / 2) ** 2 + b * b));
}

/**
 * Today's stirred face from the unstirred one and where each pixel's two generations are read (RGBA per pixel:
 * younger lat, lon, older lat, lon, as stirred-gpu.ts `carryOnGpu` returns), weighed by the kernel's generations.
 */
export function composeStill(face: TrueTexture, t: CarryTable, read: Float32Array): TrueTexture {
  const { width, height } = face;
  const levels = levelsOf(face);
  const top = levels.length - 1;
  const acc = new Float32Array(width * height * 4);
  for (let j = 0; j < height; j++) {
    const lat = ((j + 0.5) / height) * 180 - 90;
    const g = generations(t, lat);
    for (let i = 0; i < width; i++) {
      const k = (j * width + i) * 4;
      for (const gen of [0, 1] as const) {
        const wgt = g[gen][1];
        if (wgt <= 0) continue;
        // A level per doubling of the stretch, read between the two either side.
        const lv = Math.min(Math.max(Math.log2(Math.max(stretchAt(read, width, height, i, j, gen), 1)), 0), top);
        const l0 = Math.floor(lv);
        const f = lv - l0;
        const la = read[k + gen * 2]!;
        const lo = read[k + gen * 2 + 1]!;
        sample(levels[l0]!, la, lo, acc, k, wgt * (1 - f));
        if (f > 0) sample(levels[Math.min(l0 + 1, top)]!, la, lo, acc, k, wgt * f);
      }
    }
  }
  const data = new Uint8Array(width * height * 4);
  for (let k = 0; k < data.length; k++) data[k] = Math.min(255, Math.max(0, Math.round(acc[k]!)));
  // Stirring moves the colours and makes none, so the disc's mean colour is the face's own.
  return { width, height, data, mean: [...face.mean] as [number, number, number] };
}
