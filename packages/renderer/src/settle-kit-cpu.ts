// The settle kit's processor twins (FL2, SSE4-visual-flourish-design.md 4.4): Carry, Exposure and Fade worked out on
// the processor, pixel for pixel as settle-kit.ts does them on the GPU, which the tests hold to these. They also serve a
// device with no GPU path: there the design allows Held and the settle only, so only `fadeCpu` is used in a view.
//
// A picture is RGBA floats, a face `w` × `h`, equirectangular, row 0 at the south pole. All places are worked in the
// face's texels (column, row), so a carry of zero seconds reads each pixel at its own centre and returns it untouched.

import { windAt, type WindTable } from "./wind-table.ts";

const TAU = 2 * Math.PI;
/** Taps of the long exposure below its limit (design 4.4: up to eight reads). */
export const EXPOSURE_TAPS = 8;

/** A picture's colour at a texel place (column `x`, row `y`), bilinear, wrapped east-west and held at the poles. */
export function sampleAt(src: Float32Array, w: number, h: number, x: number, y: number, out: Float64Array, k: number, wgt: number): void {
  const fy = Math.min(Math.max(y, 0), h - 1);
  const x0 = Math.floor(x);
  // The top row reads itself (no neighbour past it), so a place on a texel's centre is that texel, exactly.
  const y0 = Math.min(Math.floor(fy), h - 1);
  const tx = x - x0;
  const ty = fy - y0;
  const xa = ((x0 % w) + w) % w;
  const xb = (xa + 1) % w;
  const yb = Math.min(y0 + 1, h - 1);
  for (let c = 0; c < 4; c++) {
    const v = (i: number, j: number) => src[(j * w + i) * 4 + c]!;
    out[k + c]! += wgt * ((v(xa, y0) * (1 - tx) + v(xb, y0) * tx) * (1 - ty) + (v(xa, yb) * (1 - tx) + v(xb, yb) * tx) * ty);
  }
}

/** The two loops' phases at `dt_s` (design 4.4): each one's offset from its middle, seconds, and its weight. */
export function loopPhases(dt_s: number, loop_s: number): [[number, number], [number, number]] {
  const L = loop_s;
  const phase = (k: number): [number, number] => {
    const t = dt_s + (k * L) / 2;
    const tau = t - L * Math.floor(t / L);
    // The offset runs from −L/2 to L/2 and the weight is zero where the loop starts again, so nothing jumps.
    return [tau - L / 2, 1 - Math.abs((2 * tau) / L - 1)];
  };
  return [phase(0), phase(1)];
}

/**
 * Carry: the held picture moved on by `dt_s` clock seconds (negative runs it back). Rows slide at their jets' turn,
 * exact for any length of time; the turning part is two overlapping loops of the table's `loop_s`, blended. Zero
 * seconds returns the picture untouched.
 */
export function carryCpu(src: Float32Array, w: number, h: number, t: WindTable, dt_s: number): Float32Array {
  const out = new Float32Array(src.length);
  const acc = new Float64Array(4);
  const wv = new Float64Array(4);
  const px = w / TAU;
  const py = h / Math.PI;
  const [[s0, w0], [s1, w1]] = loopPhases(dt_s, t.loop_s);
  for (let j = 0; j < h; j++) {
    for (let i = 0; i < w; i++) {
      // The jets: exactly where this row's cloud was dt_s ago.
      windAt(t, i, j, w, h, wv);
      const x0 = i - wv[0]! * dt_s * px;
      // The turning part as it stood there, in the held picture's own places.
      windAt(t, x0, j, w, h, wv);
      acc.fill(0);
      for (const [s, wt] of [[s0, w0], [s1, w1]] as const) {
        if (wt <= 0) continue;
        sampleAt(src, w, h, x0 - wv[1]! * s * px, j - wv[2]! * s * py, acc, 0, wt);
      }
      out.set(acc, (j * w + i) * 4);
    }
  }
  return out;
}

/** Each row's mean colour, RGBA (the long exposure's limit: one column worked out once per picture). */
export function rowMeans(src: Float32Array, w: number, h: number): Float32Array {
  const out = new Float32Array(h * 4);
  for (let j = 0; j < h; j++)
    for (let c = 0; c < 4; c++) {
      let s = 0;
      for (let i = 0; i < w; i++) s += src[(j * w + i) * 4 + c]!;
      out[j * 4 + c] = s / w;
    }
  return out;
}

/**
 * How far a texel's exposure has gone toward its row's mean: none until the cloud moves half a turn in one picture's
 * `span_s`, all of it at a whole turn. The rate is the jet's and the turning part's eastward together.
 */
export function meanShare(east_rad_s: number, span_s: number): number {
  const turns = (Math.abs(east_rad_s) * Math.abs(span_s)) / TAU;
  return Math.min(Math.max(2 * turns - 1, 0), 1);
}

/**
 * Exposure: the picture as a camera with its shutter open for `span_s` would see it, the time one picture stands for.
 * Up to eight reads back along the winds at the pixel (the storms and eddies over at most half a loop); past half a turn in a picture it blends to the row's mean,
 * which it is at a whole turn.
 */
export function exposureCpu(src: Float32Array, w: number, h: number, t: WindTable, span_s: number, means: Float32Array = rowMeans(src, w, h)): Float32Array {
  const out = new Float32Array(src.length);
  const acc = new Float64Array(4);
  const wv = new Float64Array(4);
  const px = w / TAU;
  const py = h / Math.PI;
  for (let j = 0; j < h; j++) {
    for (let i = 0; i < w; i++) {
      windAt(t, i, j, w, h, wv);
      const m = meanShare(wv[0]! + wv[1]!, span_s);
      acc.fill(0);
      if (m < 1) for (let k = 0; k < EXPOSURE_TAPS; k++) {
        const tk = (span_s * (k + 0.5)) / EXPOSURE_TAPS;
        // The jets for the whole stretch; the storms and eddies for no more than half a loop, the most the Carry
        // follows them (held straight for longer, they would shift whole bands north and south).
        const tt = Math.sign(tk) * Math.min(Math.abs(tk), t.loop_s / 2);
        sampleAt(src, w, h, i - (wv[0]! * tk + wv[1]! * tt) * px, j - wv[2]! * tt * py, acc, 0, (1 - m) / EXPOSURE_TAPS);
      }
      for (let c = 0; c < 4; c++) out[(j * w + i) * 4 + c] = acc[c]! + m * means[j * 4 + c]!;
    }
  }
  return out;
}

/** Fade: two pictures already brought to the same date, one weight on the second (the settle and the seam). */
export function fadeCpu(a: Float32Array, b: Float32Array, weight: number): Float32Array {
  const out = new Float32Array(a.length);
  const k = Math.min(Math.max(weight, 0), 1);
  for (let i = 0; i < a.length; i++) out[i] = a[i]! * (1 - k) + b[i]! * k;
  return out;
}
