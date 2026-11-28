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

type Level = { width: number; height: number; data: Float32Array };

/** A level halved east-west (wrapped). */
function halveLon(a: Level): Level {
  const w = a.width >> 1;
  const d = new Float32Array(w * a.height * 4);
  for (let j = 0; j < a.height; j++)
    for (let i = 0; i < w; i++)
      for (let c = 0; c < 4; c++) d[(j * w + i) * 4 + c] = (a.data[(j * a.width + 2 * i) * 4 + c]! + a.data[(j * a.width + 2 * i + 1) * 4 + c]!) / 2;
  return { width: w, height: a.height, data: d };
}
/** A level halved north-south. */
function halveLat(a: Level): Level {
  const h = a.height >> 1;
  const d = new Float32Array(a.width * h * 4);
  for (let j = 0; j < h; j++)
    for (let i = 0; i < a.width; i++)
      for (let c = 0; c < 4; c++) d[(j * a.width + i) * 4 + c] = (a.data[(2 * j * a.width + i) * 4 + c]! + a.data[((2 * j + 1) * a.width + i) * 4 + c]!) / 2;
  return { width: a.width, height: h, data: d };
}

/**
 * The face as float levels halved east-west and north-south apart (RGBA 0..255): `[ns][ew]`, down to four pixels wide
 * and two tall. Halving the two ways apart lets cloud sheared along its band be averaged along the band alone: the
 * jets drag a pixel's neighbours a hundred pixels east or west of each other and barely north or south, and averaging
 * both ways alike wiped Jupiter's bands into a plain disc (AC-ProArt, 28 Nov).
 */
export function levelsOf(face: TrueTexture): Level[][] {
  const out: Level[][] = [];
  let base: Level = { width: face.width, height: face.height, data: Float32Array.from(face.data) };
  for (;;) {
    const row = [base];
    while (row[row.length - 1]!.width >= 8) row.push(halveLon(row[row.length - 1]!));
    out.push(row);
    if (base.height < 4) break;
    base = halveLat(base);
  }
  return out;
}

/** A level's colour at a place, bilinear, wrapped east-west and held at the poles. */
function sample(l: Level, lat: number, lon: number, out: Float32Array, k: number, wgt: number) {
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

/** The map's slopes at pixel (i, j) for one generation, in the face's pixels per picture pixel: east, then north. */
function slopes(read: Float32Array, width: number, height: number, i: number, j: number, gen: 0 | 1): [[number, number], [number, number]] {
  const at = (x: number, y: number): [number, number] => {
    const k = (y * width + ((x + width) % width)) * 4 + gen * 2;
    return [read[k]!, read[k + 1]!];
  };
  const d = (a: [number, number], b: [number, number], n: number): [number, number] => [
    ((wrap(b[1] - a[1]) / 360) * width) / n,
    (((b[0] - a[0]) / 180) * height) / n,
  ];
  return [d(at(i - 1, j), at(i + 1, j), 2), d(at(i, Math.max(j - 1, 0)), at(i, Math.min(j + 1, height - 1)), j === 0 || j === height - 1 ? 1 : 2)];
}

/**
 * How far the backward map stretches cloud at pixel (i, j) for one generation, in the face's pixels per picture pixel:
 * the larger spread of the map's slopes, from the read places of the pixels either side (one side at a pole row).
 */
export function stretchAt(read: Float32Array, width: number, height: number, i: number, j: number, gen: 0 | 1): number {
  const [ex, ny] = slopes(read, width, height, i, j, gen);
  // The larger singular value of [[ex₀, ny₀], [ex₁, ny₁]].
  const a = ex[0] * ex[0] + ex[1] * ex[1];
  const b = ex[0] * ny[0] + ex[1] * ny[1];
  const c = ny[0] * ny[0] + ny[1] * ny[1];
  return Math.sqrt((a + c) / 2 + Math.sqrt(((a - c) / 2) ** 2 + b * b));
}

/** How much of the face one picture pixel covers, east-west and north-south, in the face's pixels (at least one). */
export function footprintAt(read: Float32Array, width: number, height: number, i: number, j: number, gen: 0 | 1): [number, number] {
  const [ex, ny] = slopes(read, width, height, i, j, gen);
  return [Math.max(Math.abs(ex[0]) + Math.abs(ny[0]), 1), Math.max(Math.abs(ex[1]) + Math.abs(ny[1]), 1)];
}

/**
 * Today's stirred face from the unstirred one and where each pixel's two generations are read (RGBA per pixel:
 * younger lat, lon, older lat, lon, as stirred-gpu.ts `carryOnGpu` returns), weighed by the kernel's generations.
 */
export function composeStill(face: TrueTexture, t: CarryTable, read: Float32Array): TrueTexture {
  const { width, height } = face;
  const levels = levelsOf(face);
  const topNs = levels.length - 1;
  const acc = new Float32Array(width * height * 4);
  for (let j = 0; j < height; j++) {
    const lat = ((j + 0.5) / height) * 180 - 90;
    const g = generations(t, lat);
    for (let i = 0; i < width; i++) {
      const k = (j * width + i) * 4;
      for (const gen of [0, 1] as const) {
        const wgt = g[gen][1];
        if (wgt <= 0) continue;
        // The pixel's footprint on the face, east-west and north-south apart: a level per doubling each way, read
        // between the two either side of each.
        const [fe, fn] = footprintAt(read, width, height, i, j, gen);
        const ln = Math.min(Math.log2(fn), topNs);
        const n0 = Math.floor(ln);
        const nf = ln - n0;
        const la = read[k + gen * 2]!;
        const lo = read[k + gen * 2 + 1]!;
        for (const [n, wn] of [[n0, 1 - nf], [Math.min(n0 + 1, topNs), nf]] as const) {
          if (wn <= 0) continue;
          const row = levels[n]!;
          const le = Math.min(Math.log2(fe), row.length - 1);
          const e0 = Math.floor(le);
          const ef = le - e0;
          sample(row[e0]!, la, lo, acc, k, wgt * wn * (1 - ef));
          if (ef > 0) sample(row[Math.min(e0 + 1, row.length - 1)]!, la, lo, acc, k, wgt * wn * ef);
        }
      }
    }
  }
  const data = new Uint8Array(width * height * 4);
  for (let k = 0; k < data.length; k++) data[k] = Math.min(255, Math.max(0, Math.round(acc[k]!)));
  // Stirring moves the colours and makes none, so the disc's mean colour is the face's own.
  return { width, height, data, mean: [...face.mean] as [number, number, number] };
}
