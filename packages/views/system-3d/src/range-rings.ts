// Range rings in the 3D system view (RK12, Atomic Rockets plan 13): the same rings as the chart's RangeRings.svelte,
// as point loops for the view's ring pass (GPU) and its canvas fallback. The kernel's `effect_at_range` gives the
// distances (ui-contract `rangeRings`); this only places points. The view passes its own `toScene` (metres from the
// system's centre to camera-relative scene units, in whichever scale it shows), so Readable and Real scale both work.
// Mount: spread `rangeRingLines(...)` into the frame's `rings` when `hudShows(hud, "range")`.
import { ringPoints, type RangeRing } from "../../../ui-contract/src/index.ts";
import type { Vec3 } from "../../../renderer/src/index.ts";

export interface RangeRingLine {
  points: Vec3[];
  colour: string;
  kind: RangeRing["kind"];
  words: string;
}

/** One closed loop of points per ring round `centre_m` (metres, system frame), coloured by the view's own tokens. */
export function rangeRingLines(rings: readonly RangeRing[], centre_m: readonly [number, number, number], toScene: (p_m: readonly [number, number, number]) => Vec3, colours: { ring: string; odds: string }, segments = 96): RangeRingLine[] {
  return rings.map((g) => {
    const pts = ringPoints(centre_m, g.radius_m, segments).map(toScene);
    return { points: [...pts, pts[0]!], colour: g.kind === "even odds" ? colours.odds : colours.ring, kind: g.kind, words: g.words };
  });
}
