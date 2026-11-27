// Galaxy type pictures (grand zoom GZ5 part 2, design section 7.3, step 2): a small picture of a galaxy made by rule
// from the kernel's own facts, for galaxies whose full chart has not been read. The broad type (the kernel's
// `classify` of the galaxy's state: dwarf, irregular, disc, barred, lenticular, elliptical) picks which parts it has;
// its half-light radius sets their size, its axis ratio and position angle tilt and turn it as seen from home, its
// B-V colour colours it, and its star formation lights blue knots. Two galaxies of one type differ by their own
// numbers and by a pattern keyed on their address (arm phase, knots, lumps).
//
// Light profiles are Sersic laws (Sersic 1963, Boletin de la Asociacion Argentina de Astronomia 6, 41),
// I(r) = I_e exp(-b_n ((r / R_e)^(1/n) - 1)), b_n from Ciotti and Bertin 1999 (A&A 352, 447, arXiv:astro-ph/9911078):
// n = 4 for a bulge or an elliptical (de Vaucouleurs), n = 1 for a disc (exponential). The shares of
// light between bulge and disc, the bar, the arm pitch and contrast are stand-ins (fudge GZ-F14); the young stars'
// share of the light is the True galaxy's own (`GALAXY_LIGHT`), so a type picture and a charted galaxy agree on how
// blue a galaxy of a given star formation looks.
import { cloudNoise, GALAXY_LIGHT } from "./galaxy-true.ts";
import { starRgb, tempFromBV } from "./sky.ts";

/** What a type picture is drawn from: the kernel's `galaxies_seen` facts, cut to what is needed. */
export interface GalaxyTypeFacts {
  address: string;
  /** dwarf, irregular, disc, barred, lenticular or elliptical; anything else is drawn as a dwarf. */
  morphology: string;
  half_light_kpc: number;
  axis_ratio: number;
  position_angle_deg: number;
  /** B-V colour (Vega), or null. */
  b_v: number | null;
  stellar_mass_msun: number;
  sfr_msun_yr: number;
}

/**
 * The parts each type is drawn with (fudge GZ-F14, stand-ins): bulge share of the light, the bulge's half-light
 * radius over the galaxy's, arms (count, pitch in degrees, contrast), a bar (half-length over the galaxy's half-light
 * radius), and how lumpy the light is (0 smooth, 1 fully broken).
 */
export const GALAXY_TYPES: Record<string, { bulge: number; bulge_re: number; arms: number; pitch_deg: number; arm_contrast: number; bar: number; lumps: number }> = {
  elliptical: { bulge: 1, bulge_re: 1, arms: 0, pitch_deg: 0, arm_contrast: 0, bar: 0, lumps: 0 },
  lenticular: { bulge: 0.5, bulge_re: 0.35, arms: 0, pitch_deg: 0, arm_contrast: 0, bar: 0, lumps: 0 },
  disc: { bulge: 0.15, bulge_re: 0.2, arms: 2, pitch_deg: 20, arm_contrast: 0.7, bar: 0, lumps: 0.15 },
  barred: { bulge: 0.2, bulge_re: 0.2, arms: 2, pitch_deg: 18, arm_contrast: 0.7, bar: 0.6, lumps: 0.15 },
  irregular: { bulge: 0, bulge_re: 0.2, arms: 0, pitch_deg: 0, arm_contrast: 0, bar: 0, lumps: 0.7 },
  dwarf: { bulge: 0, bulge_re: 0.2, arms: 0, pitch_deg: 0, arm_contrast: 0, bar: 0, lumps: 0.25 },
};

/** b_n of a Sersic law, so R_e holds half the light (Ciotti and Bertin 1999, arXiv:astro-ph/9911078, their asymptotic series to 1/n). */
export function sersicB(n: number): number {
  return 2 * n - 1 / 3 + 4 / (405 * n) + 46 / (25515 * n * n);
}

/** A Sersic law's whole light over I_e R_e^2: 2 pi n e^b b^(-2n) Gamma(2n), for n = 1 and n = 4 only. */
function sersicTotal(n: 1 | 4): number {
  const b = sersicB(n);
  const gamma = n === 1 ? 1 : 5040; // Gamma(2) and Gamma(8)
  return 2 * Math.PI * n * Math.exp(b) * Math.pow(b, -2 * n) * gamma;
}

const seedOf = (key: string) => {
  let v = 2166136261;
  for (const ch of key) v = Math.imul(v ^ ch.charCodeAt(0), 16777619);
  return v >>> 0;
};

/** A colour temperature from the kernel's B-V (Ballesteros 2012, by the sky's one copy, `tempFromBV`); unknown reads as the Sun's 0.65. */
export function bvTemperature(b_v: number | null): number {
  return tempFromBV(Math.min(2, b_v ?? 0.65));
}

/** The young stars' share of a galaxy's visible light, by the True galaxy's own light laws. */
export function youngShare(stellar_mass_msun: number, sfr_msun_yr: number): number {
  const old = GALAXY_LIGHT.old_l_per_m * Math.max(0, stellar_mass_msun);
  const young = GALAXY_LIGHT.young_l_per_sfr * Math.max(0, sfr_msun_yr);
  return old + young > 0 ? young / (old + young) : 0;
}

export interface GalaxyTypePicture {
  /** RGBA, `width` square, light on black with alpha as brightness (for drawing over space). */
  rgba: Uint8ClampedArray;
  width: number;
  /** Half the picture's width in kpc: the picture spans this far each way from the galaxy's centre. */
  extent_kpc: number;
}

/** How far out the picture reaches, in half-light radii: past where a disc's light has fallen a hundredfold. */
export const PICTURE_REACH_RE = 4;

/**
 * Light (linear, before the stretch) at a point of the sky picture, kpc from the centre, x to the right and y up on
 * the page. Exported for tests: the bulge and disc shares, the arms and the knots are all read here.
 */
export function typeLight(f: GalaxyTypeFacts, x: number, y: number): { old: number; young: number } {
  const T = GALAXY_TYPES[f.morphology] ?? GALAXY_TYPES.dwarf!;
  const re = Math.max(0.05, f.half_light_kpc);
  const q = Math.min(1, Math.max(0.08, f.axis_ratio));
  // Turned by the position angle (measured from up toward the left, as the sky does), then stretched back to the
  // galaxy's own plane along its minor axis.
  const pa = (f.position_angle_deg * Math.PI) / 180;
  const u = x * Math.cos(pa) + y * Math.sin(pa);
  const v = -x * Math.sin(pa) + y * Math.cos(pa);
  // A lumpy galaxy's light is lopsided: its middle sits off the picture's by a keyed share of R_e.
  const seed0 = seedOf(f.address);
  const off = T.lumps >= 0.5 ? 0.3 * re : 0;
  const X = v - off * Math.cos(seed0 % 628 / 100);
  const Y = u / q - off * Math.sin(seed0 % 628 / 100);
  const r = Math.hypot(X, Y);
  const seed = seedOf(f.address);
  // The bulge is rounder than the disc it sits in (it is a spheroid): its own axis ratio is pulled toward 1.
  const qb = T.bulge >= 1 ? q : 1 - (1 - q) * 0.3;
  const rb = Math.hypot(v, u / qb);
  const reB = re * T.bulge_re;
  const bulge = T.bulge > 0 ? (T.bulge * Math.exp(-sersicB(4) * (Math.pow(rb / reB, 0.25) - 1))) / (sersicTotal(4) * reB * reB * qb) : 0;
  let disc = 0;
  let arm = 0;
  if (T.bulge < 1) {
    disc = ((1 - T.bulge) * Math.exp(-sersicB(1) * (r / re - 1))) / (sersicTotal(1) * re * re * q);
    if (T.arms > 0) {
      const phi = Math.atan2(Y, X);
      const r0 = re * (T.bar > 0 ? T.bar : 0.25);
      const phase0 = (seed % 6283) / 1000;
      const wind = Math.log(Math.max(r, 1e-6) / r0) / Math.tan((T.pitch_deg * Math.PI) / 180);
      const c = 0.5 + 0.5 * Math.cos(T.arms * (phi - wind) + phase0);
      // Arms start at the bar's end (or a quarter of R_e) and fade in over the next half R_e.
      const on = Math.min(1, Math.max(0, (r - r0) / (0.5 * re)));
      arm = Math.pow(c, 4) * on;
      disc *= 1 - T.arm_contrast * on + T.arm_contrast * 2.2 * arm;
    }
    if (T.bar > 0) {
      const lb = re * T.bar;
      const wb = lb * GALAXY_LIGHT.bar_width;
      disc += ((1 - T.bulge) * 0.25 * Math.exp(-0.5 * ((X / lb) ** 4 + (Y / wb) ** 2))) / (sersicTotal(1) * re * re * q);
    }
    if (T.lumps > 0) {
      const n = 0.5 * cloudNoise(seed, X, Y, 0.7 * re) + 0.3 * cloudNoise(seed ^ 0x9e3779b9, X, Y, 0.3 * re) + 0.2 * cloudNoise(seed ^ 0x27d4eb2f, X, Y, 0.12 * re);
      disc *= 1 - T.lumps + T.lumps * 2 * n;
    }
  }
  // Young stars: knots where the gas is, on the arms of a spiral, anywhere in the disc of a lumpy galaxy.
  const share = youngShare(f.stellar_mass_msun, f.sfr_msun_yr);
  let young = 0;
  if (share > 0 && T.bulge < 1) {
    const k = cloudNoise(seed ^ 0x5bd1e995, X, Y, 0.08 * re);
    const knot = Math.pow(Math.max(0, k - 0.55) / 0.45, 2);
    const where = T.arms > 0 ? arm : 0.5 + T.lumps * 0.5;
    young = (share * disc * where * knot * 6) / Math.max(1e-6, 1 - share);
  }
  return { old: bulge + disc, young };
}

/**
 * How deep the picture's log stretch reaches below its brightest pixel, decades: shallower than the charted True
 * galaxy's, because a picture a few pixels across shows its middle and shape best, as a short exposure would (fudge
 * GZ-F14).
 */
export const TYPE_DECADES = 2.2;

/**
 * A galaxy's type picture, `width` pixels square, spanning PICTURE_REACH_RE half-light radii each way. Shown on a
 * log stretch `TYPE_DECADES` deep from its brightest pixel.
 */
export function galaxyTypePicture(f: GalaxyTypeFacts, width = 64): GalaxyTypePicture {
  const extent = PICTURE_REACH_RE * Math.max(0.05, f.half_light_kpc);
  const old = starRgb(bvTemperature(f.b_v));
  const blue = starRgb(15000);
  const lin = new Float32Array(width * width * 3);
  let peak = 0;
  for (let j = 0; j < width; j++)
    for (let i = 0; i < width; i++) {
      const x = (((i + 0.5) / width) * 2 - 1) * extent;
      const y = (1 - ((j + 0.5) / width) * 2) * extent;
      // Faded over the picture's outer quarter, so its square edge never shows (a long-winged elliptical reaches it).
      const rr = Math.hypot(x, y) / extent;
      const fade = rr <= 0.75 ? 1 : rr >= 1 ? 0 : 1 - ((rr - 0.75) / 0.25) ** 2 * (3 - 2 * ((rr - 0.75) / 0.25));
      const l0 = typeLight(f, x, y);
      const l = { old: l0.old * fade, young: l0.young * fade };
      const k = (j * width + i) * 3;
      for (let c = 0; c < 3; c++) lin[k + c] = l.old * old[c]! + l.young * blue[c]!;
      peak = Math.max(peak, lin[k]!, lin[k + 1]!, lin[k + 2]!);
    }
  const rgba = new Uint8ClampedArray(width * width * 4);
  const decades = TYPE_DECADES;
  for (let p = 0; p < width * width; p++) {
    const r = lin[p * 3]!;
    const g = lin[p * 3 + 1]!;
    const b = lin[p * 3 + 2]!;
    const m = Math.max(r, g, b);
    if (m <= 0 || peak <= 0) continue;
    const level = Math.pow(Math.max(0, 1 + Math.log10(m / peak) / decades), 2.2);
    rgba[p * 4] = enc((r / m) * level);
    rgba[p * 4 + 1] = enc((g / m) * level);
    rgba[p * 4 + 2] = enc((b / m) * level);
    rgba[p * 4 + 3] = enc(level);
  }
  return { rgba, width, extent_kpc: extent };
}

/** Linear light to an 8-bit sRGB value. */
const enc = (v: number) => Math.round(255 * Math.max(0, Math.min(1, v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055)));
