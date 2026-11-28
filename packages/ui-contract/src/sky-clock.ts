// The one epoch (RW17 part 2; time design section 4, TP2's first slice).
//
// The screens count a viewed system's time in years from its star's birth
// (the workspace's clock); the kernel counts universe seconds. A system the
// kernel holds reports the universe second it was born (`birth_s`), so the
// second a screen draws is `birth_s + age`, one mapping for every system.
// A real-sky system (base-sol) also carries `time.j2000_s` on its star: the
// universe second of J2000.0 TT. Its calendar and the date its door opens on
// are read from that fact, never typed again.
import leapDoc from "./leap-seconds.json" with { type: "json" };
import type { PackCalendar } from "./session.ts";
import type { OrbitPlaced } from "./orbits.ts";
import { GREGORIAN } from "./calendar.ts";
import { yearsToSeconds } from "./intent.ts";

/** The fact on a real-sky system's star that anchors it in universe time (kernel `motion::ANCHOR_FIELD`). */
export const ANCHOR_FIELD = "time.j2000_s";

/** The universe second drawn for a time on the screen's axis (years): the viewed system's birth second plus the age. */
export function drawnSecond(birth_s: bigint | null | undefined, yr: number): bigint {
  return (birth_s ?? 0n) + yearsToSeconds(yr);
}

/** The screen's years for a universe second (the inverse of `drawnSecond`). */
export function screenYears(birth_s: bigint | null | undefined, at_s: bigint): number {
  return Number(at_s - (birth_s ?? 0n)) / Number(yearsToSeconds(1));
}

/** A real-sky system's anchor, exact, from its facts as `resolved_system` gives them: the first star row carrying it. */
export function anchorOf(facts: Record<string, Record<string, unknown>>): bigint | null {
  for (const row of Object.values(facts)) {
    const v = row[ANCHOR_FIELD];
    if (typeof v === "bigint") return v;
    if (typeof v === "number" && Number.isSafeInteger(v)) return BigInt(v);
    if (typeof v === "string" && /^-?\d+$/.test(v)) return BigInt(v);
  }
  return null;
}

/** Seconds in a day, as the calendar counts them. */
const DAY = BigInt(GREGORIAN.day_s);

/**
 * The everyday calendar for a real-sky system: Gregorian days from midnight starting J2000.0's day, counted on the
 * universe clock (TT), so its epoch is the anchor less half a day. The same days the Sol today pack's calendar counts.
 */
export function skyCalendar(j2000_s: bigint): PackCalendar {
  return { ...GREGORIAN, name: "Gregorian (read on TT)", epoch_s: Number(j2000_s - DAY / 2n) } as PackCalendar;
}

/** Unix milliseconds of 12:00 on 1 January 2000 UTC; J2000.0 is that moment on TT, which ran 64.184 s ahead. */
const NOON_2000_UNIX_MS = 946_728_000_000n;

/** TAI minus UTC, in seconds, at a moment of the device's clock (Unix ms), from the cited list. */
export function taiMinusUtc(unix_ms: number): number {
  let s = leapDoc.steps[0]!.tai_utc_s;
  for (const st of leapDoc.steps) if (unix_ms >= st.from_unix_ms) s = st.tai_utc_s;
  return s;
}

/**
 * A moment of the device's clock (UTC, Unix milliseconds) as a universe second (TT, whole, rounded down), for a
 * system whose J2000.0 is `j2000_s`. Counted in integers from J2000.0; only the device's own milliseconds enter.
 */
export function universeSecondOf(unix_ms: number, j2000_s: bigint): bigint {
  const ms = BigInt(Math.trunc(unix_ms)) + BigInt(taiMinusUtc(unix_ms) * 1000 + leapDoc.tt_minus_tai_ms) - NOON_2000_UNIX_MS;
  const whole = ms >= 0n ? ms / 1000n : -((-ms + 999n) / 1000n);
  return j2000_s + whole;
}

/** What the chart says about where the bodies were placed from (RW17): only when a body names a motion model. */
export function motionWord(bodies: readonly OrbitPlaced[]): { word: "measured" | "plausible" | "mixed"; text: string } | null {
  const named = bodies.filter((b) => b.motion && !b.motion.unused);
  if (!named.length) return null;
  const says = new Set(named.map((b) => b.at?.[0]?.motion ?? "plausible"));
  if (says.size === 1 && says.has("measured")) return { word: "measured", text: "As measured: the bodies follow their published motion series at this date." };
  if (says.size === 1) return { word: "plausible", text: "Plausible: this date is outside the published motion series, so the bodies follow orbits fitted at the series' edge." };
  return { word: "mixed", text: "Partly measured: some bodies follow their published motion series at this date; the rest follow orbits fitted at the series' edge." };
}
