// The settle schedule (FL1, SSE4-visual-flourish-design.md sections 3 and 4): one rule for every picture that is dear
// to work out. A dear picture is a **proper picture**, a pure function of the date, the place and the view, drawn from
// the kernel's numbers. While the clock, the camera or the place moves faster than the device can rebuild it, a
// **stand-in** is shown; when the movement stops, the proper picture is built unasked and the view **settles** onto
// it. What is seen at rest is always the proper picture (Alex, 28 Nov 2026: "fake it if time moves too fast" and fall
// back to "proper render" when you stop).
//
// A stand-in is presentation only: nothing reads it back (no save, pack, postcard, report, rule or press), tracked
// things are always placed from the kernel, and each use is declared in `stand-ins.json` with its fudge register row.
//
// Pure and clock-free, as `RestClock` (renderer/src/still.ts) is: callers pass wall times in milliseconds and the
// picture clock's rate; nothing here reads a clock, draws or keeps a picture. The shader kit is FL2, the warm-up
// queue FL7.

/** What can make a dear picture stale. */
export type Movement = "clock" | "camera" | "place";

/** The six kinds of stand-in, most honest first (design 4.2). A use offers them in order; `hint` needs Alex's yes. */
export const STAND_IN_KINDS = ["coarse", "held", "carried", "exposure", "seam", "hint"] as const;
export type StandInKind = (typeof STAND_IN_KINDS)[number];

/** The four states of the schedule (design 4.3). */
export type SettleState = "rest" | "live" | "carried-on" | "too-fast";

/** The record a layer whose picture is dear declares (design 4.1). The builder itself stays with the view. */
export interface DearLayer {
  /** Its name in the declared list and the register, such as `giant.cloud`. */
  id: string;
  /** Which movements make it stale. */
  moves_with: readonly Movement[];
  /** Clock seconds before the picture has changed enough to see, from the kernel; null where the clock does not age it. */
  own_time_s: number | null;
  /** The shortest turn of what the picture rides on (a world's day), clock seconds; null for none. */
  cycle_s: number | null;
  /** The stand-in kinds it offers, in the order it prefers them. */
  stand_in: readonly StandInKind[];
}

/** Still this long, ms, and the view is at rest (the rest clock's figure, still.ts `REST_AFTER_MS`). */
export const SETTLE_REST_MS = 500;
/** At most one rebuild a second for each layer while anything moves (Alex's "1 fps"), ms between asks. */
export const REBUILD_CAP_MS = 1000;
/** The settle's cross-fade, ms (the interface's own motion time). */
export const SETTLE_FADE_MS = 240;
/** The sizes a proper picture is built at, by width in pixels: the smallest that covers the layer on screen. */
export const SIZE_LADDER = [64, 128, 256, 512, 1024] as const;

/** The ladder size for a layer `px` wide on screen: built at the size it is shown, never above the ladder's top. */
export function sizeFor(px: number): number {
  for (const s of SIZE_LADDER) if (px <= s) return s;
  return SIZE_LADDER[SIZE_LADDER.length - 1]!;
}

/**
 * The spacing of rebuild dates while the clock runs, clock seconds: the layer's own time, doubled until one step
 * outlasts the clock time a build and the cap between them cover (a slow device gets a coarser lattice, design 4.3).
 * Null for a layer the clock does not age.
 */
export function latticeStep(own_time_s: number | null, rate: number, build_ms: number): number | null {
  if (own_time_s === null || !(own_time_s > 0)) return null;
  const need = Math.abs(rate) * (Math.max(build_ms, REBUILD_CAP_MS) / 1000);
  let step = own_time_s;
  // Doubling is exact in binary, so two devices on the same spacing find the same dates.
  for (let k = 0; k < 64 && step < need; k++) step *= 2;
  return step;
}

/**
 * The lattice date at or before `t_s`, anchored at the campaign's epoch (clock zero), so scrubbing back and forth finds
 * pictures already made and two devices on the same spacing show the same thing.
 */
export function latticeDate(t_s: number, step_s: number): number {
  if (!(step_s > 0) || !Number.isFinite(t_s)) return t_s;
  return Math.floor(t_s / step_s) * step_s;
}

/** What the schedule needs to know at a moment. */
export interface SettleMoment {
  /** Wall time, ms (performance.now's). */
  now_ms: number;
  /** Clock seconds per real second (0 stopped). */
  rate: number;
  /** Pictures a second the view draws. */
  pictures_per_s: number;
  /** The measured (or estimated) time to build the proper picture at the size on screen, ms. */
  build_ms: number;
}

/**
 * Whether the clock outruns one picture of this layer: one picture covers more than its own time, or the thing it
 * rides on turns in under two pictures (CLK's "faster than the pictures"). A sampling limit, not the device's.
 */
export function outrunsPictures(layer: DearLayer, rate: number, pictures_per_s: number): boolean {
  if (!layer.moves_with.includes("clock") || !(rate > 0) || !(pictures_per_s > 0)) return false;
  const span = rate / pictures_per_s;
  if (layer.own_time_s !== null && span > layer.own_time_s) return true;
  return layer.cycle_s !== null && layer.cycle_s > 0 && layer.cycle_s < 2 * span;
}

/** A proper picture asked for: by layer, date, view and size. Two asks with the same key are the same picture. */
export interface ProperAsk {
  id: string;
  /** Clock seconds since the campaign's epoch: the exact date at rest, a lattice date while the clock runs. */
  date_s: number;
  /** The view's own key (camera pose and place, as the view words it). */
  view: string;
  /** Width on the size ladder. */
  size: number;
  /** True at rest (the exact date and view); false for a rebuild while moving. */
  exact: boolean;
}

/** The one key a proper picture is kept and found by. */
export function properKey(a: Pick<ProperAsk, "id" | "date_s" | "view" | "size">): string {
  return `${a.id}|${a.date_s}|${a.view}|${a.size}`;
}

/**
 * The schedule for one layer. Call `moved(what, now)` on every change of the date, camera or place that is not the
 * clock simply running (a step, a drag, a jump, a turn of the camera); `state(m)` for what to draw; `ask(...)` when
 * the frame has spare time, to learn which proper picture to build, if any; `started` and `built` round each build.
 */
export class Settle {
  readonly layer: DearLayer;
  #moved_ms: Record<Movement, number> = { clock: Number.NEGATIVE_INFINITY, camera: Number.NEGATIVE_INFINITY, place: Number.NEGATIVE_INFINITY };
  #asked_ms = Number.NEGATIVE_INFINITY;
  #building: string | null = null;
  #have: string | null = null;
  #rest_done: string | null = null;

  constructor(layer: DearLayer) {
    this.layer = layer;
  }

  /** Something the layer moves with has changed (not the clock merely running: its rate says that). */
  moved(what: Movement, now_ms: number): void {
    if (this.layer.moves_with.includes(what)) this.#moved_ms[what] = now_ms;
  }

  #recent(what: Movement, now_ms: number): boolean {
    return this.layer.moves_with.includes(what) && now_ms - this.#moved_ms[what] < SETTLE_REST_MS;
  }

  /** Whether anything this layer moves with is moving at `m`. */
  moving(m: Pick<SettleMoment, "now_ms" | "rate">): boolean {
    if (this.layer.moves_with.includes("clock") && m.rate > 0) return true;
    return this.#recent("clock", m.now_ms) || this.#recent("camera", m.now_ms) || this.#recent("place", m.now_ms);
  }

  /**
   * Whether the date is being dragged or stepped with the clock stopped: a movement of unknown rate (design 4.3). The
   * view shows `dragPicture`'s choice and nothing is rebuilt until it is let go; on release, Rest.
   */
  dragging(m: Pick<SettleMoment, "now_ms" | "rate">): boolean {
    return !(m.rate > 0) && this.#recent("clock", m.now_ms) && !this.#recent("camera", m.now_ms) && !this.#recent("place", m.now_ms);
  }

  /** The state at `m` (design 4.3). At rest it is Rest, whatever came before. */
  state(m: SettleMoment): SettleState {
    if (!this.moving(m)) return "rest";
    const l = this.layer;
    if (outrunsPictures(l, m.rate, m.pictures_per_s)) return "too-fast";
    const build_s = Math.max(0, m.build_ms) / 1000;
    if (l.moves_with.includes("clock") && m.rate > 0 && l.own_time_s !== null) {
      // A build lands while the picture it makes is still current: the viewer cannot tell it from every frame.
      return m.rate * build_s < l.own_time_s ? "live" : "carried-on";
    }
    // The camera or the place moving: live when a proper picture can be had within one picture's time.
    const frame_s = m.pictures_per_s > 0 ? 1 / m.pictures_per_s : Number.POSITIVE_INFINITY;
    return build_s <= frame_s ? "live" : "carried-on";
  }

  /**
   * Which proper picture to build now, or null. `at_s` is the date in view (clock seconds since the epoch), `view` the
   * view's key, `px` the layer's width on screen, `spare` whether the frame has time to spare (the governor under
   * budget). At rest: one build for the exact date and view, then none. While moving: a lattice date, at most once a
   * second, one at a time, never while too fast. Nothing is asked twice for a picture already made.
   */
  ask(m: SettleMoment & { at_s: number; view: string; px: number; spare: boolean }): ProperAsk | null {
    if (this.#building !== null) return null;
    const s = this.state(m);
    const size = sizeFor(m.px);
    if (s === "rest") {
      const a: ProperAsk = { id: this.layer.id, date_s: m.at_s, view: m.view, size, exact: true };
      const key = properKey(a);
      if (this.#rest_done === key || this.#have === key) {
        this.#rest_done = key;
        return null;
      }
      return a;
    }
    this.#rest_done = null;
    if (s === "too-fast" || !m.spare || this.dragging(m)) return null;
    if (m.now_ms - this.#asked_ms < REBUILD_CAP_MS) return null;
    const step = latticeStep(this.layer.own_time_s, m.rate, m.build_ms);
    const date_s = step === null ? m.at_s : latticeDate(m.at_s, step);
    const a: ProperAsk = { id: this.layer.id, date_s, view: m.view, size, exact: false };
    return properKey(a) === this.#have ? null : a;
  }

  /** A build of `a` has begun. */
  started(a: ProperAsk, now_ms: number): void {
    this.#building = properKey(a);
    this.#asked_ms = now_ms;
  }

  /** The build of `a` is done (or given up, with `ok` false). */
  built(a: ProperAsk, ok = true): void {
    const key = properKey(a);
    if (this.#building === key) this.#building = null;
    if (!ok) return;
    this.#have = key;
    if (a.exact) this.#rest_done = key;
  }

  /** The key of the last proper picture made, or null. */
  get have(): string | null {
    return this.#have;
  }
}

/** What the device can do, for choosing a stand-in. */
export interface StandInHost {
  /** Reduced motion asked for: the Carried and Hint kinds are off (Held shows). */
  reduced: boolean;
  /** A GPU tier: without one (canvas, text) only Held and the settle. */
  gpu: boolean;
}

/**
 * What to draw in a state: "proper" (with the settle fade onto it at Rest), or the stand-in kind. Live draws the
 * proper picture with the layer's first stand-in filling the frames between builds; Carried on draws the stand-in;
 * Too fast draws the Exposure kind where the use offers one, else Held.
 */
export function drawFor(layer: DearLayer, state: SettleState, host: StandInHost): "proper" | StandInKind {
  if (state === "rest") return "proper";
  const allowed = (k: StandInKind) => (host.gpu || k === "held") && !(host.reduced && (k === "carried" || k === "hint"));
  if (state === "too-fast") return layer.stand_in.includes("exposure") && allowed("exposure") ? "exposure" : "held";
  for (const k of layer.stand_in) if (k !== "exposure" && k !== "seam" && allowed(k)) return k;
  return "held";
}

/** The settle's weight on the new proper picture, 0 to 1, `since_ms` after it was ready (one plain fade, opacity only). */
export function settleWeight(since_ms: number): number {
  if (!(since_ms > 0)) return 0;
  return Math.min(1, since_ms / SETTLE_FADE_MS);
}

/**
 * Which kept picture a drag on the ruler shows at `t_s` (design 4.3): within one lattice step of a kept picture, carry
 * that one (forwards or backwards); further, the nearest kept picture held; none kept, the Too fast picture.
 */
export function dragPicture(t_s: number, kept_s: readonly number[], step_s: number): { kind: "carried" | "held"; from_s: number } | { kind: "too-fast" } {
  let best: number | null = null;
  for (const d of kept_s) if (best === null || Math.abs(d - t_s) < Math.abs(best - t_s)) best = d;
  if (best === null) return { kind: "too-fast" };
  return Math.abs(best - t_s) <= step_s ? { kind: "carried", from_s: best } : { kind: "held", from_s: best };
}

/** Asks from several layers in the order to build them: the biggest on screen first. */
export function bySize(asks: readonly ProperAsk[]): ProperAsk[] {
  return [...asks].sort((a, b) => b.size - a.size || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
}
