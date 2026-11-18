// Chart geometry. A system chart is drawn on a log-radial scale so inner
// and outer orbits are both readable; the scale is stated on the chart.

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
