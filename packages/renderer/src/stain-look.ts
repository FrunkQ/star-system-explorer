// A world's stains in True (TRUE4): the kernel says what coats its ground and how thoroughly (sse-surface stains.rs,
// `surfaces[].stains`); each stain's hue comes from its data-sheet optics (sheet-optics.ts). The kernel's tile albedo
// keeps setting how bright the ground is; a stain moves its hue toward the stain's, by its share. A stain whose sheet
// row has no optics is not drawn (visual polish rule: no number, no flourish), and the note says so.
// A stain whose row is a measured reflectance spectrum (`optical.reflectance`, MP35's shape: Io's sulphur, MP36) is
// laid on whole, brightness and all: a coating covering a share s of the ground sends back (1 - s) of the ground's
// light plus s of its own (areal mixing, Hapke 2012 section 10.2), so bright yellow sulphur makes Io bright and yellow.

import { constant } from "../../ui-contract/src/index.ts";
import { indexTableOf, seenColour } from "./sheet-optics.ts";
import { bandsOf, type ReflectanceTable } from "./ground-optics.ts";
import type { Prepared } from "./surface.ts";
import type { SurfaceTile } from "../../ui-contract/src/index.ts";
import { sampleColour } from "../../ui-contract/src/index.ts";

/**
 * A stain's grain size, µm: stains are fine coatings, settled haze and dust, not coarse rock (Mars's airborne dust has
 * an effective radius near 1.5 µm, Wolff et al. 2006, JGR 111, E12S17; tholin haze grains are smaller still). Until the
 * kernel gives a grain size per stain (fudge register).
 */
export const STAIN_GRAIN_UM = 1.5;

/** A stain as the kernel writes it. */
export interface StainFact {
  kind: string;
  material_id: string;
  share: number;
  why: string;
}

/** A stain ready to draw: its hue (linear RGB, luminance 1) and share. */
export interface StainHue {
  kind: string;
  material_id: string;
  share: number;
  hue: [number, number, number];
  /**
   * The stain's own measured reflectance at the three bands True draws (linear, 0..1), when its row is a laboratory
   * spectrum: then the stain is laid on by areal mixing, brightness and all, not as a hue on the ground's brightness.
   */
  reflectance?: [number, number, number];
  /** Where the colour comes from, for the note: "measured reflectance" or "refractive index". */
  from: "reflectance" | "refractive index";
}

/** The stains on a surface record, strongest first, or none. */
export function stainFacts(surface: unknown): StainFact[] {
  const s = (surface as { stains?: unknown } | null)?.stains;
  if (!Array.isArray(s)) return [];
  return s.filter((x): x is StainFact => !!x && typeof x.material_id === "string" && typeof x.share === "number" && x.share > 0);
}

const lum = (c: readonly number[]) => 0.2126 * c[0]! + 0.7152 * c[1]! + 0.0722 * c[2]!;

/** A sheet row's measured reflectance spectrum (MP35's shape), or null. */
function reflectanceOf(row: unknown): ReflectanceTable | null {
  const t = (row as { optical?: { reflectance?: ReflectanceTable } } | null)?.optical?.reflectance;
  return t?.lambda_nm?.length && t.r?.length === t.lambda_nm.length ? t : null;
}

/**
 * A stain's colour from its sheet row (any table: `readMatter({ id })`), or null when the row has no optics: a measured
 * reflectance spectrum where the row has one (read at the bands True draws, as the bare ground is, ground-optics.ts),
 * else a layer of its grains from its refractive index (sheet-optics.ts).
 */
export function stainHue(f: StainFact, row: unknown, star_k = constant("T_sun")): StainHue | null {
  const share = Math.min(1, f.share);
  const rt = reflectanceOf(row);
  if (rt) {
    const r = bandsOf(rt);
    const y = lum(r);
    if (!(y > 0)) return null;
    return { kind: f.kind, material_id: f.material_id, share, hue: [r[0] / y, r[1] / y, r[2] / y], reflectance: r, from: "reflectance" };
  }
  const t = indexTableOf(row);
  if (!t) return null;
  const c = seenColour(t, STAIN_GRAIN_UM, star_k).rgb;
  const y = lum(c);
  if (!(y > 0)) return null;
  return { kind: f.kind, material_id: f.material_id, share, hue: [c[0] / y, c[1] / y, c[2] / y], from: "refractive index" };
}

/** The kernel door `readStains` needs. */
export interface StainReader {
  readMatter(ask: { table?: string; id?: string }): Promise<{ ok: true; value: unknown } | { ok: false; message: string }>;
}

/** A surface record's stains, each coloured from its data-sheet row (rows with no optics are left out). */
export async function readStains(k: StainReader, surface: unknown): Promise<StainHue[]> {
  const out: StainHue[] = [];
  for (const f of stainFacts(surface)) {
    const r = await k.readMatter({ id: f.material_id });
    const h = r.ok ? stainHue(f, (r.value as { row?: unknown }).row) : null;
    if (h) out.push(h);
  }
  return out;
}

/**
 * Tiles with their bare ground stained in place (`ground_rgb`), for views that colour straight from the tiles (the
 * Body globe and the Land screen) rather than through `trueSurface`'s own staining. Lay the measured ground first
 * (ground-optics.ts `withGround`), then this; covers and every kernel value are untouched.
 */
export function stainTiles(tiles: readonly SurfaceTile[], stains: readonly StainHue[]): SurfaceTile[] {
  const ground = groundStains(stains);
  if (!ground.length) return tiles.slice();
  return tiles.map((t) => {
    const n = t.material.length;
    const rgb = new Float32Array(n * 3).fill(NaN);
    const one = new Uint8Array(3);
    let any = false;
    for (let i = 0; i < n; i++) {
      if (t.cover[i] !== 0) {
        if (t.ground_rgb) for (let j = 0; j < 3; j++) rgb[i * 3 + j] = t.ground_rgb[i * 3 + j]!;
        continue;
      }
      one.set(sampleColour(t, i, 1));
      stainTexel(one, 0, ground);
      rgb.set(one, i * 3);
      any = true;
    }
    return any ? { ...t, ground_rgb: rgb } : t;
  });
}

const toLin = (v: number) => {
  const x = v / 255;
  return x <= 0.04045 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
};
const toByte = (v: number) => Math.round(255 * Math.max(0, Math.min(1, v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055)));

/**
 * The bare ground of a prepared globe (cover none) stained in place: each sample's colour keeps its luminance and moves
 * toward each stain's hue by the stain's share (a measured-reflectance stain is mixed in whole, brightness and all),
 * strongest last so it shows most. Water, ice and magma are untouched
 * (frost is its own cover in the kernel's tiles).
 */
export function stainPrepared(p: Prepared, stains: readonly StainHue[]): Prepared {
  const ground = groundStains(stains);
  if (!ground.length) return p;
  for (const f of p.faces) {
    if (!f) continue;
    const n = f.s * f.s;
    for (let k = 0; k < n; k++) if (f.cover[k] === 0) stainTexel(f.rgb, k * 3, ground);
  }
  return p;
}

/** The stains drawn on bare ground (frost is its own cover), weakest first so the strongest shows most. */
export function groundStains(stains: readonly StainHue[]): StainHue[] {
  return stains.filter((s) => s.kind !== "frost" && s.share > 0).sort((a, b) => a.share - b.share);
}

/**
 * One sRGB texel at `o` stained in place by `ground` (from `groundStains`): its luminance kept and its hue moved, or,
 * for a stain with a measured reflectance, the stain's own reflectance mixed in by its share.
 */
export function stainTexel(rgb: Uint8Array | Uint8ClampedArray, o: number, ground: readonly StainHue[]): void {
  let c = [toLin(rgb[o]!), toLin(rgb[o + 1]!), toLin(rgb[o + 2]!)];
  const y = lum(c);
  for (const s of ground) {
    const own = s.reflectance;
    c = c.map((v, j) => (1 - s.share) * v + s.share * (own ? own[j]! : y * s.hue[j]!));
  }
  rgb[o] = toByte(c[0]!);
  rgb[o + 1] = toByte(c[1]!);
  rgb[o + 2] = toByte(c[2]!);
}

/** One line for True's note: what stains the world and where its colour comes from. */
export function stainNote(name: string, facts: readonly StainFact[], drawn: readonly StainHue[]): string {
  if (!facts.length) return "";
  const parts = facts.map((f) => {
    const h = drawn.find((x) => x.material_id === f.material_id);
    return `${f.kind.replace(/-/g, " ")} ${Math.round(f.share * 100)}%${h ? (h.from === "reflectance" ? " (its measured reflectance)" : "") : " (not drawn: no optics on the sheet yet)"}`;
  });
  return `${name}'s ground: ${parts.join(", ")}; colours from the data sheet's measured optics.`;
}
