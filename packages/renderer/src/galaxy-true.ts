// The True galaxy (V9t): a galaxy drawn from the kernel's own numbers, in any band. The kernel's galaxy chart
// (`galaxy_chart`, G1 to G4 and GH3) gives each ring's stars and gas, the arm crests, the bulge and the dated marks
// of every collision; this turns them into light and nothing more. Old stars shine by their mass, young stars by the
// star formation the gas allows (Kennicutt 1998), dust in the gas dims the visible light, and each band reads the
// part of the galaxy that shines in it. The shapes between the kernel's numbers (how wide an arm is, the bar, the
// bulge's size) are stand-ins and are in the fudge register.
//
// The picture is face-on, in kpc: the 3D view lays it on the disc's plane (bent by the warp, galaxy-marks.ts) and
// stacks it through the disc's thickness, with the bulge as a glow of its own.
import type { GalaxyChart, GalaxyRing } from "../../ui-contract/src/index.ts";
import { brightnessAt, scarsAt, type DiscScars } from "./galaxy-marks.ts";
import { starRgb } from "./sky.ts";

export type GalaxyBand = "visible" | "infrared" | "ultraviolet" | "xray" | "radio";
export const GALAXY_BANDS: readonly GalaxyBand[] = ["visible", "infrared", "ultraviolet", "xray", "radio"];

/**
 * The light laws and stand-in shapes, each with its source. Surface densities in solar masses per square parsec
 * where a law is written that way.
 */
export const GALAXY_LIGHT = {
  /** Star formation from gas: Σ_SFR = a Σ_gas^n, M☉/yr/kpc² from M☉/pc² (Kennicutt 1998, ApJ 498, 541). */
  kennicutt: { a: 2.5e-4, n: 1.4 },
  /** Visible light per solar mass of old stars, L☉/M☉ (a mass-to-light ratio of about 2.5 in V for an old disc:
   * Bell and de Jong 2001). */
  old_l_per_m: 0.4,
  /** Visible light of the young stars per unit of star formation, L☉ per M☉/yr (a population forming steadily for
   * 100 Myr: Starburst99, Leitherer et al. 1999, about 2×10⁹). */
  young_l_per_sfr: 2e9,
  /** Near-infrared (2.2 µm) light per solar mass, L☉/M☉ (mass-to-light near 1 in K: Bell and de Jong 2001), and of
   * the young stars per M☉/yr, a tenth of their visible share (they are hot and blue). */
  ir_l_per_m: 1,
  ir_young_share: 0.1,
  /** Dust in the gas: A_V per M☉/pc² of gas at the Sun's metallicity (N_H/A_V = 1.9×10²¹ cm⁻², Bohlin et al. 1978;
   * 1 M☉/pc² ≈ 1.25×10²⁰ H cm⁻²), scaled by the gas's metallicity over 0.014 (Asplund et al. 2009). */
  av_per_gas: 0.066,
  z_sun: 0.014,
  /** Ultraviolet extinction over visible (A_FUV ≈ 2.6 A_V for Milky Way dust: Cardelli, Clayton and Mathis 1989). */
  uv_over_v: 2.6,
  /** X-rays: high-mass binaries per unit star formation, erg/s per M☉/yr (Mineo, Gilfanov and Sunyaev 2012), and
   * low-mass binaries per solar mass of stars, erg/s per M☉ (Gilfanov 2004). */
  hmxb_per_sfr: 2.6e39,
  lmxb_per_m: 1e29,
  /** Stand-ins (fudge register "True galaxy shapes"): an arm crest's half-width, kpc; how much more gas sits on a
   * crest than between arms at the chart's arm contrast; the bar's half-length is where the arms start, its width a
   * share of that; the bulge's scale is a share of the disc's scale length. Widths are shares of the disc's scale
   * length, so a dwarf's arms are as thick for its size as a giant's. */
  arm_halfwidth_rd: 0.18,
  arm_gas_gain: 3,
  bar_width: 0.3,
  bulge_rd: 0.2,
  /** Clumps: the gas is broken into clouds a few hundred parsecs across, star-forming knots on the crests, by
   * value noise at these sizes (shares of the disc's scale length, never under three pixels) and this depth (0
   * smooth, 1 fully broken). The dust lane lies this far inside
   * each crest (a share of the scale length), on its concave side, where the gas is shocked before the stars light (Roberts 1969). */
  clump_rd: [0.5, 0.25, 0.12],
  clump_depth: 0.75,
  lane_rd: 0.12,
  lane_gain: 2.5,
  /** The disc's thin and thick scale heights, kpc (galaxy-marks.ts SCAR_SHAPES.heights carries the same). */
  thin_kpc: 0.3,
  /** Decades of surface brightness a mapped band spans below its brightest. */
  decades: 4,
  /** Colours of the two visible populations: an old disc's light (about 4,800 K) and young OB-rich light (about
   * 15,000 K), each as a star of that temperature. */
  old_k: 4800,
  young_k: 15000,
} as const;

export const GALAXY_LIGHT_REFS = [
  "Kennicutt 1998, ApJ 498, 541: star formation from gas",
  "Bell and de Jong 2001, ApJ 550, 212: mass-to-light ratios",
  "Leitherer et al. 1999, ApJS 123, 3: light of a young population (Starburst99)",
  "Bohlin, Savage and Drake 1978, ApJ 224, 132: gas to dust",
  "Cardelli, Clayton and Mathis 1989, ApJ 345, 245: extinction by wavelength",
  "Mineo, Gilfanov and Sunyaev 2012, MNRAS 419, 2095: X-ray binaries and star formation",
  "Gilfanov 2004, MNRAS 349, 146: low-mass X-ray binaries and stellar mass",
];

/** A face-on picture: RGBA, row 0 at +y (the top), `width` pixels across 2·outer_kpc. */
export interface GalaxyPicture {
  width: number;
  data: Uint8ClampedArray;
  outer_kpc: number;
  band: GalaxyBand;
  /** The bulge as a glow: its scale (kpc), its colour and its brightness on the picture's own scale (0..1). */
  bulge: { scale_kpc: number; rgb: [number, number, number]; level: number };
  /** The bar, when there is one: its half-length and angle (radians from +x). */
  bar: { half_kpc: number; angle: number } | null;
  /** Words for the band: what is drawn and how it is mapped. */
  words: string;
}

/** A ring's surface density of stars and of gas, M☉/kpc². */
const sigma = (g: GalaxyRing) => {
  const a = Math.PI * (g.outer_kpc * g.outer_kpc - g.inner_kpc * g.inner_kpc);
  return { stars: g.stars / Math.max(a, 1e-9), gas: g.gas / Math.max(a, 1e-9) };
};

/** Surface densities at radius r, interpolated in log between ring middles; past the last ring, falling with the disc's scale length. */
export function surfaceAt(rings: readonly GalaxyRing[], r: number, rd: number): { stars: number; gas: number } {
  if (!rings.length) return { stars: 0, gas: 0 };
  const mid = (g: GalaxyRing) => (g.inner_kpc + g.outer_kpc) / 2;
  const first = rings[0]!;
  const last = rings[rings.length - 1]!;
  if (r <= mid(first)) return sigma(first);
  if (r >= mid(last)) {
    const s = sigma(last);
    const k = Math.exp(-(r - mid(last)) / Math.max(rd, 1e-3));
    return { stars: s.stars * k, gas: s.gas * k };
  }
  let i = 0;
  while (i < rings.length - 2 && r > mid(rings[i + 1]!)) i++;
  const a = rings[i]!;
  const b = rings[i + 1]!;
  const u = (r - mid(a)) / Math.max(mid(b) - mid(a), 1e-9);
  const lerp = (x: number, y: number) => (x > 0 && y > 0 ? Math.exp(Math.log(x) * (1 - u) + Math.log(y) * u) : x * (1 - u) + y * u);
  const sa = sigma(a);
  const sb = sigma(b);
  return { stars: lerp(sa.stars, sb.stars), gas: lerp(sa.gas, sb.gas) };
}

/** How near each pixel is to an arm crest, 0..1 (a Gaussian across the crest), on a width×width grid over ±outer. */
export function armField(arms: readonly (readonly [number, number][])[], width: number, outer: number, halfwidth: number): Float32Array {
  const f = new Float32Array(width * width);
  const px = (2 * outer) / width;
  const reach = Math.ceil((3 * halfwidth) / px);
  const splat = (x: number, y: number) => {
    const ci = Math.floor((x + outer) / px);
    const cj = Math.floor((outer - y) / px);
    for (let j = cj - reach; j <= cj + reach; j++) {
      if (j < 0 || j >= width) continue;
      const yy = outer - (j + 0.5) * px;
      for (let i = ci - reach; i <= ci + reach; i++) {
        if (i < 0 || i >= width) continue;
        const xx = -outer + (i + 0.5) * px;
        const d2 = ((xx - x) ** 2 + (yy - y) ** 2) / (halfwidth * halfwidth);
        const v = Math.exp(-0.5 * d2);
        const k = j * width + i;
        if (v > f[k]!) f[k] = v;
      }
    }
  };
  for (const arm of arms) {
    for (let n = 0; n < arm.length; n++) {
      const [r, phi] = arm[n]!;
      const x = r * Math.cos(phi);
      const y = r * Math.sin(phi);
      splat(x, y);
      const next = arm[n + 1];
      if (!next) continue;
      // Fill between crest points so a wide gap far out leaves no beads.
      const x2 = next[0] * Math.cos(next[1]);
      const y2 = next[0] * Math.sin(next[1]);
      const steps = Math.floor(Math.hypot(x2 - x, y2 - y) / (halfwidth / 2));
      for (let s = 1; s < steps; s++) splat(x + ((x2 - x) * s) / steps, y + ((y2 - y) * s) / steps);
    }
  }
  return f;
}

/** Smooth value noise in 0..1 at a point (kpc), from a seed: the clouds' stand-in (fudge register "True galaxy shapes"). */
export function cloudNoise(seed: number, x: number, y: number, size: number): number {
  const h = (i: number, j: number) => {
    let v = Math.imul(i, 374761393) ^ Math.imul(j, 668265263) ^ seed;
    v = Math.imul(v ^ (v >>> 13), 1274126177);
    return ((v ^ (v >>> 16)) >>> 0) / 4294967296;
  };
  const u = x / size;
  const w = y / size;
  const i = Math.floor(u);
  const j = Math.floor(w);
  const fu = u - i;
  const fw = w - j;
  const su = fu * fu * (3 - 2 * fu);
  const sw = fw * fw * (3 - 2 * fw);
  const a = h(i, j) + (h(i + 1, j) - h(i, j)) * su;
  const b = h(i, j + 1) + (h(i + 1, j + 1) - h(i, j + 1)) * su;
  return a + (b - a) * sw;
}

const seedOf = (key: string) => {
  let v = 2166136261;
  for (const ch of key) v = Math.imul(v ^ ch.charCodeAt(0), 16777619);
  return v >>> 0;
};

/** Linear light to an 8-bit sRGB value. */
const enc = (v: number) => Math.round(255 * Math.max(0, Math.min(1, v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055)));

/**
 * The galaxy at `t_gyr` in a band, face-on, `width` pixels across. Visible is in true colour; the other bands are
 * mapped grey on a log scale `decades` deep from the brightest point (they have no colour to see).
 */
export function galaxyFace(chart: GalaxyChart, t_gyr: number, band: GalaxyBand, width = 256): GalaxyPicture {
  const L = GALAXY_LIGHT;
  const rings = chart.rings;
  const edge = rings.at(-1)?.outer_kpc ?? 1;
  // The picture reaches a little past the last ring, where the disc fades out.
  const outer = edge * 1.15;
  const seed = seedOf(chart.key || chart.halo || "galaxy");
  const rd = Math.max(chart.disc_kpc, 0.1);
  const sc: DiscScars = scarsAt(chart.assembly?.marks ?? [], t_gyr, rd);
  const arm = armField(chart.arms, width, outer, L.arm_halfwidth_rd * rd);
  const pxKpc = (2 * outer) / width;
  const clumps = L.clump_rd.map((f) => Math.max(3 * pxKpc, f * rd));
  const metal = Math.max(0.05, (chart.now.z_gas || L.z_sun) / L.z_sun);
  // The bar: from the centre to where the arms start, along the first arm's starting angle.
  const a0 = chart.arms[0]?.[0];
  const bar = chart.barred && a0 ? { half_kpc: a0[0], angle: a0[1] } : null;
  const n = width * width;
  const lin = new Float32Array(n);
  const youngShare = new Float32Array(n);
  const px = (2 * outer) / width;
  for (let j = 0; j < width; j++) {
    const y = outer - (j + 0.5) * px;
    for (let i = 0; i < width; i++) {
      const x = -outer + (i + 0.5) * px;
      const r = Math.hypot(x, y);
      const k = j * width + i;
      if (r > outer) continue;
      const phi = Math.atan2(y, x);
      const s0 = surfaceAt(rings, r, rd);
      const taper = r > edge ? Math.max(0, 1 - (r - edge) / (outer - edge)) ** 2 : 1;
      const s = { stars: s0.stars * taper, gas: s0.gas * taper };
      // Gas gathers on the crests and thins between them, its mean kept; stars follow faintly. The dust lane is the
      // crest seen from a little further out, so it lies on the crest's inner side.
      const c = chart.arm_contrast;
      const a = arm[k]!;
      const lk = r > 0.05 ? (() => {
        const f = 1 + (L.lane_rd * rd) / r;
        const li = Math.floor((x * f + outer) / px);
        const lj = Math.floor((outer - y * f) / px);
        return li >= 0 && lj >= 0 && li < width && lj < width ? arm[lj * width + li]! : 0;
      })() : 0;
      const lane = Math.max(0, lk - a);
      // Clouds: three octaves of noise break the gas into clumps.
      const cloud = (cloudNoise(seed, x, y, clumps[0]!) * 0.5 + cloudNoise(seed + 1, x, y, clumps[1]!) * 0.3 + cloudNoise(seed + 2, x, y, clumps[2]!) * 0.2);
      const clump = 1 - L.clump_depth + L.clump_depth * 2 * cloud;
      const gasK = (1 + c * L.arm_gas_gain * (a - 0.3)) * clump;
      let stars = s.stars * (1 + 0.3 * c * (a - 0.3));
      const gas = Math.max(0, s.gas * gasK);
      const dustGas = gas * (1 + L.lane_gain * c * lane);
      if (bar && r < bar.half_kpc * 1.2) {
        // The bar: the inner disc's stars drawn into a long oval, emptier to its sides.
        const t = phi - bar.angle;
        const along = r * Math.cos(t);
        const across = r * Math.sin(t);
        const inBar = Math.exp(-0.5 * (across / (L.bar_width * bar.half_kpc)) ** 2) * (Math.abs(along) < bar.half_kpc ? 1 : Math.exp(-((Math.abs(along) - bar.half_kpc) / (0.1 * bar.half_kpc))));
        stars *= 0.5 + 1.5 * inBar;
      }
      // The marks: rings, lopsided and driven brightness act on the stars' light.
      stars *= brightnessAt(sc, r, phi);
      const gasPc = gas / 1e6;
      const sfr = L.kennicutt.a * Math.pow(gasPc, L.kennicutt.n); // M☉/yr/kpc²
      const av = L.av_per_gas * (dustGas / 1e6) * metal * 0.5; // half the dust lies in front, face-on
      let old = 0;
      let young = 0;
      if (band === "visible") {
        const dim = Math.pow(10, -0.4 * av);
        old = stars * L.old_l_per_m * dim;
        young = sfr * L.young_l_per_sfr * Math.pow(10, -0.4 * av * 1.3);
      } else if (band === "infrared") {
        old = stars * L.ir_l_per_m;
        young = sfr * L.young_l_per_sfr * L.ir_young_share;
      } else if (band === "ultraviolet") {
        young = sfr * L.young_l_per_sfr * Math.pow(10, -0.4 * av * L.uv_over_v);
      } else if (band === "xray") {
        old = stars * L.lmxb_per_m;
        young = sfr * L.hmxb_per_sfr;
      } else {
        // 21 cm: the gas itself, with nothing in front.
        old = gas;
      }
      lin[k] = old + young;
      youngShare[k] = old + young > 0 ? young / (old + young) : 0;
    }
  }
  // Knots of new stars from collisions: young light, a few hundred parsecs across.
  for (const kn of sc.knots) {
    const x = kn.r_kpc * Math.cos(kn.phi);
    const y = kn.r_kpc * Math.sin(kn.phi);
    const i = Math.floor((x + outer) / px);
    const j = Math.floor((outer - y) / px);
    if (i < 0 || j < 0 || i >= width || j >= width || band === "infrared" || band === "radio") continue;
    const k = j * width + i;
    lin[k] = lin[k]! * (1 + kn.size);
    youngShare[k] = Math.min(1, youngShare[k]! + 0.3);
  }
  let hi = 0;
  for (let k = 0; k < n; k++) if (lin[k]! > hi) hi = lin[k]!;
  // The bulge: old stars in a ball, its light on the disc's scale.
  const bulgeScale = L.bulge_rd * rd;
  const bulgeL = band === "visible" ? chart.now.bulge * L.old_l_per_m : band === "infrared" ? chart.now.bulge * L.ir_l_per_m : band === "xray" ? chart.now.bulge * L.lmxb_per_m : 0;
  const bulgePeak = bulgeL / (2 * Math.PI * bulgeScale * bulgeScale);
  const top = Math.max(hi, bulgePeak, 1e-30);
  const floor = top / Math.pow(10, L.decades);
  const level = (v: number) => (v > floor ? Math.log10(v / floor) / L.decades : 0);
  const out = new Uint8ClampedArray(n * 4);
  const oldRgb = starRgb(L.old_k);
  const youngRgb = starRgb(L.young_k);
  for (let k = 0; k < n; k++) {
    const v = level(lin[k]!);
    if (v <= 0) continue;
    let rgb: [number, number, number];
    if (band === "visible") {
      const u = youngShare[k]!;
      rgb = [oldRgb[0] * (1 - u) + youngRgb[0] * u, oldRgb[1] * (1 - u) + youngRgb[1] * u, oldRgb[2] * (1 - u) + youngRgb[2] * u];
    } else rgb = [1, 1, 1];
    const m = Math.max(rgb[0], rgb[1], rgb[2]) || 1;
    // A power on the level keeps the faint outer disc visible without flattening the centre.
    const b = Math.pow(v, 2.2);
    out[k * 4] = enc((rgb[0] / m) * b);
    out[k * 4 + 1] = enc((rgb[1] / m) * b);
    out[k * 4 + 2] = enc((rgb[2] / m) * b);
    out[k * 4 + 3] = 255;
  }
  const words =
    band === "visible"
      ? "Visible light in true colour: old stars by their mass, young blue stars where the gas forms them (Kennicutt's law), dimmed by the dust in the gas."
      : band === "infrared"
        ? `Near infrared (2.2 µm), mapped grey on a log scale ${L.decades} decades deep: the old stars' mass, seen through the dust.`
        : band === "ultraviolet"
          ? `Ultraviolet, mapped grey on a log scale ${L.decades} decades deep: only the young stars, deep in the dust of the arms.`
          : band === "xray"
            ? `X-rays, mapped grey on a log scale ${L.decades} decades deep: binaries with a young star where stars form, old binaries where the stars are.`
            : `Radio at 21 cm, mapped grey on a log scale ${L.decades} decades deep: the hydrogen gas itself, reaching past the stars.`;
  const brgb = band === "visible" ? starRgb(L.old_k) : ([1, 1, 1] as [number, number, number]);
  return { width, data: out, outer_kpc: outer, band, bulge: { scale_kpc: bulgeScale, rgb: [brgb[0], brgb[1], brgb[2]], level: bulgeL > 0 ? level(bulgePeak) : 0 }, bar, words };
}

/** A round glow, RGBA, `width` across: a Gaussian ball's projected light, for the bulge. */
export function glowPicture(width: number, rgb: readonly [number, number, number], level: number): Uint8ClampedArray {
  const out = new Uint8ClampedArray(width * width * 4);
  const m = Math.max(rgb[0], rgb[1], rgb[2]) || 1;
  for (let j = 0; j < width; j++)
    for (let i = 0; i < width; i++) {
      const x = ((i + 0.5) / width) * 2 - 1;
      const y = ((j + 0.5) / width) * 2 - 1;
      // The sprite spans ±4 scale lengths.
      const d2 = (x * x + y * y) * 16;
      const v = Math.pow(Math.max(0, level + Math.log10(Math.exp(-0.5 * d2)) / GALAXY_LIGHT.decades), 2.2);
      const k = (j * width + i) * 4;
      out[k] = enc((rgb[0] / m) * v);
      out[k + 1] = enc((rgb[1] / m) * v);
      out[k + 2] = enc((rgb[2] / m) * v);
      out[k + 3] = 255;
    }
  return out;
}

/** The disc's light through its thickness: weights of `layers` sheets spread over ±2 scale heights (sech², summing to 1). */
export function sheetWeights(layers: number): { z_kpc: number; weight: number }[] {
  const h = GALAXY_LIGHT.thin_kpc;
  const out: { z_kpc: number; weight: number }[] = [];
  for (let i = 0; i < layers; i++) {
    const z = layers === 1 ? 0 : -2 * h + (4 * h * i) / (layers - 1);
    const c = Math.cosh(z / h);
    out.push({ z_kpc: z, weight: 1 / (c * c) });
  }
  const s = out.reduce((a, o) => a + o.weight, 0);
  return out.map((o) => ({ ...o, weight: o.weight / s }));
}
