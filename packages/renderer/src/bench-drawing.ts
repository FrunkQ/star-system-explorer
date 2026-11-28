// The drawing rows of the bench (VP4, SSE4-visual-polish-design.md 8.2): the speed probe's three scenes timed on
// this device, the standard True recipe's (system-true) predicted frame at each GPU tier at 1080p, and that same
// standard frame drawn for real at each tier (a star, eight surfaced worlds, a belt's haze, the shared finish), so the
// pass costs can be set from measurement rather than estimate. Built by scripts/build-harness.sh into
// tests/harness/pkg/drawing/ (three.js bundled), loaded by the bench on /harness/ only when the bench runs, and written
// into the report as `drawing`; scripts/min-spec.mjs reads it.
import systemTrue from "./recipes/system-true.json";
import { LIBRARY } from "./passes.ts";
import { BENCH_PIXELS, drawingBench, type DrawingBench } from "./probe.ts";
import { measureProbe, readCapabilities } from "./probe-gpu.ts";
import { plan, type RecipeDoc } from "./recipe.ts";
import { createGpu, type GpuDots, type GpuItem } from "./gpu.ts";
import type { Tier } from "../../ui-contract/src/index.ts";

/** One tier's standard frame as drawn on this device. */
export interface StandardFrame {
  tier: Tier;
  /** Median ms of the frame (GPU time by timestamp, else closed by a one-pixel read-back). */
  ms: number;
  method: "timestamp" | "readback";
  frames: number;
}

const W = 1920;
const H = 1080;

/** A seeded surface picture, the same on every device (no kernel needed: this times drawing, not worlds). */
function surface(seed: number) {
  const w = 512;
  const h = 256;
  const data = new Uint8Array(w * h * 4);
  let x = (0x9e3779b9 ^ seed) >>> 0;
  for (let i = 0; i < data.length; i += 4) {
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    const v = 90 + ((x >>> 0) % 120);
    data[i] = v;
    data[i + 1] = v * 0.9;
    data[i + 2] = v * 0.75;
    data[i + 3] = 255;
  }
  return { width: w, height: h, data, mean: [150, 135, 110] as [number, number, number] };
}

/** The standard frame: a star at the centre, eight worlds out along a gentle spiral, a belt's haze, seen from above. */
function standardScene(): { items: GpuItem[]; dots: GpuDots[] } {
  const items: GpuItem[] = [{ id: "star", rel: [0, 0, -30], radius: 1.2, colour: "#fff4e0", shines: true }];
  for (let i = 0; i < 8; i++) {
    const a = i * 0.9;
    const r = 3 + i * 1.6;
    items.push({ id: `w${i}`, rel: [r * Math.cos(a), r * Math.sin(a) * 0.6, -30 + r * Math.sin(a) * 0.4], radius: 0.35 + (i % 3) * 0.2, colour: "#a09080", surface: { key: `s${i}`, texture: surface(i) } });
  }
  const pts: [number, number, number][] = [];
  for (let i = 0; i < 2400; i++) {
    const a = (i * 2.399963) % (2 * Math.PI);
    const r = 9 + ((i * 7919) % 1000) / 500;
    pts.push([r * Math.cos(a), r * Math.sin(a) * 0.6, -30 + r * Math.sin(a) * 0.4]);
  }
  return { items, dots: [{ points: pts, colour: "#8a8070", size: 1.5 }] };
}

const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? s[Math.floor((s.length - 1) / 2)]! : NaN;
};

/** Draw the standard frame at each GPU tier the recipe has, at 1080p, and time it. Empty when there is no GPU path. */
export async function standardFrames(recipe: RecipeDoc = systemTrue as RecipeDoc, runs = 15): Promise<StandardFrame[]> {
  if (typeof document === "undefined") return [];
  const canvas = document.createElement("canvas");
  canvas.width = W;
  canvas.height = H;
  const gpu = await createGpu(canvas, { timing: true });
  if (!gpu) return [];
  const gl = gpu.backend === "webgl2" ? (canvas.getContext("webgl2") as WebGL2RenderingContext | null) : null;
  const px = new Uint8Array(4);
  const scene = standardScene();
  const out: StandardFrame[] = [];
  try {
    for (const tier of recipe.tiers.filter((t): t is Tier => t === "gpu-ultra" || t === "gpu-high" || t === "gpu-basic")) {
      const p = plan(recipe, LIBRARY, tier, 0);
      const fin = p.passes.find((x) => x.pass === "true-finish")?.params ?? {};
      const bodies = p.passes.find((x) => x.pass === "true-bodies")?.params ?? {};
      const draw = () =>
        gpu.draw({
          items: scene.items,
          rings: [],
          dots: scene.dots,
          forward: [0, 0, -1],
          up: [0, 1, 0],
          fovY: 0.8,
          width: W,
          height: H,
          ground: "#000000",
          light: { at: [0, 0, -30] },
          sphere_detail: Number(bodies.sphere_detail ?? 64),
          finish: { glare: true, rungs: Number(fin.glare_rungs ?? 6), glare_scale: Number(fin.glare_scale ?? 0.5), render_scale: p.render_scale },
        });
      // Warm up (shaders compile, textures upload), then time.
      for (let i = 0; i < 3; i++) draw();
      await new Promise((r) => setTimeout(r, 200));
      const wall: number[] = [];
      const gpuMs: number[] = [];
      for (let i = 0; i < runs; i++) {
        const t0 = performance.now();
        draw();
        if (gl) gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
        wall.push(performance.now() - t0);
        await new Promise((r) => setTimeout(r, 30));
        if (gpu.gpuMs !== null) gpuMs.push(gpu.gpuMs);
      }
      const timed = gpuMs.length >= Math.ceil(runs / 2);
      out.push({ tier, ms: Math.round(median(timed ? gpuMs : wall) * 100) / 100, method: timed ? "timestamp" : "readback", frames: runs });
    }
  } finally {
    gpu.dispose();
  }
  return out;
}

/** Measure this device (always afresh: the bench never reads the app's cache), place it at 1080p, and draw the standard frame. */
/** The settle kit's three pieces (FL2) on a 512-wide face, each with its upload and read-back: empty without a GPU. */
export async function kitFrames(): Promise<{ piece: string; width: number; height: number; ms: number }[]> {
  if (typeof document === "undefined") return [];
  const [{ WebGPURenderer }, { benchKit }] = await Promise.all([import("three/webgpu"), import("./settle-kit.ts")]);
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const r = new WebGPURenderer({ canvas, antialias: false });
  try {
    await r.init();
    return await benchKit(r);
  } finally {
    r.dispose();
  }
}

export async function benchDrawing(): Promise<DrawingBench & { standard: StandardFrame[]; standard_pixels: number; kit: { piece: string; width: number; height: number; ms: number }[] }> {
  const caps = await readCapabilities();
  const probe = await measureProbe(caps, { runs: 15, cap_ms: 1500 });
  const placed = drawingBench(caps, probe, systemTrue as RecipeDoc, LIBRARY);
  const standard = await standardFrames().catch(() => []);
  const kit = await kitFrames().catch(() => []);
  return { ...placed, standard, standard_pixels: BENCH_PIXELS, kit };
}
