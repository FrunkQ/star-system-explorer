// The size comparison in 3D, True and Wireframe (Alex, 24 Nov 2026): each body in the lineup a globe turning on its
// own axis, leaning by its own tilt, turning its own way, sped up to a pace that can be watched. Pure maths here: the
// globe's frame, its turn at a moment, and the lines of a graticule and a ring's edges, all in the lineup's display
// units (pixels, y up, the viewer looking down -z). gpu.ts draws them.
import type { Vec3 } from "./frames.ts";

/** A 3×3 frame: the body's own x, y and z axes (z its north pole) in the lineup's space. */
export type Frame = [Vec3, Vec3, Vec3];

/** How the lineup is seen: `elevation` above the bodies' orbit plane (0 edge-on, π/2 from above), `yaw` round its normal. */
export interface LineupView {
  elevation: number;
  yaw: number;
}

/** One turn in this many seconds when every body keeps the same pace: slow enough to follow, quick enough to see. */
export const SECONDS_PER_TURN = 40;

type M = [number, number, number, number, number, number, number, number, number];
const mul = (a: M, b: M): M => {
  const o = [0, 0, 0, 0, 0, 0, 0, 0, 0] as M;
  for (let i = 0; i < 3; i++) for (let j = 0; j < 3; j++) o[3 * i + j] = a[3 * i]! * b[j]! + a[3 * i + 1]! * b[3 + j]! + a[3 * i + 2]! * b[6 + j]!;
  return o;
};
const rx = (t: number): M => [1, 0, 0, 0, Math.cos(t), -Math.sin(t), 0, Math.sin(t), Math.cos(t)];
const ry = (t: number): M => [Math.cos(t), 0, Math.sin(t), 0, 1, 0, -Math.sin(t), 0, Math.cos(t)];
const rz = (t: number): M => [Math.cos(t), -Math.sin(t), 0, Math.sin(t), Math.cos(t), 0, 0, 0, 1];
// The body's own axes laid into the edge-on lineup: its pole up the screen (the orbit's normal), its x to the right.
const LAY: M = [1, 0, 0, 0, 0, 1, 0, -1, 0];

/**
 * A body's frame: its pole leans `tilt_rad` from the orbit's normal, sideways across the view (which way it leans is
 * not on record, as for rings: fudge register), and it has turned `turn` radians on its axis. A tilt past a right
 * angle (Venus, Uranus) turns it backwards as seen from the orbit's north, as it truly does.
 */
export function globeFrame(tilt_rad: number | undefined, view: LineupView, turn: number): Frame {
  const m = mul(mul(mul(rx(view.elevation), ry(view.yaw)), mul(rz(-(tilt_rad ?? 0)), LAY)), rz(turn));
  return [
    [m[0], m[3], m[6]],
    [m[1], m[4], m[7]],
    [m[2], m[5], m[8]],
  ];
}

/**
 * How far a body has turned after `t` seconds of watching. "same": every body one turn in SECONDS_PER_TURN, each its
 * own way. "true": the quickest in the lineup (`quickest_h`) one turn in SECONDS_PER_TURN and the rest in proportion,
 * so a slow turner is seen to be slow. A body whose day is not known stands still. A negative day turns backwards.
 */
export function spinAt(day_h: number | undefined, t: number, pace: "same" | "true", quickest_h: number): number {
  if (!day_h || !Number.isFinite(day_h)) return 0;
  const turns = pace === "same" ? t / SECONDS_PER_TURN : (t / SECONDS_PER_TURN) * (quickest_h / Math.abs(day_h));
  return Math.sign(day_h) * 2 * Math.PI * (turns % 1);
}

/** The body's own seconds gone by after `t` seconds of watching, at the pace `spinAt` turns it (0 if its day is not known). */
export function bodySeconds(day_h: number | undefined, t: number, pace: "same" | "true", quickest_h: number): number {
  if (!day_h || !Number.isFinite(day_h)) return 0;
  return (t / SECONDS_PER_TURN) * (pace === "same" ? Math.abs(day_h) : quickest_h) * 3600;
}

/** Hours of the body's day shown in each second watched, at true pace. */
export function hoursPerSecond(quickest_h: number): number {
  return quickest_h / SECONDS_PER_TURN;
}

/** Points round a circle fine enough that no chord strays more than half a pixel from it. */
export function circleSteps(radius_px: number): number {
  return Math.max(24, Math.min(2048, Math.ceil(Math.PI * Math.sqrt(Math.max(0, radius_px)))));
}

export interface GlobeLine {
  points: Vec3[];
  /** The equator and the meridian the turn is read from are marked; the rest are plain. */
  kind: "equator" | "prime" | "line";
}

const place = (f: Frame, c: Vec3, r: number, x: number, y: number, z: number): Vec3 => [c[0] + r * (x * f[0][0] + y * f[1][0] + z * f[2][0]), c[1] + r * (x * f[0][1] + y * f[1][1] + z * f[2][1]), c[2] + r * (x * f[0][2] + y * f[1][2] + z * f[2][2])];

/** A graticule on a globe of `radius` at `centre`: meridians every 30°, parallels every 30°, lifted a hair off the surface. */
export function graticule(f: Frame, centre: Vec3, radius: number): GlobeLine[] {
  const r = radius * 1.004;
  const n = circleSteps(r);
  const out: GlobeLine[] = [];
  for (let k = 0; k < 6; k++) {
    // A meridian is a whole great circle through both poles: longitude λ and λ + 180°.
    const lo = (k * Math.PI) / 6;
    const pts: Vec3[] = [];
    for (let s = 0; s <= n; s++) {
      const a = (2 * Math.PI * s) / n;
      pts.push(place(f, centre, r, Math.cos(a) * Math.cos(lo), Math.cos(a) * Math.sin(lo), Math.sin(a)));
    }
    out.push({ points: pts, kind: k === 0 ? "prime" : "line" });
  }
  for (const la of [-60, -30, 0, 30, 60]) {
    const phi = (la * Math.PI) / 180;
    const m = Math.max(12, Math.round(n * Math.cos(phi)));
    const pts: Vec3[] = [];
    for (let s = 0; s <= m; s++) {
      const a = (2 * Math.PI * s) / m;
      pts.push(place(f, centre, r, Math.cos(phi) * Math.cos(a), Math.cos(phi) * Math.sin(a), Math.sin(phi)));
    }
    out.push({ points: pts, kind: la === 0 ? "equator" : "line" });
  }
  return out;
}

/** A ring's edge in the body's equator, `edge` from its centre in display units. */
export function ringEdge(f: Frame, centre: Vec3, edge: number): Vec3[] {
  const n = circleSteps(edge);
  const pts: Vec3[] = [];
  for (let s = 0; s <= n; s++) {
    const a = (2 * Math.PI * s) / n;
    pts.push(place(f, centre, edge, Math.cos(a), Math.sin(a), 0));
  }
  return pts;
}

/**
 * The runs of a line on a globe that face the viewer (looking down -z), each a separate polyline: a shaded globe's
 * graticule is drawn over it, trimmed to its near side, so no line fights the surface it lies on.
 */
export function nearSide(points: readonly Vec3[], centre: Vec3): Vec3[][] {
  const runs: Vec3[][] = [];
  let run: Vec3[] = [];
  for (const p of points) {
    if (p[2] - centre[2] >= 0) run.push(p);
    else if (run.length) (run.length > 1 && runs.push(run), (run = []));
  }
  if (run.length > 1) runs.push(run);
  return runs;
}
