// One body's True face as a still (V14i, the class gallery's pictures): the face from the one door every True view
// uses (true-body.ts `trueLook`), drawn alone on an off-screen canvas, straight on and lit from the upper left as the
// size comparison lights it, through the shared finish (VP1), its running mean of jittered frames sharpening the
// edges. Presentation only: it never touches the kernel or the state hash.
import { trueLook, type TrueBody } from "./true-body.ts";
import { globeFrame } from "./lineup-3d.ts";
import { halton } from "./photograph.ts";

export interface TrueStill {
  canvas: HTMLCanvasElement;
  backend: "webgpu" | "webgl2";
  frames: number;
  ms: number;
  /** Whether its lit side showed (false: it stayed dark through `warm_ms`, so the picture is no use). */
  lit: boolean;
}

const nextFrame = () => new Promise<void>((r) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => r()) : setTimeout(r, 0)));

/**
 * Draw `b` alone, `size` pixels square: the globe nine tenths across, tipped by its tilt and seen from a little above
 * its equator. Null where this device has no GPU for True.
 */
export async function trueStill(b: Omit<TrueBody, "pole">, o: { size: number; tilt_rad?: number; frames?: number; budget_ms?: number; warm_ms?: number }): Promise<TrueStill | null> {
  const t0 = performance.now();
  const gl = document.createElement("canvas");
  // three.js stays a lazily loaded chunk (shell vite.config.ts): gpu.ts is fetched only when a still is drawn.
  const { createGpu } = await import("./gpu.ts");
  const gpu = await createGpu(gl);
  if (!gpu) return null;
  try {
    const f = globeFrame(o.tilt_rad, { elevation: 0.3, yaw: 0 }, 0.6);
    // Pushed back past its own radius (and its air's), so nothing crosses the near plane; straight on, depth leaves its size alone.
    const radius = o.size * 0.45;
    const back = 2 * radius + 10;
    const item = { id: b.key, rel: [0, 0, -back] as [number, number, number], radius, frame: f, ...trueLook({ ...b, pole: f[2] }) };
    const most = Math.max(1, Math.min(32, Math.round(o.frames ?? 8)));
    const out = document.createElement("canvas");
    out.width = out.height = o.size;
    const g = out.getContext("2d")!;
    const budget = o.budget_ms ?? 3000;
    const draw = (k: number) => {
      gpu.draw({
        items: [item],
        rings: [],
        forward: [0, 0, -1],
        up: [0, 1, 0],
        fovY: 0.5,
        width: o.size,
        height: o.size,
        ground: "black",
        light: { from: [-0.75, 0.45, 0.6] },
        ortho: { width: o.size, height: o.size, near: 1, far: 2 * back },
        finish: { glare: false, accumulate: k },
        ...(k > 0 ? { jitter: [halton(k, 2) - 0.5, halton(k, 3) - 0.5] as [number, number] } : {}),
      });
      // Kept in the same task as the draw, while the canvas still holds it: the finish's mean so far.
      g.clearRect(0, 0, o.size, o.size);
      g.drawImage(gl, 0, 0, o.size, o.size);
    };
    let n = 0;
    // Its shaders compile in the background on the first draws, drawing nothing until they are ready: drawn over until
    // the globe's lit side shows (the size comparison draws continuously, so it never waits), up to `warm_ms`.
    const warm = o.warm_ms ?? 15000;
    const lit = () => {
      const c = Math.round(o.size * 0.4);
      const px = g.getImageData(c, c, 1, 1).data;
      return px[0]! + px[1]! + px[2]! > 6;
    };
    let shows = false;
    for (;;) {
      draw(0);
      shows = lit();
      if (shows || performance.now() - t0 > warm) break;
      await nextFrame();
    }
    if (!shows) return { canvas: out, backend: gpu.backend, frames: 1, ms: performance.now() - t0, lit: false };
    const t1 = performance.now();
    for (let k = 0; k < most; k++) {
      draw(k);
      n = k + 1;
      if (k + 1 < most) await nextFrame();
      if (performance.now() - t1 > budget) break;
    }
    return { canvas: out, backend: gpu.backend, frames: n, ms: performance.now() - t0, lit: true };
  } finally {
    gpu.dispose();
  }
}

export interface TrueTurn {
  /** One turn as `frames` squares side by side, `size` pixels each, frame 0 on the left. */
  sheet: HTMLCanvasElement;
  frames: number;
  size: number;
  backend: "webgpu" | "webgl2";
  ms: number;
  lit: boolean;
}

/**
 * One turn of `b` on its own axis as a strip of stills (V14i part 2, the class gallery's turning globes): drawn once,
 * then any number of tiles turn it by stepping through the strip, with no GPU of their own. Lit and framed as
 * `trueStill`; `spin` is +1 for a world turning prograde, -1 for retrograde. Each frame takes `passes` jittered draws.
 * Null where this device has no GPU for True.
 */
export async function trueTurn(b: Omit<TrueBody, "pole">, o: { size: number; frames: number; tilt_rad?: number; spin?: 1 | -1; passes?: number; warm_ms?: number }): Promise<TrueTurn | null> {
  const t0 = performance.now();
  const gl = document.createElement("canvas");
  const { createGpu } = await import("./gpu.ts");
  const gpu = await createGpu(gl);
  if (!gpu) return null;
  const size = o.size;
  const frames = Math.max(1, Math.min(96, Math.round(o.frames)));
  const passes = Math.max(1, Math.min(8, Math.round(o.passes ?? 2)));
  const sheet = document.createElement("canvas");
  sheet.width = size * frames;
  sheet.height = size;
  const g = sheet.getContext("2d")!;
  try {
    const radius = size * 0.45;
    const back = 2 * radius + 10;
    const itemAt = (turn: number) => {
      const f = globeFrame(o.tilt_rad, { elevation: 0.3, yaw: 0 }, turn);
      return { id: b.key, rel: [0, 0, -back] as [number, number, number], radius, frame: f, ...trueLook({ ...b, pole: f[2] }) };
    };
    const draw = (turn: number, k: number, x: number) => {
      gpu.draw({
        items: [itemAt(turn)],
        rings: [],
        forward: [0, 0, -1],
        up: [0, 1, 0],
        fovY: 0.5,
        width: size,
        height: size,
        ground: "black",
        light: { from: [-0.75, 0.45, 0.6] },
        ortho: { width: size, height: size, near: 1, far: 2 * back },
        finish: { glare: false, accumulate: k },
        ...(k > 0 ? { jitter: [halton(k, 2) - 0.5, halton(k, 3) - 0.5] as [number, number] } : {}),
      });
      g.clearRect(x, 0, size, size);
      // The whole drawing canvas into this frame: it may be larger than `size` (the device's pixel ratio).
      g.drawImage(gl, 0, 0, gl.width, gl.height, x, 0, size, size);
    };
    // Shaders compile on the first draws, which draw nothing: drawn over until the lit side shows (as trueStill).
    const warm = o.warm_ms ?? 15000;
    const lit = () => {
      const c = Math.round(size * 0.4);
      const px = g.getImageData(c, c, 1, 1).data;
      return px[0]! + px[1]! + px[2]! > 6;
    };
    let shows = false;
    for (;;) {
      draw(0, 0, 0);
      shows = lit();
      if (shows || performance.now() - t0 > warm) break;
      await nextFrame();
    }
    if (!shows) return { sheet, frames: 1, size, backend: gpu.backend, ms: performance.now() - t0, lit: false };
    const dir = o.spin ?? 1;
    for (let i = 0; i < frames; i++) {
      const turn = (dir * 2 * Math.PI * i) / frames;
      for (let k = 0; k < passes; k++) draw(turn, k, i * size);
      // Let the page breathe between frames: the gallery stays live while a strip draws.
      if (i % 4 === 3) await nextFrame();
    }
    return { sheet, frames, size, backend: gpu.backend, ms: performance.now() - t0, lit: true };
  } finally {
    gpu.dispose();
  }
}
