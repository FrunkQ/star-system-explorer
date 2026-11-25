// The pass library (R1). Each pass is code shipped and tested here; recipes
// only name passes and set their parameters. Every parameter is described
// for Orac first (name, type, range, default, unit, meaning, group) in the
// same ParamSpec shape the views use (V11). Costs are per frame on the
// reference device and are estimates until the probe measures this device
// (probe.ts); the governor works from measured frame times either way. A GPU
// cost's `load` says which of the probe's three scenes bounds it.
import type { PassCost, PassSpec } from "./recipe.ts";

const MESH: PassCost["load"] = { geometry: 0.5, fill: 0.5 };
const LINES: PassCost["load"] = { geometry: 1 };
const MARCH: PassCost["load"] = { march: 1 };

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
  tiers: ["still", "gpu-high", "gpu-basic", "canvas"],
  params: [
    { name: "colour", type: "enum", default: "survey", options: ["survey", "height"], meaning: "Survey: water by depth and bare ground by material. Height: a height tint from the lowest ground to the highest, with water kept blue.", group: "look", promoted: true },
    { name: "shade", type: "number", default: 0.8, range: [0, 1], meaning: "How strongly slopes and the globe's own curve are shaded (0 is flat colour).", group: "look" },
    { name: "exaggeration", type: "number", default: 12, range: [1, 60], unit: "×", meaning: "Vertical exaggeration of slopes for the shading only; heights are never changed.", group: "look" },
    { name: "sun_azimuth_deg", type: "number", default: 315, range: [0, 360], unit: "°", meaning: "Where the light comes from on the screen, clockwise from straight up (315 is the upper left, the map-maker's convention).", group: "look" },
    { name: "sun_elevation_deg", type: "number", default: 40, range: [5, 90], unit: "°", meaning: "How high the light stands above the screen's plane.", group: "look" },
    { name: "ambient", type: "number", default: 0.16, range: [0, 0.6], meaning: "Light on the night side and in shadow, as a share of full light.", group: "look" },
    { name: "grid", type: "integer", default: 128, range: [16, 256], meaning: "GPU tier: mesh cells along each cube face's edge. More is rounder and finer, and costs more.", group: "look" },
    { name: "lighting", type: "enum", default: "display", options: ["display", "physical"], meaning: "Display: shaded in screen colours, the same on every tier (a map's look). Physical (GPU only): linear albedo lit by the star through the air, with the night side dark, then tone-mapped.", group: "look" },
  ],
  cost: {
    canvas: { fixed_ms: 0.5, per_mpix_ms: 60 },
    "gpu-basic": { fixed_ms: 1.5, per_mpix_ms: 1.5, scale: [{ param: "grid", ref: 128, power: 1.2 }], load: MESH },
    "gpu-high": { fixed_ms: 1.5, per_mpix_ms: 2, scale: [{ param: "grid", ref: 128, power: 1.2 }], load: MESH },
    still: { fixed_ms: 1.5, per_mpix_ms: 2, scale: [{ param: "grid", ref: 128, power: 1.2 }], load: MESH },
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
  tiers: ["still", "gpu-basic", "canvas"],
  params: [
    { name: "spacing_deg", type: "number", default: 30, range: [5, 90], unit: "°", meaning: "Degrees between lines.", group: "look", promoted: true },
    { name: "opacity", type: "number", default: 0.3, range: [0, 1], meaning: "How strongly the lines show.", group: "look" },
  ],
  cost: { canvas: { fixed_ms: 0, per_mpix_ms: 8 }, "gpu-basic": { fixed_ms: 0.3, per_mpix_ms: 0.2, load: LINES }, still: { fixed_ms: 0.3, per_mpix_ms: 0.2, load: LINES } },
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

export const ATMOSPHERE: PassSpec = {
  id: "atmosphere",
  version: "1.0.0",
  name: "Atmosphere",
  meaning: "The world's air: light scattered by its molecules (Rayleigh) towards the viewer, from the kernel's surface pressure, temperature, gravity and gas mix, and the dimming of the ground behind it. GPU tiers ray-march single scattering through a shell; the canvas tier uses a one-column approximation. Worlds without air draw nothing.",
  kind: "overlay",
  inputs: ["body.air"],
  tiers: ["still", "gpu-high", "gpu-basic", "canvas"],
  params: [
    { name: "steps", type: "integer", default: 16, range: [4, 64], meaning: "GPU: samples along each view ray through the air.", group: "look" },
    { name: "light_steps", type: "integer", default: 6, range: [2, 16], meaning: "GPU: samples from each of those towards the star.", group: "look" },
    { name: "sun", type: "number", default: 6, range: [1, 100], meaning: "The star's light at the top of the air, in the renderer's units (the ground and the sky share it, so their balance is physical).", group: "look" },
    { name: "exposure", type: "number", default: 1, range: [0.1, 8], meaning: "Camera exposure before tone mapping (the canvas tier has no tone mapping, so there it is a plain brightness gain).", group: "look", promoted: true },
    { name: "thickness", type: "number", default: 1, range: [1, 20], unit: "×", meaning: "Draw the air this many times thicker than it is (1 is true scale). Above 1 is a look, not the world, and the badge says so.", group: "look" },
  ],
  cost: {
    "gpu-high": { fixed_ms: 0.5, per_mpix_ms: 6, scale: [{ param: "steps", ref: 16, power: 1 }, { param: "light_steps", ref: 6, power: 1 }], load: MARCH },
    still: { fixed_ms: 0.5, per_mpix_ms: 6, scale: [{ param: "steps", ref: 16, power: 1 }, { param: "light_steps", ref: 6, power: 1 }], load: MARCH },
    "gpu-basic": { fixed_ms: 0.5, per_mpix_ms: 6, scale: [{ param: "steps", ref: 16, power: 1 }, { param: "light_steps", ref: 6, power: 1 }], load: MARCH },
    canvas: { fixed_ms: 0, per_mpix_ms: 6 },
  },
  references: [
    { text: "Nishita, T., Sirai, T., Tadamura, K., Nakamae, E. (1993). Display of the Earth taking into account atmospheric scattering. SIGGRAPH 93, 175-182.", doi: "10.1145/166117.166140" },
    { text: "Bucholtz, A. (1995). Rayleigh-scattering calculations for the terrestrial atmosphere. Applied Optics 34(15), 2765-2773.", doi: "10.1364/AO.34.002765" },
    { text: "Kasten, F., Young, A. T. (1989). Revised optical air mass tables and approximation formula. Applied Optics 28(22), 4735-4738.", doi: "10.1364/AO.28.004735" },
  ],
};

export const AURORA: PassSpec = {
  id: "aurora",
  version: "1.0.0",
  name: "Aurora",
  meaning: "Where the kernel says particles come down into the air (read_field's aurora: wind and spin ovals round the magnetic poles, a diffuse glow on airy worlds with no field), each line the air gives off as a thin glowing shell at its own height, stacked at the limb. Colours and heights come from the air's own lines; worlds where the kernel finds no aurora draw nothing. GPU tiers only.",
  kind: "overlay",
  inputs: ["body.field", "body.air"],
  tiers: ["still", "gpu-high", "gpu-basic"],
  params: [
    { name: "exposure", type: "enum", default: "long", options: ["long", "eye"], meaning: "Long exposure (the default): faint light is lifted by one stated ratio so the aurora shows in full colour, as a camera's long exposure records it. Eye: one exposure for the sunlit world, the aurora at its true brightness and grey where it is too faint for colour vision.", group: "look", promoted: true },
    { name: "lift_target", type: "number", default: 0.6, range: [0.05, 2], meaning: "Long exposure: the brightest line is lifted to this share of the sunlit ground's brightness; the other lines keep their true ratio to it.", group: "look" },
  ],
  cost: {
    "gpu-high": { fixed_ms: 0.2, per_mpix_ms: 1 },
    "gpu-basic": { fixed_ms: 0.2, per_mpix_ms: 1 },
    still: { fixed_ms: 0.2, per_mpix_ms: 1 },
  },
  references: [
    { text: "Chamberlain, J. W. (1961). Physics of the Aurora and Airglow. Academic Press, New York.", doi: "10.1029/SP041" },
    { text: "Hunten, D. M., Roach, F. E., Chamberlain, J. W. (1956). A photometric unit for the airglow and aurora. Journal of Atmospheric and Terrestrial Physics 8, 345-346.", doi: "10.1016/0021-9169(56)90111-8" },
  ],
};

export const NARRATION: PassSpec = {
  id: "text-narration",
  version: "1.0.0",
  name: "Narration",
  meaning: "The globe in words: the face turned to the viewer, what covers it, the highest and lowest ground in view and which way they lie.",
  kind: "text",
  inputs: ["surface.tiles", "body.label"],
  tiers: ["text", "canvas", "gpu-basic", "gpu-high", "still"],
  params: [{ name: "detail", type: "enum", default: "full", options: ["brief", "full"], meaning: "Brief: one sentence. Full: cover, heights and where they lie.", group: "content" }],
  cost: { text: { fixed_ms: 2, per_mpix_ms: 0 }, canvas: { fixed_ms: 0, per_mpix_ms: 0 }, "gpu-basic": { fixed_ms: 0, per_mpix_ms: 0 }, "gpu-high": { fixed_ms: 0, per_mpix_ms: 0 }, still: { fixed_ms: 0, per_mpix_ms: 0 } },
};

/** Every pass this release ships. */
export const LIBRARY: readonly PassSpec[] = [RELIEF, GRATICULE, CONTOURS, ATMOSPHERE, AURORA, NARRATION];
