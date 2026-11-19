// Glyph rules for the hex map (X3c): which world fields draw which marks.
// Data, from the pack that configures the map, so another game draws its
// own marks with the same view. Marks are named; the theme draws them with
// its tokens, so no colour lives here.

import type { GridConfig } from "./grid.ts";

/** One world on the map, as the scene gives it: its hex label and fields. */
export interface MapWorld {
  hex: string;
  name: string;
  /** Any fields the glyph rules read: "starport", "gas_giants", "bases", "zone", "uwp"... */
  fields: Record<string, string | number | boolean | string[]>;
  /** Address in the universe, when the world is a system there. */
  address?: string;
}

export interface When {
  field: string;
  is?: string | number | boolean;
  in?: (string | number)[];
  gt?: number;
  /** The field is a list that holds this value. */
  has?: string;
}

export interface MarkRule {
  when?: When;
  mark: string;
  /** Where in the hex: centre, top, bottom, top-left, top-right, bottom-left, bottom-right, ring. */
  at: string;
  /** Text from a field (the starport letter), else none. */
  text_field?: string;
  /** Tone token name the theme maps: "ink", "muted", "accent", "caution", "danger". */
  tone?: string;
}

export interface GlyphRules {
  /** Field shown as the label under the world. */
  label_field: string;
  marks: MarkRule[];
}

export interface HexMapConfig {
  grid: GridConfig;
  glyphs: GlyphRules;
}

export interface Mark {
  mark: string;
  at: string;
  text?: string;
  tone: string;
  rule: number;
}

export function holds(w: MapWorld, c: When | undefined): boolean {
  if (!c) return true;
  const v = w.fields[c.field];
  if (v === undefined) return false;
  if (c.is !== undefined && v !== c.is) return false;
  if (c.in !== undefined && !c.in.includes(v as string | number)) return false;
  if (c.gt !== undefined && !(typeof v === "number" && v > c.gt)) return false;
  if (c.has !== undefined && !(Array.isArray(v) && v.includes(c.has))) return false;
  return true;
}

/** The marks a world draws, in rule order. Rules on the same spot: the first that holds wins. */
export function marksFor(w: MapWorld, g: GlyphRules): Mark[] {
  const taken = new Set<string>();
  const out: Mark[] = [];
  g.marks.forEach((r, i) => {
    if (!holds(w, r.when)) return;
    const key = r.at === "ring" ? `ring:${i}` : r.at;
    if (taken.has(key)) return;
    taken.add(key);
    const t = r.text_field ? w.fields[r.text_field] : undefined;
    out.push({ mark: r.mark, at: r.at, text: t === undefined ? undefined : String(t), tone: r.tone ?? "ink", rule: i });
  });
  return out;
}

/** Where a spot sits relative to the hex centre, as fractions of the hex size. */
export const SPOTS: Record<string, [number, number]> = {
  centre: [0, 0],
  top: [0, -0.55],
  bottom: [0, 0.55],
  "top-left": [-0.45, -0.3],
  "top-right": [0.45, -0.3],
  "bottom-left": [-0.45, 0.3],
  "bottom-right": [0.45, 0.3],
  ring: [0, 0],
};
