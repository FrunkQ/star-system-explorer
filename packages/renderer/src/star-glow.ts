// A star's glow past its edge (SG1). Nothing glows round a star in empty space: the halo in every photograph and
// every glance at the Sun is light scattered inside the eye or the lens. So the glow is drawn as the eye's own
// scatter, the CIE general disability glare equation (Vos and van den Berg 1999, adopted as CIE 146:2002), applied
// to the star's own disc and brightness. Its size on the screen is an angle, not a distance: it is the same few
// degrees whether the star is near or far, as in a real photograph, and it brightens when the star does (a flare).
//
// Exposure: True is exposed so a white surface lit by the star reads white (the worlds' own daylight). Under that
// exposure the glare relative to white is π · (F_seen / F_exposed) · PSF(θ), whatever the star: a white surface at
// the observer's distance gets the same flux as the eye. `exposure` is F_seen / F_exposed; 1 is that daylight
// exposure, and a view that knows the real flux at its eye (a world's sky, a ship far out) can pass the ratio.
import { thermalColour } from "../../ui-contract/src/index.ts";
import { srgb255 } from "./srgb.ts";
import { limbDarkening } from "./star.ts";

export const GLOW_REFERENCES = [
  { text: "Vos, J. J. and van den Berg, T. J. T. P. (1999). Report on disability glare. CIE Collection 135/1, 1-9 (the general glare equation L/E = 10/θ³ + (5/θ² + 0.1 p/θ)(1 + (A/62.5)⁴) + 0.0025 p, θ in degrees from 0.1 to 100).", url: "https://cie.co.at/publications" },
  { text: "CIE 146:2002 / CIE 147:2002. CIE equations for disability glare; glare from small, large and complex sources (the same equation adopted by the CIE).", url: "https://cie.co.at/publications" },
  { text: "Spencer, G., Shirley, P., Zimmerman, K. and Greenberg, D. P. (1995). Physically-based glare effects for digital images. SIGGRAPH 95, 325-334 (drawing the eye's scatter round bright sources).", doi: "10.1145/218380.218466" },
];

/** The eye the glare is drawn for: age 25 (the equation's young eye) and pigmentation 0.5 (brown eyes; 0 very dark, 1 blue). */
export const GLARE_EYE = { age_y: 25, pigment: 0.5 };
/** The glare is drawn out to this angle from the star's centre, degrees; beyond it the falling part is under a hundredth of white. */
export const GLOW_REACH_DEG = 20;
/** Smallest angle the equation holds for, degrees (CIE: 0.1 to 100). */
const THETA_MIN_DEG = 0.1;

/**
 * The CIE general disability glare equation: veiling luminance over the illuminance at the eye, per steradian, at
 * `theta_deg` from a point source. The constant term 0.0025 p is the eye's even veil over the whole field; it is left
 * out (`withVeil` false) when drawing a halo, because it lifts the whole picture rather than ringing the star.
 */
export function glarePsf(theta_deg: number, eye = GLARE_EYE, withVeil = true): number {
  const t = Math.max(THETA_MIN_DEG, theta_deg);
  const age = 1 + (eye.age_y / 62.5) ** 4;
  return 10 / t ** 3 + (5 / (t * t) + (0.1 * eye.pigment) / t) * age + (withVeil ? 0.0025 * eye.pigment : 0);
}

/**
 * The glare round a disc of angular radius `disc_deg`, as luminance over a lit white surface's, at `n` angles from
 * 0 to `reach_deg` from the centre. The disc's light is spread by its limb darkening and the PSF summed over it, so a
 * large disc's glow hugs its limb and a point's falls as 1/θ³. Multiply by `exposure` (and a flare's extra light).
 */
export function glareProfile(disc_deg: number, n = 128, reach_deg = GLOW_REACH_DEG, eye = GLARE_EYE): Float64Array {
  const out = new Float64Array(n);
  // The disc in rings of equal width, each sampled round; weights are each patch's share of the disc's light.
  const rings = 16;
  const round = 32;
  const pts: { x: number; y: number; w: number }[] = [];
  let total = 0;
  for (let i = 0; i < rings; i++) {
    const r = (i + 0.5) / rings;
    const w = limbDarkening(Math.sqrt(1 - r * r)) * r;
    for (let j = 0; j < round; j++) {
      const a = ((j + 0.5) / round) * 2 * Math.PI;
      pts.push({ x: r * Math.cos(a) * disc_deg, y: r * Math.sin(a) * disc_deg, w });
      total += w;
    }
  }
  for (let k = 0; k < n; k++) {
    const th = (k / (n - 1)) * reach_deg;
    let s = 0;
    for (const p of pts) s += p.w * glarePsf(Math.hypot(th - p.x, p.y), eye, false);
    out[k] = (Math.PI * s) / total;
  }
  return out;
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

export interface GlowDraw {
  teff_k: number;
  /** The drawn disc's angular radius as the camera sees it, degrees. */
  disc_deg: number;
  /** Flux at the eye over the exposure's (1: the worlds' own daylight). */
  exposure?: number;
  /** The star's light now over its quiet light (1 + a flare's share). */
  boost?: number;
  /** A precomputed `glareProfile` for this disc and reach (it depends on nothing else). */
  profile?: Float64Array;
  /** The picture's half-width as an angle from the centre, degrees (`haloReachDeg`); GLOW_REACH_DEG unless given. */
  reach_deg?: number;
}

/**
 * How far the halo picture reaches from the star's centre, degrees: the glare's own reach, or, for a star seen large
 * (the close-up), far enough past its limb to hold what stands over it, two and a half times its outline on the
 * picture's plane, never past 70 degrees, where a flat picture facing the camera stretches too far to hold a shape.
 */
export function haloReachDeg(disc_deg: number): number {
  const d = (Math.min(80, Math.max(0, disc_deg)) * Math.PI) / 180;
  return Math.min(70, Math.max(GLOW_REACH_DEG, (Math.atan(2.5 * Math.tan(d)) * 180) / Math.PI));
}

/**
 * The glow as a square picture reaching `reach_deg` (GLOW_REACH_DEG unless given) from the centre: a flat picture at
 * the star's distance facing the camera, so a point r of its half-width out lies atan(r tan(reach)) from the centre.
 * Linear light over white is squeezed by L/(1 + L) so the bright core saturates to the star's colour and the falling
 * skirt keeps its shape; inside the disc it is clear, because the disc is drawn at its own exposure (the face).
 * Alpha is the light itself: the picture is added to the scene.
 */
export function drawGlow(out: { width: number; data: Uint8ClampedArray }, g: GlowDraw): void {
  const n = out.width;
  const reach = g.reach_deg ?? GLOW_REACH_DEG;
  const k = (g.exposure ?? 1) * (g.boost ?? 1);
  // The glow changes only with the zoom and the star's light, not with each picture: a picture already drawn for
  // the same numbers is copied (the profile, when given, is a function of the disc and reach in the key).
  const key = `${n}|${g.teff_k}|${g.disc_deg}|${reach}|${k}`;
  const kept = glows.get(key);
  if (kept && out.data.length === kept.length) {
    out.data.set(kept);
    return;
  }
  paintGlow(out, g, reach, k);
  if (glows.size >= GLOWS_KEPT) glows.delete(glows.keys().next().value!);
  glows.set(key, out.data.slice());
}

const GLOWS_KEPT = 4;
const glows = new Map<string, Uint8ClampedArray>();

function paintGlow(out: { width: number; data: Uint8ClampedArray }, g: GlowDraw, reach: number, k: number): void {
  const n = out.width;
  const d = out.data;
  const prof = g.profile ?? glareProfile(g.disc_deg, 128, reach);
  const m = prof.length;
  const base = (thermalColour(g.teff_k).match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"]).map((h) => decode(parseInt(h, 16) / 255));
  const c = n / 2;
  const rad = Math.PI / 180;
  const tanReach = Math.tan(reach * rad);
  const discR = Math.tan(g.disc_deg * rad) / tanReach;
  for (let py = 0; py < n; py++)
    for (let px = 0; px < n; px++) {
      const o = (py * n + px) * 4;
      const r = Math.hypot(px + 0.5 - c, py + 0.5 - c) / c;
      // Clear well inside the disc (drawn by the face at its own exposure); across the limb the glare comes up from
      // nothing, so the face's darkened edge and any lump of the limb blend into it without a seam.
      const inner = 0.7 * discR;
      if (r >= 1 || r <= inner) {
        d[o] = d[o + 1] = d[o + 2] = d[o + 3] = 0;
        continue;
      }
      const into = r >= discR ? 1 : (r - inner) / (discR - inner);
      // The profile is sampled evenly in angle out to the reach; this pixel's angle from the centre, as a share of it.
      const f = (Math.atan(r * tanReach) / (reach * rad)) * (m - 1);
      const i = Math.min(m - 2, Math.floor(f));
      const L = k * (prof[i]! + (f - i) * (prof[i + 1]! - prof[i]!));
      // Fade the outer part (beyond 40% of the reach, where the glare is already under a fifth of white) smoothly to
      // nothing, so the picture never shows its edge as a rim.
      const u = Math.max(0, Math.min(1, (1 - r) / 0.6));
      const v = (L / (1 + L)) * u * u * (3 - 2 * u) * into * into;
      d[o] = srgb255(base[0]! * v);
      d[o + 1] = srgb255(base[1]! * v);
      d[o + 2] = srgb255(base[2]! * v);
      d[o + 3] = srgb255(v);
    }
}

/** The angle at which the glow falls to a share of white (for words and checks), degrees. */
export function glowReach(profile: Float64Array, share = 0.5, reach_deg = GLOW_REACH_DEG, exposure = 1): number {
  for (let k = profile.length - 1; k >= 0; k--) if (exposure * profile[k]! >= share) return (k / (profile.length - 1)) * reach_deg;
  return 0;
}

/** The glow in words. */
export function narrateGlow(profile: Float64Array, exposure = 1, reach_deg = GLOW_REACH_DEG): string {
  const half = glowReach(profile, 1, reach_deg, exposure);
  return `The glow round the star is light scattered in the eye (the CIE glare equation), not anything in space: it outshines a sunlit surface out to about ${half.toFixed(1)}° from the star and fades over ${Math.round(reach_deg)}°.`;
}
