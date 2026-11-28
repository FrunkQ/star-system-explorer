// The sky behind a black hole, as the light-path stage looks it up (XO4, design 3.1 item 1): the kernel's stars
// (`sky_from_here`) laid into two pictures in the camera's frame, a fine patch round the hole's direction and the whole
// sky coarsely, each star's light spread over its texel as radiance. Lensing keeps surface brightness, so the shader
// reading radiance through the bent ray gives the smeared arcs and the Einstein ring their true brightness, with no
// rule of its own. Units as VP2 (stars-behind.ts): flux as a share of the Sun's at 1 au, so π · share / Ω is 1 for a
// star filling Ω as brightly as a white surface in the Sun's light.
import { constant } from "../../ui-contract/src/index.ts";
import { starTint } from "./stars-behind.ts";
import type { HoleSky } from "./gpu-light-path.ts";

export interface HoleSkyStar {
  /** Galactic unit direction. */
  dir: [number, number, number];
  luminosity_lsun: number;
  distance_pc: number;
  teff_k: number;
}

type V3 = [number, number, number];
const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (a: V3): V3 => {
  const l = Math.hypot(...a) || 1;
  return [a[0] / l, a[1] / l, a[2] / l];
};

/** A galactic direction from longitude and latitude, degrees. */
export function galactic(l_deg: number, b_deg: number): V3 {
  const l = (l_deg * Math.PI) / 180;
  const b = (b_deg * Math.PI) / 180;
  return [Math.cos(b) * Math.cos(l), Math.cos(b) * Math.sin(l), Math.sin(b)];
}

/**
 * The camera's frame looking at `toward` (galactic), galactic north up: right, up, and back (the camera looks down
 * −back), as the shader's frame (x right, y up, −z forward).
 */
export function skyFrame(toward: V3): { right: V3; up: V3; back: V3 } {
  const f = norm(toward);
  const n: V3 = Math.abs(f[2]) > 0.99 ? [1, 0, 0] : [0, 0, 1];
  const right = norm(cross(f, n));
  const up = cross(right, f);
  return { right, up, back: [-f[0], -f[1], -f[2]] };
}

/** Lay `s` (per channel) times `k` at fractional texel (fx, fy) over the 3 × 3 texels round it, Gaussian weights summing to 1. */
function splat(out: Float32Array, w: number, h: number, fx: number, fy: number, s: V3, k: number, wrap: boolean): void {
  const ci = Math.round(fx);
  const cj = Math.round(fy);
  const wts: [number, number, number][] = [];
  let sum = 0;
  for (let dj = -1; dj <= 1; dj++)
    for (let di = -1; di <= 1; di++) {
      const g = Math.exp(-((ci + di - fx) ** 2 + (cj + dj - fy) ** 2) / (2 * 0.6 * 0.6));
      wts.push([ci + di, cj + dj, g]);
      sum += g;
    }
  for (const [i0, j, g] of wts) {
    if (j < 0 || j >= h) continue;
    const i = wrap ? ((i0 % w) + w) % w : i0;
    if (i < 0 || i >= w) continue;
    const at = (j * w + i) * 4;
    for (let c = 0; c < 3; c++) out[at + c] = out[at + c]! + (s[c]! * k * g) / sum;
    out[at + 3] = 1;
  }
}

/** Each star's flux as a share of the Sun's at 1 au, per channel by its colour (as VP2's `starsBehind`). */
function shares(stars: readonly HoleSkyStar[]): { d: V3; s: V3 }[] {
  const auPerPc = constant("pc") / constant("au");
  return stars.map((x) => {
    const dd = x.distance_pc * auPerPc;
    const f = dd > 0 ? x.luminosity_lsun / (dd * dd) : 0;
    const c = starTint(x.teff_k);
    return { d: x.dir, s: [f * c[0], f * c[1], f * c[2]] };
  });
}

/**
 * The two pictures: a patch `size` texels square spanning ±`half` (tangent of the angle off the hole's direction),
 * and the whole sky `wholeW` by `wholeW / 2`. `fine` and `all` are star lists for each (a deep cone round the hole and
 * the bright whole sky); a star in both is laid into each once. Also the gain that fills the `n`th brightest star in
 * the patch to white at its own texel (a long exposure, VP2's rule).
 */
export function holeSky(key: string, toward: V3, fine: readonly HoleSkyStar[], all: readonly HoleSkyStar[], o: { size: number; half: number; wholeW: number; n?: number }): { sky: HoleSky; gain: number; stars: number } {
  const fr = skyFrame(toward);
  const n = o.size;
  const patch = new Float32Array(n * n * 4);
  const texel = (2 * o.half) / n;
  const omegaPatch = texel * texel;
  const lum: number[] = [];
  for (const { d, s } of shares(fine)) {
    const z = -dot(d, fr.back);
    if (z <= 0) continue;
    const x = dot(d, fr.right) / z;
    const y = dot(d, fr.up) / z;
    if (Math.abs(x) >= o.half || Math.abs(y) >= o.half) continue;
    // Spread over the texels round it (a small Gaussian, its weights summing to one), so a star squeezed near the
    // shadow still lands on a pixel; its light is kept. Radiance is light over the texels' solid angle (the gnomonic
    // texel's cos³ left out).
    const fx = ((x / o.half + 1) / 2) * n - 0.5;
    const fy = ((y / o.half + 1) / 2) * n - 0.5;
    splat(patch, n, n, fx, fy, s, Math.PI / omegaPatch, false);
    lum.push((s[0] + s[1] + s[2]) / 3);
  }
  const W = o.wholeW;
  const H = Math.max(1, Math.round(W / 2));
  const whole = new Float32Array(W * H * 4);
  for (const { d, s } of shares(all)) {
    const x = dot(d, fr.right);
    const y = dot(d, fr.up);
    const z = -dot(d, fr.back);
    const lon = Math.atan2(x, z);
    const lat = Math.asin(Math.max(-1, Math.min(1, y)));
    const omega = ((2 * Math.PI) / W) * (Math.PI / H) * Math.max(1e-3, Math.cos(lat));
    splat(whole, W, H, (lon / (2 * Math.PI) + 0.5) * W - 0.5, (lat / Math.PI + 0.5) * H - 0.5, s, Math.PI / omega, true);
  }
  lum.sort((a, b) => b - a);
  const nth = lum[Math.min(o.n ?? 30, lum.length) - 1] ?? 0;
  const gain = nth > 0 ? Math.max(1, omegaPatch / (Math.PI * nth)) : 1;
  return { sky: { key, patch: { data: patch, size: n, half: o.half }, whole: { data: whole, width: W, height: H } }, gain, stars: fine.length };
}
