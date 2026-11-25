// Placing range rings on the system chart (RK12 mount): the chart's distances are log-scaled about its centre, so a
// ring round anything but the centre is bent. `chartProject` turns metres from the centre (orbital plane x, y) into
// chart px the same way the chart places a world (`polar(cx, cy, rr(au), phase)`), and `holderAt` gives a world's
// own place in metres from its orbit. RangeRings.svelte projects each ring point through these.
import { KM_PER_AU } from "../../../ui-contract/src/index.ts";
import { polar } from "../../../chart-grammar/src/scale.ts";

const M_PER_AU = KM_PER_AU * 1000;

/** Metres from the chart's centre to chart px, by the chart's own radial scale `rr` (AU to px). */
export function chartProject(cx: number, cy: number, rr: (au: number) => number): (x_m: number, y_m: number) => [number, number] {
  return (x_m, y_m) => {
    const au = Math.hypot(x_m, y_m) / M_PER_AU;
    const deg = (Math.atan2(y_m, x_m) * 180) / Math.PI;
    return polar(cx, cy, rr(au), deg);
  };
}

/** A world's place in metres from the chart's centre, from its orbit's size and phase (the centre itself: 0, 0). */
export function holderAt(orbit: { a_au: number; phase_deg: number } | undefined | null): [number, number] {
  if (!orbit) return [0, 0];
  const t = (orbit.phase_deg * Math.PI) / 180;
  return [orbit.a_au * M_PER_AU * Math.cos(t), orbit.a_au * M_PER_AU * Math.sin(t)];
}
