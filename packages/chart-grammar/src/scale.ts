// Chart geometry. A system chart is drawn on a log-radial scale so inner
// and outer orbits are both readable; the scale is stated on the chart.
import { roundTo } from "../../ui-contract/src/index.ts";

export interface RadialScale {
  inner_au: number;
  outer_au: number;
  r_min: number;
  r_max: number;
}

/** Chart radius (px) of a distance in AU, logarithmic between inner and outer. */
export function radius(a_au: number, s: RadialScale): number {
  const lo = Math.log(s.inner_au);
  const hi = Math.log(s.outer_au);
  const t = (Math.log(Math.max(a_au, s.inner_au * 1e-3)) - lo) / (hi - lo);
  return s.r_min + t * (s.r_max - s.r_min);
}

/** Inverse of `radius`: the distance in AU drawn at a chart radius. */
export function distanceAt(r: number, s: RadialScale): number {
  const lo = Math.log(s.inner_au);
  const hi = Math.log(s.outer_au);
  return Math.exp(lo + ((r - s.r_min) / (s.r_max - s.r_min)) * (hi - lo));
}

/** Choose inner and outer bounds that fit a set of orbits with a margin. */
export function fitScale(distances_au: number[], r_min: number, r_max: number): RadialScale {
  const d = distances_au.filter((x) => x > 0);
  const inner = d.length ? Math.min(...d) / 1.6 : 0.1;
  const outer = d.length ? Math.max(...d) * 1.25 : 10;
  return { inner_au: inner, outer_au: outer, r_min, r_max };
}

export function polar(cx: number, cy: number, r: number, deg: number): [number, number] {
  const t = (deg * Math.PI) / 180;
  return [cx + r * Math.cos(t), cy - r * Math.sin(t)];
}

/**
 * The chart's scale drawn as rings (the grid of a log-radial chart): a solid
 * ring at every power of ten of the unit, and fainter rings at 2 and 5 times it
 * when the decades sit far enough apart to read them. `unit_au` is one display
 * unit in AU (1 for AU; 1 / km-per-AU for kilometres). Rings run from the
 * scale's inner edge out to `r_out` pixels.
 */
export function scaleRings(s: RadialScale, r_out: number, unit_au = 1, minGapPx = 56): { au: number; units: number; major: boolean }[] {
  const lo = Math.max(s.inner_au, distanceAt(s.r_min, s));
  const hi = distanceAt(r_out, s);
  if (!(hi > lo)) return [];
  const decadePx = (s.r_max - s.r_min) / (Math.log10(s.outer_au) - Math.log10(s.inner_au));
  const minors = decadePx >= minGapPx * 2;
  const out: { au: number; units: number; major: boolean }[] = [];
  for (let e = Math.floor(Math.log10(lo / unit_au)); e <= Math.ceil(Math.log10(hi / unit_au)); e++) {
    for (const m of minors ? [1, 2, 5] : [1]) {
      const units = roundTo(m * Math.pow(10, e), 12);
      const au = units * unit_au;
      if (au >= lo * 0.999 && au <= hi) out.push({ au, units, major: m === 1 });
    }
  }
  return out;
}
