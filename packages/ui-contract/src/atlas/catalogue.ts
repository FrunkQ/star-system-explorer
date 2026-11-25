// The Atlas's catalogue of fact fields (atlas design section 12.6, check 2): which fields an edition may name, and how
// each numeric one is said through the figures door (figures.ts `say`). The scene carries SI; this table only says
// which quantity a field is and what it is about, so the units engine picks AU for an orbit and Earth masses for a
// planet. Nothing here formats a number.
import { say, type About, type FigureKind } from "../figures.ts";
import type { Depth } from "../depth.ts";
import { describeFact, fact, wordsFor, type PlayerFact, type PlayerSubject, type Vocab } from "../player-scene.ts";

export interface FieldFigure {
  kind: FigureKind;
  /** Which figure of the subject it is ("orbit", "size", "mass", "light"), for the established unit. */
  what?: string;
}

/** Numeric fields of a player scene and the quantity each one is. */
export const FIGURE_FIELDS: Readonly<Record<string, FieldFigure>> = {
  luminosity_w: { kind: "power", what: "light" },
  mass_kg: { kind: "mass", what: "mass" },
  radius_m: { kind: "length", what: "size" },
  "orbit.semi_major_axis_m": { kind: "length", what: "orbit" },
  "orbit.period_s": { kind: "time" },
  "atmosphere.pressure_pa": { kind: "pressure" },
  "surface.temp_mean_k": { kind: "temperature" },
  "ocean.fraction": { kind: "share" },
  "interior.core_fraction": { kind: "share" },
};

/** Fields that are counts: said as a whole number, no unit. */
export const COUNT_FIELDS: readonly string[] = ["settlements.count"];

/** Text fields an edition may name (their words come from the vocabulary layer). */
export const TEXT_FIELDS: readonly string[] = ["name", "kind", "class", "size_class", "colour", "spectral", "atmosphere.composition", "surface.class", "hazards", "resources"];

/** True when an edition may name this field: catalogued here, or a story line. */
export function isCatalogued(field: string): boolean {
  return field in FIGURE_FIELDS || COUNT_FIELDS.includes(field) || TEXT_FIELDS.includes(field) || field.startsWith("story.");
}

/** The figures engine's scale class for a subject, from what the audience knows of its kind and class. */
export function classOf(s: PlayerSubject): string {
  const k = fact(s, "kind")?.value;
  const c = fact(s, "class")?.value;
  if (k === "star") return "star";
  if (k === "moon") return "moon";
  if (k === "station" || k === "ship" || k === "construct") return "construct";
  if (typeof c === "string" && /giant|jupiter|neptune|puff|brown-dwarf/.test(c)) return "giant";
  return "planet";
}

/** One fact as the Atlas shows it: its state, its text, and for a figure what the figures door was asked. */
export type FactShown =
  | { state: "known"; text: string; figure?: { si: number; kind: FigureKind; about: About } }
  | { state: "band"; text: string; figure: { si: [number, number]; kind: FigureKind; about: About } }
  | { state: "gap" };

export interface SayIn {
  vocab: Vocab;
  depth?: Depth;
}

/** Whole numbers for counts (no unit, no rounding to say). */
function countText(n: number): string {
  return Number.isInteger(n) ? String(n) : String(Math.round(n));
}

/**
 * A fact for the Atlas. A fact the audience has not been given, whether the scene carries a placeholder for it or
 * carries nothing, is the same `gap`: the Atlas never reads a placeholder, because the kernel only writes one where a
 * value exists (design section 5: a gap is drawn from the template, never from the value).
 */
export function showFact(s: PlayerSubject, field: string, opts: SayIn): FactShown {
  const f: PlayerFact | undefined = fact(s, field);
  if (!f || (f.value === undefined && !f.band)) return { state: "gap" };
  const fig = FIGURE_FIELDS[field];
  const about: About = { class: classOf(s), ...(fig?.what ? { what: fig.what } : {}) };
  const sayOpts = { depth: opts.depth ?? "science" };
  if (fig && f.band) {
    const lo = say(f.band[0], fig.kind, about, sayOpts);
    const hi = say(f.band[1], fig.kind, about, { ...sayOpts, hold: lo.rung });
    return { state: "band", text: `${lo.number} to ${hi.text}`, figure: { si: [f.band[0], f.band[1]], kind: fig.kind, about } };
  }
  if (fig && typeof f.value === "number") {
    return { state: "known", text: say(f.value, fig.kind, about, sayOpts).text, figure: { si: f.value, kind: fig.kind, about } };
  }
  if (COUNT_FIELDS.includes(field) && typeof f.value === "number") return { state: "known", text: countText(f.value) };
  if (typeof f.value === "number") {
    // An uncatalogued number: the vocabulary's own reading. No shipped template names one (a test holds this).
    return { state: "known", text: describeFact(opts.vocab, f).text };
  }
  return { state: "known", text: describeFact(opts.vocab, f).text };
}

/** The label a line has: the edition's `line.<field>` word, else the vocabulary's. */
export function lineLabel(labels: Record<string, string>, vocab: Vocab, field: string): string {
  return labels[`line.${field}`] ?? wordsFor(vocab, field).label;
}
