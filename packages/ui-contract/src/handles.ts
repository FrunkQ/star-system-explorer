// Handles and strips (ED1, editing design sections 2 and 3): the few true inputs of a world, a star and a system,
// read from the handle sheet (handles.json), and the strip each opens on: a line laid out by the value's own scale,
// with the reference worlds and this system's other worlds as dots. Presentation arithmetic only: every figure on a
// strip is a kernel fact or a catalogued one, said by the units engine, and a moved handle is one set_value.
import sheet from "./handles.json" with { type: "json" };
import { constant } from "./sheet.ts";
import { say, type About, type Said, type SayOptions } from "./figures.ts";
import type { Json } from "./json.ts";
import type { SolSheet } from "./sol.ts";
import { toSI } from "./sol.ts";

export interface HandlePart {
  field: string;
  kind: string;
  what?: string;
  scale: "log" | "linear" | "bar";
  /** The sheet constant the field is held in multiples of ("M_ocean_earth": surface water in Earth oceans). */
  per?: string;
  /** None at all is a value the strip offers (no air, no seas), at its left end. */
  zero?: boolean;
  /** The sign means something else (a negative day spins backwards): the strip lays out the size. */
  signed?: boolean;
}

export interface Handle {
  id: string;
  on: "world" | "star" | "system";
  names: { curious: string; science: string };
  part: HandlePart;
  also?: { field: string; kind: string; label: string };
  refs: string[];
  moon_refs?: string[];
  desks: string[];
  waits?: string;
}

export const HANDLES: readonly Handle[] = (sheet as unknown as { handles: Handle[] }).handles;

/** The handles of a world or a star, in row order. */
export const handlesFor = (on: Handle["on"]): Handle[] => HANDLES.filter((h) => h.on === on);

/** A subject kind as the handle sheet's "on". */
export const handleSubject = (kind: string): Handle["on"] | null => (kind === "star" ? "star" : kind === "planet" || kind === "moon" || kind === "dwarf" ? "world" : null);

/**
 * The units engine's scale class for a body: a star, a moon, a giant planet (ten Earth masses and up, where gas
 * envelopes take over; a display choice, not a class), or a planet.
 */
export function scaleClass(kind: string, mass_kg: number | undefined): string {
  if (kind === "star") return "star";
  if (kind === "moon") return "moon";
  if (mass_kg !== undefined && mass_kg >= 10 * constant("M_earth")) return "giant";
  return "planet";
}

/** A scene subject's scale class, from its kind and (for a planet) its mass fact in Earth masses. */
export function subjectClass(s: { kind: string; facts?: readonly { field: string; value: unknown }[] }): string {
  if (s.kind === "belt" || s.kind === "ring") return "ring";
  if (s.kind === "station" || s.kind === "construct" || s.kind === "craft") return "construct";
  const m = s.facts?.find((f) => f.field === "mass_earth")?.value;
  return scaleClass(s.kind, typeof m === "number" ? m * constant("M_earth") : undefined);
}

export const aboutFor = (h: Handle, cls: string): About => ({ class: cls, ...(h.part.what ? { what: h.part.what } : {}) });

const perOf = (p: { per?: string }): number => (p.per ? constant(p.per) : 1);

/** A fact as the handle's SI value (null when the body has no such fact). */
export function handleValue(h: Handle, facts: Record<string, Json>): number | null {
  const v = facts[h.part.field];
  return typeof v === "number" ? v * perOf(h.part) : null;
}

/** An SI value back as the kernel field's value (Earth oceans for surface water). */
export const fieldValue = (h: Handle, si: number): number => si / perOf(h.part);

/** The handle's figure: "0.52 AU", "31 h, tilt 23.4°", or null where the body has no value. */
export function handleFigure(h: Handle, facts: Record<string, Json>, cls: string, opts: SayOptions = {}): { main: Said; also?: Said } | null {
  const v = handleValue(h, facts);
  if (v === null) return null;
  const main = say(h.part.signed ? Math.abs(v) : v, h.part.kind, aboutFor(h, cls), opts);
  const a = h.also ? facts[h.also.field] : undefined;
  return typeof a === "number" ? { main, also: say(a, h.also!.kind, { class: cls }, opts) } : { main };
}

// ---------------------------------------------------------------------------------------------------------- strips

export interface StripDot {
  label: string;
  si: number;
  /** "reference" for a catalogued Solar System body, "here" for another world of this system. */
  from: "reference" | "here";
}

export interface StripDomain {
  scale: "log" | "linear";
  lo: number;
  hi: number;
  /** The strip's left end is "none" (no air, no seas). */
  zero: boolean;
}

/**
 * The strip's range: where play is interesting for this value, not a limit. It spans the dots and the value with room
 * either side (a factor of four on a log strip), and stretches to take a typed value beyond it.
 */
export function stripDomain(h: Handle, value: number | null, dots: readonly StripDot[]): StripDomain {
  const scale = h.part.scale === "linear" ? "linear" : "log";
  const vs = [...dots.map((d) => Math.abs(d.si)), ...(value !== null ? [Math.abs(value)] : [])].filter((x) => Number.isFinite(x) && (scale === "linear" || x > 0));
  const zero = !!h.part.zero;
  if (!vs.length) return { scale, lo: scale === "log" ? 1 : 0, hi: scale === "log" ? 1e3 : 1, zero };
  const min = Math.min(...vs);
  const max = Math.max(...vs);
  if (scale === "linear") {
    const pad = (max - min || Math.abs(max) || 1) * 0.15;
    return { scale, lo: min - pad, hi: max + pad, zero };
  }
  return { scale, lo: min / 4, hi: max === min ? max * 16 : max * 4, zero };
}

/** Share of the zero end's width on a strip that offers "none". */
const ZERO_GAP = 0.06;

/** Where a value sits along the strip, 0 to 1 (values beyond it are clamped; a typed one stretches it first). */
export function stripPos(d: StripDomain, v: number): number {
  const a = Math.abs(v);
  if (d.zero && a === 0) return 0;
  const g = d.zero ? ZERO_GAP : 0;
  const t = d.scale === "log" ? (Math.log10(Math.max(a, d.lo)) - Math.log10(d.lo)) / (Math.log10(d.hi) - Math.log10(d.lo)) : (v - d.lo) / (d.hi - d.lo);
  return g + (1 - g) * Math.min(1, Math.max(0, t));
}

/** The value at a place along the strip (the size; the caller keeps the sign). */
export function stripValue(d: StripDomain, pos: number): number {
  const g = d.zero ? ZERO_GAP : 0;
  if (d.zero && pos < g / 2) return 0;
  const t = Math.min(1, Math.max(0, (pos - g) / (1 - g)));
  return d.scale === "log" ? 10 ** (Math.log10(d.lo) + t * (Math.log10(d.hi) - Math.log10(d.lo))) : d.lo + t * (d.hi - d.lo);
}

/** The strip widened to hold a value (a typed value past the end stretches the strip). */
export function stretchTo(d: StripDomain, v: number): StripDomain {
  const a = d.scale === "log" ? Math.abs(v) : v;
  if (d.scale === "log" && a === 0) return d;
  if (a < d.lo) return { ...d, lo: d.scale === "log" ? a / 2 : a - (d.hi - d.lo) * 0.1 };
  if (a > d.hi) return { ...d, hi: d.scale === "log" ? a * 2 : a + (d.hi - d.lo) * 0.1 };
  return d;
}

/** One arrow-key step along the strip (Shift for ten times bigger): a hundredth of its width. */
export function stripStep(d: StripDomain, v: number, dir: 1 | -1, big = false): number {
  const p = stripPos(d, v);
  return stripValue(d, Math.min(1, Math.max(0, p + dir * (big ? 0.1 : 0.01))));
}

/**
 * The reference dots for a handle: the catalogued Solar System bodies it names (a moon's own list for a moon), from
 * the base-sol sheet, in SI. A body the sheet lacks, or with no value for the field, is left out.
 */
export function referenceDots(h: Handle, sol: SolSheet | null, cls: string): StripDot[] {
  if (!sol) return [];
  const ids = cls === "moon" && h.moon_refs ? [...h.moon_refs, ...h.refs.slice(0, 2)] : h.refs;
  const out: StripDot[] = [];
  for (const id of ids) {
    const b = sol.bodies.find((x) => x.id === id);
    const x = b?.values[h.part.field];
    if (!b || !x || typeof x.v !== "number") continue;
    const si = x.unit === "earth_oceans" ? x.v * perOf(h.part) : toSI(x, `${id} ${h.part.field}`);
    out.push({ label: b.name, si, from: "reference" });
  }
  return out;
}

export interface ResultRow {
  key: string;
  label: string;
  kind?: string;
  reading?: string;
  yes?: string;
  no?: string;
}

/** The results "what followed" watches, in the sheet's order. */
export const RESULTS: readonly ResultRow[] = (sheet as unknown as { results: ResultRow[] }).results;

/** A world's results from a resolved system: its surface entry flattened to "core.dynamo_now" keys, arrays left out. */
export function resultsOf(surfaces: unknown, address: string): Record<string, Json> {
  const entry = Array.isArray(surfaces) ? (surfaces as { address?: string }[]).find((s) => s?.address === address) : undefined;
  const out: Record<string, Json> = {};
  const walk = (o: Record<string, unknown>, pre: string) => {
    for (const [k, v] of Object.entries(o)) {
      if (Array.isArray(v)) continue;
      if (v && typeof v === "object") walk(v as Record<string, unknown>, `${pre}${k}.`);
      else if (typeof v === "number" || typeof v === "string" || typeof v === "boolean") out[`${pre}${k}`] = v;
    }
  };
  if (entry) walk(entry as Record<string, unknown>, "");
  return out;
}

export interface Followed {
  label: string;
  before: string;
  after: string;
}

/**
 * What followed a change (design section 6, step 5): each watched result that moved (a number by more than half a
 * percent, a label or a yes/no that changed), said by the units engine, in the sheet's order.
 */
export function whatFollowed(before: Record<string, Json>, after: Record<string, Json>, cls: string, opts: SayOptions = {}): Followed[] {
  const out: Followed[] = [];
  for (const r of RESULTS) {
    const b = before[r.key];
    const a = after[r.key];
    if (a === undefined || b === undefined || a === b) continue;
    if (typeof a === "number" && typeof b === "number") {
      const scale = Math.max(Math.abs(a), Math.abs(b));
      if (!(scale > 0 && Math.abs(a - b) / scale > 0.005) || !r.kind) continue;
      const about = { class: cls };
      out.push({ label: r.label, before: say(b, r.kind, about, { ...opts, ...(r.reading ? { reading: r.reading } : {}) }).text, after: say(a, r.kind, about, { ...opts, ...(r.reading ? { reading: r.reading } : {}) }).text });
    } else if (typeof a === "boolean" && typeof b === "boolean") {
      out.push({ label: r.label, before: (b ? r.yes : r.no) ?? String(b), after: (a ? r.yes : r.no) ?? String(a) });
    } else out.push({ label: r.label, before: String(b), after: String(a) });
  }
  return out;
}
