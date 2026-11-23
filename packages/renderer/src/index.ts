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
// Time-lapse layers on the Body globe (surface_frames): read, bracket, colour, pick, narrate.
export { bracket, cellOf, eventsBetween, FIELDS, LAYERS, layerColour, layersFor, narrateFrame, pickAt, readFrames, recolour, seaLevelAt, valueAt } from "./timelapse.ts";
export type { FieldName, Frame, FrameEvent, LayerSpec, RawFrames, TimeLapse } from "./timelapse.ts";
export { addLayers, CLOUD_G, cloudReflectance, HAZE_G, liveSystems, narrateWeather, readWeather, slabReflectance, stormAt, WEATHER_REFS, weatherAt, withWeather } from "./weather.ts";
export type { WeatherCell, WeatherLayer, WeatherMap, WeatherSystem, WeatherTile } from "./weather.ts";
export { ang2pixRing, drawSky, exposureFor, glowLevel, glowPixel, glowSample, narrateSky, pickStar, pix2angRing, pointSpread, projectSky, readSky, skyCamera, skyView, systemView, toGalactic, toSystem, SKY_POINTS_MAX, SKY_WHITE_MAG_ARCSEC2, starRgb, tempFromBV, unprojectSky } from "./sky.ts";
export type { Sky, SkyCamera, SkyGlowBand, SkyStar, SkyView, SystemFrame } from "./sky.ts";
export { bubbleOutline, chartCut, compressed, cutPaths, degreeOf, dipole, dipoleAxis, dipoleB0, drawField3D, FIELD_LINES, FIELD_REFS, fieldAt, fieldFromRead, inBubble, insideBubble, lineBuffers, narrateField, project3D, schmidt, seeds, traceField, traceLine } from "./field.ts";
export type { Bubble, ChartCut, FieldCamera, FieldLine, FieldModel, FieldReadJson, FieldTrace, Pen, Run, TraceOptions } from "./field.ts";
// V3 gap item 14, part 1: ship hulls and drive plumes built from the ship record's own physics.
export { bandOf, driveFacts, HULL_SHAPE, hullFromShip, hullProfile, narrateShip, plumeShape, SHIP_REFS, shipPaths, readPropellants, tanksOf } from "./ship.ts";
export type { DriveFacts, Hull, HullPart, HullPartKind, PlumeShape, PropellantPart, Propellants, ShipRecord, TankNeed } from "./ship.ts";
// V3 gap item 14, stage 2: ships designed from their parts, in styles from the base-ships pack.
export { designShip, narrateDesign, needsOf, remixes, shipViews } from "./ship-design.ts";
export type { Callout, DesignOverrides, DesignRecipe, DesignRules, DesignShip, Needs, Piece, ShipLayout, ShipStyle, ShipView, SystemKind, ViewPath } from "./ship-design.ts";
// V3 gap item 14, stage 3: the 3D model from the same layout, with panel textures from the style's skin.
export { panelTexture, PANEL_UNIT, shipMesh, SURFACE_OF } from "./ship-mesh.ts";
export type { ShipMesh, SkinSpec, Surface, SurfaceBuffers, SurfaceLook } from "./ship-mesh.ts";
// GH9: every collision leaves its mark: warps, ripples, rings, lopsided discs, drawn from the kernel's dated marks.
export { brightnessAt, edgeOnPaths, faceOnPaths, heightAt, markStrengthAt, SCAR_SHAPES, scarsAt, scarWords } from "./galaxy-marks.ts";
export type { DiscScars, Frame as ScarFrame, Ring as ScarRing, Ripple } from "./galaxy-marks.ts";
// V3 gap item 14, stage 4: abilities (aerobraking, landing, skimming) size a shield, lift, legs, chute and scoop.
export { fitAbilities, fittedChecks, outlineOf } from "./ship-abilities.ts";
export type { Abilities, AbilityRules, AskShipAtWorld, FitResult, Fitted, ShieldKind } from "./ship-abilities.ts";
export type { ShipView3D } from "./gpu-ship.ts";

/** Draw a designed ship in 3D (WebGPU, else WebGL2), or null when this device has neither. three.js loads only here. */
export async function startShip(...a: Parameters<typeof import("./gpu-ship.ts").mountShip>) {
  const { mountShip } = await import("./gpu-ship.ts");
  return mountShip(...a);
}
