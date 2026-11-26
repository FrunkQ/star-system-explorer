// The way in (GZ1, design/SSE4-grand-zoom-design.md section 5.1): how a place is arrived at. Going to a bookmark,
// a postcard, a Playbook Show step or a place link is a cut today; a way in says it may be travelled instead: where
// the journey starts, whether it stops at each level, what is shown on the way and how fast. It sits BESIDE a
// bookmark wherever one is used, never inside it, so "bookmarks define what" stays true.
//
// This file is the data: the ladder of rungs a journey passes (section 5.2), the `sse.approach/1` shape with its
// defaults, and its link form. The plan and the camera made from it are in journey.ts. Presentation only: none of
// this reaches the kernel or the state hash.
import { placeRank, type Place, type PlaceScale } from "./place.ts";

export const APPROACH_SCHEMA = "sse.approach/1";

/**
 * The rungs of the ladder, largest first: the place line's five scales and the two the grand zoom adds, "stars"
 * (Local stars, between Galaxy and System) and "site" (below World). The place line itself (`PLACE_SCALES`) is not
 * changed by this file: the Run desk's stepper (TM7) and the short form (GZ4) draw the seven from here.
 */
export const RUNGS = ["web", "cluster", "galaxy", "stars", "system", "world", "site"] as const;
export type LadderRung = (typeof RUNGS)[number];

export interface RungInfo {
  rung: LadderRung;
  /** The words on the place line. */
  words: string;
  /** The picture's width at the rung's good view (its stop), metres. A view or `place_chain` may give a better one
   * for a particular target (a big system, a small moon): `JourneyAsk.widths`. */
  stop_m: number;
  /** The place line scale that draws this rung today ("site" rides on the world's link and the Land screen). */
  scale: PlaceScale;
  /** Why no view draws this rung yet; absent when one does. A journey passes over a held rung (journey.ts). */
  held?: string;
}

const PC = 3.0857e16;
const AU = 1.495978707e11;

/**
 * Why Local stars is held. The conserved budget chain (accepted 26 Nov 2026, its section 13) redraws every star
 * nobody has looked at once, when mass is tied through the middle (its stage 3). So the rung is declared here and
 * nothing draws it, and nothing in this contract names a particular generated star.
 */
export const STARS_HELD = "Local stars is held until the star field is settled (the conserved budget chain's stage 3)";

/** The ladder (design 5.2's table). Widths are the stop's picture width, rough by design: a view sharpens its own. */
export const RUNG_INFO: Record<LadderRung, RungInfo> = {
  web: { rung: "web", words: "Cosmic web", stop_m: 100e6 * PC, scale: "web" },
  cluster: { rung: "cluster", words: "Galaxies nearby", stop_m: 3e6 * PC, scale: "cluster" },
  galaxy: { rung: "galaxy", words: "Galaxy", stop_m: 60e3 * PC, scale: "galaxy" },
  stars: { rung: "stars", words: "Local stars", stop_m: 20 * PC, scale: "galaxy", held: STARS_HELD },
  system: { rung: "system", words: "System", stop_m: 100 * AU, scale: "system" },
  world: { rung: "world", words: "World", stop_m: 2.5e7, scale: "world" },
  site: { rung: "site", words: "Site", stop_m: 2e3, scale: "world" },
};

export const RUNG_WORDS: Record<LadderRung, string> = Object.fromEntries(RUNGS.map((r) => [r, RUNG_INFO[r].words])) as Record<LadderRung, string>;

export function rungRank(r: LadderRung): number {
  return RUNGS.indexOf(r);
}
export function isRung(r: unknown): r is LadderRung {
  return typeof r === "string" && (RUNGS as readonly string[]).includes(r);
}
/** Is a rung drawn by a view today (not held)? */
export function rungDrawn(r: LadderRung): boolean {
  return RUNG_INFO[r].held === undefined;
}

/** The screen a landing site is on (the Land screen). A world place with a site on it is at the "site" rung. */
export const SITE_SCREEN = "players.land.screen.json";

/** The rung a place is at: its scale, or "site" when it is a world with a landing site on the Land screen. */
export function rungOfPlace(p: Place): LadderRung {
  return p.scale === "world" && p.site && p.screen === SITE_SCREEN ? "site" : p.scale;
}

/**
 * The lowest rung two places share: a journey between them rises to it and comes down again (design 5.2). Today a
 * campaign has one galaxy and no system knows its star cell, so two systems share the galaxy; `place_chain` (GZ2)
 * will say when they share a cell or are in different galaxies.
 */
export function sharedRung(a: Place, b: Place): LadderRung {
  const ra = rungOfPlace(a);
  const rb = rungOfPlace(b);
  if (placeRank(a.scale) < placeRank("system") || placeRank(b.scale) < placeRank("system")) return rungRank(ra) < rungRank(rb) ? ra : rb;
  if (a.system !== b.system) return "galaxy";
  if (ra === "system" || rb === "system" || a.world !== b.world) return "system";
  if (ra === "site" && rb === "site" && a.site!.lat_deg === b.site!.lat_deg && a.site!.lon_deg === b.site!.lon_deg) return "site";
  return "world";
}

export type ApproachFrom = LadderRung | "here";
export type ApproachStops = "each" | "none" | LadderRung[];
export type ApproachGoOn = "press" | { after_s: number };
export type ApproachHud = "at_stops" | "always" | "never";
export type ApproachWords = "captions" | "names" | "none";
export type ApproachStyle = "true" | "chart" | "viewer";
/** "short" is the everyday zoom between two levels (about 0.6 s a rung, design 5.6); the others are rates. */
export type ApproachPace = "short" | "brisk" | "stately" | "slow" | { total_s: number };
export type ApproachEnds = "camera" | "landing";

/** `sse.approach/1`. Every key may be left out; `fullApproach` says what each takes then. */
export interface Approach {
  schema?: typeof APPROACH_SCHEMA;
  /** Where the journey starts: a rung, or "here" (from where the viewer is, up to the rung both places share and
   * down again). Left out: the rung above the place. */
  from?: ApproachFrom;
  /** Stop at every rung on the way down, at none, or at the named ones. */
  stops?: ApproachStops;
  /** At a stop: wait for Go on, or go on by itself after so many seconds. */
  go_on?: ApproachGoOn;
  /** Left out: the HUD as the viewer has it. */
  hud?: ApproachHud;
  words?: ApproachWords;
  /** The view style travelled in. "viewer", or left out: the desk's remembered style (three desks note 13A). */
  style?: ApproachStyle;
  pace?: ApproachPace;
  /** The camera simply arrives, or the last leg is the kernel's flown descent (a site only). */
  ends?: ApproachEnds;
}

/** An approach with every choice made. `hud: "viewer"` leaves the HUD as the viewer has it. */
export interface FullApproach {
  from: ApproachFrom;
  stops: ApproachStops;
  go_on: ApproachGoOn;
  hud: ApproachHud | "viewer";
  words: ApproachWords;
  style: ApproachStyle;
  pace: ApproachPace;
  ends: ApproachEnds;
}

/** The default way in when nothing is said (design 5.1 and 5.6): the short zoom from the rung above, no stops, no
 * words, the HUD and the style as the viewer has them. */
export const SHORT_FORM: Omit<FullApproach, "from"> = { stops: "none", go_on: "press", hud: "viewer", words: "none", style: "viewer", pace: "short", ends: "camera" };

/** What "Zoom in from" starts with (question 7, accepted 25 Nov): True, a stop at each level, HUD and captions at
 * stops, go on by a press, at the stately pace. */
export const ZOOM_IN_FROM: Approach = { schema: APPROACH_SCHEMA, stops: "each", go_on: "press", hud: "at_stops", words: "captions", style: "true", pace: "stately" };

/** "One smooth zoom" is one switch that turns stops, HUD and words off together (question 7). */
export function oneSmoothZoom(a: Approach): Approach {
  return { ...a, stops: "none", hud: "never", words: "none" };
}

/** The rung above a rung on the ladder, passing over held ones; the web for the web. */
export function rungAbove(r: LadderRung): LadderRung {
  for (let i = rungRank(r) - 1; i >= 0; i--) if (rungDrawn(RUNGS[i]!)) return RUNGS[i]!;
  return "web";
}

/** Every choice made, for a journey to the rung `to`. */
export function fullApproach(a: Approach | null | undefined, to: LadderRung): FullApproach {
  const x = a ?? {};
  return {
    from: x.from ?? rungAbove(to),
    stops: x.stops ?? SHORT_FORM.stops,
    go_on: x.go_on ?? SHORT_FORM.go_on,
    hud: x.hud ?? SHORT_FORM.hud,
    words: x.words ?? SHORT_FORM.words,
    style: x.style ?? SHORT_FORM.style,
    pace: x.pace ?? SHORT_FORM.pace,
    ends: x.ends ?? SHORT_FORM.ends,
  };
}

const finitePos = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x) && x > 0;
const one = <T extends string>(v: unknown, of: readonly T[]): v is T => typeof v === "string" && (of as readonly string[]).includes(v);

/** The longest a journey may be told to take, or to wait at a stop by itself: an hour. */
export const APPROACH_MAX_S = 3600;

/** What is wrong with a way in, one line; null when it is a good one. */
export function checkApproach(raw: unknown): string | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return "the way in is not an object";
  const o = raw as Record<string, unknown>;
  for (const k of Object.keys(o)) if (!["schema", "from", "stops", "go_on", "hud", "words", "style", "pace", "ends"].includes(k)) return `the way in has an unknown key "${k}"`;
  if (o.schema !== undefined && o.schema !== APPROACH_SCHEMA) return `the way in is not ${APPROACH_SCHEMA}`;
  if (o.from !== undefined && o.from !== "here" && !isRung(o.from)) return "the way in starts at no rung";
  if (o.stops !== undefined && o.stops !== "each" && o.stops !== "none" && !(Array.isArray(o.stops) && o.stops.every(isRung))) return "the way in's stops are not each, none or a list of rungs";
  if (o.go_on !== undefined && o.go_on !== "press") {
    const g = o.go_on as Record<string, unknown> | null;
    if (!g || typeof g !== "object" || !finitePos(g.after_s) || g.after_s > APPROACH_MAX_S) return "the way in's go on is not press or a wait in seconds";
  }
  if (o.hud !== undefined && !one(o.hud, ["at_stops", "always", "never"])) return "the way in's HUD is not at stops, always or never";
  if (o.words !== undefined && !one(o.words, ["captions", "names", "none"])) return "the way in's words are not captions, names or none";
  if (o.style !== undefined && !one(o.style, ["true", "chart", "viewer"])) return "the way in's style is not true, chart or viewer";
  if (o.pace !== undefined && !one(o.pace, ["short", "brisk", "stately", "slow"])) {
    const p = o.pace as Record<string, unknown> | null;
    if (!p || typeof p !== "object" || !finitePos(p.total_s) || p.total_s > APPROACH_MAX_S) return "the way in's pace is not short, brisk, stately, slow or a length in seconds";
  }
  if (o.ends !== undefined && !one(o.ends, ["camera", "landing"])) return "the way in does not end with the camera or the landing";
  return null;
}

/** A way in read from a save, a postcard, a preset or a pack: null when it is not a good one (the place is then
 * arrived at by the default). Lists of stops come back in ladder order without repeats. */
export function approachFrom(raw: unknown): Approach | null {
  if (checkApproach(raw) !== null) return null;
  const o = raw as Approach;
  const out: Approach = {};
  if (o.schema) out.schema = o.schema;
  if (o.from !== undefined) out.from = o.from;
  if (o.stops !== undefined) out.stops = Array.isArray(o.stops) ? RUNGS.filter((r) => (o.stops as LadderRung[]).includes(r)) : o.stops;
  if (o.go_on !== undefined) out.go_on = o.go_on === "press" ? "press" : { after_s: o.go_on.after_s };
  if (o.hud !== undefined) out.hud = o.hud;
  if (o.words !== undefined) out.words = o.words;
  if (o.style !== undefined) out.style = o.style;
  if (o.pace !== undefined) out.pace = typeof o.pace === "string" ? o.pace : { total_s: o.pace.total_s };
  if (o.ends !== undefined) out.ends = o.ends;
  return out;
}

// The link form (design 5.1): `&from=web&stops=each` after the place, e.g. `#chart/world/sys/body~screen?site=1,2&from=web`
// or `#chart/galaxy?from=web` when the place has no settings of its own. Keys in this order, each only when set:
// from, stops (each, none or rungs joined by "."), go_on (press or seconds), hud, words, style, pace (a name or
// seconds), ends.
const LINK_KEYS = ["from", "stops", "go_on", "hud", "words", "style", "pace", "ends"] as const;

// Link text, not a figure for people: rounded to a thousandth of a second.
const num = (x: number) => String(Math.round(x * 1000) / 1000);

/** A way in as link settings, without the leading "?" or "&"; "" when nothing is set. */
export function approachQuery(a: Approach): string {
  const out: string[] = [];
  if (a.from) out.push(`from=${a.from}`);
  if (a.stops) out.push(`stops=${Array.isArray(a.stops) ? a.stops.join(".") : a.stops}`);
  if (a.go_on) out.push(`go_on=${a.go_on === "press" ? "press" : num(a.go_on.after_s)}`);
  if (a.hud) out.push(`hud=${a.hud}`);
  if (a.words) out.push(`words=${a.words}`);
  if (a.style) out.push(`style=${a.style}`);
  if (a.pace) out.push(`pace=${typeof a.pace === "string" ? a.pace : num(a.pace.total_s)}`);
  if (a.ends) out.push(`ends=${a.ends}`);
  return out.join("&");
}

/** A place link with a way in added after it (any way in it carried is replaced). */
export function withApproach(hash: string, a: Approach | null): string {
  const bare = splitApproach(hash).hash;
  const q = a ? approachQuery(a) : "";
  if (!q) return bare;
  return `${bare}${bare.includes("?") ? "&" : "?"}${q}`;
}

/**
 * A place link split into the place (for `parsePlaceHash`) and the way in it carries. Settings that are not a way
 * in's stay with the place; a way in that does not check is dropped (the place is arrived at by the default).
 */
export function splitApproach(hash: string): { hash: string; approach: Approach | null } {
  const q = hash.indexOf("?");
  if (q < 0) return { hash, approach: null };
  const head = hash.slice(0, q);
  const kept: string[] = [];
  const raw: Record<string, unknown> = {};
  let any = false;
  for (const part of hash.slice(q + 1).split("&")) {
    const eq = part.indexOf("=");
    const k = eq < 0 ? part : part.slice(0, eq);
    const v = eq < 0 ? "" : part.slice(eq + 1);
    if (!(LINK_KEYS as readonly string[]).includes(k)) {
      if (part) kept.push(part);
      continue;
    }
    any = true;
    if (k === "stops") raw.stops = v === "each" || v === "none" ? v : v.split(".");
    else if (k === "go_on") raw.go_on = v === "press" ? "press" : { after_s: Number(v) };
    else if (k === "pace") raw.pace = /^\d/.test(v) ? { total_s: Number(v) } : v;
    else raw[k] = v;
  }
  const back = kept.length ? `${head}?${kept.join("&")}` : head;
  return { hash: back, approach: any ? approachFrom(raw) : null };
}
