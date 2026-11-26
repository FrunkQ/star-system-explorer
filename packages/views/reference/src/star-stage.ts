// Drawing a page of live stars in True with one GPU (SC1): every star on the page is an item in one straight-on
// scene, placed over its tile, its face from the one door the system's True uses (star-now.ts `liveFace` through
// true-body.ts `trueLook`), at its own clock. One canvas and one draw a frame, so a page of 48 stars costs one view.
import { liveFace, trueLook, type Gpu, type GpuItem, type LiveStar, type Vec3 } from "../../../renderer/src/index.ts";
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

/** Draw `stars` at screen time `t_s` (seconds since the page showed); `calm` holds each at its present (reduced motion). */
export function drawStage(gpu: Gpu, o: { width: number; height: number; dpr: number; ground: string; t_s: number; calm: boolean; stars: readonly StageStar[] }): void {
  const back = 2 * Math.max(10, ...o.stars.map((s) => s.r)) + 10;
  const items: GpuItem[] = [];
  // A face drawn every tenth of a second of the screen's time, each at its own star's clock.
  const tick = Math.floor(o.t_s * 10) / 10;
  for (const s of o.stars) {
    const px = 2 * s.r * o.dpr;
    const face = liveFace(s.key, s.live, { t: s.live.age_s + tick * s.clock, clock: s.clock, span: s.clock / 10, calm: o.calm, px: Math.max(16, px) }).face;
    const c: Vec3 = [s.x - o.width / 2, o.height / 2 - s.y, -back];
    items.push({ id: s.key, rel: c, radius: s.r, ...trueLook({ key: s.key, star: true, colour: thermalColour(s.live.teff), radius_m: s.live.radius, face, pole: [0, 1, 0] }) });
  }
  gpu.draw({
    items,
    rings: [],
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
