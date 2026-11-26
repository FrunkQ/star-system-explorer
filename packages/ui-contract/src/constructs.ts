// Ships, stations and sites made from pack templates (J7 in the kernel; V3
// gap 13 on the chart room): the kernel's `templates` list, typed for views,
// and the pure steps a construct desk needs. Nothing names a game: the
// templates, their kinds, facts and words come from whichever packs are
// stacked.
import type { PlayerScene, PlayerSubject } from "./player-scene.ts";
import type { HistoryEvent } from "./session.ts";
import type { Json } from "./json.ts";
import { constant } from "./sheet.ts";

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
  what: "made" | "edited" | "retired" | "kept" | "note" | "event" | "aboard" | "flight";
  text: string;
  who: string;
  why?: string;
  /** The kept kind or event kind, for Engine depth. */
  key: string;
}

/** A number, word or yes/no as the log writes it; anything bigger (a design, a picture) is just "changed". */
function factWords(v: unknown): string | null {
  if (v === undefined || v === null) return null;
  if (typeof v === "number") return String(Math.round(v * 1000) / 1000);
  if (typeof v === "boolean") return v ? "yes" : "no";
  if (typeof v === "string") {
    // The picture a construct opens on (views/pictures): "picture:0" is the GM's first own picture.
    const own = /^picture:(\d+)$/.exec(v);
    if (own) return `picture ${Number(own[1]) + 1}`;
    return v.length <= 60 && !v.startsWith("data:") ? v : null;
  }
  return null;
}

/** One edit's line: "Changed: Jump drive 2 → 3; Name Kestrel → Kestrel II; Design changed". */
export function factEditWords(changes: NonNullable<HistoryEvent["changes"]>, was: ReadonlyMap<string, unknown>, label: (field: string) => string = (f) => f): string {
  const parts = changes.map((c) => {
    const name = label(c.field);
    if (c.unset) return `${name} cleared`;
    const to = c.large ? null : factWords(c.value);
    const from = factWords(was.get(c.field));
    if (to === null) return `${name} changed`;
    return from === null || from === to ? `${name} ${to}` : `${name} ${from} → ${to}`;
  });
  return `Changed: ${parts.join("; ")}`;
}

/** The order of entries at the same moment: made first, taken out of play last. */
export const LOG_RANK: Record<ShipLogEntry["what"], number> = { made: 0, edited: 1, event: 1, flight: 1, kept: 2, aboard: 2, note: 3, retired: 4 };

const DAY_S = constant("day");

/** One event of a construct's flight log (kernel `flight_log`, sse-nav `LogEvent`): its kind's own fields flattened in. */
type FlightEvent = { at_s: number; kind: string; mark?: string; why?: string; writer?: string } & Record<string, unknown>;

/**
 * The ship's flights for its log (MC7, Alex 24 Nov: "the log record all flights"), from its flight log on the record
 * (the travel track's V8): one line per voyage, "Flew Tarn to Regina (12 days)" on arrival, or "Left Tarn" while it
 * is under way; and launches, burns, flybys, landings, entries, splits, joins, refuels, loads and unloads, each on
 * its own date. What was planned or estimated, not recorded, says so beside the reason.
 */
export function flightLines(log: Json | null | undefined, yr: (at_s: number) => number, place: (address: string) => string): ShipLogEntry[] {
  const events = ((log as { events?: FlightEvent[] } | null)?.events ?? []).filter((e) => e && typeof e.at_s === "number").sort((a, b) => a.at_s - b.at_s);
  const out: ShipLogEntry[] = [];
  // Departures not yet arrived, newest last: a planned voyage can still be under way when the next one is planned
  // (an arrival closes the newest open departure).
  const open: FlightEvent[] = [];
  const placeOf = (e: FlightEvent) => (typeof e.place_id === "string" ? place(e.place_id) : null);
  const said = (e: FlightEvent, text: string, at_s = e.at_s) => {
    const marks = [e.mark && e.mark !== "recorded" ? e.mark : null, typeof e.why === "string" ? e.why : null].filter(Boolean);
    out.push({ at_yr: yr(at_s), what: "flight", text, who: writerWords(e.writer ?? "gm"), ...(marks.length ? { why: marks.join(": ") } : {}), key: `flight.${e.kind === "stop" ? String(e.type) : e.kind}` });
  };
  const num = (v: unknown, d = 1) => (typeof v === "number" ? String(Math.round(v * 10 ** d) / 10 ** d) : "?");
  for (const e of events) {
    const at = typeof e.place_id === "string" ? place(e.place_id) : null;
    if (e.kind === "stop" && e.type === "depart") {
      open.push(e);
      continue;
    }
    if (e.kind === "stop" && e.type === "arrive") {
      const left = open.pop() ?? null;
      const from = left ? placeOf(left) : null;
      const days = left ? (e.at_s - left.at_s) / DAY_S : null;
      said(e, `Flew ${from ? `${from} to ` : "to "}${at ?? "somewhere"}${days !== null ? ` (${days < 2 ? `${num(days * 24, 0)} hours` : `${num(days, 0)} days`})` : ""}`);
      continue;
    }
    const text =
      e.kind === "launch" ? "Launched"
      : e.kind === "burn" ? `Burn of ${num((e.dv_m_s as number) / 1000, 2)} km/s${typeof e.propellant_t === "number" && e.propellant_t > 0 ? `, ${num(e.propellant_t)} t of propellant` : ""}`
      : e.kind === "flyby" ? `Flew by ${typeof e.body === "string" ? place(e.body) : "a world"}`
      : e.kind === "entry" ? "Entered the air"
      : e.kind === "landing" ? "Landed"
      : e.kind === "split" ? `Split: ${Array.isArray(e.into) ? e.into.map((x) => place(String(x))).join(", ") : "parts"} went their own way`
      : e.kind === "join" ? `Joined ${typeof e.with === "string" ? place(e.with) : "another craft"}`
      : e.kind === "stop" && e.type === "refuel" ? `Refuelled${at ? ` at ${at}` : ""}`
      : e.kind === "stop" && (e.type === "load" || e.type === "unload") ? `${e.type === "load" ? "Loaded" : "Unloaded"}${typeof e.tonnes === "number" ? ` ${num(e.tonnes)} t` : ""}${typeof e.resource_key === "string" ? ` of ${e.resource_key.replace(/[._-]/g, " ")}` : ""}${at ? ` at ${at}` : ""}`
      : e.kind === "stop" ? `${String(e.type ?? "stop").replace(/^./, (c) => c.toUpperCase())}${at ? ` at ${at}` : ""}`
      : e.kind;
    said(e, text);
  }
  for (const left of open) said(left, `Left ${placeOf(left) ?? "port"}, under way`);
  return out;
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
  // Each fact's value so far, so an edit (a refit, a rename) says what it changed from and to.
  const was = new Map<string, unknown>();
  for (const e of [...history].sort((a, b) => a.at_yr - b.at_yr)) {
    const what = e.kind === "template.create" ? "made" : e.kind === "template.retire" ? "retired" : e.kind === "authored.set" || e.kind === "authored.batch" ? "edited" : "event";
    const said = what === "edited" && e.changes?.length ? factEditWords(e.changes, was, label) : null;
    for (const c of e.changes ?? []) was.set(c.field, c.unset || c.large ? undefined : c.value);
    const text = what === "made" ? "Made" : what === "retired" ? "Taken out of play" : what === "edited" ? (said ?? "Facts edited") : e.kind;
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
  const rank: Record<ShipLogEntry["what"], number> = LOG_RANK;
  return out.sort((a, b) => a.at_yr - b.at_yr || rank[a.what] - rank[b.what]);
}
