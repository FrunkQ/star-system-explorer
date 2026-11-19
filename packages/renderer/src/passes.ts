// The pass library (R1). Each pass is code shipped and tested here; recipes
// only name passes and set their parameters. Every parameter is described
// for Orac first (name, type, range, default, unit, meaning, group) in the
// same ParamSpec shape the views use (V11). Costs are per frame on the
// reference device and are estimates until the probe measures this device
// (probe.ts); the governor works from measured frame times either way.
import type { PassSpec } from "./recipe.ts";

const HILLSHADE_REF = {
  text: "Horn, B. K. P. (1981). Hill shading and the reflectance map. Proceedings of the IEEE 69(1), 14-47.",
  doi: "10.1109/PROC.1981.11918",
};

export const RELIEF: PassSpec = {
  id: "relief-globe",
  version: "1.0.0",
  name: "Relief globe",
  meaning: "The world's surface tiles on a globe, coloured by cover and bare material, shaded by its real slopes (hill shading from the elevation layer) under one light.",
  kind: "image",
  inputs: ["surface.tiles"],
  tiers: ["gpu-basic", "canvas"],
  params: [
    { name: "colour", type: "enum", default: "survey", options: ["survey", "height"], meaning: "Survey: water by depth and bare ground by material. Height: a height tint from the lowest ground to the highest, with water kept blue.", group: "look", promoted: true },
    { name: "shade", type: "number", default: 0.8, range: [0, 1], meaning: "How strongly slopes and the globe's own curve are shaded (0 is flat colour).", group: "look" },
    { name: "exaggeration", type: "number", default: 12, range: [1, 60], unit: "×", meaning: "Vertical exaggeration of slopes for the shading only; heights are never changed.", group: "look" },
    { name: "sun_azimuth_deg", type: "number", default: 315, range: [0, 360], unit: "°", meaning: "Where the light comes from on the screen, clockwise from straight up (315 is the upper left, the map-maker's convention).", group: "look" },
    { name: "sun_elevation_deg", type: "number", default: 40, range: [5, 90], unit: "°", meaning: "How high the light stands above the screen's plane.", group: "look" },
    { name: "ambient", type: "number", default: 0.16, range: [0, 0.6], meaning: "Light on the night side and in shadow, as a share of full light.", group: "look" },
    { name: "grid", type: "integer", default: 128, range: [16, 256], meaning: "GPU tier: mesh cells along each cube face's edge. More is rounder and finer, and costs more.", group: "look" },
  ],
  cost: {
    canvas: { fixed_ms: 0.5, per_mpix_ms: 60 },
    "gpu-basic": { fixed_ms: 1.5, per_mpix_ms: 1.5, scale: [{ param: "grid", ref: 128, power: 1.2 }] },
  },
  references: [HILLSHADE_REF],
};

export const GRATICULE: PassSpec = {
  id: "graticule",
  version: "1.0.0",
  name: "Graticule",
  meaning: "Lines of latitude and longitude over the globe, with the equator drawn a little stronger.",
  kind: "overlay",
  inputs: [],
  tiers: ["gpu-basic", "canvas"],
  params: [
    { name: "spacing_deg", type: "number", default: 30, range: [5, 90], unit: "°", meaning: "Degrees between lines.", group: "look", promoted: true },
    { name: "opacity", type: "number", default: 0.3, range: [0, 1], meaning: "How strongly the lines show.", group: "look" },
  ],
  cost: { canvas: { fixed_ms: 0, per_mpix_ms: 8 }, "gpu-basic": { fixed_ms: 0.3, per_mpix_ms: 0.2 } },
};

export const CONTOURS: PassSpec = {
  id: "contours",
  version: "1.0.0",
  name: "Contours",
  meaning: "Lines of equal height over the globe at a fixed interval, and the shoreline where cover meets bare ground.",
  kind: "overlay",
  inputs: ["surface.tiles"],
  tiers: ["canvas"],
  params: [
    { name: "interval_m", type: "number", default: 1000, range: [100, 5000], unit: "m", meaning: "Height between contour lines.", group: "look", promoted: true },
    { name: "opacity", type: "number", default: 0.45, range: [0, 1], meaning: "How strongly the lines show.", group: "look" },
    { name: "shoreline", type: "boolean", default: true, meaning: "Draw the shoreline (where water, magma or hydrocarbon meets bare ground) as a firmer line.", group: "content" },
  ],
  cost: { canvas: { fixed_ms: 0, per_mpix_ms: 10 } },
};

export const NARRATION: PassSpec = {
  id: "text-narration",
  version: "1.0.0",
  name: "Narration",
  meaning: "The globe in words: the face turned to the viewer, what covers it, the highest and lowest ground in view and which way they lie.",
  kind: "text",
  inputs: ["surface.tiles", "body.label"],
  tiers: ["text", "canvas", "gpu-basic"],
  params: [{ name: "detail", type: "enum", default: "full", options: ["brief", "full"], meaning: "Brief: one sentence. Full: cover, heights and where they lie.", group: "content" }],
  cost: { text: { fixed_ms: 2, per_mpix_ms: 0 }, canvas: { fixed_ms: 0, per_mpix_ms: 0 }, "gpu-basic": { fixed_ms: 0, per_mpix_ms: 0 } },
};

/** Every pass this release ships. */
export const LIBRARY: readonly PassSpec[] = [RELIEF, GRATICULE, CONTOURS, NARRATION];
