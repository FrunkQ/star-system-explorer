// Renderer packages (H8.1, SSE4-surfaces-and-rendering.md). Frames and the
// camera-relative rule are plain f64 code here; the GPU wrapper loads three.js
// only when a view asks for it, so pages without 3D never fetch it.
export { tree, pathTo, relative, basis, project, projectExact, projectFlattened, rebase, frameFor } from "./frames.ts";
export type { Vec3, FrameNode, Tree, Camera, Pixel } from "./frames.ts";
export type { Gpu, GpuDots, GpuGlow, GpuItem, GpuLight, GpuRing, GpuSheet } from "./gpu.ts";
export { armField, cloudNoise, GALAXY_BANDS, GALAXY_LIGHT, GALAXY_LIGHT_REFS, galaxyFace, glowPicture, sheetWeights, surfaceAt } from "./galaxy-true.ts";
export type { GalaxyBand, GalaxyPicture } from "./galaxy-true.ts";
export { cubeEdges, densityLevel, galaxyLight, mixCss, SPACE_CSS, webGlow, webLights, webParcelLevels, webParcels, WEB_DARK_GLOW, WEB_DENSITY_LEVELS, WEB_LEVEL_INK, WEB_LEVELS, WEB_KINDS, WEB_MARKS, webMarks, WEB_LIGHT_K, WEB_LIGHT_SHOW } from "./web-cube.ts";
export type { WebCubeData, WebGalaxyData, WebKind, WebLightGroup } from "./web-cube.ts";
// V10a: True in the system view, each world wrapped in its own surface.
export { equirect, starCss, thermalGrey, THERMAL_DECADES, THERMAL_IR_M, trueSurface } from "./true-look.ts";
export type { TrueTexture } from "./true-look.ts";
// R1 to R3, R6: air in True (clouds, hazes, giants from their cloud decks), true-air.ts.
export { flowFace, gasAbove, gasTint, readCloudRows, readTrueAir, readTrueFace, sheetColours, CLOUD_ROW_NAMES, trueGiant, withAir, AIR_REFS, type AirFlow, type AirLook, type FaceReader, type FlowStorm } from "./true-air.ts";
// VP-A3: the air's look-up tables (Hillaire 2020) for its rim and terminator in True.
export { airLuts, airShapeOf, AIR_LUT_REFS, type AirLuts, type AirShape } from "./air-luts.ts";
// VP3 part 5: bare ground in its measured colour (data sheet reflectance rows, kernel SW1 maturity), ground-optics.ts.
export { bandsOf, groundReflectance, readGroundOptics, reflectanceAt, withGround, GROUND_BANDS_NM, GROUND_ROWS, type GroundOptics, type MatterReader, type ReflectanceTable } from "./ground-optics.ts";
// TA6: a world flattened by its spin (Darwin-Radau flow), true-shape.ts.
export { flattenScale, readTrueShape, SHAPE_FLOW, type ShapeReader, type TrueShape } from "./true-shape.ts";
// V10f: rings and belts in True, from the universe's ring and belt data.
export { BELT_DUST, beltAt, beltHaze, beltParticles, kernelProfile, poleOf, RING_FALLBACK, RING_LOOK_SCALE, ringLook, ringProfile } from "./rings.ts";
export type { BeltParticle } from "./rings.ts";
export type { KernelRing, RingBand, RingLookData, RingPhase, RingProfile } from "./rings.ts";
// R1: render recipes, the pass library, the governor and the globe runtime.
export { checkRecipe, plan, estimateMs, pickRecipe, bestTier, maxLevel, stepsFrom, TIER_ORDER, DEFAULT_BUDGET_MS } from "./recipe.ts";
export type { RecipeDoc, RecipePass, PassSpec, PassCost, Knob, DegradeStep, Plan, RecipeCheck } from "./recipe.ts";
export { Governor, badge } from "./governor.ts";
export type { Badge, GovernorOptions, GovernorMove } from "./governor.ts";
export { LIBRARY } from "./passes.ts";
export { mountGlobe, airRender, deviceReadout, measureAgain, passShares, quality, setQuality } from "./runtime.ts";
// VP4 part 2b, VP5: one quality controller per True view (placement, governor, heat, sharpen at rest).
export { TrueRun, type TrueSettings, type TrueStatus } from "./true-run.ts";
export { REST_FRAMES, RestClock, restJitter, type RestState } from "./still.ts";
// VP4: the speed probe and placement by measured speed.
export { BENCH_PIXELS, drawingBench, factorsFrom, gpuTiersAllowed, HeatWatch, loadProbe, loadQuality, place, PLACE_MARGIN, PROBE_SCENES, probeKey, QUALITY_WORDS, tierWords, ULTRA_HEADROOM } from "./probe.ts";
export type { Capabilities, DeviceFactors, DrawingBench, Placement, ProbeResult, ProbeScene, QualityChoice, TierPrediction } from "./probe.ts";
export { air, airFactsOf, gasMixOf, surfacePressureOf, airFromOptics, airRequest, KERNEL_GASES } from "./atmosphere.ts";
export type { Air, AirFacts } from "./atmosphere.ts";
export type { GlobeRun, GlobeRunOptions, GlobeStatus } from "./runtime.ts";

/** Start the GPU path (WebGPU, else WebGL2), or null when this device has neither. */
export async function startGpu(canvas: HTMLCanvasElement, opts: { timing?: boolean } = {}) {
  const { createGpu } = await import("./gpu.ts");
  return createGpu(canvas, opts);
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
export { cycleOf, drawLiveStar, flarePlace, flareProfile, flaresAt, PROMINENCE_WINDOW_S, readLiveStar, spotsOf, targetsFromFacts, turnAt, type ActivityTarget } from "./live-star.ts";
export type { Flare, GivenStar, LiveStar, StarReader } from "./live-star.ts";
export { drawGlow, glarePsf, glareProfile, glowReach, GLARE_EYE, GLOW_REACH_DEG, GLOW_REFERENCES, narrateGlow } from "./star-glow.ts";
export { BIG_CELL_DT_MAX, BIG_CELLS_LEAST, bigCellContrast, cellsAcross, churningBigCells, GAMMA_PHOTOSPHERE, CELL_DRIFT, CHURN_REFERENCES, churningGranules, granuleBlur, limbLumps, LUMP_SCALE_HEIGHTS, narrateLumps } from "./star-churn.ts";
export { addEjections, capFlashes, EJECTION_SCALE, ejectionShare, ejectionShown, ejectionsAt, EVENT_REFERENCES, FLASHES_PER_SECOND_MAX, flareBoost, flareWhiteLight, narrateEvents, patchContrast, patchRadiusDeg, shownFlares } from "./star-events.ts";
export type { EjectionNow, FlashEvent, ShownFlare } from "./star-events.ts";
export { drawStarNow, FACE_STEPS, liveFace } from "./star-now.ts";
export { addProminences, narrateProminences, PROMINENCE_REFERENCES, PROMINENCE_SCALE, prominenceColour, prominencePoints, prominenceShown, prominencesAt } from "./star-prominences.ts";
export type { Prominence, ProminenceNow } from "./star-prominences.ts";
export { rateShareOf, regionLon, regionShare, spotsFromRegions, type Region, type Regions } from "./star-regions.ts";
export type { StarNow, StarNowAsk } from "./star-now.ts";
// Time-lapse layers on the Body globe (surface_frames): read, bracket, colour, pick, narrate.
export { bracket, cellOf, drawnCellOf, eventsBetween, FIELDS, LAYERS, layerColour, layersFor, narrateFrame, pickAt, plateCellAngle, readFrames, recolour, seaLevelAt, valueAt } from "./timelapse.ts";
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
export type { Callout, DesignBay, DesignOverrides, DesignRecipe, DesignRules, DesignShip, Needs, Piece, ShipLayout, ShipStyle, ShipView, SystemKind, ViewPath } from "./ship-design.ts";
// V3 gap item 14, stage 3: the 3D model from the same layout, with panel textures from the style's skin.
export { compoundMesh, panelTexture, PANEL_UNIT, shipMesh, SURFACE_OF } from "./ship-mesh.ts";
export { baysFromFacts, designCompound, mountFromFacts, placeCompound, portsFromFacts, reachOf, ringPoints, sizeBays } from "./compound-layout.ts";
export type { BayFacts, CompoundLayout, CompoundMember, CompoundRules, MountFacts, MountPlace, PlacedMember, PortFacts, PortMark } from "./compound-layout.ts";
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
export type { LanderPose, LanderShip } from "./surface-lander.ts";

/** Draw the ground round a site on the GPU (WebGPU, else WebGL2), or null when this device has neither. three.js loads only here. */
export async function startSurface(canvas: HTMLCanvasElement) {
  const { createSurfaceGpu } = await import("./gpu-surface.ts");
  return createSurfaceGpu(canvas);
}
// The size comparison's turning globes (3D, True, Wireframe).
export { circleSteps, globeFrame, graticule, bodySeconds, hoursPerSecond, nearSide, ringEdge, SECONDS_PER_TURN, spinAt } from "./lineup-3d.ts";
export type { Frame as GlobeFrame, GlobeLine, LineupView } from "./lineup-3d.ts";
export { byDensitySquared, density as shellDensity, drawShells, dustGlow10um, narrateShells, nebulaColour, SHELL_DECADES, SHELL_REFERENCES, wavelengthRgb } from "./star-shells.ts";
export type { NebulaShape, ShellView } from "./star-shells.ts";
export { baumbach, CORONA_DECADES, CORONA_REFERENCES, coronaShape, cycleActivity, drawCorona, narrateCorona } from "./star-corona.ts";
export { DIFFERENTIAL_ROTATION, rateShare, spotsAt } from "./live-star.ts";
// A body's face in True, one door for every view (true-body.ts).
export { meanColour, ringsAssumed, trueLook } from "./true-body.ts";
export { trueStill, trueTurn, type TrueStill, type TrueTurn } from "./true-still.ts";
export type { TrueBody, TrueLook } from "./true-body.ts";
// Hot things glow in Visible and plumes above the limb (third audit R10 to R12, track TRUE).
export { drawnGlow, GLOW_FLOOR, hotGlow } from "./hot-glow.ts";
export type { HotGlow } from "./hot-glow.ts";
export { plumeBrightness, plumeFacts, plumeGrains, plumeNote } from "./plumes.ts";
export type { PlumeFacts } from "./plumes.ts";
// Lava tube openings cut into the ground at a landing site (third audit, TRUE6).
export { caveCut, caveOpenings, nearestOpening, openingLine } from "./cave-mouths.ts";
export type { CaveOpening } from "./cave-mouths.ts";
// Surface stains hued from the data sheet's optics (TRUE4).
export { grainAlbedo, indexTableOf, layerReflectance, mixAlbedo, seenColour, GRAIN_UM } from "./sheet-optics.ts";
export type { GrainShare, IndexTable } from "./sheet-optics.ts";
export { readStains, stainFacts, stainHue, stainNote, stainPrepared, stainTiles, STAIN_GRAIN_UM } from "./stain-look.ts";
export type { StainFact, StainHue, StainReader } from "./stain-look.ts";
// Small bodies' own shapes in True (SB3, small-body.ts).
export { figureRadius, meanRadius, smallBodyColour, smallBodyMesh, smallBodyShape } from "./small-body.ts";
export { COMET_REFERENCES, comaPicture, cometNote, cometTails, hexLinear, ionColour, isActive, linearCss, showing, tailGrains } from "./comet.ts";
export type { CometActivity } from "./comet.ts";
export type { SmallFigure, SmallLobe, SmallMesh } from "./small-body.ts";
// The stars over a landing site and the eye's adaptation (night-sky.ts).
export { ADAPT_RANGE, adaptedExposure, starsOverSite } from "./night-sky.ts";
export type { SiteSkyFrame, SiteSkyStar, SiteStar } from "./night-sky.ts";
// Aurorae (VP-AU5): the plan from read_field and the air, and its exposure.
export { AURORA_REFERENCES, auroraSky, lineName, cieXyz, COLOUR_VISION_CD_M2, exposeAurora, loadAurora, skyGlowAlong, toSite, kiloRayleighRadiance, lineRgb, oxygenShare, OXYGEN_STAND_IN, planAurora, RAYLEIGH_PH_M2_S_SR } from "./aurora.ts";
export type { AuroraDraw, AuroraDrawLayer, AuroraSession, AuroraSky, AuroraEmissionJson, AuroraExposure, AuroraJson, AuroraLayer, AuroraOvalJson, AuroraPlan, AuroraWorld } from "./aurora.ts";
// Close-up: the camera down to a grazing orbit over a world in True, and the ground under it.
export { CLOSE_FLOOR_M, CLOSE_FLOOR_SHARE, CLOSE_REACH, closeMin, grazeLook, groundUnder, orbitEye, orbitOf, zoomNear, type GroundView, type Orbit } from "./close-up.ts";
// The shared finish (VP1) and the sky's stars behind True (VP2).
export { startFinish, type Finish, type FinishFrame } from "./finish.ts";
export { GLARE_FLOOR_DEG, glareLadder, glareShare, glareTotal, rungScale, type GlareRung } from "./finish-glare.ts";
export { longGain, starsBehind, starTint, type SkyStar as BehindStar } from "./stars-behind.ts";
export type { GpuStars } from "./gpu.ts";
// VP6: the detail ladder (rung table, bindings, seed and clock, HUD detail line). The GPU node is ladder-node.ts,
// imported by GPU code directly so pages without 3D never load three.js.
export { checkLayer, detailLine, profileShare, discScale, EDDY_EXPONENT, FLOW_REACH, LADDER_LAYERS, ladderAt, ladderLayer, ladderSeed, MAX_RUNGS, MAX_STRETCH, resolveLayer, RUNG_FADE, RUNG_MIN_PX, rungOrigin, rungs, sizeWords, TIER_RUNGS } from "./ladder.ts";
export type { Bound, LadderFacts, LadderLayer, NoiseKind, NotDrawn, ResolvedLayer, Rung, ViewScale } from "./ladder.ts";
export { cell, curl, NOISE_REFERENCES, NUM, PERIOD, ridged, simplex, SIMPLEX_SCALE, WRAP } from "./ladder-noise.ts";
export type { Ops, V3 } from "./ladder-noise.ts";
// VP3: the texture ladder (face sizes by screen, tiles to read, face pictures from any tile levels, relief normals).
// The GPU bake is texture-ladder-gpu.ts, used by gpu.ts.
export { BAKE_BUDGET, composeFaces, COMPOSE_MAX, cubeBytes, directionFace, FACE_CAP, FACE_MIN, faceDirection, faceSize, faceWords, levelFor as tileLevelFor, reliefNormal, sampleFaces, texelAngle, TILE_SAMPLES, tilesFacing, withFiner, composePatch, frameAt, patchDirection, patchFor, patchLevel, PATCH_N, PATCH_PX_PER_SAMPLE, PATCH_TILES, samplePatch, tilesNear } from "./texture-ladder.ts";
export type { Composite, FacePicture, GroundAsk, PatchPicture, TileKey } from "./texture-ladder.ts";
export { discCover, GLINT_FALLBACK, glintBrdf, MAX_CASTERS, ringPassAt, seenFrom, shadeFor, starLightAt, turnOnto, umbraLight, type Caster, type Placed, type Shade, type ShadowLightRead } from "./eclipse.ts";
export { readSeaGlint, readUmbras, umbraKey, type GlintReader, type SeaGlint, type UmbraPair, type UmbraReader } from "./eclipse-read.ts";
