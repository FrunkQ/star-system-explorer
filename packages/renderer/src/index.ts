// Renderer packages (H8.1, SSE4-surfaces-and-rendering.md). Frames and the
// camera-relative rule are plain f64 code here; the GPU wrapper loads three.js
// only when a view asks for it, so pages without 3D never fetch it.
export { tree, pathTo, relative, basis, project, projectExact, projectFlattened, rebase, frameFor } from "./frames.ts";
export type { Vec3, FrameNode, Tree, Camera, Pixel } from "./frames.ts";
export type { Gpu, GpuDots, GpuGlow, GpuItem, GpuLight, GpuRing, GpuSheet } from "./gpu.ts";
export { armField, cloudNoise, GALAXY_BANDS, GALAXY_LIGHT, GALAXY_LIGHT_REFS, galaxyFace, glowPicture, sheetWeights, surfaceAt } from "./galaxy-true.ts";
export type { GalaxyBand, GalaxyPicture } from "./galaxy-true.ts";
export { cubeEdges, galaxyLight, SPACE_CSS, webGlow, webLights, webParcels, WEB_DARK_GLOW, WEB_KINDS, WEB_LIGHT_K, WEB_LIGHT_SHOW } from "./web-cube.ts";
export type { WebCubeData, WebGalaxyData, WebKind, WebLightGroup } from "./web-cube.ts";
// V10a: True in the system view, each world wrapped in its own surface.
export { equirect, starCss, thermalGrey, THERMAL_DECADES, THERMAL_IR_M, trueSurface } from "./true-look.ts";
export type { TrueTexture } from "./true-look.ts";
// R1 to R3, R6: air in True (clouds, hazes, giants from their cloud decks), true-air.ts.
export { flowFace, gasAbove, gasTint, readTrueFace, sheetColours, trueGiant, withAir, AIR_REFS, type AirFlow, type AirLook, type FaceReader, type FlowStorm } from "./true-air.ts";
// V10f: rings and belts in True, from the universe's ring and belt data.
export { BELT_DUST, BELT_SECONDS_PER_SECOND, beltAt, beltHaze, beltParticles, poleOf, RING_FALLBACK, ringProfile } from "./rings.ts";
export type { BeltParticle } from "./rings.ts";
export type { RingBand, RingProfile } from "./rings.ts";
// R1: render recipes, the pass library, the governor and the globe runtime.
export { checkRecipe, plan, estimateMs, pickRecipe, bestTier, maxLevel, stepsFrom, TIER_ORDER, DEFAULT_BUDGET_MS } from "./recipe.ts";
export type { RecipeDoc, RecipePass, PassSpec, PassCost, Knob, DegradeStep, Plan, RecipeCheck } from "./recipe.ts";
export { Governor, badge } from "./governor.ts";
export type { Badge, GovernorOptions, GovernorMove } from "./governor.ts";
export { LIBRARY } from "./passes.ts";
export { mountGlobe, airRender, deviceReadout, measureAgain, passShares, quality, setQuality } from "./runtime.ts";
// VP4: the speed probe and placement by measured speed.
export { BENCH_PIXELS, drawingBench, factorsFrom, gpuTiersAllowed, HeatWatch, loadProbe, loadQuality, place, PLACE_MARGIN, PROBE_SCENES, probeKey, QUALITY_WORDS, tierWords, ULTRA_HEADROOM } from "./probe.ts";
export type { Capabilities, DeviceFactors, DrawingBench, Placement, ProbeResult, ProbeScene, QualityChoice, TierPrediction } from "./probe.ts";
export { air, airFactsOf, gasMixOf, surfacePressureOf, airFromOptics, airRequest, KERNEL_GASES } from "./atmosphere.ts";
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
export type { Convection, FlarePatch, Granules, Spot, StarDraw } from "./star.ts";
export { cycleOf, drawLiveStar, flarePlace, flareProfile, flaresAt, readLiveStar, spotsOf, STAR_SECONDS_PER_SECOND, turnAt } from "./live-star.ts";
export type { Flare, GivenStar, LiveStar, StarReader } from "./live-star.ts";
export { drawGlow, glarePsf, glareProfile, glowReach, GLARE_EYE, GLOW_REACH_DEG, GLOW_REFERENCES, narrateGlow } from "./star-glow.ts";
export { CELL_DRIFT, CHURN_REFERENCES, churningGranules, granuleBlur, limbLumps, LUMP_SCALE_HEIGHTS, narrateLumps } from "./star-churn.ts";
export { addEjections, EJECTION_SCALE, ejectionShare, ejectionShown, ejectionsAt, EVENT_REFERENCES, FLASHES_PER_SECOND_MAX, flareBoost, flareWhiteLight, narrateEvents, patchContrast, patchRadiusDeg, shownFlares } from "./star-events.ts";
export type { EjectionNow, ShownFlare } from "./star-events.ts";
export { drawStarNow } from "./star-now.ts";
export type { StarNow, StarNowAsk } from "./star-now.ts";
// Time-lapse layers on the Body globe (surface_frames): read, bracket, colour, pick, narrate.
export { bracket, cellOf, eventsBetween, FIELDS, LAYERS, layerColour, layersFor, narrateFrame, pickAt, readFrames, recolour, seaLevelAt, valueAt } from "./timelapse.ts";
export type { FieldName, Frame, FrameEvent, LayerSpec, RawFrames, TimeLapse } from "./timelapse.ts";
export { addLayers, CLOUD_G, cloudReflectance, HAZE_G, liveSystems, narrateWeather, readWeather, slabReflectance, stormAt, WEATHER_REFS, weatherAt, withWeather } from "./weather.ts";
export type { WeatherCell, WeatherLayer, WeatherMap, WeatherSystem, WeatherTile } from "./weather.ts";
export { ang2pixRing, drawSky, exposureFor, glowLevel, glowPixel, glowSample, narrateSky, pickStar, pix2angRing, pointSpread, projectSky, readSky, skyCamera, skyView, systemView, toGalactic, toSystem, SKY_POINTS_MAX, SKY_WHITE_MAG_ARCSEC2, starRgb, tempFromBV, unprojectSky } from "./sky.ts";
export type { Sky, SkyCamera, SkyGlowBand, SkyStar, SkyView, SystemFrame } from "./sky.ts";
export { bubbleOutline, chartCut, compressed, cutPaths, degreeOf, dipole, dipoleAxis, dipoleB0, drawField3D, FIELD_LINES, FIELD_REFS, fieldAt, fieldFromRead, inBubble, insideBubble, lineBuffers, narrateField, project3D, schmidt, seeds, traceField, traceLine } from "./field.ts";
export { MAG_SYSTEM_LINES, magExtent, magShape, placeMag } from "./mag-true.ts";
export type { MagRun, MagShape } from "./mag-true.ts";
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
export { fitAbilities, fittedChecks, outlineOf, passGlow } from "./ship-abilities.ts";
export type { Abilities, AbilityRules, AskShipAtWorld, FitResult, Fitted, ShieldKind } from "./ship-abilities.ts";
export type { ShipView3D } from "./gpu-ship.ts";

/** Draw a designed ship in 3D (WebGPU, else WebGL2), or null when this device has neither. three.js loads only here. */
export async function startShip(...a: Parameters<typeof import("./gpu-ship.ts").mountShip>) {
  const { mountShip } = await import("./gpu-ship.ts");
  return mountShip(...a);
}
// Stations and land constructs built from their description (Alex, 24 Nov): a self-contained builder, swappable.
export { designStation, hoopNeed, narrateStation, pickShape, stationPackData, stationViews } from "./station-design.ts";
export type { Axis, DockClass, Megastructure, Part, StationForm, StationLayout, StationLimits, StationOverrides, StationPlace, StationRecipe, StationRecord, StationRules, StationShape, StationSurface, StationView, StrengthMark } from "./station-design.ts";
export { stationMesh } from "./station-mesh.ts";
export type { StationMesh } from "./station-mesh.ts";
export type { StationSkin, StationView3D } from "./gpu-station.ts";
/** The station's 3D model, loaded only when asked (three.js is heavy). */
export async function startStation(...a: Parameters<typeof import("./gpu-station.ts").mountStation>) {
  const { mountStation } = await import("./gpu-station.ts");
  return mountStation(...a);
}

export { curveDrop, decoration, descentTrack, floorFromRefusal, groundColours, groundMesh, heightAt as groundHeightAt, horizonDistance, levelFor, localToDir, patchKeys, patchPoint, playbackKeys, readPatch, siteFrame, tileSpot } from "./surface-patch.ts";
export type { GroundPoint, Patch, TileKey as SurfaceTileKey } from "./surface-patch.ts";
export { fresnel, MAX_TRAINS, readSeaOptics, SEA_STEPS, seaBody, seaWords } from "./sea.ts";
export type { SeaOptics, WaveTrain } from "./sea.ts";
export { airMass, rayleighPhase, skyIrradiance, skyRadiance, sunAt, sunDirection, sunTransmittance } from "./surface-sky.ts";
export type { SurfaceFrame, SurfaceGpu, SurfaceMesh } from "./gpu-surface.ts";

/** Draw the ground round a site on the GPU (WebGPU, else WebGL2), or null when this device has neither. three.js loads only here. */
export async function startSurface(canvas: HTMLCanvasElement) {
  const { createSurfaceGpu } = await import("./gpu-surface.ts");
  return createSurfaceGpu(canvas);
}
// The size comparison's turning globes (3D, True, Wireframe).
export { circleSteps, globeFrame, graticule, bodySeconds, hoursPerSecond, nearSide, ringEdge, SECONDS_PER_TURN, spinAt } from "./lineup-3d.ts";
export type { Frame as GlobeFrame, GlobeLine, LineupView } from "./lineup-3d.ts";
// A body's face in True, one door for every view (true-body.ts).
export { meanColour, ringsAssumed, trueLook } from "./true-body.ts";
export type { TrueBody, TrueLook } from "./true-body.ts";
// The stars over a landing site and the eye's adaptation (night-sky.ts).
export { ADAPT_RANGE, adaptedExposure, starsOverSite } from "./night-sky.ts";
export type { SiteSkyFrame, SiteSkyStar, SiteStar } from "./night-sky.ts";
// The shared finish (VP1) and the sky's stars behind True (VP2).
export { startFinish, type Finish, type FinishFrame } from "./finish.ts";
export { GLARE_FLOOR_DEG, glareLadder, glareShare, glareTotal, rungScale, type GlareRung } from "./finish-glare.ts";
export { longGain, starsBehind, starTint, type SkyStar as BehindStar } from "./stars-behind.ts";
export type { GpuStars } from "./gpu.ts";
