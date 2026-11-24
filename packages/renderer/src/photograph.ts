// The still tier and "take photograph" (R3, SSE4-surfaces-and-rendering.md
// 6.4). A still is drawn off screen with the recipe's `still` passes (finer
// mesh, more air samples) and progressively refined: several frames, each
// shifted by a fraction of a pixel along a Halton sequence, are averaged,
// which smooths edges and the air's sampling. With no GPU it falls back to
// the canvas tier drawn at twice the size and scaled down. Presentation
// only: a photograph never touches the kernel or the state hash.
import type { Globe, Tier } from "../../ui-contract/src/index.ts";
import { pickRecipe, plan, type PassSpec, type RecipeDoc } from "./recipe.ts";
import { prepare } from "./surface.ts";
import { drawCanvas } from "./canvas.ts";
import { air as airOf, type AirFacts } from "./atmosphere.ts";
import { airRender, type GlobeRunOptions } from "./runtime.ts";
import { exposeAurora } from "./aurora.ts";

export interface PhotoOptions {
  registry: readonly RecipeDoc[];
  library: readonly PassSpec[];
  recipe: string;
  knobs: Record<string, number>;
  globe: Globe;
  air?: AirFacts | null;
  /** The world's aurora, as the globe runtime takes it (runtime.ts `GlobeRunOptions.aurora`). */
  aurora?: GlobeRunOptions["aurora"];
  label: string;
  lon: number;
  lat: number;
  /** Pixels across. */
  size: number;
  /** Most frames to average on the GPU (1 to 64). */
  frames: number;
  /** Time to spend refining, in ms: refinement stops when the next frame would pass it (default 6000). The first frame is always made. */
  budget_ms?: number;
  onProgress?(done: number, of: number): void;
}

export interface Photo {
  canvas: HTMLCanvasElement;
  tier: Tier;
  backend: "webgpu" | "webgl2" | "canvas2d";
  recipe: string;
  frames: number;
  ms: number;
  /** How it was made, in words, for the caption. */
  note: string;
}

/** The k-th point of the Halton sequence in a base, in [0, 1). */
export function halton(k: number, base: number): number {
  let f = 1;
  let r = 0;
  for (let i = k; i > 0; i = Math.floor(i / base)) {
    f /= base;
    r += f * (i % base);
  }
  return r;
}

const nextFrame = () => new Promise<void>((r) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => r()) : setTimeout(r, 0)));

export async function photograph(o: PhotoOptions): Promise<Photo> {
  const t0 = performance.now();
  const out = document.createElement("canvas");
  out.width = out.height = o.size;
  const acc = out.getContext("2d")!;
  const still = pickRecipe(o.registry, o.recipe, "still");
  const a = o.air ? airOf(o.air) : null;
  const airPhys = a && o.air ? { air: a, radius_m: o.air.radius_m } : null;
  if (still) {
    const { createGlobeGpu } = await import("./gpu-globe.ts");
    const glCanvas = document.createElement("canvas");
    const gpu = await createGlobeGpu(glCanvas);
    if (gpu) {
      try {
        const pl = plan(still.doc, o.library, "still", 0, o.knobs);
        const relief = pl.passes.find((x) => x.pass === "relief-globe")?.params ?? {};
        const prepared = prepare(o.globe, String(relief.colour ?? "survey"), Number(relief.exaggeration ?? 12));
        const atm = pl.passes.find((x) => x.pass === "atmosphere")?.params;
        const air = atm && airPhys ? airRender(airPhys, Number(atm.thickness ?? 1)) : null;
        const auPass = pl.passes.find((x) => x.pass === "aurora")?.params;
        const aurora = auPass && o.aurora ? exposeAurora(o.aurora.plan, o.aurora.radius_m, auPass.exposure === "eye" ? "eye" : "long", Number(atm?.sun ?? 6), o.aurora.flux_w_m2 ?? 1361, Number(auPass.lift_target ?? 0.6)) : null;
        const most = Math.max(1, Math.min(64, Math.round(o.frames)));
        const budget = o.budget_ms ?? 6000;
        let n = 0;
        let slowest = 0;
        for (let k = 0; k < most; k++) {
          const f0 = performance.now();
          gpu.draw(prepared, pl, o.lon, o.lat, o.size, air, [halton(k + 1, 2) - 0.5, halton(k + 1, 3) - 0.5], aurora);
          // Running average: frame k weighs 1/(k+1). Read back in the same task as the draw.
          acc.globalAlpha = 1 / (k + 1);
          acc.drawImage(glCanvas, 0, 0, o.size, o.size);
          n = k + 1;
          o.onProgress?.(n, most);
          await nextFrame();
          // The first frame carries shader compiling, so later frames are timed on their own.
          const took = performance.now() - f0;
          if (k > 0) slowest = Math.max(slowest, took);
          if (performance.now() - t0 + (k > 0 ? slowest : took) > budget) break;
        }
        acc.globalAlpha = 1;
        const note = `${still.doc.name}, still tier on ${gpu.backend === "webgpu" ? "WebGPU" : "WebGL2"}: ${n} frame${n === 1 ? "" : "s"} averaged at ${o.size} px${n < most ? ` (this device had time for ${n} of ${most})` : ""}.${still.stood_in ? ` ${still.stood_in}.` : ""}`;
        return { canvas: out, tier: "still", backend: gpu.backend, recipe: still.doc.id, frames: n, ms: performance.now() - t0, note };
      } finally {
        gpu.dispose();
      }
    }
  }
  // No GPU: the canvas tier at twice the size, scaled down.
  const c = pickRecipe(o.registry, o.recipe, "canvas");
  if (!c) throw new Error(`no recipe of ${o.recipe}'s family draws on a canvas`);
  const pl = plan(c.doc, o.library, "canvas", 0, o.knobs);
  const relief = pl.passes.find((x) => x.pass === "relief-globe")?.params ?? {};
  const prepared = prepare(o.globe, String(relief.colour ?? "survey"), Number(relief.exaggeration ?? 12));
  const atm = pl.passes.find((x) => x.pass === "atmosphere")?.params;
  const big = document.createElement("canvas");
  drawCanvas(big, prepared, pl, o.lon, o.lat, o.size * 2, {}, atm && airPhys ? airRender(airPhys, Number(atm.thickness ?? 1)) : null);
  acc.imageSmoothingEnabled = true;
  acc.imageSmoothingQuality = "high";
  acc.drawImage(big, 0, 0, o.size, o.size);
  o.onProgress?.(1, 1);
  return { canvas: out, tier: "canvas", backend: "canvas2d", recipe: c.doc.id, frames: 1, ms: performance.now() - t0, note: `${c.doc.name} on a 2D canvas (no GPU here), drawn at twice the size and scaled down.` };
}
