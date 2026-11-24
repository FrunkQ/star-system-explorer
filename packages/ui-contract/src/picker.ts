// The one picker (Alex, 24 Nov 2026): the place line's "find a thing here" list, the same at every scale with other
// content. The list rule is plain functions, so it is tested without a page (picker.test). Learned from V3's body
// picker and done once, generally:
//  - Browsing shows the real hierarchy: the thing at this level, what is directly in it, and one level further folded
//    behind a count ("4 moons") that opens in place; searching shows a flat list, since indenting matches under
//    parents nobody asked about only buries them.
//  - Kind chips add together, show everything when none is pressed, and count before the others apply, so the
//    numbers do not jump as they are pressed.
//  - A filter keeps the path: a parent hidden by the filter but holding a match stays, quiet and not pickable, so
//    the match is seen where it is (a station under Earth, not a lone station).
//  - Order is the order of the place itself (distance from the star; a moon right after its planet), never A to Z.
//  - A loop in the data (A in B, B in A) still lists its items at the top rather than an empty list.
import type { SceneSubject } from "./scene.ts";

export interface PickItem {
  id: string;
  label: string;
  /** The item it sits in (an id in the same list); absent or unknown: it is at the top. */
  parent?: string | null;
  /** Kinds for the chips; an item may have several (a planet with life: planet and life). */
  kinds: string[];
  /** Order among its siblings, smaller first (distance from the star at the system scale). */
  order?: number;
  /** A short line saying where it is or what it is ("1.0 AU", "orbits Earth"). */
  caption?: string;
}

export interface PickRow {
  id: string;
  label: string;
  depth: number;
  caption?: string;
  /** Kept only to show where a match is: quiet, and not pickable. */
  context?: boolean;
  /** Children folded behind it: how many, and the word for them ("moons"). */
  folded?: { count: number; words: string };
  /** Its children are showing because it was opened. */
  opened?: boolean;
}

export interface PickAsk {
  query?: string;
  /** Kind chips pressed; none means everything. */
  kinds?: readonly string[];
  /** Items whose folded children are open. */
  open?: readonly string[];
  /** How many levels show before folding: 2 is the thing, what is in it, then folded (default). */
  levels?: number;
  /** The word for an item's children by kind, for the fold ("moons"); default "inside". */
  childWords?: (children: readonly PickItem[]) => string;
}

export const SEARCH_CAP = 200;

function tree(items: readonly PickItem[]) {
  const ids = new Set(items.map((i) => i.id));
  const kids = new Map<string | null, PickItem[]>();
  // An item whose chain of parents loops is put at the top, so a loop never empties the list.
  const loops = (i: PickItem) => {
    const seen = new Set<string>([i.id]);
    for (let p = i.parent; p && ids.has(p); p = items.find((x) => x.id === p)?.parent) {
      if (seen.has(p)) return true;
      seen.add(p);
    }
    return false;
  };
  for (const i of items) {
    const p = i.parent && ids.has(i.parent) && !loops(i) ? i.parent : null;
    const list = kids.get(p) ?? [];
    list.push(i);
    kids.set(p, list);
  }
  const ord = (a: PickItem, b: PickItem) => (a.order ?? Infinity) - (b.order ?? Infinity) || items.indexOf(a) - items.indexOf(b);
  for (const l of kids.values()) l.sort(ord);
  return kids;
}

const matchesKinds = (i: PickItem, kinds: readonly string[]) => !kinds.length || i.kinds.some((k) => kinds.includes(k));

/** The rows to show for an ask: a folded tree when browsing, a flat list when searching. */
export function pickerRows(items: readonly PickItem[], ask: PickAsk = {}): { rows: PickRow[]; mode: "browse" | "search"; more: number } {
  const kinds = ask.kinds ?? [];
  const q = (ask.query ?? "").trim().toLowerCase();
  if (q) {
    const hit = items.filter((i) => i.label.toLowerCase().includes(q) && matchesKinds(i, kinds));
    const byId = new Map(items.map((i) => [i.id, i]));
    const rows = hit.slice(0, SEARCH_CAP).map((i): PickRow => {
      // Where a match is: "in Earth" for a moon; a planet keeps its own caption (its distance from the star).
      const host = i.parent ? byId.get(i.parent) : undefined;
      const caption = host && host.parent && byId.has(host.parent) ? `in ${host.label}` : i.caption;
      return { id: i.id, label: i.label, depth: 0, ...(caption ? { caption } : {}) };
    });
    return { rows, mode: "search", more: Math.max(0, hit.length - SEARCH_CAP) };
  }
  const kids = tree(items);
  const levels = ask.levels ?? 2;
  const open = new Set(ask.open ?? []);
  const words = ask.childWords ?? (() => "inside");
  // What a filter keeps: an item that matches, or one with a match below it (kept as context).
  const keeps = new Map<string, "match" | "context">();
  const mark = (i: PickItem): boolean => {
    let below = false;
    for (const c of kids.get(i.id) ?? []) below = mark(c) || below;
    const me = matchesKinds(i, kinds);
    if (me) keeps.set(i.id, "match");
    else if (below) keeps.set(i.id, "context");
    return me || below;
  };
  for (const r of kids.get(null) ?? []) mark(r);
  const rows: PickRow[] = [];
  const walk = (i: PickItem, depth: number) => {
    const keep = keeps.get(i.id);
    if (!keep) return;
    const children = (kids.get(i.id) ?? []).filter((c) => keeps.has(c.id));
    // Fold below the levels shown, unless opened or a filter is on (a filter shows its matches wherever they are).
    const fold = children.length > 0 && depth + 1 >= levels && !open.has(i.id) && !kinds.length;
    rows.push({
      id: i.id,
      label: i.label,
      depth,
      ...(i.caption ? { caption: i.caption } : {}),
      ...(keep === "context" ? { context: true } : {}),
      ...(fold ? { folded: { count: children.length, words: words(children) } } : {}),
      ...(open.has(i.id) && children.length && depth + 1 >= levels ? { opened: true } : {}),
    });
    if (!fold) for (const c of children) walk(c, depth + 1);
  };
  for (const r of kids.get(null) ?? []) walk(r, 0);
  return { rows, mode: "browse", more: 0 };
}

/** The kind chips with their counts, counted over everything so a count never moves as chips are pressed. */
export function kindChips(items: readonly PickItem[], order: readonly string[] = []): { kind: string; count: number }[] {
  const counts = new Map<string, number>();
  for (const i of items) for (const k of new Set(i.kinds)) counts.set(k, (counts.get(k) ?? 0) + 1);
  const rank = (k: string) => (order.includes(k) ? order.indexOf(k) : order.length);
  return [...counts].map(([kind, count]) => ({ kind, count })).sort((a, b) => rank(a.kind) - rank(b.kind) || a.kind.localeCompare(b.kind));
}

/** The picker's kinds for a scene subject, in plain words a newcomer reads at once. */
export function subjectKinds(s: Pick<SceneSubject, "kind">): string[] {
  const k = s.kind;
  if (k === "star" || k === "brown_dwarf" || k === "remnant") return ["Stars"];
  if (k === "planet" || k === "dwarf_planet") return ["Planets"];
  if (k === "moon") return ["Moons"];
  if (k === "belt" || k === "ring" || k === "disc" || k === "cloud") return ["Belts and rings"];
  if (k === "zone") return ["Zones"];
  if (k === "station" || k === "ship" || k === "construct" || k === "base" || k === "outpost") return ["Ships and stations"];
  return ["Other"];
}
export const SUBJECT_KIND_ORDER = ["Stars", "Planets", "Moons", "Belts and rings", "Ships and stations", "Zones", "Other"] as const;

/** A system's subjects as picker items: the star at the top, ordered by distance out, each saying where it is. */
export function subjectItems(subjects: readonly SceneSubject[]): PickItem[] {
  const byId = new Map(subjects.map((s) => [s.address, s]));
  // Distance from the star, summed up the chain of orbits, so a moon sorts right after its planet.
  // A belt sorts by the middle of its band, so the asteroid belt falls between Mars and Jupiter.
  const at = (x: SceneSubject) => x.orbit?.a_au ?? (x.band_au ? (x.band_au[0] + x.band_au[1]) / 2 : 0);
  const out = (s: SceneSubject): number => {
    let d = 0;
    const seen = new Set<string>();
    for (let x: SceneSubject | undefined = s; x && !seen.has(x.address); x = x.parent ? byId.get(x.parent) : undefined) {
      seen.add(x.address);
      d += at(x);
    }
    return d;
  };
  return subjects.map((s) => {
    const host = s.parent ? byId.get(s.parent) : undefined;
    const top = host && !host.parent;
    const caption = !top ? undefined : s.orbit ? auWords(s.orbit.a_au) : s.band_au ? `${auWords(s.band_au[0]).replace(" AU", "")} to ${auWords(s.band_au[1])}` : undefined;
    return { id: s.address, label: s.label, parent: s.parent ?? null, kinds: subjectKinds(s), order: out(s), ...(caption ? { caption } : {}) };
  });
}

function auWords(au: number): string {
  if (!(au > 0)) return "";
  const v = au >= 100 ? Math.round(au) : au >= 10 ? Number(au.toFixed(0)) : Number(au.toPrecision(2));
  return `${v.toLocaleString("en-GB")} AU`;
}

/** The fold's word for a list of children: "moons" when they are all moons, "ring" for one ring, else "inside". */
export function childWordsFor(children: readonly PickItem[]): string {
  const ks = new Set(children.flatMap((c) => c.kinds));
  const one = children.length === 1;
  if (ks.size !== 1) return "inside";
  const k = [...ks][0]!;
  const all = (re: RegExp) => children.every((c) => re.test(c.label));
  if (k === "Belts and rings") return all(/ring/i) ? (one ? "ring" : "rings") : all(/belt/i) ? (one ? "belt" : "belts") : one ? "belt or ring" : "belts and rings";
  const words: Record<string, [string, string]> = { Stars: ["star", "stars"], Planets: ["planet", "planets"], Moons: ["moon", "moons"], Zones: ["zone", "zones"], "Ships and stations": ["ship or station", "ships and stations"] };
  const w = words[k];
  return w ? w[one ? 0 : 1] : k.toLowerCase();
}
