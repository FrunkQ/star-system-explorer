// The journey (GZ1, design/SSE4-grand-zoom-design.md sections 5.2 to 5.6 and 5.10): one way in (approach.ts),
// played. `journeyPlan` lays out the rungs passed, the stops, the seams and the timings; `journeyAt` gives, for one
// journey time, each view's camera. The journey is a pure function of journey time: nothing here reads a clock, so a
// journey can be paused, stepped back, played at any frame rate and filmed frame by frame (the trailer rig asks for
// time t and gets exactly that frame). Presentation only: never read by physics, never in the state hash.
//
// How it moves (5.3): within a run between two rests (the start, each stop, the turn of a journey that rises first,
// and the end) the picture's width shrinks or grows at one rate in powers of ten, easing in and out at the rests.
// Seams are not rests: the zoom goes on through them while the two views cross-fade (5.4). Directions are each
// rung's own: a rung's aim says the line of sight it opens at (`arrive`), its good viewing angle (`good`, at its stop
// width) and the one it leaves at (`leave`, which should be the next rung's `arrive` seen in this rung's frame:
// `place_chain`, GZ2, supplies that). The last rung ends exactly on the bookmark's pose.
import { REDUCED_FADE_MS } from "./delivery.ts";
import type { Pose, Vec3 } from "./pose.ts";
import { RUNG_INFO, RUNG_WORDS, fullApproach, rungDrawn, rungRank, RUNGS, type Approach, type FullApproach, type LadderRung } from "./approach.ts";

export const JOURNEY_SCHEMA = "sse.journey/1";

/** The shortest cross-fade at a seam, seconds (design 5.4: about a third of a second, never stopping the zoom). */
export const SEAM_FADE_S = 1 / 3;
/** The cross-fade spans this many factors of ten either side of the seam's width, so it is as long in the picture at
 * every pace: the recovered mock-up Alex liked (27 Nov) blends over 0.22 a side, about 0.9 s at stately. A third of a
 * second read as a hard cut on the first real-GPU capture. Never shorter than `SEAM_FADE_S`. */
export const SEAM_FADE_DECADES = 0.22;
/** The short form, per change of rung (design 5.6: about 0.6 s). */
export const SHORT_RUNG_S = 0.6;
/** Seconds per factor of ten in width (5.3: stately is about two; question 8: brisk about 25 s and slow about 90 s
 * for the whole grand zoom against stately's 45). */
export const PACE_S_PER_DECADE = { brisk: 1.1, stately: 2, slow: 4 } as const;
/** At each rest the zoom eases in or out over this many factors of ten. */
export const RAMP_DECADES = 0.3;
/** Under reduced motion each rung passed is held about this long (5.10). */
export const REDUCED_HOLD_S = 2;
/** Under reduced motion a change of rung is the plain fade used everywhere else. */
export const REDUCED_FADE_S = REDUCED_FADE_MS / 1000;
/** Two stops on one chain must differ at least this much in width, or the journey has nothing to zoom through. */
export const MIN_STOP_RATIO = 2;

/** What one rung's view is aimed at, for one target. Directions are unit vectors from the target towards the eye in
 * the rung's own frame (z its pole), as `Pose.eye_dir`; every key may be left out. */
export interface RungAim {
  /** The target's address at this rung (the galaxy, the star, the world), for the view's port. */
  target?: string;
  /** The picture's width at this rung's stop, metres. Left out: `RUNG_INFO[rung].stop_m`. */
  width_m?: number;
  arrive?: Vec3;
  good?: Vec3;
  leave?: Vec3;
  up?: Vec3;
}

export interface JourneyAsk {
  /** The rung of the place arrived at (`rungOfPlace`). */
  to: LadderRung;
  approach?: Approach | null;
  /** Where the viewer is now: needed when the way in is from "here". */
  here?: LadderRung;
  /** The lowest rung here and the place share (`sharedRung`). Left out: the higher of here and the place. */
  shared?: LadderRung;
  /** The highest rung the role travelling knows (5.8): the journey never starts above it. */
  ceiling?: LadderRung;
  /** Aims along the place's chain, by rung. */
  down?: Partial<Record<LadderRung, RungAim>>;
  /** Aims along here's chain, for a journey that rises first. */
  up?: Partial<Record<LadderRung, RungAim>>;
  /** The picture's width where a journey from "here" starts (the view in front now). Left out: here's stop. */
  start_m?: number;
  /** Where it ends: the bookmark's pose as a width and a direction (`journeyEnd`). Left out: the place's stop. */
  end?: { width_m: number; eye_dir?: Vec3; up?: Vec3 };
  reduced_motion?: boolean;
  /** Which rungs a view can draw. Left out: every rung that is not held (`rungDrawn`). */
  drawn?: (r: LadderRung) => boolean;
}

/** One rung passed, in the order passed. A journey that rises first passes "up" rungs, turns at the shared rung and
 * passes "down" rungs; one from a rung passes only "down" rungs (the first is where it starts). */
export interface JourneyEntry {
  rung: LadderRung;
  side: "up" | "turn" | "down";
  target?: string;
  /** The picture's width at this rung's good view. */
  stop_m: number;
  /** When this rung's view is first drawn and last drawn (seam windows included). The port must be ready by `t_in`:
   * read ahead of it, and hold the journey on the rung before if it is not (5.4). */
  t_in: number;
  t_out: number;
}

/** A stop: the zoom at rest, the HUD up (5.5). `hold_s` null: until Go on (`journeyHold`). */
export interface JourneyStop {
  index: number;
  entry: number;
  rung: LadderRung;
  t: number;
  width_m: number;
  hold_s: number | null;
}

/** A change of rung: both views drawn and the outgoing one fading over the incoming one during `fade`. */
export interface JourneySeam {
  from_entry: number;
  to_entry: number;
  t: number;
  width_m: number;
  fade: [number, number];
  /** Under reduced motion nothing zooms: each view keeps its own width through the fade. */
  out_m?: number;
  in_m?: number;
}

export type JourneyLeg = { kind: "move"; t0: number; t1: number; w0: number; w1: number; k: number } | { kind: "rest"; t0: number; t1: number; w: number };

export interface JourneyNote {
  code: "knowledge" | "held" | "widths" | "landing";
  /** One line for the GM's preview or the package log. */
  words: string;
}

interface Key {
  t: number;
  w: number;
  dir: Vec3;
  up: Vec3;
}

export interface JourneyPlan {
  schema: typeof JOURNEY_SCHEMA;
  approach: FullApproach;
  /** The rung the journey starts at, after the role's ceiling and held rungs. */
  from: LadderRung;
  path: JourneyEntry[];
  legs: JourneyLeg[];
  stops: JourneyStop[];
  seams: JourneySeam[];
  total_s: number;
  reduced: boolean;
  /** The last leg is the kernel's flown descent (ends "landing" to a site): the player replaces it (GZ3). */
  landing: boolean;
  notes: JourneyNote[];
  /** Per entry, the camera's directions through time (journeyAt reads them). */
  keys: Key[][];
}

export type JourneyPlanned = { ok: true; plan: JourneyPlan } | { ok: false; why: string };

/** One view's camera at a moment: what the journey port is told (`JourneyPort.aim`). */
export interface JourneyCamera {
  entry: number;
  rung: LadderRung;
  target?: string;
  width_m: number;
  eye_dir: Vec3;
  up: Vec3;
  /** 1 except for the outgoing view in a seam, which is drawn over the incoming one. */
  opacity: number;
}

/** The journey at one moment. */
export interface JourneyFrame {
  t: number;
  /** The entry in front: the place line's current step and the caption's subject. */
  entry: number;
  rung: LadderRung;
  width_m: number;
  /** Views to draw, bottom first: one, or two in a seam (the incoming one under the outgoing one). */
  views: JourneyCamera[];
  stop: JourneyStop | null;
  moving: boolean;
  /** Whether the HUD is up; null leaves it as the viewer has it. */
  hud: boolean | null;
  /** The words at a stop; "none" while moving. */
  words: "captions" | "names" | "none";
  /** The target ringed in the accent (at a stop, unless HUD and words are both off). */
  ring: boolean;
  /** The one line of width while moving (5.7): on unless the way in says HUD never (with the viewer's HUD, only when
   * that is on). */
  width_line: boolean;
  done: boolean;
}

/**
 * The journey port (5.3): what a view that can be travelled through answers. Declared here for GZ3, which adds it
 * to `ViewModule`; views keep their own renderers and their own camera code.
 */
export interface JourneyPort {
  /** The rungs this view draws. */
  readonly rungs: readonly LadderRung[];
  /** Put your camera here. Called every frame the view is drawn on a journey. */
  aim(cam: JourneyCamera): void;
  /** Are your reads in for this target? The player holds the journey on the rung before until they are. */
  ready(cam: Pick<JourneyCamera, "rung" | "target">): boolean;
  /** What is at the middle of the frame, for the caption and the place line. */
  centre(): { address: string; label: string } | null;
}

// --- vectors ---------------------------------------------------------------------------------------------------
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const norm = (v: Vec3): Vec3 => {
  const l = Math.hypot(v[0], v[1], v[2]);
  return l > 1e-12 ? [v[0] / l, v[1] / l, v[2] / l] : [0, 0, 1];
};
const cross = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
/** Any unit vector square to v. */
const perp = (v: Vec3): Vec3 => norm(cross(v, Math.abs(v[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0]));
/** Along the great circle from a to b. */
function slerp(a0: Vec3, b0: Vec3, u: number): Vec3 {
  const a = norm(a0);
  const b = norm(b0);
  const c = Math.max(-1, Math.min(1, dot(a, b)));
  if (c > 0.9999995) return norm([a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u, a[2] + (b[2] - a[2]) * u]);
  const axis = c < -0.9999995 ? perp(a) : null;
  const th = Math.acos(c);
  if (axis) {
    // Opposite directions: turn about any square axis.
    const o = norm(cross(axis, a));
    const ang = Math.PI * u;
    return norm([a[0] * Math.cos(ang) + o[0] * Math.sin(ang), a[1] * Math.cos(ang) + o[1] * Math.sin(ang), a[2] * Math.cos(ang) + o[2] * Math.sin(ang)]);
  }
  const s = Math.sin(th);
  const wa = Math.sin((1 - u) * th) / s;
  const wb = Math.sin(u * th) / s;
  return norm([a[0] * wa + b[0] * wb, a[1] * wa + b[1] * wb, a[2] * wa + b[2] * wb]);
}
/** Up made square to the line of sight. */
function squareUp(up: Vec3, dir: Vec3): Vec3 {
  const d = dot(up, dir);
  const u: Vec3 = [up[0] - d * dir[0], up[1] - d * dir[1], up[2] - d * dir[2]];
  return Math.hypot(u[0], u[1], u[2]) < 1e-6 ? perp(dir) : norm(u);
}

/** A rung's good viewing angle when nothing better is given: tilted, as the shared view controls' "Tilted". */
const TILT = 0.55;
export const DEFAULT_GOOD: Vec3 = [Math.cos(TILT), 0, Math.sin(TILT)];
export const DEFAULT_UP: Vec3 = [-Math.sin(TILT), 0, Math.cos(TILT)];

/** The picture's width a pose shows at its target, metres: what `JourneyAsk.end` takes from a bookmark's pose. */
export function poseWidthM(p: Pose): number {
  return 2 * p.eye_m * Math.tan(((p.fov_deg * Math.PI) / 180) / 2);
}
/** A bookmark's pose as the journey's end. */
export function journeyEnd(p: Pose): NonNullable<JourneyAsk["end"]> {
  return { width_m: poseWidthM(p), eye_dir: p.eye_dir, up: p.up };
}

// --- the zoom profile ------------------------------------------------------------------------------------------
// A run of D factors of ten between two rests, at 1 s a factor of ten: ramps of 2R s at each end (R factors of ten
// each), steady between; a short run is two ramps meeting. Every pace is this profile stretched by k s a decade.
const R = RAMP_DECADES;
const unitTime = (D: number) => (D >= 2 * R ? D + 2 * R : 2 * Math.sqrt(2 * R * D));
/** Factors of ten travelled after τ unit seconds of a run of D. */
function travelled(D: number, tau: number): number {
  const f = unitTime(D);
  const a = 1 / (2 * R);
  if (tau <= 0) return 0;
  if (tau >= f) return D;
  if (D >= 2 * R) {
    const e = 2 * R;
    if (tau < e) return (a * tau * tau) / 2;
    if (tau <= f - e) return R + (tau - e);
    return D - (a * (f - tau) ** 2) / 2;
  }
  const h = f / 2;
  return tau < h ? (a * tau * tau) / 2 : D - (a * (f - tau) ** 2) / 2;
}
/** The unit time at which a run of D has travelled p (the inverse of `travelled`). */
function timeOf(D: number, p: number): number {
  const f = unitTime(D);
  const a = 1 / (2 * R);
  if (p <= 0) return 0;
  if (p >= D) return f;
  if (D >= 2 * R) {
    const e = 2 * R;
    if (p < R) return Math.sqrt((2 * p) / a);
    if (p <= D - R) return e + (p - R);
    return f - Math.sqrt((2 * (D - p)) / a);
  }
  return p < D / 2 ? Math.sqrt((2 * p) / a) : f - Math.sqrt((2 * (D - p)) / a);
}
const lg = Math.log10;

// --- the plan --------------------------------------------------------------------------------------------------
interface Way {
  w: number;
  rest: boolean;
  /** A stop at this rest, on this entry. */
  stop?: number;
  /** A seam from entry `seam` to the next. */
  seam?: number;
  entry: number;
}

/** Lay out a journey. Fails only when there is nothing to travel to: a held rung, or no "here" for "from here". */
export function journeyPlan(ask: JourneyAsk): JourneyPlanned {
  const a = fullApproach(ask.approach, ask.to);
  const drawn = ask.drawn ?? rungDrawn;
  const notes: JourneyNote[] = [];
  const to = ask.to;
  if (!drawn(to)) return { ok: false, why: RUNG_INFO[to].held ?? `${RUNG_WORDS[to]} is not drawn here` };

  // 1. The rungs passed.
  type E = { rung: LadderRung; side: JourneyEntry["side"]; aim: RungAim };
  const entries: E[] = [];
  const between = (lo: number, hi: number) => RUNGS.filter((r) => rungRank(r) > lo && rungRank(r) < hi);
  const heldBetween = (lo: number, hi: number) => between(lo, hi).filter((r) => !drawn(r));
  let from: LadderRung;
  if (a.from === "here") {
    const here = ask.here;
    if (!here) return { ok: false, why: "a journey from here needs to know where here is" };
    if (!drawn(here)) return { ok: false, why: RUNG_INFO[here].held ?? `${RUNG_WORDS[here]} is not drawn here` };
    let shared = ask.shared ?? (rungRank(here) < rungRank(to) ? here : to);
    if (rungRank(shared) > rungRank(here) || rungRank(shared) > rungRank(to)) return { ok: false, why: "the rung both places share must be above both" };
    if (!drawn(shared)) shared = RUNGS.slice(0, rungRank(shared)).filter(drawn).at(-1) ?? "web";
    from = here;
    const upAim = (r: LadderRung) => ask.up?.[r] ?? (r === shared ? (ask.down?.[r] ?? {}) : {});
    // Rising: here, then each drawn rung above it, to just below the shared rung; then the turn; then down.
    if (here !== shared) for (const r of [here, ...between(rungRank(shared), rungRank(here)).filter(drawn).reverse()]) entries.push({ rung: r, side: "up", aim: upAim(r) });
    entries.push({ rung: shared, side: entries.length ? "turn" : "down", aim: ask.down?.[shared] ?? ask.up?.[shared] ?? {} });
    for (const r of between(rungRank(shared), rungRank(to)).filter(drawn)) entries.push({ rung: r, side: "down", aim: ask.down?.[r] ?? {} });
    if (to !== shared) entries.push({ rung: to, side: "down", aim: ask.down?.[to] ?? {} });
    const held = [...heldBetween(rungRank(shared), rungRank(here)), ...heldBetween(rungRank(shared), rungRank(to))];
    for (const r of [...new Set(held)]) notes.push({ code: "held", words: `${RUNG_WORDS[r]} is passed over: ${RUNG_INFO[r].held ?? "no view draws it here"}.` });
  } else {
    from = a.from;
    if (rungRank(from) >= rungRank(to)) return { ok: false, why: `a way in from ${RUNG_WORDS[from]} starts at or below ${RUNG_WORDS[to]}` };
    const capped = !!ask.ceiling && rungRank(from) < rungRank(ask.ceiling);
    if (capped) from = ask.ceiling!;
    if (!drawn(from)) {
      // A held rung to start at: start at the drawn rung above it, or below it when the role knows no higher.
      const up = capped ? undefined : RUNGS.slice(0, rungRank(from)).filter(drawn).at(-1);
      const down = RUNGS.slice(rungRank(from) + 1, rungRank(to)).filter(drawn)[0];
      const next = up ?? down ?? to;
      if (!capped) notes.push({ code: "held", words: `Starts at ${RUNG_WORDS[next]}: ${RUNG_INFO[from].held ?? `${RUNG_WORDS[from]} is not drawn here`}.` });
      from = next;
    }
    if (rungRank(from) > rungRank(to)) from = to;
    if (capped) notes.push({ code: "knowledge", words: `Starts at ${RUNG_WORDS[from]}: they do not know where this place lies above it.` });
    for (const r of [from, ...between(rungRank(from), rungRank(to)).filter(drawn), ...(from === to ? [] : [to])]) entries.push({ rung: r, side: "down", aim: ask.down?.[r] ?? {} });
    for (const r of heldBetween(rungRank(from), rungRank(to))) notes.push({ code: "held", words: `${RUNG_WORDS[r]} is passed over: ${RUNG_INFO[r].held}.` });
  }
  if (a.ends === "landing" && to !== "site") notes.push({ code: "landing", words: "Ends with the camera: only a site has a flown descent." });

  // 2. Widths: each stop narrower than the one above it on its chain, the turn widest.
  const stopW = entries.map((e) => (e.aim.width_m && e.aim.width_m > 0 ? e.aim.width_m : RUNG_INFO[e.rung].stop_m));
  const turnAt = entries.findIndex((e) => e.side === "turn");
  const fix = (i: number, above: number) => {
    if (stopW[i]! * MIN_STOP_RATIO > above) {
      stopW[i] = above / MIN_STOP_RATIO;
      notes.push({ code: "widths", words: `${RUNG_WORDS[entries[i]!.rung]}'s width was more than its parent's; it was narrowed.` });
    }
  };
  const top = turnAt >= 0 ? turnAt : 0;
  for (let i = top + 1; i < entries.length; i++) fix(i, stopW[i - 1]!);
  for (let i = top - 1; i >= 0; i--) fix(i, stopW[i + 1]!);
  const seamW = entries.slice(1).map((_, i) => Math.sqrt(stopW[i]! * stopW[i + 1]!));
  const n = entries.length;
  let startW = a.from === "here" && ask.start_m && ask.start_m > 0 ? ask.start_m : stopW[0]!;
  let endW = ask.end?.width_m && ask.end.width_m > 0 ? ask.end.width_m : stopW[n - 1]!;
  if (n > 1) {
    // The start must lie inside the first rung's band and the end inside the last's, or a seam is crossed twice.
    const firstBand = entries[0]!.side === "up" ? seamW[0]! : Infinity;
    if (entries[0]!.side === "up" && startW * MIN_STOP_RATIO > firstBand) startW = firstBand / MIN_STOP_RATIO;
    if (entries[0]!.side !== "up" && startW < seamW[0]! * MIN_STOP_RATIO) startW = Math.max(startW, seamW[0]! * MIN_STOP_RATIO);
    if (endW * MIN_STOP_RATIO > seamW[n - 2]!) {
      endW = seamW[n - 2]! / MIN_STOP_RATIO;
      notes.push({ code: "widths", words: "The end was wider than its rung; it was narrowed to fit." });
    }
  }

  // 3. The ways: rests (start, stops, the turn, the end) and seams, in the order passed.
  const stopsOn = (i: number) => {
    const e = entries[i]!;
    if (e.side === "up") return false;
    if (a.from === "here" && i === 0) return false;
    if (i === n - 1) return false;
    return a.stops === "each" || (Array.isArray(a.stops) && a.stops.includes(e.rung));
  };
  const reduced = !!ask.reduced_motion;
  const short = a.pace === "short";
  const ways: Way[] = [];
  if (reduced) {
    // Nothing zooms: each rung held (two seconds, or as a stop says), then the plain fade. The short form is one fade.
    const shown = short ? (n > 1 ? [0, n - 1] : [0]) : entries.map((_, i) => i);
    shown.forEach((i, j) => {
      const w = i === 0 ? startW : i === n - 1 ? endW : stopW[i]!;
      ways.push({ w, rest: true, entry: i, ...(stopsOn(i) ? { stop: i } : {}) });
      if (j < shown.length - 1) ways.push({ w, rest: false, seam: i, entry: shown[j + 1]! });
    });
  } else {
    ways.push({ w: startW, rest: true, entry: 0, ...(stopsOn(0) ? { stop: 0 } : {}) });
    if (n === 1 && Math.abs(lg(endW / startW)) > 1e-9) ways.push({ w: endW, rest: true, entry: 0 });
    for (let i = 1; i < n; i++) {
      ways.push({ w: seamW[i - 1]!, rest: false, seam: i - 1, entry: i });
      if (i === n - 1) ways.push({ w: endW, rest: true, entry: i });
      else if (entries[i]!.side === "turn" || stopsOn(i)) ways.push({ w: stopW[i]!, rest: true, entry: i, ...(stopsOn(i) ? { stop: i } : {}) });
    }
  }

  // 4. Timings.
  const legs: JourneyLeg[] = [];
  const stops: JourneyStop[] = [];
  const seams: JourneySeam[] = [];
  const holdOf = (stop: boolean) => (stop ? (a.go_on === "press" ? null : a.go_on.after_s) : 0);
  let t = 0;
  if (reduced) {
    let prevW = ways[0]!.w;
    let prevEntry = 0;
    for (let i = 0; i < ways.length; i++) {
      const wy = ways[i]!;
      if (wy.rest) {
        const isStop = wy.stop !== undefined;
        const last = i === ways.length - 1;
        const hold = isStop ? holdOf(true) : last ? 0 : short ? 0 : REDUCED_HOLD_S;
        if (isStop) stops.push({ index: stops.length, entry: wy.entry, rung: entries[wy.entry]!.rung, t, width_m: wy.w, hold_s: hold });
        legs.push({ kind: "rest", t0: t, t1: t + (hold ?? 0), w: wy.w });
        t += hold ?? 0;
        prevW = wy.w;
        prevEntry = wy.entry;
      } else {
        const next = ways[i + 1]!;
        const ts = t + REDUCED_FADE_S / 2;
        seams.push({ from_entry: prevEntry, to_entry: wy.entry, t: ts, width_m: next.w, fade: [t, t + REDUCED_FADE_S], out_m: prevW, in_m: next.w });
        legs.push({ kind: "rest", t0: t, t1: ts, w: prevW });
        legs.push({ kind: "rest", t0: ts, t1: t + REDUCED_FADE_S, w: next.w });
        t += REDUCED_FADE_S;
      }
    }
  } else {
    const rests = ways.map((w, i) => (w.rest ? i : -1)).filter((i) => i >= 0);
    const runs = rests.slice(1).map((r, j) => ({ from: rests[j]!, to: r, D: Math.abs(lg(ways[r]!.w / ways[rests[j]!]!.w)) }));
    const unit = runs.reduce((s, r) => s + unitTime(r.D), 0);
    const kOf = () => {
      if (typeof a.pace === "object") return unit > 0 ? a.pace.total_s / unit : 0;
      if (a.pace === "short") return unit > 0 ? (SHORT_RUNG_S * Math.max(1, n - 1)) / unit : 0;
      return PACE_S_PER_DECADE[a.pace];
    };
    const k = kOf();
    const restAt = (wi: number) => {
      const wy = ways[wi]!;
      const isStop = wy.stop !== undefined;
      const hold = holdOf(isStop);
      if (isStop) stops.push({ index: stops.length, entry: wy.entry, rung: entries[wy.entry]!.rung, t, width_m: wy.w, hold_s: hold });
      if ((hold ?? 0) > 0) legs.push({ kind: "rest", t0: t, t1: t + hold!, w: wy.w });
      t += hold ?? 0;
    };
    restAt(0);
    for (const run of runs) {
      const w0 = ways[run.from]!.w;
      const w1 = ways[run.to]!.w;
      const dur = unitTime(run.D) * k;
      for (let wi = run.from + 1; wi < run.to; wi++) {
        const wy = ways[wi]!;
        const p = Math.abs(lg(wy.w / w0));
        const ts = t + timeOf(run.D, p) * k;
        const lo = Math.min(ts - SEAM_FADE_S / 2, t + timeOf(run.D, Math.max(0, p - SEAM_FADE_DECADES)) * k);
        const hi = Math.max(ts + SEAM_FADE_S / 2, t + timeOf(run.D, Math.min(run.D, p + SEAM_FADE_DECADES)) * k);
        seams.push({ from_entry: wy.seam!, to_entry: wy.entry, t: ts, width_m: wy.w, fade: [lo, hi] });
      }
      if (dur > 0) legs.push({ kind: "move", t0: t, t1: t + dur, w0, w1, k });
      t += dur;
      restAt(run.to);
    }
  }
  const total = t;
  // Seam windows stay inside the journey, so the first and last frames show one view.
  for (const s of seams) {
    const len = s.fade[1] - s.fade[0];
    const lo = Math.max(0, Math.min(s.fade[0], total - len));
    s.fade = [lo, Math.min(total, lo + len)];
  }
  if (!legs.length) legs.push({ kind: "rest", t0: 0, t1: 0, w: endW });

  // 5. The entries' times and directions.
  const path: JourneyEntry[] = entries.map((e, i) => {
    const sIn = seams.find((s) => s.to_entry === i);
    const sOut = seams.find((s) => s.from_entry === i);
    return { rung: e.rung, side: e.side, ...(e.aim.target ? { target: e.aim.target } : {}), stop_m: stopW[i]!, t_in: sIn ? sIn.fade[0] : 0, t_out: sOut ? sOut.fade[1] : total };
  });
  const plan: JourneyPlan = { schema: JOURNEY_SCHEMA, approach: a, from, path, legs, stops, seams, total_s: total, reduced, landing: a.ends === "landing" && to === "site", notes, keys: [] };
  plan.keys = entries.map((e, i) => {
    const good = e.aim.good ?? DEFAULT_GOOD;
    const up = e.aim.up ?? DEFAULT_UP;
    const keys: Key[] = [];
    const sIn = seams.find((s) => s.to_entry === i);
    const sOut = seams.find((s) => s.from_entry === i);
    const atT = (tt: number, dir: Vec3, u: Vec3) => keys.push({ t: tt, w: widthAt(plan, tt), dir: norm(dir), up: u });
    atT(sIn ? sIn.t : 0, i === 0 ? good : (e.aim.arrive ?? good), up);
    if (!reduced) {
      const tg = crossing(plan, stopW[i]!, sIn ? sIn.t : 0, sOut ? sOut.t : total);
      if (tg !== null && tg > keys[0]!.t) atT(tg, good, up);
    }
    if (i === n - 1) atT(total, ask.end?.eye_dir ?? good, ask.end?.up ?? up);
    else if (sOut) atT(sOut.t, e.aim.leave ?? good, up);
    return keys.sort((x, y) => x.t - y.t);
  });
  return { ok: true, plan };
}

/** The first time in [t0, t1] the picture's width is w (to within a hundredth of a factor of ten), or null. */
function crossing(plan: JourneyPlan, w: number, t0: number, t1: number): number | null {
  for (const l of plan.legs) {
    if (l.t1 < t0 || l.t0 > t1) continue;
    if (l.kind === "rest") {
      if (Math.abs(lg(l.w / w)) < 0.01) return Math.max(l.t0, t0);
      continue;
    }
    const lo = Math.min(l.w0, l.w1);
    const hi = Math.max(l.w0, l.w1);
    if (w < lo * 0.977 || w > hi * 1.023) continue;
    const D = Math.abs(lg(l.w1 / l.w0));
    const tt = l.t0 + timeOf(D, Math.min(D, Math.abs(lg(w / l.w0)))) * l.k;
    if (tt >= t0 - 1e-9 && tt <= t1 + 1e-9) return tt;
  }
  return null;
}

/** The picture's width at journey time t. */
export function widthAt(plan: JourneyPlan, t: number): number {
  const tt = Math.max(0, Math.min(plan.total_s, t));
  let found: JourneyLeg | undefined;
  for (const l of plan.legs) if (tt >= l.t0 - 1e-12) found = l;
  const l = found ?? plan.legs[0]!;
  if (l.kind === "rest") return l.w;
  if (tt >= l.t1) return l.w1;
  const D = Math.abs(lg(l.w1 / l.w0));
  const p = travelled(D, (tt - l.t0) / l.k);
  return l.w0 * 10 ** (l.w1 < l.w0 ? -p : p);
}

function aimAt(plan: JourneyPlan, entry: number, t: number, w: number): { dir: Vec3; up: Vec3 } {
  const keys = plan.keys[entry]!;
  if (t <= keys[0]!.t || keys.length === 1) return { dir: keys[0]!.dir, up: squareUp(keys[0]!.up, keys[0]!.dir) };
  const last = keys[keys.length - 1]!;
  if (t >= last.t) return { dir: last.dir, up: squareUp(last.up, last.dir) };
  let j = 0;
  while (j < keys.length - 2 && t > keys[j + 1]!.t) j++;
  const k0 = keys[j]!;
  const k1 = keys[j + 1]!;
  // Eased by the width, not the clock, so the line of sight is tied to how far the zoom has gone.
  const span = lg(k1.w / k0.w);
  const u = Math.abs(span) > 1e-9 ? Math.max(0, Math.min(1, lg(w / k0.w) / span)) : (t - k0.t) / (k1.t - k0.t);
  const dir = slerp(k0.dir, k1.dir, u);
  return { dir, up: squareUp(slerp(k0.up, k1.up, u), dir) };
}

const smooth = (x: number) => {
  const c = Math.max(0, Math.min(1, x));
  return c * c * (3 - 2 * c);
};

/** The journey at time t (clamped to the journey). */
export function journeyAt(plan: JourneyPlan, t: number): JourneyFrame {
  const tt = Math.max(0, Math.min(plan.total_s, t));
  const w = widthAt(plan, tt);
  let front = 0;
  for (const s of plan.seams) if (tt >= s.t) front = s.to_entry;
  const cam = (entry: number, width: number, opacity: number): JourneyCamera => {
    const e = plan.path[entry]!;
    const { dir, up } = aimAt(plan, entry, tt, width);
    return { entry, rung: e.rung, ...(e.target ? { target: e.target } : {}), width_m: width, eye_dir: dir, up, opacity };
  };
  const seam = plan.seams.find((s) => tt > s.fade[0] && tt < s.fade[1]);
  const views = seam ? [cam(seam.to_entry, seam.in_m ?? w, 1), cam(seam.from_entry, seam.out_m ?? w, 1 - smooth((tt - seam.fade[0]) / (seam.fade[1] - seam.fade[0])))] : [cam(front, w, 1)];
  const stop = plan.stops.find((s) => tt >= s.t - 1e-9 && tt <= s.t + (s.hold_s ?? 0) + 1e-9) ?? null;
  const leg = plan.legs.find((l) => tt >= l.t0 && tt < l.t1);
  const moving = !stop && leg?.kind === "move";
  const a = plan.approach;
  const hud = a.hud === "viewer" ? null : a.hud === "always" ? true : a.hud === "never" ? false : !!stop;
  const words = stop ? a.words : "none";
  return { t: tt, entry: front, rung: plan.path[front]!.rung, width_m: w, views, stop, moving, hud, words, ring: !!stop && !(hud === false && words === "none"), width_line: moving && a.hud !== "never", done: tt >= plan.total_s };
}

/** Journey time as far as the player may go: up to the first stop waiting on Go on that has not been pressed past.
 * `released` counts the presses of Go on so far. */
export function journeyHold(plan: JourneyPlan, t: number, released: number): number {
  const waits = plan.stops.filter((s) => s.hold_s === null);
  const at = waits[released];
  return Math.max(0, Math.min(t, at ? at.t : plan.total_s));
}

/** Back up (5.5): the time of the stop before t (0 when there is none), and how many Go on presses it leaves. */
export function stopBefore(plan: JourneyPlan, t: number): { t: number; released: number } {
  const before = plan.stops.filter((s) => s.t < t - 1e-6);
  const s = before.at(-1);
  if (!s) return { t: 0, released: 0 };
  return { t: s.t, released: plan.stops.filter((x) => x.hold_s === null && x.t < s.t).length };
}
