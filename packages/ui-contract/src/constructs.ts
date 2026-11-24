// Ships, stations and sites made from pack templates (J7 in the kernel; V3
// gap 13 on the chart room): the kernel's `templates` list, typed for views,
// and the pure steps a construct desk needs. Nothing names a game: the
// templates, their kinds, facts and words come from whichever packs are
// stacked.
import type { PlayerScene, PlayerSubject } from "./player-scene.ts";
import type { HistoryEvent } from "./session.ts";
import type { Json } from "./json.ts";

export interface ConstructTemplate {
  id: string;
  kind: string;
  label: string;
  facts?: Record<string, number | string | boolean>;
  kept?: Record<string, number | string | boolean>;
  tags?: string[];
  programs?: unknown[];
  explain?: string;
  pack: string;
  version: string;
}

export interface Created {
  address: string;
  event_id: string;
  kept: string[];
  state_hash: string;
  record_hash: string;
}

/** Words for a template kind: "ship" becomes "Ships". A pack's own kind keeps its word, capitalised and made plural. */
export function kindWords(kind: string): string {
  const w = kind.replace(/[_-]+/g, " ");
  const plural = /s$/.test(w) ? w : /y$/.test(w) ? `${w.slice(0, -1)}ies` : `${w}s`;
  return plural.charAt(0).toUpperCase() + plural.slice(1);
}

/** Templates grouped by kind, kinds in first-seen order, each group in label order. */
export function byKind(templates: readonly ConstructTemplate[]): { kind: string; words: string; templates: ConstructTemplate[] }[] {
  const out = new Map<string, ConstructTemplate[]>();
  for (const t of templates) (out.get(t.kind) ?? out.set(t.kind, []).get(t.kind)!).push(t);
  return [...out].map(([kind, ts]) => ({ kind, words: kindWords(kind), templates: [...ts].sort((a, b) => (a.label < b.label ? -1 : a.label > b.label ? 1 : 0)) }));
}

/** The kind an address names last ("u:k/sys:1/body:3/ship:2" is "ship"). */
export function kindOf(address: string): string {
  const last = address.split("/").pop() ?? "";
  return last.includes(":") ? last.slice(0, last.indexOf(":")) : "";
}

/**
 * The constructs a scene holds: subjects whose own kind is one the templates
 * make (or tagged `construct`), not retired, in address order.
 */
export function constructsIn(scene: PlayerScene, kinds: readonly string[]): PlayerSubject[] {
  return scene.subjects
    .filter((s) => (kinds.includes(kindOf(s.address)) || s.tags.includes("construct")) && !s.facts.some((f) => f.field === "retired" && f.value === true))
    .sort((a, b) => (a.address < b.address ? -1 : a.address > b.address ? 1 : 0));
}

/** A subject's fact value by field. */
export function factOf(s: PlayerSubject, field: string): unknown {
  return s.facts.find((f) => f.field === field)?.value;
}

/** The template a construct came from (the kernel writes `template.id` and `template.pack` on it). */
export function templateOf(s: PlayerSubject, templates: readonly ConstructTemplate[]): ConstructTemplate | undefined {
  const id = factOf(s, "template.id");
  const pack = factOf(s, "template.pack");
  return typeof id === "string" ? templates.find((t) => t.id === id && (pack === undefined || t.pack === pack)) : undefined;
}

/** Fields the kernel writes for its own bookkeeping, kept out of a construct's card below Engine depth. */
export const BOOKKEEPING = ["template.id", "template.pack", "template.version", "template.programs", "retired"];

// ---- The ship's log (V3 gap 16): a view of the campaign record, so it cannot drift from what happened ----

/** One kept value's change, as the kernel's `record` gives it. */
export interface KeptChange {
  event: string;
  at_s: number;
  value?: Json | null;
  writer: string;
  reason?: string;
  delta?: number;
}
export interface RecordRead {
  address: string;
  notes: { id: string; at_s: number; text: string; by: string; tags?: string[]; audience?: Json }[];
  kept: { kind: string; value: Json; describes?: string; unit?: string; kept_by?: string; history: KeptChange[] }[];
  actors: { address: string; name: string; kind: string }[];
}
/** One line of the log. `who` is the writer ("gm", a template, a pack rule); `why` the reason given, if any. */
export interface ShipLogEntry {
  at_yr: number;
  what: "made" | "edited" | "retired" | "kept" | "note" | "event" | "aboard";
  text: string;
  who: string;
  why?: string;
  /** The kept kind or event kind, for Engine depth. */
  key: string;
}

/** Words for who wrote an entry: the GM, a template, or a pack's rule. */
export function writerWords(w: string): string {
  if (w === "gm") return "the GM";
  if (w.startsWith("template.")) return `template ${w.slice(9).replace(/_/g, " ")}`;
  if (w.startsWith("effect.")) return `rule ${w.slice(7)}`;
  return w;
}

/**
 * The log of one construct, oldest first: its universe events (made, edited, taken out of play), every change of a
 * kept value with its writer and reason, and every note at it. Built from the record and the history alone, so it
 * says only what happened. `label` gives a kept kind its words; `yr` turns record seconds into campaign years.
 */
export function shipLog(
  history: HistoryEvent[],
  record: RecordRead | null,
  yr: (at_s: number) => number,
  label: (kind: string) => string = (k) => k,
): ShipLogEntry[] {
  const out: ShipLogEntry[] = [];
  for (const e of history) {
    const what = e.kind === "template.create" ? "made" : e.kind === "template.retire" ? "retired" : e.kind === "authored.set" || e.kind === "authored.batch" ? "edited" : "event";
    const text = what === "made" ? "Made" : what === "retired" ? "Taken out of play" : what === "edited" ? "Facts edited" : e.kind;
    out.push({ at_yr: e.at_yr, what, text, who: e.evidence === "authored" ? "the GM" : e.evidence, key: e.kind });
  }
  for (const k of record?.kept ?? []) {
    k.history.forEach((c, i) => {
      // The value a template set when the object was made is part of "Made"; later changes are the log.
      if (i === 0 && c.writer.startsWith("template.")) return;
      const v = c.value === null || c.value === undefined ? "cleared" : typeof c.value === "number" ? String(Math.round(c.value * 1000) / 1000) : String(c.value);
      const d = c.delta !== undefined && c.delta !== 0 ? ` (${c.delta > 0 ? "+" : ""}${Math.round(c.delta * 1000) / 1000})` : "";
      out.push({ at_yr: yr(c.at_s), what: "kept", text: `${label(k.kind)}: ${v}${d}`, who: writerWords(c.writer), ...(c.reason ? { why: c.reason } : {}), key: k.kind });
    });
  }
  for (const n of record?.notes ?? []) out.push({ at_yr: yr(n.at_s), what: "note", text: n.text, who: writerWords(n.by), key: "note" });
  const rank: Record<ShipLogEntry["what"], number> = { made: 0, edited: 1, event: 1, kept: 2, aboard: 2, note: 3, retired: 4 };
  return out.sort((a, b) => a.at_yr - b.at_yr || rank[a.what] - rank[b.what]);
}
