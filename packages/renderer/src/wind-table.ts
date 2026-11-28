// The wind table the settle kit carries a picture along (FL2, SSE4-visual-flourish-design.md 4.4).
//
// A small grid over a face (equirectangular, row 0 at the south pole, as every face in the renderer), each cell four
// floats: the steady eastward turn of its row (the jets, radians of longitude a second), and the turning part as it
// stood when the held picture was made (storms and eddies: radians of longitude and of latitude a second, beside the
// jets'). The steady part is exact for any length of time; the turning part is run as two overlapping short loops of
// `loop_s` (Max and Becker 1995, Vlachos 2010; both unverified until 1 Dec 2026). Kernel numbers only: a use bakes its
// table from its own kernel read (a giant's from `want: "carry"`, FL3), never from constants here.

import { constant } from "../../ui-contract/src/index.ts";

/** RGBA per cell: jet east (rad/s), turning east (rad/s of longitude), turning north (rad/s), unused. */
export interface WindTable {
  width: number;
  height: number;
  data: Float32Array;
  /** The turning part's loop, clock seconds: the shorter of the jets' shear time and a radian of the fastest eddy's turn. */
  loop_s: number;
}

/** Latitude of row `j` of `h`, radians. */
export const rowLat = (j: number, h: number): number => (((j + 0.5) / h) * 180 - 90) * (Math.PI / 180);

/**
 * A table from three rules by place (latitude and longitude in radians): the row's jet, and the turning part. Cells
 * are sampled at their centres. `loop_s` is worked out from the table itself unless given.
 */
export function windTable(
  width: number,
  height: number,
  jet: (lat: number) => number,
  turn: (lat: number, lon: number) => [number, number] = () => [0, 0],
  loop_s?: number,
): WindTable {
  const data = new Float32Array(width * height * 4);
  for (let j = 0; j < height; j++) {
    const lat = rowLat(j, height);
    const w = jet(lat);
    for (let i = 0; i < width; i++) {
      const lon = ((i + 0.5) / width) * 2 * Math.PI - Math.PI;
      const [e, n] = turn(lat, lon);
      data.set([w, e, n, 0], (j * width + i) * 4);
    }
  }
  const t: WindTable = { width, height, data, loop_s: 0 };
  t.loop_s = loop_s ?? loopLength(t);
  return t;
}

/** Rows nearer a pole than this (cos φ under it, about 3°) do not set the loop. */
const POLE_COS = 0.05;

/** The longest loop allowed, clock seconds (a Julian year, from the sheet): a table with no shear and no eddies still needs a finite loop. */
export const LOOP_MAX_S = constant("yr_julian");

/**
 * The loop's length from the table (design 4.4): the shortest of the jets' shear time (one over the largest rate at
 * which neighbouring rows draw apart, in radians a second) and the time the fastest eddy takes to turn a radian (one
 * over half the largest vorticity of the turning part).
 */
export function loopLength(t: WindTable): number {
  const { width: w, height: h, data } = t;
  const dlat = Math.PI / h;
  const dlon = (2 * Math.PI) / w;
  let fastest = 0;
  for (let j = 0; j < h; j++) {
    const c = Math.max(Math.cos(rowLat(j, h)), 0.02);
    for (let i = 0; i < w; i++) {
      const k = (j * w + i) * 4;
      // Rows within 3° of a pole are left out: there a face's pixels close up to a point and the clamped turn rates
      // stand for no real speed.
      if (j + 1 < h && Math.min(c, Math.cos(rowLat(j + 1, h))) >= POLE_COS) {
        // Shear: how fast neighbouring rows draw apart, radians a second (the jets' turn rates differ by row; a turn
        // shared by every row, such as an air that super-rotates as one, moves nothing apart).
        const c1 = Math.max(Math.cos(rowLat(j + 1, h)), 0.02);
        const k1 = ((j + 1) * w + i) * 4;
        fastest = Math.max(fastest, (Math.abs(data[k1]! - data[k]!) / dlat) * Math.min(c, c1));
        // Vorticity of the turning part on the unit sphere, ζ = (∂v/∂λ − ∂(u cos φ)/∂φ) / cos φ; an eddy turns a
        // radian in 2/ζ.
        const ip = ((j * w + ((i + 1) % w)) * 4);
        const dvdx = (data[ip + 2]! - data[k + 2]!) / dlon;
        const dudy = (data[k1 + 1]! * c1 * c1 - data[k + 1]! * c * c) / dlat;
        const cm = Math.max(Math.cos(rowLat(j + 0.5, h)), 0.02);
        fastest = Math.max(fastest, Math.abs(dvdx - dudy) / cm / 2);
      }
    }
  }
  return fastest > 0 ? Math.min(1 / fastest, LOOP_MAX_S) : LOOP_MAX_S;
}

/** The table's four values at a face's texel place (column `x`, row `y`, in a `w` × `h` face's texels), bilinear. */
export function windAt(t: WindTable, x: number, y: number, w: number, h: number, out: Float64Array = new Float64Array(4)): Float64Array {
  const tx = ((x + 0.5) / w) * t.width - 0.5;
  const ty = Math.min(Math.max(((y + 0.5) / h) * t.height - 0.5, 0), t.height - 1);
  const x0 = Math.floor(tx);
  const y0 = Math.min(Math.floor(ty), t.height - 1);
  const fx = tx - x0;
  const fy = ty - y0;
  const xa = ((x0 % t.width) + t.width) % t.width;
  const xb = (xa + 1) % t.width;
  const yb = Math.min(y0 + 1, t.height - 1);
  for (let c = 0; c < 4; c++) {
    const v = (i: number, j: number) => t.data[(j * t.width + i) * 4 + c]!;
    out[c] = (v(xa, y0) * (1 - fx) + v(xb, y0) * fx) * (1 - fy) + (v(xa, yb) * (1 - fx) + v(xb, yb) * fx) * fy;
  }
  return out;
}
