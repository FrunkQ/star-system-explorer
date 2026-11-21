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
export { air, airFactsOf, airFromOptics, airRequest, KERNEL_GASES } from "./atmosphere.ts";
export type { Air, AirFacts } from "./atmosphere.ts";
export type { GlobeRun, GlobeRunOptions, GlobeStatus } from "./runtime.ts";

/** Start the GPU path (WebGPU, else WebGL2), or null when this device has neither. */
export async function startGpu(canvas: HTMLCanvasElement) {
  const { createGpu } = await import("./gpu.ts");
  return createGpu(canvas);
}
// R3: the still tier, "take photograph", and photographs kept on this device.
export { photograph, halton } from "./photograph.ts";
export type { Photo, PhotoOptions } from "./photograph.ts";
export { keepPhoto, listPhotos, removePhoto } from "./media.ts";
export type { MediaItem } from "./media.ts";
// R4: recipe editing through checked edits, costed against the device budget.
export { applyEdits, editableSettings, tierCosts } from "./recipe-edit.ts";
export type { EditContext, EditResult, RecipeEdit, TierCost } from "./recipe-edit.ts";
// R5a: stars drawn from their physics (photosphere, spots, corona in X-rays).
// O10 convection: granules drawn where the disc resolves them.
export { coronaScaleHeight, CORONA_K, drawStar, GRANULE_DT, granulesFor, granulesShow, limbDarkening, narrateGranules, narrateStar, SPOT_DELTA_K, spotsFor, STAR_REFERENCES } from "./star.ts";
export type { Convection, Granules, Spot, StarDraw } from "./star.ts";
