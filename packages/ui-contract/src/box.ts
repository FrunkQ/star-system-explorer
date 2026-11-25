// A handle's box (ED12, editing design section 11F): the Orbit and Spin boxes, and any box a pack declares in the
// handle sheet. The parts are strips and rings on kernel fields; the chips are one set_value each; the results under
// the box are the kernel's (the year from orbits_at, the verdict from orbit_stability) or the orbit's own geometry
// (closest and farthest point, the sunlight's swing, the sun's day). Presentation arithmetic only: nothing here is a
// physical model, and every figure is said by the units engine.
import type { Depth } from "./depth.ts";
import type { BoxChip, BoxPart, Handle } from "./handles.ts";
import type { StabilityCheck } from "./session.ts";

const ORDER: Record<Depth, number> = { curious: 0, science: 1, engine: 2 };
const TAU = 2 * Math.PI;

/** Shown at this depth: a part or chip marked "science" is hidden at Curious. */
export const atDepth = (x: { depth: Depth }, depth: Depth): boolean => ORDER[depth] >= ORDER[x.depth];

/** The box's parts and chips at a depth (none for a handle without a box). */
export function boxAt(h: Handle, depth: Depth): { parts: BoxPart[]; chips: BoxChip[]; results: string[] } {
  if (!h.box) return { parts: [], chips: [], results: [] };
  return { parts: h.box.parts.filter((p) => atDepth(p, depth)), chips: h.box.chips.filter((c) => atDepth(c, depth)), results: h.box.results };
}

/** A box part as a handle of its own, so the one strip draws it (no reference dots; its own range). */
export function partHandle(h: Handle, p: BoxPart): Handle {
  return { id: `${h.id}.${p.id}`, on: h.on, names: p.names, part: p.part, refs: [], desks: h.desks };
}

/** An angle wrapped into 0 to 2π. */
export const wrap = (x: number): number => ((x % TAU) + TAU) % TAU;

/** The mirror of a lean past 90 degrees: the Retrograde chip (a lean of 10° becomes 170°, and back). */
export const mirrorAngle = (x: number): number => Math.PI - x;

/** The true anomaly (radians) for a mean anomaly on an orbit of eccentricity e (Kepler's equation by Newton's method). */
export function trueFromMean(M: number, e: number): number {
  const m = wrap(M);
  let E = e < 0.8 ? m : Math.PI;
  for (let k = 0; k < 50; k++) {
    const d = (E - e * Math.sin(E) - m) / (1 - e * Math.cos(E));
    E -= d;
    if (Math.abs(d) < 1e-13) break;
  }
  return wrap(2 * Math.atan2(Math.sqrt(1 + e) * Math.sin(E / 2), Math.sqrt(1 - e) * Math.cos(E / 2)));
}

/** The mean anomaly (radians) for a true anomaly (radians). */
export function meanFromTrueRad(nu: number, e: number): number {
  const E = 2 * Math.atan2(Math.sqrt(1 - e) * Math.sin(nu / 2), Math.sqrt(1 + e) * Math.cos(nu / 2));
  return wrap(E - e * Math.sin(E));
}

/**
 * The orbit's place now as the true anomaly (angle from the closest point, radians), from the kernel's position
 * relative to the host (orbits_at `r_m`) and the orbit's angles: the position measured against the line to the
 * closest point, so it holds at any lean.
 */
export function trueAnomalyFrom(r: readonly [number, number, number], o: { inc: number; node: number; peri: number }): number {
  const [co, so, cw, sw, ci, si] = [Math.cos(o.node), Math.sin(o.node), Math.cos(o.peri), Math.sin(o.peri), Math.cos(o.inc), Math.sin(o.inc)];
  const P = [co * cw - so * sw * ci, so * cw + co * sw * ci, sw * si];
  const Q = [-co * sw - so * cw * ci, -so * sw + co * cw * ci, cw * si];
  return wrap(Math.atan2(r[0] * Q[0]! + r[1] * Q[1]! + r[2] * Q[2]!, r[0] * P[0]! + r[1] * P[1]! + r[2] * P[2]!));
}

/**
 * The mean anomaly at the orbit's epoch that puts the world at true anomaly `nu` at the campaign's now: what the
 * "Where it is now" ring writes.
 */
export function epochMeanFor(nu: number, e: number, period_s: number, now_s: number, epoch_s: number): number {
  return wrap(meanFromTrueRad(nu, e) - (TAU * (now_s - epoch_s)) / period_s);
}

/** Closest and farthest distance, and how much more sunlight the closest point gets than the farthest (a share). */
export function orbitShape(a: number, e: number): { closest: number; farthest: number; swing: number } {
  return { closest: a * (1 - e), farthest: a * (1 + e), swing: ((1 + e) / (1 - e)) ** 2 - 1 };
}

/**
 * The sun's day (noon to noon) for a turn of `rotation_s` (negative spins backwards) on an orbit of `year_s`, or null
 * when the two are equal (the same face always looks at the star).
 */
export function solarDay(rotation_s: number, year_s: number): number | null {
  const f = 1 / rotation_s - 1 / year_s;
  if (Math.abs(f) * Math.abs(rotation_s) < 1e-3) return null;
  return Math.abs(1 / f);
}

export interface Resonance {
  /** "3:2 with Neptune". */
  label: string;
  /** What it means, said in words. */
  says: string;
  /** The size that puts this world's year in that ratio. */
  a: number;
  /** Already there, within a thousandth. */
  on: boolean;
}

const RATIOS: readonly [number, number][] = [
  [2, 1],
  [3, 2],
  [4, 3],
  [5, 3],
  [5, 2],
  [3, 1],
];

/**
 * The "In step with" chips: for the nearest neighbour inside and outside (round the same host), the ratio of years
 * nearest to where this world is now, if within a quarter. The size comes from the year by Kepler's third law with
 * this world's own year as the yardstick (a ∝ P^2/3 round one host), so the kernel's own period is kept.
 */
export function resonances(self: { a: number; year: number }, others: readonly { label: string; a: number; year: number }[]): Resonance[] {
  const inner = others.filter((o) => o.a < self.a).sort((x, y) => y.a - x.a)[0];
  const outer = others.filter((o) => o.a > self.a).sort((x, y) => x.a - y.a)[0];
  const out: Resonance[] = [];
  for (const [n, outside] of [
    [inner, true],
    [outer, false],
  ] as const) {
    if (!n) continue;
    const now = outside ? self.year / n.year : n.year / self.year;
    let best: [number, number] | null = null;
    for (const r of RATIOS) if (!best || Math.abs(Math.log(r[0] / r[1] / now)) < Math.abs(Math.log(best[0] / best[1] / now))) best = r;
    const [p, q] = best!;
    if (Math.abs(Math.log(p / q / now)) > Math.log(1.25)) continue;
    const year = outside ? (n.year * p) / q : (n.year * q) / p;
    out.push({
      label: `${p}:${q} with ${n.label}`,
      says: outside ? `${n.label} goes round ${p} times while this goes round ${q}` : `this goes round ${p} times while ${n.label} goes round ${q}`,
      a: self.a * (year / self.year) ** (2 / 3),
      on: Math.abs(year / self.year - 1) < 1e-3,
    });
  }
  return out;
}

const LEVEL = ["Lasts", "Watch", "Will not last", "The orbits cross"];

/** The stability verdict for one world: its worst check (of those naming it), in words, with what that check says. */
export function stabilityFor(checks: readonly StabilityCheck[], address: string): { level: number; word: string; says: string; check: string } | null {
  const mine = checks.filter((c) => c.bodies?.includes(address));
  if (!mine.length) return null;
  const worst = mine.reduce((w, c) => (c.level > w.level ? c : w));
  return { level: worst.level, word: LEVEL[Math.min(3, Math.max(0, worst.level))]!, says: worst.says, check: worst.check };
}

/**
 * The outer wall on an orbit's size strip, from the kernel's stability checks: the widest orbit that lasts round its
 * host (a moon round its planet against the star's pull; a world round one star of a pair). Null when no check gives one.
 */
export function outerWall(checks: readonly StabilityCheck[], address: string): { at: number; says: string } | null {
  const c = checks.find((x) => (x.check === "moon" || x.check === "binary_one_star") && x.bodies?.[0] === address && typeof x.results?.limit === "number");
  if (!c) return null;
  return { at: c.results!.limit!, says: c.check === "moon" ? "the star pulls it away beyond" : "the other star throws it out beyond" };
}
