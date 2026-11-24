// A world's stains in True (TRUE4): the kernel says what coats its ground and how thoroughly (sse-surface stains.rs,
// `surfaces[].stains`); each stain's hue comes from its data-sheet optics (sheet-optics.ts). The kernel's tile albedo
// keeps setting how bright the ground is; a stain moves its hue toward the stain's, by its share. A stain whose sheet
// row has no optics is not drawn (visual polish rule: no number, no flourish), and the note says so.

import { constant } from "../../ui-contract/src/index.ts";
import { indexTableOf, seenColour } from "./sheet-optics.ts";
import type { Prepared } from "./surface.ts";

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
}

/** The stains on a surface record, strongest first, or none. */
export function stainFacts(surface: unknown): StainFact[] {
  const s = (surface as { stains?: unknown } | null)?.stains;
  if (!Array.isArray(s)) return [];
  return s.filter((x): x is StainFact => !!x && typeof x.material_id === "string" && typeof x.share === "number" && x.share > 0);
}

const lum = (c: readonly number[]) => 0.2126 * c[0]! + 0.7152 * c[1]! + 0.0722 * c[2]!;

/** A stain's hue from its sheet row (any table: `readMatter({ id })`), or null when the row has no optics. */
export function stainHue(f: StainFact, row: unknown, star_k = constant("T_sun")): StainHue | null {
  const t = indexTableOf(row);
  if (!t) return null;
  const c = seenColour(t, STAIN_GRAIN_UM, star_k).rgb;
  const y = lum(c);
  if (!(y > 0)) return null;
  return { kind: f.kind, material_id: f.material_id, share: Math.min(1, f.share), hue: [c[0] / y, c[1] / y, c[2] / y] };
}

const toLin = (v: number) => {
  const x = v / 255;
  return x <= 0.04045 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
};
const toByte = (v: number) => Math.round(255 * Math.max(0, Math.min(1, v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055)));

/**
 * The bare ground of a prepared globe (cover none) stained in place: each sample's colour keeps its luminance and moves
 * toward each stain's hue by the stain's share, strongest last so it shows most. Water, ice and magma are untouched
 * (frost is its own cover in the kernel's tiles).
 */
export function stainPrepared(p: Prepared, stains: readonly StainHue[]): Prepared {
  const ground = stains.filter((s) => s.kind !== "frost" && s.share > 0).sort((a, b) => a.share - b.share);
  if (!ground.length) return p;
  for (const f of p.faces) {
    if (!f) continue;
    const n = f.s * f.s;
    for (let k = 0; k < n; k++) {
      if (f.cover[k] !== 0) continue;
      let c = [toLin(f.rgb[k * 3]!), toLin(f.rgb[k * 3 + 1]!), toLin(f.rgb[k * 3 + 2]!)];
      const y = lum(c);
      for (const s of ground) c = c.map((v, j) => (1 - s.share) * v + s.share * y * s.hue[j]!);
      f.rgb[k * 3] = toByte(c[0]!);
      f.rgb[k * 3 + 1] = toByte(c[1]!);
      f.rgb[k * 3 + 2] = toByte(c[2]!);
    }
  }
  return p;
}

/** One line for True's note: what stains the world and where its colour comes from. */
export function stainNote(name: string, facts: readonly StainFact[], drawn: readonly StainHue[]): string {
  if (!facts.length) return "";
  const parts = facts.map((f) => {
    const d = drawn.some((h) => h.material_id === f.material_id);
    return `${f.kind.replace(/-/g, " ")} ${Math.round(f.share * 100)}%${d ? "" : " (not drawn: no optics on the sheet yet)"}`;
  });
  return `${name}'s ground: ${parts.join(", ")}; hues from the data sheet's measured optics.`;
}
