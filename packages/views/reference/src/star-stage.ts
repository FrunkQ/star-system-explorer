// Drawing a page of live stars in True with one GPU (SC1): every star on the page is an item in one straight-on
// scene, placed over its tile, its face from the one door the system's True uses (star-now.ts `liveFace` through
// true-body.ts `trueLook`), at its own clock. One canvas and one draw a frame, so a page of 48 stars costs one view.
import { liveFace, trueLook, type Gpu, type GpuItem, type GpuRing, type LiveStar, type Vec3 } from "../../../renderer/src/index.ts";
import { thermalColour } from "../../../ui-contract/src/index.ts";

export interface StageStar {
  key: string;
  live: LiveStar;
  /** Centre over the canvas, CSS pixels from its top left. */
  x: number;
  y: number;
  /** Drawn radius, CSS pixels. */
  r: number;
  /** Star seconds per screen second. */
  clock: number;
}

/** CPU time per drawing for moving faces on, milliseconds (a page is drawn ten times a second: about a sixth of the time). */
export const FACE_BUDGET_MS = 16;
/** The tick each star's face was last drawn at, by key. */
const shownTick = new Map<string, number>();

/** A lit body beside the stars (Earth on its orbit in one star's life), over the canvas as the stars are. */
export interface StageBody {
  key: string;
  x: number;
  y: number;
  r: number;
  colour: string;
}

/** A closed line over the canvas (Earth's orbit), CSS pixels; `over` draws it over the stars (an orbit inside a star). */
export interface StageLine {
  points: { x: number; y: number }[];
  colour: string;
  over?: boolean;
}

/**
 * Draw `stars` at screen time `t_s` (seconds since the page showed); `calm` holds each at its present (reduced motion).
 * `bodies` and `lines` are drawn in the stars' plane, so a body inside a star's disc is hidden by it.
 */
export function drawStage(gpu: Gpu, o: { width: number; height: number; dpr: number; ground: string; t_s: number; calm: boolean; stars: readonly StageStar[]; bodies?: readonly StageBody[]; lines?: readonly StageLine[] }): void {
  const back = 2 * Math.max(10, ...o.stars.map((s) => s.r)) + 10;
  const at = (x: number, y: number): Vec3 => [x - o.width / 2, o.height / 2 - y, -back];
  const items: GpuItem[] = [];
  // A face drawn every tenth of a second of the screen's time, each at its own star's clock. Faces are drawn on the
  // CPU, so a page of stars moves on within a budget each frame, the longest unchanged first; the rest keep their last
  // face until their turn (otherwise a full page starves the page's own work, the kernel's answers among it).
  const tick = Math.floor(o.t_s * 10) / 10;
  const start = performance.now();
  if (shownTick.size > 256) shownTick.clear();
  const order = [...o.stars].sort((a, b) => (shownTick.get(a.key) ?? -1) - (shownTick.get(b.key) ?? -1));
  for (const s of order) {
    const px = 2 * s.r * o.dpr;
    const had = shownTick.get(s.key);
    const at_tick = had === undefined || (had !== tick && performance.now() - start < FACE_BUDGET_MS) ? tick : had;
    shownTick.set(s.key, at_tick);
    const face = liveFace(s.key, s.live, { t: s.live.age_s + at_tick * s.clock, clock: s.clock, span: s.clock / 10, calm: o.calm, px: Math.max(16, px) }).face;
    const c = at(s.x, s.y);
    items.push({ id: s.key, rel: c, radius: s.r, ...trueLook({ key: s.key, star: true, colour: thermalColour(s.live.teff), radius_m: s.live.radius, face, pole: [0, 1, 0] }) });
  }
  for (const b of o.bodies ?? []) items.push({ id: b.key, rel: at(b.x, b.y), radius: b.r, colour: b.colour });
  const rings: GpuRing[] = (o.lines ?? []).map((l) => ({ points: l.points.map((p) => at(p.x, p.y)), colour: l.colour, ...(l.over ? { over: true } : {}) }));
  gpu.draw({
    items,
    rings,
    forward: [0, 0, -1],
    up: [0, 1, 0],
    fovY: 0.5,
    width: Math.round(o.width * o.dpr),
    height: Math.round(o.height * o.dpr),
    ground: o.ground,
    light: { from: [-0.75, 0.45, 0.6] },
    ortho: { width: o.width, height: o.height, near: 1, far: 2 * back + 10 },
    // Through the shared finish as every True view; no glare straight on (sizes side by side, as the size comparison).
    finish: { glare: false },
  });
}
