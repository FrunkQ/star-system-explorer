// Renderer packages (H8.1, SSE4-surfaces-and-rendering.md). Frames and the
// camera-relative rule are plain f64 code here; the GPU wrapper loads three.js
// only when a view asks for it, so pages without 3D never fetch it.
export { tree, pathTo, relative, basis, project, projectExact, projectFlattened, rebase, frameFor } from "./frames.ts";
export type { Vec3, FrameNode, Tree, Camera, Pixel } from "./frames.ts";
export type { Gpu, GpuItem, GpuRing } from "./gpu.ts";
// R1: render recipes, the pass library, the governor and the globe runtime.
export { checkRecipe, plan, estimateMs, pickRecipe, bestTier, maxLevel, stepsFrom, TIER_ORDER, DEFAULT_BUDGET_MS } from "./recipe.ts";
export type { RecipeDoc, RecipePass, PassSpec, PassCost, Knob, DegradeStep, Plan, RecipeCheck } from "./recipe.ts";
export { Governor, badge } from "./governor.ts";
export type { Badge, GovernorOptions, GovernorMove } from "./governor.ts";
export { LIBRARY } from "./passes.ts";
export { mountGlobe, airRender } from "./runtime.ts";
export { air, airFactsOf } from "./atmosphere.ts";
export type { Air, AirFacts } from "./atmosphere.ts";
export type { GlobeRun, GlobeRunOptions, GlobeStatus } from "./runtime.ts";

/** Start the GPU path (WebGPU, else WebGL2), or null when this device has neither. */
export async function startGpu(canvas: HTMLCanvasElement) {
  const { createGpu } = await import("./gpu.ts");
  return createGpu(canvas);
}
