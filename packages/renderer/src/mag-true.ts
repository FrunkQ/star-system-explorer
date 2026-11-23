// Mag through the system view (V10e): each world's magnetosphere drawn round it in 3D, in the Mag band. The same
// kernel facts as the Body screen's Mag layer (read_field), traced by the same tracer (field.ts), only in 3D and
// round every world at once. Nothing here computes physics: it traces the field the kernel gives and places it.
//
// The body frame is field.ts's: z along the spin axis (the ecliptic taken as the spin equator), x toward the star.
import { bubbleOutline, fieldFromRead, traceField, type FieldReadJson, type V3 } from "./field.ts";

/** Lines a world gets in the system view: four meridians rather than six, since every world draws at once (a few ms a world, traced once). */
export const MAG_SYSTEM_LINES = { perHemisphere: 12, meridians: 4 } as const;

/** One stretch of a field line with one ink: the field pointing away from the world (out) or toward it. */
export interface MagRun {
  points: V3[];
  out: boolean;
  open: boolean;
}

/** A world's magnetosphere in its own frame, in its radii. */
export interface MagShape {
  runs: MagRun[];
  /** The bubble's edge (magnetopause) as three outlines: through the poles, along the equator, and round its waist. */
  bubble: V3[][];
  /** Where the bubble's nose sits toward the star, radii (null when the field does not hold the wind off). */
  standoff: number | null;
  /** How far out the lines are drawn, radii (the Body screen's cut draws to the same reach). */
  extent: number;
  notes: string[];
}

/** The reach drawn round a world: past the bubble's nose and some way down its tail, as on the Body screen's cut. */
export const magExtent = (standoff: number | null): number => (standoff ? Math.min(90, Math.max(4, standoff * 1.8)) : 6);

/** Cut a polyline at a radius: the stretches inside it. */
function clip(points: V3[], r: number): V3[][] {
  const out: V3[][] = [];
  let cur: V3[] = [];
  for (const p of points) {
    if (Math.hypot(p[0], p[1], p[2]) <= r) cur.push(p);
    else if (cur.length) (cur.length > 1 && out.push(cur), (cur = []));
  }
  if (cur.length > 1) out.push(cur);
  return out;
}

/** Trace a world's field from its read_field facts: its lines, split where they turn round, and its bubble. Null with no field. */
export function magShape(r: FieldReadJson, lines: { perHemisphere: number; meridians: number } = MAG_SYSTEM_LINES): MagShape | null {
  const f = fieldFromRead(r);
  if (!f) return null;
  const standoff = f.bubble?.standoff_radii ?? null;
  const extent = magExtent(standoff);
  const trace = traceField(f.model, f.bubble, lines);
  const runs: MagRun[] = [];
  for (const l of trace.lines) {
    const open = l.kind !== "closed";
    // A run ends where the field turns from pointing away to pointing in (the line's farthest point).
    let start = 0;
    for (let i = 1; i <= l.points.length; i++) {
      if (i < l.points.length && l.outward[i] === l.outward[start]) continue;
      const seg = l.points.slice(start, Math.min(l.points.length, i + 1));
      for (const s of clip(seg, extent)) runs.push({ points: s, out: l.outward[start] === 1, open });
      start = i;
    }
  }
  const bubble: V3[][] = [];
  if (f.bubble) {
    const o = bubbleOutline(f.bubble, 64, 150);
    bubble.push(...clip(o.map((p) => [p.x, 0, p.y] as V3), extent));
    bubble.push(...clip(o.map((p) => [p.x, p.y, 0] as V3), extent));
    // The waist: the bubble's cross-section over the world's terminator (θ = 90°, r = r0 2^α).
    const w = f.bubble.standoff_radii * Math.pow(2, f.bubble.flaring);
    if (w <= extent) bubble.push(Array.from({ length: 65 }, (_, i) => [0, w * Math.cos((i / 64) * 2 * Math.PI), w * Math.sin((i / 64) * 2 * Math.PI)] as V3));
  }
  return { runs, bubble, standoff, extent, notes: f.notes };
}

/**
 * A world's shape placed in the system view: centre and size of the drawn world, the direction to the star in the
 * view's plane, and how far it may reach (display units). Points beyond the reach are cut away.
 */
export function placeMag(shape: MagShape, centre: V3, radius: number, sunward: [number, number], reach = Infinity): { runs: { points: V3[]; out: boolean; open: boolean }[]; bubble: V3[][] } {
  const n = Math.hypot(sunward[0], sunward[1]) || 1;
  const ux = sunward[0] / n;
  const uy = sunward[1] / n;
  const lim = Math.min(shape.extent, reach / Math.max(1e-12, radius));
  // Body x toward the star, z up out of the plane, y = z × x.
  const to = (p: V3): V3 => [centre[0] + radius * (p[0] * ux - p[1] * uy), centre[1] + radius * (p[0] * uy + p[1] * ux), centre[2] + radius * p[2]];
  const runs = shape.runs.flatMap((r) => clip(r.points, lim).map((s) => ({ points: s.map(to), out: r.out, open: r.open })));
  const bubble = shape.bubble.flatMap((b) => clip(b, lim).map((s) => s.map(to)));
  return { runs, bubble };
}
