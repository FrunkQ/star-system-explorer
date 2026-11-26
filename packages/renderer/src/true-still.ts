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
