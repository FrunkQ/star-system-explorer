// Roll call (track SH, design `handover/design/SSE4-spaceflight-history-design.md`, accepted 25 Nov 2026): who was in
// flight on a day, aboard what, and where each craft was. A pure look-up over two things that already exist: flight
// logs (dated pieces: in orbit, on a course, landed, docked, gone) and stays (who was aboard which construct, from when
// to when). Nothing is simulated to answer it, and nothing here feeds physics or the state hash.
//
// It is general (ruling 2): a campaign's own people and logs give its fleet history (`sourceFromManifest`), and the
// "Sol today" pack's crews and logs give the real history of spaceflight (`sourceFromCrews`). There is no real-history
// mode: both are one `RollSource`.
//
// Rules of restraint (design 8.1, standing): names are shown as the record gives them, nothing is invented, and the
// words are plain. Every row keeps its source's mark (recorded or estimated) and why, so an estimated row is drawn in
// pencil and says so.
import type { Json } from "./json.ts";
import type { AboardEntry } from "./manifest.ts";
import { PERSON } from "./manifest.ts";
import { say } from "./figures.ts";
import { constant } from "./sheet.ts";

// ---- What it reads ----

/** A person who was ever aboard anything. Flight facts only (ruling 9). */
export interface RollPerson {
  id: string;
  /** As the record credits them. */
  name: string;
  /** Kind of being, never assumed human (MC ruling): "human", "dog", "chimpanzee"... */
  being?: string;
  /** Who they flew for, as text. */
  flew_for?: string;
}

/** One stay aboard a construct: from when to when. `to_s` null while still aboard. */
export interface RollStay {
  person: string;
  from_s: number;
  to_s: number | null;
  /** The construct's log id (or address); null where no log is held for it. */
  on: string | null;
  /** The craft as the source names it, for when there is no log. */
  craft: string;
  role?: string;
  mark: "recorded" | "estimated";
  why?: string;
  src?: string;
  /**
   * True when the source says the stay is a flight (the pack's legs). False or absent: whether the person is in
   * flight is read from the construct's log at the time (a campaign ship landed on a world is not in flight).
   */
  flight?: boolean;
  /** A short hop that went up and came down; with its highest point when the source has one (null: not recorded). */
  suborbital?: boolean;
  peak_km?: number | null;
  peak_why?: string;
  /** False when the flight never reached space (the source says why): listed on its day, in flight, never counted. */
  reached_space?: false;
  reached_why?: string;
}

/** One piece of a flight log (the kernel's `record.flight` shape, as the pack writes it). */
export interface RollPiece {
  from_s: number;
  to_s?: number;
  state: { kind: string; around?: string; body?: string; to?: string; lat_deg?: number; lon_deg?: number; how?: string; course?: Json };
  mark?: string;
  src?: string;
  why?: string;
}

export interface RollLog {
  construct: string;
  name: string;
  /** The catalogue id other logs dock to ("gcat:S16609"), when it has one. */
  gcat?: string;
  pieces: RollPiece[];
  events?: { at_s: number; kind: string }[];
}

export interface RollSource {
  people: RollPerson[];
  stays: RollStay[];
  logs: RollLog[];
  /** Where space starts, km: the pack's value (ruling 11: 100 by default). Null: every stay counts. */
  edge_km: number | null;
  /** The lowest height the source lists at all (McDowell's 80 km), when it has one. */
  listed_from_km?: number;
  /** The place the campaign calls home, nearest first in the roster (Earth for the real pack). */
  home?: string;
  /** Where the source says its people came from, for the credit line. */
  credit?: string;
}

// ---- What it says ----

export interface RollRow {
  person: string;
  name: string;
  being?: string;
  role?: string;
  /** Day of the flight on the day in view, from 1; `of` its length in days (null while still up). */
  day: number;
  of: number | null;
  mark: "recorded" | "estimated";
  why?: string;
  /** Under the edge of space: listed because the source lists it, not counted. */
  below_edge?: string;
  /** A short hop whose height the source does not give: counted, and said. */
  height_unknown?: boolean;
}

export interface RollCraft {
  /** The log's construct id, or null where there is no log. */
  on: string | null;
  name: string;
  /** Where it was, in words: "in orbit", "on the surface", "docked to Mir", "a short flight up to 107 km". */
  where: string;
  mark: "recorded" | "estimated";
  why?: string;
  people: RollRow[];
}

export interface RollGroup {
  /** The body (address) the group is about, or a word for no body ("unrecorded"). */
  key: string;
  /** "Around Earth", "At the Moon", "On the way to Mars", "Around the Sun", "Where not recorded". */
  label: string;
  rank: number;
  crafts: RollCraft[];
}

export interface RollCall {
  from_s: number;
  to_s: number;
  /** People in space (counted against the edge) during the window, and the craft they were aboard. */
  people: number;
  humans: number;
  crafts: number;
  groups: RollGroup[];
  /** Craft in flight with nobody aboard, folded to a count per place. */
  also: { key: string; label: string; count: number }[];
  /** Nobody up: the last person down before the window and the next up after it. */
  empty?: { last?: { person: string; name: string; at_s: number }; next?: { person: string; name: string; at_s: number }; before_first?: boolean };
}

export interface RollOptions {
  /** A body's name for the words ("Earth", "Moon"); the address's last part when absent. */
  nameOf?: (address: string) => string;
  /** Where space starts, km, overriding the source's (the Build desk's value). */
  edge_km?: number | null;
}

// ---- The index, built once per source ----

export interface RollIndex {
  source: RollSource;
  people: Map<string, RollPerson>;
  logs: Map<string, RollLog>;
  /** Stays sorted by start. */
  stays: RollStay[];
  /** Each person's flights: runs of stays that follow on without a gap, for "day 5 of 8". */
  flights: Map<string, { from_s: number; to_s: number | null }[]>;
}

export function indexRoll(source: RollSource): RollIndex {
  const people = new Map(source.people.map((p) => [p.id, p]));
  const logs = new Map<string, RollLog>();
  for (const l of source.logs) {
    logs.set(l.construct, l);
    if (l.gcat) logs.set(l.gcat.startsWith("gcat:") ? l.gcat : `gcat:${l.gcat}`, l);
  }
  const stays = [...source.stays].sort((a, b) => a.from_s - b.from_s || cmp(a.person, b.person));
  const flights = new Map<string, { from_s: number; to_s: number | null }[]>();
  for (const s of stays) {
    const list = flights.get(s.person) ?? [];
    const last = list[list.length - 1];
    // A stay that starts where the last one ended (or within it) is the same flight: a change of craft in space.
    if (last && (last.to_s === null || s.from_s <= last.to_s)) last.to_s = last.to_s === null || s.to_s === null ? null : Math.max(last.to_s, s.to_s);
    else list.push({ from_s: s.from_s, to_s: s.to_s });
    flights.set(s.person, list);
  }
  return { source, people, logs, stays, flights };
}

/** A name as the record gives it, less McDowell's "*" (it marks the name the person went by); nothing else changes. */
export const shownName = (name: string): string => name.replace(/\*/g, "");

const cmp = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);
const DAY = constant("day");

/** The piece of a log covering `t`, or null. */
export function pieceAt(log: RollLog, t: number): RollPiece | null {
  for (const p of log.pieces) if (p.from_s <= t && (p.to_s === undefined || t < p.to_s)) return p;
  return null;
}

const OFF_GROUND = new Set(["orbit", "course", "docked"]);

/** Whether a construct is off the ground and not yet gone at `t`, by its log. */
export function inFlight(log: RollLog, t: number): boolean {
  const p = pieceAt(log, t);
  return !!p && OFF_GROUND.has(p.state.kind);
}

/** Does a stay count as "in space" against the edge? `below` gives the words when it is listed but not counted. */
/** Whether a stay counts as time in space against the source's edge (records and "On this day", SH5). */
export const againstEdgeCounts = (s: RollStay, ix: RollIndex, edge_km: number | null = ix.source.edge_km): boolean => againstEdge(s, edge_km, ix.source.listed_from_km).counts;

function againstEdge(s: RollStay, edge: number | null, listed: number | undefined): { counts: boolean; below?: string; unknown?: boolean } {
  if (s.reached_space === false) return { counts: false, below: s.reached_why ? `did not reach space: ${s.reached_why}` : "did not reach space" };
  if (!s.suborbital || edge === null) return { counts: true };
  if (s.peak_km === null || s.peak_km === undefined) return { counts: true, unknown: true };
  if (s.peak_km >= edge) return { counts: true };
  const km = (x: number) => say(x * 1000, "length", { class: "construct", what: "height" }, { sig: 3 }).text;
  return { counts: false, below: listed !== undefined && s.peak_km >= listed ? `by the ${km(listed)} line, below the ${km(edge)} edge` : `below the ${km(edge)} edge` };
}

/** Is this stay in flight at `t`? Pack legs are flights by definition; a campaign's are read from the log. */
function stayFlying(ix: RollIndex, s: RollStay, t: number): boolean {
  if (s.from_s > t || (s.to_s !== null && t >= s.to_s)) return false;
  if (s.flight) return true;
  const log = s.on ? ix.logs.get(s.on) : undefined;
  return !!log && inFlight(log, t);
}

/** A stay's own window clipped to [from, to): the moment to read its craft at, or null if it does not overlap. */
function overlap(s: RollStay, from: number, to: number, at: number): number | null {
  const a = Math.max(s.from_s, from);
  const b = Math.min(s.to_s ?? Number.POSITIVE_INFINITY, to);
  if (a >= b && !(s.from_s === s.to_s && s.from_s >= from && s.from_s < to)) return null;
  return at >= a && at < b ? at : a;
}

// ---- Places and their words ----

const lastPart = (address: string) => {
  const seg = address.split("/").pop() ?? address;
  const w = seg.includes(":") ? seg.slice(seg.indexOf(":") + 1) : seg;
  return w ? w[0]!.toUpperCase() + w.slice(1).replace(/[-_]/g, " ") : address;
};
/** "the Moon", "the Sun": a body whose name is a common noun takes the article in English. */
const withArticle = (n: string) => (/^(Moon|Sun)$/.test(n) ? `the ${n}` : n);
const isStar = (address: string) => !address.split("/").pop()?.startsWith("body:");

interface Placed {
  key: string;
  label: string;
  rank: number;
  where: string;
  mark: "recorded" | "estimated";
  why?: string;
}

function place(ix: RollIndex, log: RollLog | undefined, t: number, opts: RollOptions, depth = 0): Placed | null {
  const name = (a: string) => withArticle(opts.nameOf?.(a) ?? lastPart(a));
  const home = ix.source.home;
  const rankOf = (body: string) => (body === home ? 0 : home && body.startsWith(`${home}/`) ? 1 : isStar(body) ? 4 : 3);
  const labelOf = (body: string) => (body === home || isStar(body) ? `Around ${name(body)}` : `At ${name(body)}`);
  if (!log) return null;
  const p = pieceAt(log, t);
  if (!p) return null;
  const mark = p.mark === "estimated" ? "estimated" : "recorded";
  const why = p.why;
  const k = p.state.kind;
  if (k === "orbit" && p.state.around) return { key: p.state.around, label: labelOf(p.state.around), rank: rankOf(p.state.around), where: "in orbit", mark, ...(why ? { why } : {}) };
  if (k === "landed" && p.state.body) return { key: p.state.body, label: labelOf(p.state.body), rank: rankOf(p.state.body), where: "on the surface", mark, ...(why ? { why } : {}) };
  if (k === "docked" && p.state.to && depth < 4) {
    const host = ix.logs.get(p.state.to);
    const at = place(ix, host, t, opts, depth + 1);
    const hostName = host?.name ?? "another craft";
    if (at) return { ...at, where: `docked to ${hostName}`, mark: mark === "estimated" || at.mark === "estimated" ? "estimated" : "recorded", ...(why ? { why } : {}) };
    return { key: "unrecorded", label: "Where not recorded", rank: 6, where: `docked to ${hostName}`, mark, ...(why ? { why } : {}) };
  }
  if (k === "course") {
    // On its way: to the body of the next piece that names one.
    const next = log.pieces.find((q) => q.from_s >= (p.to_s ?? Number.POSITIVE_INFINITY) && (q.state.around || q.state.body));
    const to = next?.state.around ?? next?.state.body;
    if (to) return { key: `course:${to}`, label: `On the way to ${name(to)}`, rank: 2, where: "on its way", mark, ...(why ? { why } : {}) };
    return { key: "course", label: "On a course", rank: 2, where: "on its way", mark, ...(why ? { why } : {}) };
  }
  return null;
}

// ---- The look-up ----

/**
 * Who was in flight during [from_s, to_s) (a day, usually), aboard what, and where, grouped by place, nearest first.
 * `at_s` is the moment each craft is placed at (the clock), clipped into each stay.
 */
export function rollCall(ix: RollIndex, from_s: number, to_s: number, at_s: number = from_s, opts: RollOptions = {}): RollCall {
  const edge = opts.edge_km !== undefined ? opts.edge_km : ix.source.edge_km;
  const crafts = new Map<string, { craft: RollCraft; group: Placed }>();
  const counted = new Set<string>();
  let humans = 0;
  const humanSeen = new Set<string>();
  for (const s of ix.stays) {
    if (s.from_s >= to_s) break;
    const t = overlap(s, from_s, to_s, at_s);
    if (t === null || !stayFlying(ix, s, t)) continue;
    const e = againstEdge(s, edge, ix.source.listed_from_km);
    const log = s.on ? ix.logs.get(s.on) : undefined;
    // The clock's moment first; if the log has no piece then (an ascent before its first orbit, a lander before it
    // undocks), the middle of the stay's part of the window, then its end, then its start.
    const b = Math.min(s.to_s ?? Number.POSITIVE_INFINITY, to_s);
    const a = Math.max(s.from_s, from_s);
    let placed: Placed | null = null;
    for (const at of [t, (a + Math.min(b, a + DAY)) / 2, Math.min(b, to_s) - 1, a]) if ((placed = place(ix, log, at, opts))) break;
    const home = ix.source.home;
    const peak = s.peak_km !== null && s.peak_km !== undefined ? say(s.peak_km * 1000, "length", { class: "construct", what: "height" }, { sig: 3 }).text : null;
    const group: Placed =
      placed ??
      (s.suborbital && home
        ? { key: home, label: `Around ${withArticle(opts.nameOf?.(home) ?? lastPart(home))}`, rank: 0, where: peak ? `a short flight, up to ${peak}` : "a short flight; its height is not recorded", mark: s.mark }
        : home
          ? { key: home, label: `Around ${withArticle(opts.nameOf?.(home) ?? lastPart(home))}`, rank: 0, where: "no flight log is held for it", mark: s.mark }
          : { key: "unrecorded", label: "Where not recorded", rank: 6, where: "no flight log is held for it", mark: s.mark });
    const id = s.on ?? `craft:${s.craft}:${s.from_s}`;
    let c = crafts.get(id);
    if (!c) {
      c = { craft: { on: s.on, name: log?.name ?? s.craft, where: group.where, mark: group.mark, ...(group.why ? { why: group.why } : {}), people: [] }, group };
      crafts.set(id, c);
    }
    if (c.craft.people.some((r) => r.person === s.person)) continue;
    const p = ix.people.get(s.person);
    const fl = (ix.flights.get(s.person) ?? []).find((f) => f.from_s <= t && (f.to_s === null || t <= f.to_s));
    const start = fl?.from_s ?? s.from_s;
    const end = fl ? fl.to_s : s.to_s;
    const row: RollRow = {
      person: s.person,
      name: shownName(p?.name ?? s.person),
      ...(p?.being ? { being: p.being } : {}),
      ...(s.role ? { role: s.role } : {}),
      day: Math.floor((t - start) / DAY) + 1,
      of: end === null ? null : Math.max(1, Math.ceil((end - start) / DAY)),
      mark: s.mark,
      ...(s.why ? { why: s.why } : {}),
      ...(e.below ? { below_edge: e.below } : {}),
      ...(e.unknown ? { height_unknown: true } : {}),
    };
    c.craft.people.push(row);
    if (e.counts && !counted.has(s.person)) {
      counted.add(s.person);
      if ((p?.being ?? "human") === "human" && !humanSeen.has(s.person)) {
        humanSeen.add(s.person);
        humans++;
      }
    }
  }
  // Craft in flight with nobody aboard.
  const alsoMap = new Map<string, { key: string; label: string; rank: number; count: number }>();
  const seenLog = new Set<RollLog>();
  for (const l of ix.source.logs) {
    if (seenLog.has(l) || crafts.has(l.construct)) continue;
    seenLog.add(l);
    const t = Math.min(Math.max(at_s, from_s), to_s - 1);
    if (!inFlight(l, t)) continue;
    const pl = place(ix, l, t, opts);
    if (!pl) continue;
    const a = alsoMap.get(pl.key) ?? { key: pl.key, label: pl.label, rank: pl.rank, count: 0 };
    a.count++;
    alsoMap.set(pl.key, a);
  }
  const groups = new Map<string, RollGroup>();
  for (const { craft, group } of crafts.values()) {
    const g = groups.get(group.key) ?? { key: group.key, label: group.label, rank: group.rank, crafts: [] };
    g.crafts.push(craft);
    groups.set(group.key, g);
  }
  const out: RollGroup[] = [...groups.values()].sort((a, b) => a.rank - b.rank || cmp(a.label, b.label));
  for (const g of out) {
    g.crafts.sort((a, b) => cmp(a.name, b.name));
    for (const c of g.crafts) c.people.sort((a, b) => (a.below_edge ? 1 : 0) - (b.below_edge ? 1 : 0));
  }
  const rc: RollCall = {
    from_s,
    to_s,
    people: counted.size,
    humans,
    crafts: [...crafts.values()].filter((c) => c.craft.people.some((r) => !r.below_edge)).length,
    groups: out,
    also: [...alsoMap.values()].sort((a, b) => a.rank - b.rank || cmp(a.label, b.label)).map(({ key, label, count }) => ({ key, label, count })),
  };
  if (!counted.size) rc.empty = emptyWords(ix, from_s, to_s, edge);
  return rc;
}

function emptyWords(ix: RollIndex, from_s: number, to_s: number, edge: number | null): NonNullable<RollCall["empty"]> {
  let last: RollStay | undefined;
  let next: RollStay | undefined;
  for (const s of ix.stays) {
    if (!s.flight && !s.on) continue;
    if (!againstEdge(s, edge, ix.source.listed_from_km).counts) continue;
    if (s.to_s !== null && s.to_s <= from_s && (!last || s.to_s > last.to_s!)) last = s;
    if (s.from_s >= to_s && (!next || s.from_s < next.from_s)) next = s;
  }
  const who = (s: RollStay, at_s: number) => ({ person: s.person, name: shownName(ix.people.get(s.person)?.name ?? s.person), at_s });
  return { ...(last ? { last: who(last, last.to_s!) } : { before_first: true }), ...(next ? { next: who(next, next.from_s) } : {}) };
}

/** The one sentence under the date: "3 people in space, aboard 2 craft. 9 more craft in flight with nobody aboard." */
export function rollSentence(rc: RollCall, past = true): string {
  const also = rc.also.reduce((n, a) => n + a.count, 0);
  const tail = also ? ` ${also} more ${also === 1 ? "craft" : "craft"} in flight with nobody aboard.` : "";
  if (!rc.people) return tail.trim();
  const all = rc.humans === rc.people;
  const n = rc.people;
  const who = all ? `${n} ${n === 1 ? "person" : "people"}` : `${n}`;
  return `${who} ${past ? (n === 1 ? "was" : "were") : n === 1 ? "is" : "are"} in space, aboard ${rc.crafts} ${rc.crafts === 1 ? "craft" : "craft"}.${tail}`;
}

// ---- Through time ----

/** People in space through time, against the edge: [t, count] steps, the count holding until the next step. */
export function countSteps(ix: RollIndex, edge_km: number | null = ix.source.edge_km): [number, number][] {
  const d = new Map<number, number>();
  for (const s of ix.stays) {
    if (!s.flight) continue;
    if (!againstEdge(s, edge_km, ix.source.listed_from_km).counts) continue;
    d.set(s.from_s, (d.get(s.from_s) ?? 0) + 1);
    if (s.to_s !== null) d.set(s.to_s, (d.get(s.to_s) ?? 0) - 1);
  }
  const out: [number, number][] = [];
  let n = 0;
  for (const t of [...d.keys()].sort((a, b) => a - b)) {
    n += d.get(t)!;
    if (out.length && out[out.length - 1]![1] === n) continue;
    out.push([t, n]);
  }
  return out;
}

/** The count at `t` from a step list. */
export function countAt(steps: readonly (readonly [number, number])[], t: number): number {
  let lo = 0;
  let hi = steps.length - 1;
  let n = 0;
  while (lo <= hi) {
    const m = (lo + hi) >> 1;
    if (steps[m]![0] <= t) {
      n = steps[m]![1];
      lo = m + 1;
    } else hi = m - 1;
  }
  return n;
}

/**
 * The next (dir 1) or previous (dir -1) dated event after or before `t`: a person going up or coming down, or a piece
 * of a crewed craft's log beginning (a docking, a landing, a course). Null when there is none.
 */
export function stepEvent(ix: RollIndex, t: number, dir: 1 | -1): number | null {
  let best: number | null = null;
  const take = (x: number) => {
    if (dir > 0 ? x > t : x < t) if (best === null || (dir > 0 ? x < best : x > best)) best = x;
  };
  const crewed = new Set<string>();
  for (const s of ix.stays) {
    take(s.from_s);
    if (s.to_s !== null) take(s.to_s);
    if (s.on) crewed.add(s.on);
  }
  for (const id of crewed) {
    const l = ix.logs.get(id);
    if (!l) continue;
    for (const p of l.pieces) take(p.from_s);
    for (const e of l.events ?? []) take(e.at_s);
  }
  return best;
}

// ---- Sources ----

/** The "Sol today" pack's `crews.json` (RS9, track RS) and its flight logs, as one source. */
export function sourceFromCrews(crews: Json, logs: readonly Json[]): RollSource {
  const c = crews as { people?: Json[]; aboard?: Json[]; edge_of_space_km?: number; listed_from_km?: number; credit?: string };
  const str = (v: Json | undefined) => (typeof v === "string" && v ? v : undefined);
  const people: RollPerson[] = (c.people ?? []).map((p) => {
    const o = p as Record<string, Json>;
    return { id: String(o.id), name: String(o.name), ...(str(o.being) ? { being: str(o.being)! } : {}), ...(str(o.citizen) ? { flew_for: str(o.citizen)! } : {}) };
  });
  const stays: RollStay[] = (c.aboard ?? []).map((a) => {
    const o = a as Record<string, Json>;
    return {
      person: String(o.person),
      from_s: Number(o.from_s),
      to_s: typeof o.to_s === "number" ? o.to_s : null,
      on: str(o.on) ?? null,
      craft: String(o.craft ?? ""),
      ...(str(o.role) ? { role: str(o.role)! } : {}),
      mark: o.mark === "estimated" ? "estimated" : "recorded",
      ...(str(o.why) ? { why: str(o.why)! } : {}),
      ...(str(o.src) ? { src: str(o.src)! } : {}),
      flight: true,
      ...(o.suborbital === true ? { suborbital: true, peak_km: typeof o.peak_km === "number" ? o.peak_km : null } : {}),
      ...(str(o.peak_why) ? { peak_why: str(o.peak_why)! } : {}),
      ...(o.reached_space === false ? { reached_space: false as const, ...(str(o.reached_why) ? { reached_why: str(o.reached_why)! } : {}) } : {}),
    };
  });
  return {
    people,
    stays,
    logs: logs.map(logFrom),
    edge_km: typeof c.edge_of_space_km === "number" ? c.edge_of_space_km : null,
    ...(typeof c.listed_from_km === "number" ? { listed_from_km: c.listed_from_km } : {}),
    home: "u:sol/sys:sun/body:earth",
    ...(str(c.credit) ? { credit: str(c.credit)! } : {}),
  };
}

/** A flight log document (pack file or the kernel's `flight_log` answer) as the look-up reads it. */
export function logFrom(doc: Json): RollLog {
  const o = doc as Record<string, Json>;
  const pieces = (Array.isArray(o.pieces) ? o.pieces : []) as unknown as RollPiece[];
  const events = (Array.isArray(o.events) ? o.events : []) as unknown as { at_s: number; kind: string }[];
  return { construct: String(o.construct), name: String(o.name ?? o.construct), ...(typeof o.gcat === "string" ? { gcat: o.gcat } : {}), pieces, events };
}

/**
 * A campaign's own people (manifest entries of kind person, with their dated `aboard.on` moves) and its constructs'
 * flight logs. A stay counts as in flight when the construct's log has it off the ground at the time.
 */
export function sourceFromManifest(entries: readonly AboardEntry[], logs: readonly RollLog[], home?: string): RollSource {
  const people: RollPerson[] = [];
  const stays: RollStay[] = [];
  for (const e of entries) {
    if (e.kind !== PERSON) continue;
    people.push({ id: e.address, name: e.name, ...(e.being ? { being: e.being } : {}) });
    const moves = [...e.moves].sort((a, b) => a.at_s - b.at_s);
    moves.forEach((m, i) => {
      const on = typeof m.value === "string" && m.value ? m.value : null;
      if (!on) return;
      const next = moves[i + 1];
      stays.push({ person: e.address, from_s: m.at_s, to_s: next ? next.at_s : null, on, craft: on, mark: "recorded", ...(m.reason ? { why: m.reason } : {}), src: "campaign" });
    });
  }
  return { people, stays, logs: [...logs], edge_km: null, ...(home ? { home } : {}) };
}
