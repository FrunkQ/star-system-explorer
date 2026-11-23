// Ships, stations and sites made from pack templates (J7 in the kernel; V3
// gap 13 on the chart room): the kernel's `templates` list, typed for views,
// and the pure steps a construct desk needs. Nothing names a game: the
// templates, their kinds, facts and words come from whichever packs are
// stacked.
import type { PlayerScene, PlayerSubject } from "./player-scene.ts";

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
