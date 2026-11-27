// Comets in True (track SB part SB4 part 2, render audit build item 4): an icy body's coma and its two tails, drawn
// from the kernel's activity (sse-small comet.rs, on every `small_body` answer for a body that holds ice). Grades
// (visual polish design section 3):
// - fact: whether it is active at the distance it is now at, which ice drives it, the coma's size, the dust and ion
//   tails' lengths, the gas and dust it gives off each second;
// - drawn from the numbers: the coma's light falls as one over the distance from the nucleus (a steady outflow's
//   column density, Haser 1957), each tail points straight away from the star (the ion tail rides the star's wind;
//   radiation pressure pushes the dust), its grains thinning along it as a steady flow spreads;
// - stand-ins (docs/fudges.md, rows COMET-*): how bright each part is drawn (from its production on a log scale, as
//   there is no grain size or ion density yet), the tails' widths, and the dust tail drawn straight (its curve needs
//   the grains' sizes, which the kernel does not give).
import { constant, say, sheetLine } from "../../ui-contract/src/index.ts";
import { wavelengthRgb } from "./star-shells.ts";

export const COMET_REFERENCES = [
  { text: "Haser, L. (1957). Distribution d'intensité dans la tête d'une comète. Bulletin de la Société Royale des Sciences de Liège 43, 740 (a steady outflow from the nucleus: the coma's column density falls as one over the distance from it).", url: "https://ui.adsabs.harvard.edu/abs/1957BSRSL..43..740H" },
  { text: "Brandt, J. C. and Chapman, R. D. (2004). Introduction to Comets, 2nd ed. Cambridge University Press (ion tails point away from the Sun along the solar wind, glowing in CO+ near 426 nm and H2O+ near 615 nm; dust tails shine by reflected sunlight).", doi: "10.1017/CBO9781139165327" },
];

/** The kernel's activity for an icy body (sse-small `Activity`). */
export interface CometActivity {
  r_au: number;
  active: boolean;
  dominant?: string | null;
  gas_kg_s: number;
  dust_kg_s: number;
  coma_km: number;
  dust_tail_km: number;
  ion_tail_km: number;
}

/** Productions drawn from faint to full, kg/s: 1 kg/s barely shows, a Hale-Bopp at perihelion (about 1e6) is full (fudge COMET-glow). */
export const COMET_FAINT_KG_S = 1;
export const COMET_FULL_KG_S = 1e6;
/** Grains drawn in each tail at full strength (a drawing's count, not the comet's: fudge register). */
export const TAIL_GRAINS = 900;
/** Ion tails are narrow beside the coma (a few tenths of its width); the dust tail starts as wide as the coma. */
export const ION_WIDTH = 0.25;

/** How strongly a production shows, 0..1, on a log scale between faint and full (fudge COMET-glow). */
export function showing(kg_s: number): number {
  if (!(kg_s > COMET_FAINT_KG_S)) return 0;
  return Math.min(1, Math.log10(kg_s / COMET_FAINT_KG_S) / Math.log10(COMET_FULL_KG_S / COMET_FAINT_KG_S));
}

/** Whether a body's activity gives anything to draw. */
export const isActive = (a: CometActivity | undefined | null): a is CometActivity => !!a && a.active && a.coma_km > 0 && showing(Math.max(a.gas_kg_s, a.dust_kg_s)) > 0;

/**
 * The ion tail's colour, linear: the light of its commonest ion. CO and CO2 comets glow in CO+ (about 426 nm, blue);
 * a water comet's tail is H2O+ (about 615 nm, orange), which is why water-driven tails look less blue (Brandt and
 * Chapman 2004). The wavelengths are the ion rows' lines on the master data sheet.
 */
export function ionColour(a: CometActivity): [number, number, number] {
  const w = sheetLine(a.dominant === "h2o" ? "h2o+" : "co+").at_m;
  const c = wavelengthRgb(w);
  const m = Math.max(c[0], c[1], c[2]) || 1;
  return [c[0] / m, c[1] / m, c[2] / m];
}

/** Deterministic unit hash, so a tail looks the same every picture. */
const unit = (i: number, k: number) => {
  let h = Math.imul(i ^ 0x2545f491, 0x9e3779b1) ^ Math.imul(k + 0x6a09e667, 0x85ebca6b);
  h ^= h >>> 15;
  h = Math.imul(h, 0x2c1b3c6d);
  h ^= h >>> 12;
  return (h >>> 0) / 4294967296;
};

/**
 * A tail's grains, each as (s, w, h): metres along the tail away from the star from the nucleus, and across it in the
 * orbit's plane and out of it. A steady flow along the tail keeps its grains per metre falling as one over the
 * distance from the head (they spread as they go; Haser 1957), so the distances are drawn by that law, from a tenth of
 * the coma out to the tail's length; widths round the axis are a disc of the tail's width.
 */
export function tailGrains(length_m: number, width_m: number, n: number, seed: number): Float64Array {
  const out = new Float64Array(n * 3);
  const s0 = Math.max(1, Math.min(length_m * 1e-3, width_m * 0.1));
  const span = Math.log(Math.max(s0 * 1.0001, length_m) / s0);
  for (let i = 0; i < n; i++) {
    // Density 1/s between s0 and L: s = s0 · e^(u ln(L/s0)).
    const s = s0 * Math.exp(unit(i, seed) * span);
    const r = width_m * Math.sqrt(unit(i, seed + 1));
    const a = 2 * Math.PI * unit(i, seed + 2);
    out[i * 3] = s;
    out[i * 3 + 1] = r * Math.cos(a);
    out[i * 3 + 2] = r * Math.sin(a);
  }
  return out;
}

/** The two tails of an active body, in metres in the tail's own frame (tailGrains). Empty when the body is quiet. */
export function cometTails(a: CometActivity): { dust: Float64Array; ion: Float64Array; dustShows: number; ionShows: number } {
  const none = new Float64Array(0);
  if (!isActive(a)) return { dust: none, ion: none, dustShows: 0, ionShows: 0 };
  const coma = a.coma_km * 1000;
  const dustShows = showing(a.dust_kg_s);
  const ionShows = showing(a.gas_kg_s);
  const dust = a.dust_tail_km > 0 && dustShows > 0 ? tailGrains(a.dust_tail_km * 1000, coma, Math.round(TAIL_GRAINS * dustShows), 11) : none;
  const ion = a.ion_tail_km > 0 && ionShows > 0 ? tailGrains(a.ion_tail_km * 1000, ION_WIDTH * coma, Math.round(TAIL_GRAINS * ionShows), 23) : none;
  return { dust, ion, dustShows, ionShows };
}

/**
 * The coma as a camera-facing glow picture, `n` pixels square, the coma's edge at the square's edge: light falling as
 * one over the distance from the nucleus (Haser 1957), in the colour of the starlight its dust reflects (`rgb`,
 * linear), at `strength` 0..1. sRGB bytes, alpha the light.
 */
export function comaPicture(rgb: readonly [number, number, number], strength: number, n = 64): { width: number; data: Uint8ClampedArray } {
  const data = new Uint8ClampedArray(n * n * 4);
  const enc = (c: number) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);
  // The core is unresolved inside a twentieth of the coma; past it the light goes as 1/ρ, fading to nothing at the edge.
  const core = 0.05;
  for (let y = 0; y < n; y++)
    for (let x = 0; x < n; x++) {
      const r = Math.hypot((x + 0.5) / n - 0.5, (y + 0.5) / n - 0.5) * 2;
      const v = r >= 1 ? 0 : (core / Math.max(core, r)) * (1 - r) * strength;
      const o = (y * n + x) * 4;
      data[o] = 255 * enc(rgb[0] * v);
      data[o + 1] = 255 * enc(rgb[1] * v);
      data[o + 2] = 255 * enc(rgb[2] * v);
      data[o + 3] = 255 * enc(v);
    }
  return { width: n, data };
}

/** A CSS hex colour as linear 0..1. */
export function hexLinear(hex: string): [number, number, number] {
  const dec = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const m = hex.match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"];
  return [dec(parseInt(m[0] ?? "ff", 16) / 255), dec(parseInt(m[1] ?? "ff", 16) / 255), dec(parseInt(m[2] ?? "ff", 16) / 255)];
}

/** A linear colour as a CSS rgb() colour. */
export function linearCss(c: readonly [number, number, number]): string {
  const enc = (v: number) => Math.round(255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * Math.max(0, v) ** (1 / 2.4) - 0.055));
  return `rgb(${enc(c[0])}, ${enc(c[1])}, ${enc(c[2])})`;
}

/** One sentence for True's note. */
export function cometNote(name: string, a: CometActivity): string {
  // Sizes through the units engine, as a star's surroundings are measured.
  const len = (km: number) => say(km * 1000, "length", { class: "star", what: "orbit" }, { sig: 2 }).text;
  const ice = a.dominant === "h2o" ? "water" : a.dominant === "co2" ? "carbon dioxide" : a.dominant === "co" ? "carbon monoxide" : "its ice";
  return `${name} is active ${say(a.r_au * constant("au"), "length", { class: "star", what: "orbit" }, { sig: 3 }).text} from its star, driven by ${ice}: a coma ${len(a.coma_km)} across, a dust tail ${len(a.dust_tail_km)} and an ion tail ${len(a.ion_tail_km)} long (kernel activity; tails drawn straight away from the star).`;
}
