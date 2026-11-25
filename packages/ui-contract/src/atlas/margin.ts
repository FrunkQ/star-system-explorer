// Margin notes (track AT, AT7; atlas design section 6 and 12.5): the Guide's jokes as data. A note belongs to a kind
// of page and fits by a condition on what the audience knows, never on the truth, so a joke can never be a leak. Real
// figures go in through slots, said by the figures door with the audience's own value; a note may not type a figure.
// The pick is seeded by the page and the edition, so every player sees the same note and print matches screen.
import type { Json } from "../json.ts";
import { fact, type PlayerScene, type PlayerSubject } from "../player-scene.ts";
import { seedOf } from "./ascii.ts";
import { isCatalogued, showFact, type SayIn } from "./catalogue.ts";
import { editionWord, PAGE_KINDS, type Edition } from "./edition.ts";
import { typedFigures } from "./loader.ts";
import type { AtlasPage } from "./page.ts";
import guideNotes from "./notes/guide.notes.json" with { type: "json" };

export const MARGIN_SCHEMA = "sse.margin-notes/1";
/** Where a note may sit: a kind of page, ships (later pages), or anywhere. */
export const NOTE_ABOUT: readonly string[] = [...PAGE_KINDS, "ship", "any"];
export const NOTE_MAX_LENGTH = 200;
/**
 * What a line is for (Alex, 25 Nov: "categorised by use case ... tag sensitive to match the scene"). `page` is the
 * Atlas margin; the rest are the one-liners a kit shows at the top and bottom of a screen, chosen by the situation.
 */
export const NOTE_USES = ["page", "arrival", "travel", "danger", "settled", "empty", "waiting", "ship"] as const;
export type NoteUse = (typeof NOTE_USES)[number];
/** The uses a screen's banners draw on in each situation of an interface kit. */
export const SITUATION_USES: Readonly<Record<string, readonly NoteUse[]>> = {
  sector: ["travel", "empty"],
  system: ["arrival", "danger", "settled", "empty"],
  world: ["arrival", "danger", "settled", "empty"],
  ship: ["ship", "travel"],
  landing: ["danger", "arrival"],
  holding: ["waiting"],
};

/** The small condition language (design 6.2): a fact at least, at most, between, equal to, present; a tag; all; any. Numbers are SI. */
export type NoteCondition =
  | { fact: string; at_least?: number; at_most?: number; between?: [number, number]; equals?: Json; present?: true }
  | { tag: string }
  | { all: NoteCondition[] }
  | { any: NoteCondition[] };

export interface MarginNote {
  id: string;
  about: string;
  when?: NoteCondition;
  /** The words; `{field}` slots are filled by the figures door with the audience's value. */
  text: string;
  /** How keen it is to be picked (default 1). */
  weight?: number;
  /** What it is for (`NOTE_USES`); left out, a ship line is for ships and travel and any other is for the page. */
  uses?: NoteUse[];
}

/** A note's uses, with the default for one that does not say. */
export function usesOf(n: MarginNote): readonly NoteUse[] {
  return n.uses ?? (n.about === "ship" ? ["ship", "travel"] : ["page"]);
}

/**
 * True when a text's typed figures are plainly a joke: all percentages, and together (or alone) over 100%. Alex,
 * 25 Nov: "have them as the proportion that adds up over 100% - then you know you can rely on what's left".
 */
export function obviousJoke(figures: readonly string[]): boolean {
  if (!figures.length) return false;
  const pcts = figures.map((f) => (/^\s*[\d.]+\s*%$/.test(f) ? Number(f.replace("%", "")) : NaN));
  if (pcts.some((x) => !Number.isFinite(x))) return false;
  return pcts.reduce((a, x) => a + x, 0) > 100 || pcts.some((x) => x > 100);
}

export interface NoteSet {
  schema: typeof MARGIN_SCHEMA;
  id: string;
  name: string;
  notes: MarginNote[];
}

/** The note sets shipped with the base Atlas: the Guide's (Alex's own list and the slotted notes). */
export const GUIDE_NOTES = guideNotes as unknown as NoteSet;
export const SHIPPED_NOTE_SETS: readonly NoteSet[] = [GUIDE_NOTES];

const known = (s: PlayerSubject, field: string) => {
  const f = fact(s, field);
  return f && (f.value !== undefined || f.band) ? f : undefined;
};

/** True when the subject, as the audience knows it, meets the condition. An unknown fact meets nothing. */
export function meets(s: PlayerSubject, c: NoteCondition): boolean {
  if ("all" in c) return c.all.every((x) => meets(s, x));
  if ("any" in c) return c.any.some((x) => meets(s, x));
  if ("tag" in c) return s.tags.includes(c.tag);
  const f = known(s, c.fact);
  if (!f) return false;
  if (c.equals !== undefined) {
    const v = f.value;
    if (Array.isArray(v) ? !v.includes(c.equals) : v !== c.equals) return false;
  }
  const range: [number, number] | null = f.band ? [f.band[0], f.band[1]] : typeof f.value === "number" ? [f.value, f.value] : null;
  const numeric = c.at_least !== undefined || c.at_most !== undefined || c.between !== undefined;
  if (numeric && !range) return false;
  // A band meets a bound only when all of it does: the Guide does not joke about what it is not sure of.
  if (c.at_least !== undefined && range![0] < c.at_least) return false;
  if (c.at_most !== undefined && range![1] > c.at_most) return false;
  if (c.between && (range![0] < c.between[0] || range![1] > c.between[1])) return false;
  return true;
}

/** The fields a condition reads. */
export function conditionFields(c: NoteCondition | undefined): string[] {
  if (!c) return [];
  if ("all" in c) return c.all.flatMap(conditionFields);
  if ("any" in c) return c.any.flatMap(conditionFields);
  if ("tag" in c) return [];
  return [c.fact];
}

const SLOT = /\{([^}]+)\}/g;

/** The note's words with its slots said for this subject and audience, or null when a slot's fact is not known. */
export function fillNote(n: MarginNote, s: PlayerSubject, opts: SayIn): string | null {
  let ok = true;
  const text = n.text.replace(SLOT, (_m, field: string) => {
    if (field === "name") {
      const v = known(s, "name")?.value;
      if (typeof v !== "string") ok = false;
      return typeof v === "string" ? v : "";
    }
    const shown = showFact(s, field, opts);
    if (shown.state === "gap") ok = false;
    return shown.state === "gap" ? "" : shown.text;
  });
  return ok ? text : null;
}

/** A small deterministic generator from a seed (mulberry32): no clock, no Math.random. */
function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export interface MarginOptions extends SayIn {
  /** Note ids the GM has turned away on this page ("Not this one"). */
  exclude?: readonly string[];
}

/**
 * The notes for a page: those that fit its kind and the audience's facts, picked by weight from a seed of the page's
 * address and the edition's id, at most `max`. The same page in the same edition with the same knowledge always gives
 * the same notes.
 */
export function pickNotes(scene: PlayerScene, page: AtlasPage, ed: Edition, sets: readonly NoteSet[], opts: MarginOptions): { id: string; text: string }[] {
  if (!ed.margin) return [];
  const s = scene.subjects.find((x) => x.address === page.address);
  if (!s) return [];
  const want = new Set(ed.margin.sets);
  const fitting: { id: string; text: string; weight: number }[] = [];
  for (const set of sets) {
    if (!want.has(set.id)) continue;
    for (const n of set.notes) {
      if (n.about !== "any" && n.about !== page.kind) continue;
      if (!usesOf(n).includes("page")) continue;
      if (opts.exclude?.includes(n.id)) continue;
      if (n.when && !meets(s, n.when)) continue;
      const text = fillNote(n, s, opts);
      if (text !== null) fitting.push({ id: n.id, text, weight: n.weight ?? 1 });
    }
  }
  return weighted(fitting, seedOf(`${page.address}|${ed.id}`), ed.margin.max);
}

/** Up to `max` of the fitting lines, picked by weight from the seed, never the same one twice. */
function weighted(fitting: { id: string; text: string; weight: number }[], seed: number, max: number): { id: string; text: string }[] {
  const next = rng(seed);
  const out: { id: string; text: string }[] = [];
  while (out.length < max && fitting.length) {
    const total = fitting.reduce((a, n) => a + n.weight, 0);
    let r = next() * total;
    let i = 0;
    while (i < fitting.length - 1 && r >= fitting[i]!.weight) r -= fitting[i++]!.weight;
    const [hit] = fitting.splice(i, 1);
    out.push({ id: hit!.id, text: hit!.text });
  }
  return out;
}

export interface BannerOptions extends MarginOptions {
  /** Mixed into the seed so a screen's lines change when its owner wants (a new day, a new scene); never the clock. */
  seed?: string;
}

/**
 * The one-liners for the top and bottom of a screen (SSE3's guide tips, by use case): lines whose use fits the
 * situation, whose kind fits the subject (or "any"), and whose condition the audience's facts and tags meet. With no
 * subject (a holding screen) only lines that need no fact can show. Two different lines, seeded and repeatable.
 */
export function pickBanners(scene: PlayerScene, address: string | null, situation: string, sets: readonly NoteSet[], opts: BannerOptions): { top?: { id: string; text: string }; bottom?: { id: string; text: string } } {
  const uses = SITUATION_USES[situation] ?? [];
  const s = address === null ? undefined : scene.subjects.find((x) => x.address === address);
  const kind = s ? String(fact(s, "kind")?.value ?? "") : "";
  const fitting: { id: string; text: string; weight: number }[] = [];
  for (const set of sets) {
    for (const n of set.notes) {
      if (!usesOf(n).some((u) => uses.includes(u))) continue;
      if (opts.exclude?.includes(n.id)) continue;
      if (n.about !== "any" && n.about !== situation && !(s && n.about === kind) && !(situation === "ship" && n.about === "ship")) continue;
      if (!s) {
        if (n.when || /\{[^}]+\}/.test(n.text)) continue;
        fitting.push({ id: n.id, text: n.text, weight: n.weight ?? 1 });
        continue;
      }
      if (n.when && !meets(s, n.when)) continue;
      const text = fillNote(n, s, opts);
      if (text !== null) fitting.push({ id: n.id, text, weight: n.weight ?? 1 });
    }
  }
  const [top, bottom] = weighted(fitting, seedOf(`${situation}|${address ?? ""}|${opts.seed ?? ""}`), 2);
  return { ...(top ? { top } : {}), ...(bottom ? { bottom } : {}) };
}

/** The page with its margin notes (and the one line saying they are opinions); unchanged when none fit. */
export function withMargin(page: AtlasPage, scene: PlayerScene, ed: Edition, sets: readonly NoteSet[], opts: MarginOptions): AtlasPage {
  const notes = pickNotes(scene, page, ed, sets, opts);
  if (!notes.length) return page;
  return {
    ...page,
    blocks: [
      ...page.blocks,
      { type: "margin", signature: ed.margin!.signature || editionWord(ed, "margin.signature", "Margin note"), notes },
      { type: "note", text: editionWord(ed, "note.margin", "Margin notes are opinions and are not facts.") },
    ],
  };
}

/** Why a note set will not load, as sentences that say what to change; empty when it loads (design 12.5, 12.6). */
export function checkNoteSet(doc: unknown): string[] {
  const d = doc as NoteSet;
  if (!d || typeof d !== "object") return ["A note set is a JSON object."];
  if (d.schema !== MARGIN_SCHEMA) return [`A note set's schema is "${MARGIN_SCHEMA}".`];
  const out: string[] = [];
  for (const k of ["id", "name"] as const) if (typeof d[k] !== "string" || !d[k]) out.push(`A note set needs a ${k}.`);
  if (!Array.isArray(d.notes)) return [...out, "A note set has a list of notes."];
  const seen = new Set<string>();
  for (const n of d.notes) {
    const at = `The note ${typeof n?.id === "string" ? n.id : "(no id)"}`;
    if (!n || typeof n.id !== "string" || !n.id) {
      out.push("Every note needs an id.");
      continue;
    }
    if (seen.has(n.id)) out.push(`${at} appears twice; give each note its own id.`);
    seen.add(n.id);
    if (!NOTE_ABOUT.includes(n.about)) out.push(`${at} is about "${n.about}"; use one of ${NOTE_ABOUT.join(", ")}.`);
    if (typeof n.text !== "string" || !n.text.trim()) {
      out.push(`${at} has no words.`);
      continue;
    }
    if (n.text.length > NOTE_MAX_LENGTH) out.push(`${at} is ${n.text.length} characters; keep it under ${NOTE_MAX_LENGTH} so it fits a margin.`);
    if (n.weight !== undefined && !(typeof n.weight === "number" && n.weight > 0)) out.push(`${at} has a weight that is not a positive number.`);
    if (n.uses !== undefined && (!Array.isArray(n.uses) || !n.uses.length || n.uses.some((u) => !(NOTE_USES as readonly string[]).includes(u)))) out.push(`${at} has uses that are not a list of ${NOTE_USES.join(", ")}.`);
    const typed = typedFigures(n.text.replace(SLOT, ""));
    if (!obviousJoke(typed)) for (const f of typed) out.push(`${at} states a figure ("${f}"). Put the fact in a slot, such as {ocean.fraction}, so it is the audience's own value. (Percentages that plainly cannot be true, over 100% together, may stay as a joke.)`);
    const reads = conditionFields(n.when);
    for (const f of reads) if (!isCatalogued(f)) out.push(`${at} tests "${f}", which is not a fact the Atlas knows.`);
    for (const m of n.text.matchAll(SLOT)) {
      const f = m[1]!;
      if (f === "name") continue;
      if (!isCatalogued(f)) out.push(`${at} has a slot {${f}}, which is not a fact the Atlas knows.`);
      else if (!reads.includes(f)) out.push(`${at} has a slot {${f}} that its "when" does not test; add a condition on ${f} so the note only fits where it is known.`);
    }
  }
  return out;
}
