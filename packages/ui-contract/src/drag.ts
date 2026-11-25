// Dragging on the chart (ED3 part 2, editing design section 4): geared dragging, Shift finer, and light magnetism to
// round figures and to resonances with the neighbours, named as it catches, Alt off (question 10, Alex yes). Tipping
// points join the catches when the kernel's sweep (K1) gives them. Presentation arithmetic only: a catch is a place
// on the line, never a physical result.
import { resonances } from "./box.ts";
import { sayNumber } from "./figures.ts";

/**
 * How much of the pointer's move along the radius counts: all of it on the line, less the further the pointer is
 * from it (a tenth more slowly with Shift), so a fine adjustment is a matter of moving the hand away from the line.
 */
export function gearing(offLinePx: number, finer: boolean): number {
  return (finer ? 0.1 : 1) / (1 + Math.max(0, offLinePx) / 80);
}

/** A place a drag catches at: a value (in AU here) and its name. */
export interface Catch {
  at: number;
  says: string;
}

/**
 * Round figures near a value, in the unit it is shown in (`per` units of the value: AU per AU, or km per AU for a
 * moon's orbit): 1, 1.5, 2 … 9.5 times a power of ten, said "round" plus the figure.
 */
export function roundFigures(v: number, per: number, unit: string): Catch[] {
  if (!(v > 0)) return [];
  const shown = v * per;
  const e = Math.floor(Math.log10(shown));
  const out: Catch[] = [];
  for (const p of [e - 1, e, e + 1])
    for (let m = 1; m < 10; m += 0.5) {
      const x = m * 10 ** p;
      out.push({ at: x / per, says: `a round ${sayNumber(x, 2)} ${unit}` });
    }
  return out;
}

/**
 * The "in step with" catches for an orbit round one host, from the neighbours' sizes: a year goes as the size to
 * the 3/2 round one host (Kepler's third law with the small bodies' masses left out, fine for a place to catch at;
 * the Orbit box's chip uses the kernel's own years).
 */
export function resonanceCatches(a: number, others: readonly { label: string; a: number }[]): Catch[] {
  const yr = (x: number) => x ** 1.5;
  return resonances({ a, year: yr(a) }, others.map((o) => ({ label: o.label, a: o.a, year: yr(o.a) }))).map((r) => ({ at: r.a, says: `${r.label}` }));
}

/** The catch nearest a place on the line, if it is within `tolPx` there (`px` turns a value into its place). */
export function nearestCatch(atPx: number, catches: readonly Catch[], px: (v: number) => number, tolPx = 6): Catch | null {
  let best: Catch | null = null;
  let gap = tolPx;
  for (const c of catches) {
    const d = Math.abs(px(c.at) - atPx);
    if (d <= gap) ((best = c), (gap = d));
  }
  return best;
}
