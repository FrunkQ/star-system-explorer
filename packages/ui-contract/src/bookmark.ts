// Bookmarks (PB2b, design SSE4-playbook-and-switchboard-design.md section 4A): where the GM is and how they are
// looking at it, saved. "Bookmarks define what, layouts define how it looks, playbooks store them" (Alex, 24 Nov).
// A bookmark holds nothing about players: no role, no look, no knowledge. It is the top-level design's place mark, so
// tours, presets and layouts all share it. Kept with the campaign; plain data, never read by physics.
import type { ViewChoice } from "./view-controls.ts";

export const BOOKMARK_SCHEMA = "sse.bookmark/1";

export interface Bookmark {
  schema: typeof BOOKMARK_SCHEMA;
  id: string;
  name: string;
  /** The place: the system (or other scope) in view. */
  scope: string;
  /** The ruler's date, in years; null follows the campaign's now. */
  at_yr: number | null;
  /** The instrument in front (a screen document id: "system", "body", "globe", "surface", ...). */
  instrument: string;
  /** The rail's scale when kept (system, world, galaxy, ...), so going back opens the same rail. */
  scale?: string;
  /** How it was looked at: view style, seen from, scale, band, HUD. */
  view?: ViewChoice;
  /** What was selected, if anything. */
  selected: string | null;
}

/** Where the GM is now, as the shell knows it. */
export interface Whereabouts {
  scope: string;
  now_yr: number;
  instrument: string;
  scale?: string;
  view?: ViewChoice;
  selected: string | null;
}

const ID = /^b(\d+)$/;

/** The next free id: b1, b2, ... (never reused while the list holds a higher one). */
export function nextBookmarkId(list: readonly Bookmark[]): string {
  let n = 0;
  for (const b of list) {
    const m = ID.exec(b.id);
    if (m) n = Math.max(n, Number(m[1]));
  }
  return `b${n + 1}`;
}

/** A plain name for where the GM is: "Mars, Globe", "The Solar System, Chart". A repeat gets " 2", " 3". */
export function bookmarkName(list: readonly Bookmark[], what: string, instrument: string): string {
  const base = instrument && instrument.toLowerCase() !== what.toLowerCase() ? `${what}, ${instrument}` : what;
  const taken = new Set(list.map((b) => b.name));
  if (!taken.has(base)) return base;
  for (let i = 2; ; i++) if (!taken.has(`${base} ${i}`)) return `${base} ${i}`;
}

/** Bookmark where the GM is. `what` and `instrumentWords` make the name. */
export function makeBookmark(list: readonly Bookmark[], at: Whereabouts, what: string, instrumentWords: string): Bookmark {
  return {
    schema: BOOKMARK_SCHEMA,
    id: nextBookmarkId(list),
    name: bookmarkName(list, what, instrumentWords),
    scope: at.scope,
    at_yr: at.now_yr,
    instrument: at.instrument,
    ...(at.scale ? { scale: at.scale } : {}),
    ...(at.view ? { view: { ...at.view } } : {}),
    selected: at.selected,
  };
}

export function addBookmark(list: readonly Bookmark[], b: Bookmark): Bookmark[] {
  return [...list.filter((x) => x.id !== b.id), b];
}
export function renameBookmark(list: readonly Bookmark[], id: string, name: string): Bookmark[] {
  const n = name.trim();
  return n ? list.map((b) => (b.id === id ? { ...b, name: n } : b)) : [...list];
}
export function removeBookmark(list: readonly Bookmark[], id: string): Bookmark[] {
  return list.filter((b) => b.id !== id);
}
/** Drag reorder: the bookmark at `from` goes to `to`. */
export function moveBookmark(list: readonly Bookmark[], from: number, to: number): Bookmark[] {
  if (from < 0 || from >= list.length || to < 0 || to >= list.length || from === to) return [...list];
  const next = [...list];
  const [b] = next.splice(from, 1);
  next.splice(to, 0, b!);
  return next;
}
/** "Update from live": the same bookmark, now where the GM is. Keeps its id and name. */
export function updateBookmark(list: readonly Bookmark[], id: string, at: Whereabouts): Bookmark[] {
  return list.map((b) => (b.id === id ? { ...makeBookmark([], at, b.name, ""), id: b.id, name: b.name } : b));
}

/** Can the shell go back to it here? Another campaign's place cannot be opened from this one. */
export function bookmarkReachable(b: Bookmark, scope: string): boolean {
  return b.scope === scope;
}

/** Bookmarks read from a save or a pack: anything that is not one is left out, never guessed at. */
export function bookmarksFrom(raw: unknown): Bookmark[] {
  if (!Array.isArray(raw)) return [];
  const out: Bookmark[] = [];
  const ids = new Set<string>();
  for (const r of raw) {
    if (!r || typeof r !== "object") continue;
    const o = r as Record<string, unknown>;
    if (o.schema !== BOOKMARK_SCHEMA || typeof o.id !== "string" || ids.has(o.id) || typeof o.name !== "string" || typeof o.scope !== "string" || typeof o.instrument !== "string") continue;
    if (o.at_yr !== null && typeof o.at_yr !== "number") continue;
    ids.add(o.id);
    out.push({
      schema: BOOKMARK_SCHEMA,
      id: o.id,
      name: o.name,
      scope: o.scope,
      at_yr: o.at_yr as number | null,
      instrument: o.instrument,
      ...(typeof o.scale === "string" ? { scale: o.scale } : {}),
      ...(o.view && typeof o.view === "object" ? { view: o.view as ViewChoice } : {}),
      selected: typeof o.selected === "string" ? o.selected : null,
    });
  }
  return out;
}
