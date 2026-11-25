// The shared finish's glare (VP1), as pure numbers: the eye's scatter (the CIE general disability glare equation,
// star-glow.ts `glarePsf`) split into a ladder of blurs. A picture convolved with the eye's point spread function is
// the picture as an eye would see it; the ladder approximates that convolution with Gaussians whose widths double,
// each carrying the share of the light the equation scatters into its ring of angles. Light inside the smallest ring
// (the equation's 0.1° floor, or one pixel if that is wider) stays where it is.
import { GLARE_EYE, glarePsf, GLOW_REACH_DEG } from "./star-glow.ts";

/** Smallest angle the CIE equation holds for, degrees. */
export const GLARE_FLOOR_DEG = 0.1;

export interface GlareRung {
  /** The Gaussian's sigma, in pixels of the full picture. */
  sigma_px: number;
  /** Its share of the picture's light (0..1). */
  weight: number;
}

const RAD = Math.PI / 180;

/** The share of a point's light the eye scatters between two angles from it (degrees), the even veil left out. */
export function glareShare(from_deg: number, to_deg: number, eye = GLARE_EYE): number {
  if (!(to_deg > from_deg)) return 0;
  // ∫ PSF(θ) dΩ with dΩ = 2π θ dθ in radians, by Simpson's rule on ln θ (the integrand falls steeply).
  const n = 64;
  const a = Math.log(from_deg);
  const b = Math.log(to_deg);
  const h = (b - a) / n;
  let s = 0;
  for (let i = 0; i <= n; i++) {
    const t = Math.exp(a + i * h);
    const f = glarePsf(t, eye, false) * 2 * Math.PI * t * t * RAD * RAD;
    s += f * (i === 0 || i === n ? 1 : i % 2 ? 4 : 2);
  }
  return (s * h) / 3;
}

/**
 * The ladder of blurs for a picture whose pixels are `pixel_deg` across: rung k has sigma 2^k pixels (from the first
 * that reaches past the floor) and carries the share scattered between its inner and outer edge (about two sigmas),
 * up to `reach_deg`. At most `rungs` rungs; the last one takes everything out to the reach.
 */
export function glareLadder(pixel_deg: number, rungs = 6, reach_deg = GLOW_REACH_DEG, eye = GLARE_EYE): GlareRung[] {
  const out: GlareRung[] = [];
  if (!(pixel_deg > 0)) return out;
  let inner = Math.max(GLARE_FLOOR_DEG, pixel_deg);
  let sigma = Math.max(1, Math.pow(2, Math.ceil(Math.log2(inner / (2 * pixel_deg)))));
  for (let k = 0; k < rungs && inner < reach_deg; k++) {
    const last = k === rungs - 1;
    const outer = last ? reach_deg : Math.min(reach_deg, 2 * sigma * pixel_deg);
    if (outer > inner) out.push({ sigma_px: sigma, weight: glareShare(inner, outer, eye) });
    inner = Math.max(inner, outer);
    sigma *= 2;
  }
  return out;
}

/** The share of light the ladder moves out of place (what the sharp picture gives up to the glare). */
export function glareTotal(l: readonly GlareRung[]): number {
  return l.reduce((s, r) => s + r.weight, 0);
}

/** The blur's own kernel, in samples each side (GaussianBlurNode: true sigma is about a third of the kernel). */
export const KERNEL = 8;
/** Each rung is blurred at a resolution where its sigma is about this many of that picture's pixels. */
const LOW_SIGMA_PX = 3;

/** The resolution scale and sample spacing at which to draw a blur of `sigma_px` full-size pixels. */
export function rungScale(sigma_px: number): { scale: number; step: number } {
  const scale = Math.min(1, Math.max(1 / 64, LOW_SIGMA_PX / sigma_px));
  // GaussianBlurNode's sigma for a kernel of K samples a side is about (K + 1) / 3 samples, each `step` apart.
  const step = (sigma_px * scale * 3) / (KERNEL + 1);
  return { scale, step };
}
