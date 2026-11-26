// The picture's clock (CLK1): one source of time for everything that moves on screen. Alex, 26 Nov 2026: "advancing
// time fast animates a sun, planets whizz round and asteroids/belts circle at their correct speeds". Every motion a view
// draws (a star's spin, spots and cells, a world's orbit and turn, a belt's members, a sea's waves) reads its time
// here, so at real time the motion is the real motion, at a day a second it is a day a second, and with the clock
// stopped the picture is still. Rates come from the physics (an orbit's period, a spin, a cell's life), never from here.
//
// The time is counted in seconds since the campaign's epoch (its start), not as years: years near the age of a star
// only resolve about 30 s in a 64-bit float, which would make real-time motion stutter. The ruler and the Run desk's
// clock set the rate; between their updates (ten a second) the picture runs on smoothly from the last anchor.
import { YEAR_S } from "./intent.ts";
import { constant } from "./sheet.ts";

const YR_S = Number(YEAR_S);
/** Seconds in the kernel's year (the one `yearsToSeconds` counts in), as a plain number. */
export const YEAR_SECONDS = YR_S;

export interface PictureClock {
  /** The campaign's epoch, years: the zero of `at_s`. */
  epoch_yr: number;
  /** Clock seconds since the epoch at the wall time `wall_ms`. */
  at_s: number;
  /** The wall time of the anchor, milliseconds (performance.now's). */
  wall_ms: number;
  /** Clock seconds per real second: 1 is real time, 86,400 a day a second, 0 stopped. */
  rate: number;
}

/** A stopped clock at a time in years. */
export function stoppedAt(now_yr: number, epoch_yr: number, wall_ms = 0): PictureClock {
  return { epoch_yr, at_s: (now_yr - epoch_yr) * YR_S, wall_ms, rate: 0 };
}

/** The clock's time at a wall time, seconds since the epoch. */
export function clockAt(c: PictureClock, wall_ms: number): number {
  return c.at_s + (c.rate * (wall_ms - c.wall_ms)) / 1000;
}

/** Years from a clock time, for anything that reads years (the scene, the ruler). */
export function clockYears(c: PictureClock, t_s: number): number {
  return c.epoch_yr + t_s / YR_S;
}

/**
 * The time being viewed moved (a step, a drag, a tick of play): follow it. While running, a tick that agrees with the
 * clock to within `slack_s` keeps the anchor, so the picture is not knocked about by the ruler's rounding; a jump
 * (a drag, a step, "Back to now") re-anchors. Stopped, the clock always takes the new time.
 */
export function followTime(c: PictureClock, now_yr: number, epoch_yr: number, wall_ms: number): PictureClock {
  const at_s = (now_yr - epoch_yr) * YR_S;
  if (c.rate === 0 || epoch_yr !== c.epoch_yr) return { epoch_yr, at_s, wall_ms, rate: c.rate };
  const drift = Math.abs(clockAt(c, wall_ms) - at_s);
  // The ruler sends ten times a second and rounds the time to its float: half a second of clock time is the most a
  // tick can honestly lag, and anything a 64-bit year cannot resolve (about 30 s at a star's age) is rounding too.
  const slack_s = Math.max(c.rate * 0.5, Math.abs(now_yr) * Number.EPSILON * 4 * YR_S);
  return drift <= slack_s ? c : { epoch_yr, at_s, wall_ms, rate: c.rate };
}

/** Start, stop or change the rate, from the time the clock shows now (so nothing jumps). */
export function withRate(c: PictureClock, rate: number, wall_ms: number): PictureClock {
  return { epoch_yr: c.epoch_yr, at_s: clockAt(c, wall_ms), wall_ms, rate: Math.max(0, rate) };
}

/** Clock seconds one picture stands for, at `pictures_per_s` pictures a second (0 while stopped). */
export function pictureSpan(c: PictureClock, pictures_per_s: number): number {
  return pictures_per_s > 0 ? c.rate / pictures_per_s : 0;
}

/**
 * Whether a motion of this period is faster than the pictures can show it. Fewer than two pictures a cycle (the
 * sampling limit) and a motion cannot be followed: it seems to jump, stand still or run backwards. Views draw such a
 * motion where it truly is at each picture (never slowed to look smooth) and say so in their note.
 */
export function tooFast(period_s: number, span_s: number): boolean {
  return Number.isFinite(period_s) && period_s > 0 && span_s > 0 && Math.abs(period_s) < 2 * span_s;
}

/** A turn of `period_s` at clock time `t_s`, 0 to 1 (0 for no period). */
export function turnOf(period_s: number, t_s: number): number {
  if (!Number.isFinite(period_s) || period_s === 0) return 0;
  const f = (t_s / period_s) % 1;
  return f < 0 ? f + 1 : f;
}

/**
 * A body's phase on its orbit at clock time `t_s`, degrees: from the kernel's phase at `at_s` (its `orbits_at`), run
 * on at the mean motion 360° per period. Between the kernel's answers this keeps a body moving smoothly; the next
 * answer puts it back where Kepler's equation says (fudge register, "Orbits run on between the kernel's answers").
 */
export function phaseAt(phase_deg: number, at_s: number, period_s: number | undefined, t_s: number): number {
  if (!period_s || !Number.isFinite(period_s) || period_s <= 0) return phase_deg;
  const p = (phase_deg + (360 * (t_s - at_s)) / period_s) % 360;
  return p < 0 ? p + 360 : p;
}

/** The rate in words, for a view's note: "stopped", "real time", "a day a second" and so on. */
export function rateWords(rate: number): string {
  if (!(rate > 0)) return "stopped";
  if (Math.abs(rate - 1) < 1e-9) return "real time";
  const day = constant("day");
  const units: [number, string][] = [
    [YR_S, "year"],
    [7 * day, "week"],
    [day, "day"],
    [day / 24, "hour"],
    [day / 1440, "minute"],
  ];
  for (const [s, name] of units) {
    const n = rate / s;
    const whole = Math.abs(n - Math.round(n)) < 0.01 * n;
    // Weeks only when whole: thirty days a second reads better than 4.3 weeks.
    if (n < 0.999 || (name === "week" && !whole)) continue;
    const said = whole ? Math.round(n) : Math.round(n * 10) / 10;
    return said === 1 ? `${name === "hour" ? "an" : "a"} ${name} a second` : `${said} ${name}s a second`;
  }
  return `${Math.round(rate * 10) / 10} seconds a second`;
}

/**
 * What a view says about its motion: at what rate it runs, and which named motions are faster than its pictures.
 * Empty `fast` gives the rate alone.
 */
export function motionWords(c: PictureClock, fast: readonly string[]): string {
  if (!(c.rate > 0)) return "The clock is stopped, so the picture is still: press play on the ruler (or Run on the clock) and everything moves at its own speed.";
  const head = `Everything moves on the campaign's clock, ${rateWords(c.rate)}.`;
  if (!fast.length) return head;
  const list = fast.length === 1 ? fast[0]! : `${fast.slice(0, -1).join(", ")} and ${fast[fast.length - 1]}`;
  return `${head} ${list} ${fast.length === 1 ? "goes" : "go"} round faster than the screen can show, so ${fast.length === 1 ? "it is" : "they are"} drawn where ${fast.length === 1 ? "it" : "they"} truly ${fast.length === 1 ? "is" : "are"} at each picture and may seem to jump or run backwards.`;
}
