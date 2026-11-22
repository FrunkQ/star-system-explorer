// The night sky drawn (Y5 part 1). The kernel gives two halves of one sky:
// `sky_from_here` lists the stars that reach the limit in a band, each with its
// direction, magnitude, temperature and address; `sky_glow` gives the light of
// every star it leaves out, as a HEALPix map (RING order) of surface brightness
// per band. Drawn together they add up to the whole sky without counting any
// star twice. This file reads both, projects them for a camera, draws them on a
// canvas, and puts the sky into words. Nothing here is astronomy: positions,
// brightness and colour are the kernel's; only the display (exposure and how a
// point of light spreads on a screen) is chosen here, and both are in the
// fudge register.

import { thermalColour } from "../../ui-contract/src/index.ts";
import { dirIndex } from "./timelapse.ts";

export type V3 = [number, number, number];

export interface SkyStar {
  address: string;
  /** Unit direction, galactic: x toward l = 0, z toward b = +90. */
  dir: V3;
  l_deg: number;
  b_deg: number;
  /** Magnitude in the sky's limit band (Vega). */
  mag: number;
  /** Vega magnitude in each band the kernel gave. */
  mags: Record<string, number>;
  teff_k: number;
  distance_pc: number;
  /** Years its light took to arrive: its distance in light years. */
  light_time_yr: number;
  luminosity_lsun: number;
  cls: number;
  double: boolean;
}

export interface SkyGlowBand {
  band: string;
  /** Per HEALPix RING pixel, Vega magnitudes per square arcsecond. */
  mag_arcsec2: Float32Array;
  /** The same as linear light, 10^(-0.4 μ), for blending. */
  flux: Float32Array;
  whole_sky_mag: number;
}

/** Pixel centres of a HEALPix map as unit vectors, with a direction lookup (built once per nside). */
const centres = new Map<number, { dirs: Float32Array; index: ReturnType<typeof dirIndex> }>();
function centresOf(nside: number) {
  let c = centres.get(nside);
  if (!c) {
    const n = 12 * nside * nside;
    const dirs = new Float32Array(n * 3);
    for (let p = 0; p < n; p++) {
      const { z, phi } = pix2angRing(nside, p);
      const r = Math.sqrt(Math.max(0, 1 - z * z));
      dirs.set([r * Math.cos(phi), r * Math.sin(phi), z], p * 3);
    }
    c = { dirs, index: dirIndex(dirs, n) };
    centres.set(nside, c);
  }
  return c;
}

/**
 * A map's value at a direction, blended from nearby pixel centres with a Gaussian of about 0.7 of a pixel, so the
 * pixels' diamond shapes do not show. The blend is of linear light (intensity), not of magnitudes.
 */
export function glowSample(nside: number, values: Float32Array, d: V3): number {
  const { dirs, index } = centresOf(nside);
  const { cell, grid } = index;
  // About 0.7 of a pixel spacing: wide enough that the pixel pattern does not show, inside the lookup's reach.
  const sig = (cell / 1.5) * 0.7;
  const inv = 1 / (2 * sig * sig);
  const i0 = Math.floor(d[0] / cell);
  const j0 = Math.floor(d[1] / cell);
  const k0 = Math.floor(d[2] / cell);
  let ws = 0;
  let v = 0;
  for (let i = i0 - 1; i <= i0 + 1; i++)
    for (let j = j0 - 1; j <= j0 + 1; j++)
      for (let k = k0 - 1; k <= k0 + 1; k++) {
        const b = grid.get((i + 512) * 1048576 + (j + 512) * 1024 + (k + 512));
        if (!b) continue;
        for (const p of b) {
          const dx = d[0] - dirs[p * 3]!;
          const dy = d[1] - dirs[p * 3 + 1]!;
          const dz = d[2] - dirs[p * 3 + 2]!;
          const g = Math.exp(-(dx * dx + dy * dy + dz * dz) * inv);
          if (g < 1e-4) continue;
          ws += g;
          v += g * values[p]!;
        }
      }
  return ws > 0 ? v / ws : values[glowPixel(nside, d)]!;
}

export interface Sky {
  /** The band the stars were chosen in (they reach limit_mag in it); the glow is everyone else, in every band. */
  band: string;
  limit_mag: number;
  stars: SkyStar[];
  truncated: boolean;
  counts: { mag: number; n: number }[];
  glow: { nside: number; bands: SkyGlowBand[] } | null;
}

type RawStar = { address: string; dir: number[]; l_deg: number; b_deg: number; mag: number; teff_k?: number; components?: { teff_k: number }[]; distance_pc: number; light_time_yr: number; luminosity_lsun: number; class: number; double?: boolean; magnitudes?: { band: string; m_vega: number }[] };

/** Read the two kernel replies into one sky. `glow` may be absent (the stars alone). */
export function readSky(sky: { band: string; limit_mag: number; stars: RawStar[]; truncated?: boolean; counts?: { mag: number; n: number }[] }, glow?: { nside: number; order: string; bands: { band: string; mag_arcsec2: number[]; whole_sky_mag: number }[] } | null): Sky {
  if (glow && glow.order !== "ring") throw new Error(`sky_glow: order ${glow.order}, expected ring`);
  const stars = sky.stars
    .map((s) => ({
      address: s.address,
      dir: [s.dir[0]!, s.dir[1]!, s.dir[2]!] as V3,
      l_deg: s.l_deg,
      b_deg: s.b_deg,
      mag: s.mag,
      mags: Object.fromEntries((s.magnitudes ?? []).map((m) => [m.band, m.m_vega])),
      teff_k: s.teff_k ?? s.components?.[0]?.teff_k ?? 5800,
      distance_pc: s.distance_pc,
      light_time_yr: s.light_time_yr,
      luminosity_lsun: s.luminosity_lsun,
      cls: s.class,
      double: !!s.double,
    }))
    .sort((a, b) => b.mag - a.mag);
  const g = glow
    ? {
        nside: glow.nside,
        bands: glow.bands.map((b) => {
          if (b.mag_arcsec2.length !== 12 * glow.nside * glow.nside) throw new Error(`sky_glow: ${b.band} has ${b.mag_arcsec2.length} pixels for nside ${glow.nside}`);
          return { band: b.band, mag_arcsec2: Float32Array.from(b.mag_arcsec2), flux: Float32Array.from(b.mag_arcsec2, (m) => Math.pow(10, -0.4 * m)), whole_sky_mag: b.whole_sky_mag };
        }),
      }
    : null;
  return { band: sky.band, limit_mag: sky.limit_mag, stars, truncated: !!sky.truncated, counts: sky.counts ?? [], glow: g };
}

// ── HEALPix, RING order (Górski et al. 2005, ApJ 622, 759, section 4) ─────────

/** The RING pixel holding a direction given as z = cos θ and φ (radians, 0 to 2π). */
export function ang2pixRing(nside: number, z: number, phi: number): number {
  const za = Math.abs(z);
  let tt = (phi / (Math.PI / 2)) % 4;
  if (tt < 0) tt += 4;
  const ncap = 2 * nside * (nside - 1);
  const npix = 12 * nside * nside;
  if (za <= 2 / 3) {
    const temp1 = nside * (0.5 + tt);
    const temp2 = nside * z * 0.75;
    const jp = Math.floor(temp1 - temp2);
    const jm = Math.floor(temp1 + temp2);
    const ir = nside + 1 + jp - jm;
    const kshift = 1 - (ir & 1);
    let ip = Math.floor((jp + jm - nside + kshift + 1) / 2);
    ip = ((ip % (4 * nside)) + 4 * nside) % (4 * nside);
    return ncap + (ir - 1) * 4 * nside + ip;
  }
  const tp = tt - Math.floor(tt);
  const tmp = nside * Math.sqrt(3 * (1 - za));
  const jp = Math.floor(tp * tmp);
  const jm = Math.floor((1 - tp) * tmp);
  const ir = jp + jm + 1;
  let ip = Math.floor(tt * ir);
  ip = ((ip % (4 * ir)) + 4 * ir) % (4 * ir);
  return z > 0 ? 2 * ir * (ir - 1) + ip : npix - 2 * ir * (ir + 1) + ip;
}

/** The centre of a RING pixel as z = cos θ and φ. */
export function pix2angRing(nside: number, p: number): { z: number; phi: number } {
  const npix = 12 * nside * nside;
  const ncap = 2 * nside * (nside - 1);
  if (p < ncap) {
    const iring = Math.floor((1 + Math.sqrt(1 + 2 * p)) / 2);
    const iphi = p + 1 - 2 * iring * (iring - 1);
    return { z: 1 - (iring * iring) / (3 * nside * nside), phi: ((iphi - 0.5) * Math.PI) / (2 * iring) };
  }
  if (p < npix - ncap) {
    const ip = p - ncap;
    const iring = Math.floor(ip / (4 * nside)) + nside;
    const iphi = (ip % (4 * nside)) + 1;
    const fodd = ((iring + nside) & 1) === 1 ? 1 : 0.5;
    return { z: ((2 * nside - iring) * 2) / (3 * nside), phi: ((iphi - fodd) * Math.PI) / (2 * nside) };
  }
  const ip = npix - p;
  const iring = Math.floor((1 + Math.sqrt(2 * ip - 1)) / 2);
  const iphi = 4 * iring + 1 - (ip - 2 * iring * (iring - 1));
  return { z: -1 + (iring * iring) / (3 * nside * nside), phi: ((iphi - 0.5) * Math.PI) / (2 * iring) };
}

/** The glow pixel a galactic direction falls in. */
export function glowPixel(nside: number, d: V3): number {
  let phi = Math.atan2(d[1], d[0]);
  if (phi < 0) phi += 2 * Math.PI;
  return ang2pixRing(nside, Math.max(-1, Math.min(1, d[2])), phi);
}

// ── Camera: stereographic, so a wide field keeps its shapes ─────────────────

export interface SkyCamera {
  /** Centre of view, galactic degrees, and the field across the picture. */
  l_deg: number;
  b_deg: number;
  field_deg: number;
  size_px: number;
  /** Basis: forward, right, up (galactic unit vectors). */
  f: V3;
  r: V3;
  u: V3;
  /** Pixels per unit of the stereographic plane. */
  k: number;
}

const unit = (l_deg: number, b_deg: number): V3 => {
  const l = (l_deg * Math.PI) / 180;
  const b = (b_deg * Math.PI) / 180;
  return [Math.cos(b) * Math.cos(l), Math.cos(b) * Math.sin(l), Math.sin(b)];
};

export function skyCamera(l_deg: number, b_deg: number, field_deg: number, size_px: number): SkyCamera {
  const f = unit(l_deg, b_deg);
  // Right is toward decreasing l on the sky as seen from inside (east to the left, as on a star chart).
  const north: V3 = Math.abs(f[2]) > 0.999 ? [1, 0, 0] : [0, 0, 1];
  let r: V3 = [f[1] * north[2] - f[2] * north[1], f[2] * north[0] - f[0] * north[2], f[0] * north[1] - f[1] * north[0]];
  const rl = Math.hypot(...r);
  r = [r[0] / rl, r[1] / rl, r[2] / rl];
  const u: V3 = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
  const half = (Math.min(359, field_deg) * Math.PI) / 360;
  const k = size_px / 2 / (2 * Math.tan(half / 2));
  return { l_deg, b_deg, field_deg, size_px, f, r, u, k };
}

/** Screen position of a direction, or null when it is outside the picture. */
export function projectSky(c: SkyCamera, d: V3, margin = 0): { x: number; y: number } | null {
  const cz = d[0] * c.f[0] + d[1] * c.f[1] + d[2] * c.f[2];
  if (cz <= -0.999) return null;
  const s = (2 * c.k) / (1 + cz);
  const x = c.size_px / 2 + s * (d[0] * c.r[0] + d[1] * c.r[1] + d[2] * c.r[2]);
  const y = c.size_px / 2 - s * (d[0] * c.u[0] + d[1] * c.u[1] + d[2] * c.u[2]);
  if (x < -margin || y < -margin || x > c.size_px + margin || y > c.size_px + margin) return null;
  return { x, y };
}

/** The direction under a screen point (the inverse stereographic projection). */
export function unprojectSky(c: SkyCamera, x: number, y: number): V3 {
  const X = (x - c.size_px / 2) / (2 * c.k);
  const Y = (c.size_px / 2 - y) / (2 * c.k);
  const q = X * X + Y * Y;
  const a = (2 * X) / (1 + q);
  const b = (2 * Y) / (1 + q);
  const w = (1 - q) / (1 + q);
  return [a * c.r[0] + b * c.u[0] + w * c.f[0], a * c.r[1] + b * c.u[1] + w * c.f[1], a * c.r[2] + b * c.u[2] + w * c.f[2]];
}

// ── Display: exposure and points ───────────────────────────────────────────

/** Surface brightness that shows as full white, mag per square arcsecond (display exposure, see docs/fudges.md). */
export const SKY_WHITE_MAG_ARCSEC2 = 18;

/**
 * The display exposure for a band: V uses SKY_WHITE_MAG_ARCSEC2; another band is shifted by how much brighter or
 * fainter the whole sky is in it, so each band is shown as a camera set for that band would show it.
 */
export function exposureFor(sky: Sky, band: string): number {
  const v = sky.glow?.bands.find((b) => b.band === "V")?.whole_sky_mag;
  const b = sky.glow?.bands.find((x) => x.band === band)?.whole_sky_mag;
  return v !== undefined && b !== undefined ? SKY_WHITE_MAG_ARCSEC2 + (b - v) : SKY_WHITE_MAG_ARCSEC2;
}

/** Linear display value (0 to 1, before sRGB encoding) of a surface brightness. */
export function glowLevel(mag_arcsec2: number, white = SKY_WHITE_MAG_ARCSEC2): number {
  return Math.min(1, Math.pow(10, -0.4 * (mag_arcsec2 - white)));
}

/**
 * Colour temperature from a B−V colour (Ballesteros 2012, EPL 97, 34008):
 * T = 4600 (1 / (0.92 (B−V) + 1.7) + 1 / (0.92 (B−V) + 0.62)).
 */
export function tempFromBV(bv: number): number {
  const x = Math.max(-0.4, Math.min(2.5, bv));
  return 4600 * (1 / (0.92 * x + 1.7) + 1 / (0.92 * x + 0.62));
}

const colourCache = new Map<number, [number, number, number]>();
/** Linear sRGB chromaticity (brightest channel 1) of a thermal source, cached per 100 K. */
export function starRgb(t_k: number): [number, number, number] {
  const key = Math.round(Math.max(1000, Math.min(40000, t_k)) / 100) * 100;
  let c = colourCache.get(key);
  if (!c) {
    const h = thermalColour(key);
    const dec = (i: number) => {
      const v = parseInt(h.slice(1 + 2 * i, 3 + 2 * i), 16) / 255;
      return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    };
    c = [dec(0), dec(1), dec(2)];
    colourCache.set(key, c);
  }
  return c;
}

/**
 * How a star spreads on screen: a Gaussian of `sigma_px` whose total light is the star's flux. Its peak, in units of
 * full white, makes a star at the sky's limit just visible (peak 2/255); brighter stars have higher peaks and so show
 * wider before they fade under one level. Returns the peak and the radius where it falls to that level.
 */
export function pointSpread(mag: number, limit_mag: number, sigma_px = 0.8): { peak: number; radius_px: number } {
  const peak = (2 / 255) * Math.pow(10, -0.4 * (mag - limit_mag));
  const r = peak > 1 / 255 ? sigma_px * Math.sqrt(2 * Math.log(peak * 255)) : 0;
  return { peak, radius_px: Math.max(0.7, r) };
}

/** Points a tier draws at most (night sky design ruling: about 20,000 on phones, 200,000 on desktops). */
export const SKY_POINTS_MAX: Record<string, number> = { "gpu-high": 200_000, "gpu-basic": 200_000, canvas: 20_000, text: 0 };

const enc = (v: number) => 255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(Math.max(0, v), 1 / 2.4) - 0.055);

/**
 * Draw the sky for a camera into an RGBA buffer: the glow per pixel (at `glowStep` pixel blocks, smoothed),
 * then each star as its spread of light, added, faintest first. Returns the stars drawn and their screen places
 * (for picking) and how many were left out by the point budget.
 */
export function drawSky(out: { width: number; height: number; data: Uint8ClampedArray }, sky: Sky, c: SkyCamera, opts: { maxPoints?: number; glowStep?: number; white?: number; band?: string } = {}): { drawn: { i: number; x: number; y: number; r: number }[]; leftOut: number } {
  const W = out.width;
  const H = out.height;
  const lin = new Float32Array(W * H * 3);
  const glowB = sky.glow?.bands.find((b) => b.band === "B");
  const band = opts.band ?? sky.band;
  const magOf = (st: SkyStar) => st.mags[band] ?? st.mag;
  // Display limit in the band shown: the faintest chosen star there is just visible.
  let limit = -Infinity;
  for (const st of sky.stars) limit = Math.max(limit, magOf(st));
  if (!Number.isFinite(limit)) limit = sky.limit_mag;
  const glowV = sky.glow?.bands.find((b) => b.band === band) ?? sky.glow?.bands.find((b) => b.band === "V");
  // Only V is the eye's own colour; other bands are drawn in grey (mapped), as their own exposure.
  const trueColour = band === "V";
  const white = opts.white ?? exposureFor(sky, band);
  if (sky.glow && glowV) {
    const step = Math.max(1, Math.round(opts.glowStep ?? 2));
    const n = sky.glow.nside;
    // Coarse grid of glow values, then bilinear to every pixel.
    const gw = Math.ceil(W / step) + 1;
    const gh = Math.ceil(H / step) + 1;
    const grid = new Float32Array(gw * gh * 3);
    for (let j = 0; j < gh; j++)
      for (let i = 0; i < gw; i++) {
        const d = unprojectSky(c, i * step, j * step);
        const fv = glowSample(n, glowV.flux, d);
        const level = glowLevel(-2.5 * Math.log10(Math.max(1e-30, fv)), white);
        const rgb = !trueColour ? [1, 1, 1] : glowB ? starRgb(tempFromBV(-2.5 * Math.log10(Math.max(1e-30, glowSample(n, glowB.flux, d)) / Math.max(1e-30, fv)))) : [1, 1, 1];
        // Chromaticity has its brightest channel at one; rescale so the luminance is the level.
        const Y = 0.2126 * rgb[0]! + 0.7152 * rgb[1]! + 0.0722 * rgb[2]!;
        for (let k = 0; k < 3; k++) grid[(j * gw + i) * 3 + k] = (rgb[k]! / Y) * level;
      }
    for (let y = 0; y < H; y++) {
      const gy = y / step;
      const j0 = Math.min(gh - 2, Math.floor(gy));
      const ty = gy - j0;
      for (let x = 0; x < W; x++) {
        const gx = x / step;
        const i0 = Math.min(gw - 2, Math.floor(gx));
        const tx = gx - i0;
        for (let k = 0; k < 3; k++) {
          const a = grid[(j0 * gw + i0) * 3 + k]!;
          const b = grid[(j0 * gw + i0 + 1) * 3 + k]!;
          const cc = grid[((j0 + 1) * gw + i0) * 3 + k]!;
          const dd = grid[((j0 + 1) * gw + i0 + 1) * 3 + k]!;
          lin[(y * W + x) * 3 + k] = (a * (1 - tx) + b * tx) * (1 - ty) + (cc * (1 - tx) + dd * tx) * ty;
        }
      }
    }
  }
  const max = opts.maxPoints ?? Infinity;
  // The brightest are kept when the budget is short; sky.stars is faintest first.
  const start = Math.max(0, sky.stars.length - max);
  const drawn: { i: number; x: number; y: number; r: number }[] = [];
  const sigma = 0.8;
  for (let i = start; i < sky.stars.length; i++) {
    const s = sky.stars[i]!;
    const p = projectSky(c, s.dir, 4);
    if (!p) continue;
    const { peak, radius_px } = pointSpread(magOf(s), limit, sigma);
    const rgb: [number, number, number] = trueColour ? starRgb(s.teff_k) : [1, 1, 1];
    const Y = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    const R = Math.ceil(radius_px + 1);
    const inv = 1 / (2 * sigma * sigma);
    for (let dy = -R; dy <= R; dy++) {
      const y = Math.round(p.y) + dy;
      if (y < 0 || y >= H) continue;
      for (let dx = -R; dx <= R; dx++) {
        const x = Math.round(p.x) + dx;
        if (x < 0 || x >= W) continue;
        const ex = x - p.x;
        const ey = y - p.y;
        const v = peak * Math.exp(-(ex * ex + ey * ey) * inv);
        if (v < 1 / 1024) continue;
        for (let k = 0; k < 3; k++) lin[(y * W + x) * 3 + k]! += (rgb[k]! / Y) * v;
      }
    }
    drawn.push({ i, x: p.x, y: p.y, r: radius_px });
  }
  for (let q = 0; q < W * H; q++) {
    // Over-white light stays white (a saturated point), keeping its hue only below full.
    const r = lin[q * 3]!;
    const g = lin[q * 3 + 1]!;
    const b = lin[q * 3 + 2]!;
    const m = Math.max(r, g, b, 1);
    const sat = m > 1 ? Math.min(1, (m - 1) / m) : 0;
    out.data[q * 4] = enc(r / m + sat * (1 - r / m));
    out.data[q * 4 + 1] = enc(g / m + sat * (1 - g / m));
    out.data[q * 4 + 2] = enc(b / m + sat * (1 - b / m));
    out.data[q * 4 + 3] = 255;
  }
  return { drawn, leftOut: start };
}

/** The star under a screen point (within a few pixels of its drawn place), brightest first. */
export function pickStar(sky: Sky, drawn: readonly { i: number; x: number; y: number; r: number }[], x: number, y: number, slop = 6): SkyStar | null {
  let best: { i: number; d: number; mag: number } | null = null;
  for (const p of drawn) {
    const d = Math.hypot(p.x - x, p.y - y);
    if (d > Math.max(slop, p.r)) continue;
    const mag = sky.stars[p.i]!.mag;
    if (!best || mag - d * 0.2 < best.mag - best.d * 0.2) best = { i: p.i, d, mag };
  }
  return best ? sky.stars[best.i]! : null;
}

const fmt = (x: number, d = 1) => x.toLocaleString("en-GB", { maximumFractionDigits: d });
// A star's light time in years is its distance in light years.
const ly = (yr: number) => (yr < 100 ? `${fmt(yr)} light years` : `${fmt(Math.round(yr), 0)} light years`);

/** The sky in words: how many stars, the brightest, and the glow. */
export function narrateSky(sky: Sky, leftOut = 0, band = sky.band): string {
  const n = sky.stars.length;
  const m = (s: SkyStar) => s.mags[band] ?? s.mag;
  const bright = [...sky.stars].sort((a, b) => m(a) - m(b)).slice(0, 3);
  const glowV = sky.glow?.bands.find((b) => b.band === band);
  const parts = [`${fmt(n, 0)} stars brighter than magnitude ${fmt(sky.limit_mag)} in ${sky.band}${sky.truncated ? " (the kernel's list stops there; fainter ones are in the glow)" : ""}`];
  if (band !== sky.band) parts.push(`shown in ${band}`);
  if (bright.length) parts.push(`brightest${band !== sky.band ? ` in ${band}` : ""}: ${bright.map((s) => `magnitude ${fmt(m(s))}, ${ly(s.light_time_yr)} away`).join("; ")}`);
  if (glowV) parts.push(`the rest of the galaxy's starlight, as a glow, adds up to magnitude ${fmt(glowV.whole_sky_mag)} in ${band} over the whole sky`);
  if (leftOut > 0) parts.push(`${fmt(leftOut, 0)} of the faintest are left to the glow on this device`);
  return `${parts.join("; ")}.`;
}
