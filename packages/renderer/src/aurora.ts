// Aurorae drawn from the kernel's numbers (VP-AU5, visual polish design section 16). One mechanism for every world:
// a driver puts energetic particles into some air at some place, and the air answers in its own lines, each at the
// height where that line survives. Where the glow is comes from `read_field`'s `aurora` (MF9 part 1: wind and spin
// ovals, moon spots, a diffuse glow); how bright, what colour and how high come from MF9 part 2 (light thread):
// `power_w`, `width_deg` and `emissions` with `peak_height_m`, `thickness_m` and a colour. Nothing here asks what a
// world is called or what class it is (rule 5). Until part 2 lands, a marked stand-in draws oxygen's two lines,
// and only on a world whose air holds oxygen; anywhere else the glow waits and says so.
//
// This file is the plan: pure numbers, no GPU. gpu-aurora.ts draws it; the Body globe hands it in.
import type { FieldReadJson, V3 } from "./field.ts";
import { dipoleAxis, fieldFromRead } from "./field.ts";
import { constant, say } from "../../ui-contract/src/index.ts";

/** One line the air gives off in an aurora (MF9 part 2's shape, agreed with the light thread 24 Nov 2026). */

/**
 * A kernel line as words: the kernel's own `name` when it gives one, otherwise its matter and wavelength ("O 557.7 nm").
 * The light table's names are not in the client API yet.
 */
export function lineName(em: { line: string; matter?: string; wavelength_m?: number; name?: string }): string {
  if (em.name) return em.name;
  if (!em.matter || !em.wavelength_m) return em.line;
  const tenths = Math.round(em.wavelength_m * 1e10);
  // Sheet ids are lower case formulas; capitals read as one (O, N2, CO2) until the kernel names its lines.
  const formula = em.matter.toUpperCase();
  return `${formula} ${Math.floor(tenths / 10)}.${tenths % 10} nm`;
}

export interface AuroraEmissionJson {
  /** The line in words, when the kernel gives it. */
  name?: string;
  line: string;
  /** Element or substance id on the data sheet. */
  matter?: string;
  wavelength_m: number;
  /** Total for both hemispheres, W. */
  power_w: number;
  share?: number;
  /** Where the line's lifetime equals the time between collisions, or the particles stop: m above the surface. */
  peak_height_m: number;
  /** Full height range where it is above half its peak, m. */
  thickness_m?: number;
  /** Chromaticity, max channel 1 (sse_light::colour); brightness is in power_w. */
  colour?: { hex?: string; linear_srgb: [number, number, number] };
}

export interface AuroraOvalJson {
  driver: string;
  maps_to_m: number;
  magnetic_latitude_deg: number;
  latitude_range_deg?: [number, number];
  spin_latitude_range_deg: [number, number];
  /** Part 2: both hemispheres, W. */
  power_w?: number;
  power_by_hemisphere?: [number, number];
  /** Part 2: full width, equatorward to poleward edge, degrees. */
  width_deg?: number;
  /** Part 2b: magnetic latitude of the equatorward and poleward edges, degrees. */
  edges_deg?: [number, number];
  emissions?: AuroraEmissionJson[];
  colour?: { hex?: string; linear_srgb: [number, number, number] };
}

export interface AuroraFootprintJson {
  moon: string;
  magnetic_latitude_deg: number;
  lead_deg?: number;
  power_w?: number;
  emissions?: AuroraEmissionJson[];
}

export interface AuroraJson {
  ovals: AuroraOvalJson[];
  footprints: AuroraFootprintJson[];
  diffuse: boolean;
  /** Part 2: the diffuse glow's own lines (an airy world with no field). */
  diffuse_emissions?: AuroraEmissionJson[];
  /** Part 2b's name for it: the glow with its own power, lines and colour. */
  diffuse_glow?: { power_w?: number; emissions?: AuroraEmissionJson[] } | null;
}

/** One glowing shell: a line of one driver, at its height, over its place. */
export interface AuroraLayer {
  kind: "oval" | "diffuse";
  driver: string;
  line: string;
  wavelength_m: number;
  /** Magnetic colatitude of the oval's centre, radians (0 for diffuse: unused). */
  colatitude_rad: number;
  /** Full width of the oval, radians. */
  width_rad: number;
  /** Peak height and half-peak thickness, m. */
  height_m: number;
  thickness_m: number;
  /** Linear sRGB chromaticity, max channel 1. */
  rgb: [number, number, number];
  /** Radiance seen face-on (looking straight down through the layer), W m⁻² sr⁻¹. */
  radiance_w_m2_sr: number;
  grade: "kernel" | "stand-in";
}

export interface AuroraPlan {
  layers: AuroraLayer[];
  /** The magnetic axis in the globe's frame (z the spin pole), pointing to the pole where the field enters. */
  axis: V3;
  /** True when any layer is the oxygen stand-in. */
  stand_in: boolean;
  /** What is drawn and why, in plain words; or why nothing is. */
  note: string;
  /** Things set aside, said. */
  caveats: string[];
}

/** The world's side of the plan: its size and air. */
export interface AuroraWorld {
  radius_m: number;
  /** The kernel's surface scale height (read_air_optics), m. */
  scale_height_m?: number;
  /** Mole fractions by formula. */
  composition?: Record<string, number>;
  /** Tilt of the magnetic axis from the spin axis, degrees, when the read has no field block (MF3's `field.tilt`). */
  tilt_deg?: number;
}

const PLANCK = constant("h");
const C = constant("c");
/** One rayleigh is 10¹⁰ photons per second per square metre of column, into 4π (Hunten, Roach and Chamberlain 1956). */
export const RAYLEIGH_PH_M2_S_SR = 1e10 / (4 * Math.PI);

/** Radiance of a column of `kR` kilorayleighs at wavelength `lambda_m`, W m⁻² sr⁻¹. */
export function kiloRayleighRadiance(kR: number, lambda_m: number): number {
  return kR * 1000 * RAYLEIGH_PH_M2_S_SR * ((PLANCK * C) / lambda_m);
}

/**
 * The oxygen stand-in (docs/fudges.md, "Aurora stand-in"): until MF9 part 2 gives each line's height and power,
 * oxygen's green (557.7 nm, O(¹S)) and red (630.0 nm, O(¹D)) lines at Earth's measured peak heights and typical
 * oval brightness, with heights and thicknesses scaled by the air's scale height over Earth's. Measured values:
 * green peaks near 110 km and red near 250 km; a moderate oval is about 10 kR in green with red a fifth of that
 * (Chamberlain 1961; Rees 1989). Brightness does not scale yet: that needs the driver's power.
 */
export const OXYGEN_STAND_IN = {
  earth_scale_height_m: 8500,
  /** Oxygen's share of the air below which the stand-in draws nothing. */
  min_share: 0.01,
  lines: [
    { line: "O I 557.7 nm", matter: "O", wavelength_m: 557.7e-9, height_m: 110e3, thickness_m: 30e3, kR: 10 },
    { line: "O I 630.0 nm", matter: "O", wavelength_m: 630.0e-9, height_m: 250e3, thickness_m: 100e3, kR: 2 },
  ],
  /** An oval's full width when the kernel gives none, degrees (Earth's is about 3° to 10°: Feldstein and Starkov 1967). */
  width_deg: 5,
} as const;

/** Oxygen's share of an air (O₂ counts twice its atoms' worth of O as one share, O as itself). */
export function oxygenShare(comp: Record<string, number> | undefined): number {
  if (!comp) return 0;
  const tot = Object.values(comp).reduce((a, x) => a + Math.max(0, x), 0);
  if (!(tot > 0)) return 0;
  return ((comp.O2 ?? 0) + (comp.O ?? 0)) / tot;
}

/**
 * CIE 1931 2° colour matching functions, multi-lobe fit of Wyman, Sloan and Shirley 2013 (JCGT 2(2), 1-11),
 * wavelength in nm.
 */
export function cieXyz(nm: number): [number, number, number] {
  const g = (x: number, mu: number, s1: number, s2: number) => {
    const t = (x - mu) / (x < mu ? s1 : s2);
    return Math.exp(-0.5 * t * t);
  };
  const x = 1.056 * g(nm, 599.8, 37.9, 31.0) + 0.362 * g(nm, 442.0, 16.0, 26.7) - 0.065 * g(nm, 501.1, 20.4, 26.2);
  const y = 0.821 * g(nm, 568.8, 46.9, 40.5) + 0.286 * g(nm, 530.9, 16.3, 31.1);
  const z = 1.217 * g(nm, 437.0, 11.8, 36.0) + 0.681 * g(nm, 459.0, 26.0, 13.8);
  return [x, y, z];
}

/**
 * A single line's colour as linear sRGB chromaticity, max channel 1. Pure spectral colours lie outside sRGB; the
 * colour is brought inside by adding white until no channel is negative (the hue kept, saturation lost).
 */
export function lineRgb(lambda_m: number): [number, number, number] {
  const [X, Y, Z] = cieXyz(lambda_m * 1e9);
  let r = 3.2406 * X - 1.5372 * Y - 0.4986 * Z;
  let g = -0.9689 * X + 1.8758 * Y + 0.0415 * Z;
  let b = 0.0557 * X - 0.204 * Y + 1.057 * Z;
  const lo = Math.min(r, g, b);
  if (lo < 0) (r -= lo), (g -= lo), (b -= lo);
  const hi = Math.max(r, g, b) || 1;
  return [r / hi, g / hi, b / hi];
}

/** Area of a band of colatitudes [c − w/2, c + w/2] on a sphere of radius r, m² (one hemisphere). */
function bandArea(r: number, c: number, w: number): number {
  const a = Math.max(0, c - w / 2);
  const b = Math.min(Math.PI / 2, c + w / 2);
  return 2 * Math.PI * r * r * Math.max(0, Math.cos(a) - Math.cos(b));
}

/**
 * The plan for one world: its read_field body (for the aurora and the field's axis) and its air. Part 2's
 * emissions are drawn as given; with none, the oxygen stand-in only where the air holds oxygen.
 */
export function planAurora(read: (FieldReadJson & { aurora?: AuroraJson | null }) | null | undefined, world: AuroraWorld): AuroraPlan {
  const caveats: string[] = [];
  const empty = (note: string): AuroraPlan => ({ layers: [], axis: [0, 0, 1], stand_in: false, note, caveats });
  if (!read) return empty("No aurora drawn: the kernel has not read this world's field yet.");
  const a = read.aurora;
  if (!a || (!a.diffuse && !a.ovals.length)) return empty(read.obstacle?.kind === "surface" ? "No aurora: nothing but bare ground meets the wind here, and there is no air to glow." : "No aurora: the kernel finds no place where particles come down into air.");
  const f = fieldFromRead(read);
  const t0 = ((world.tilt_deg ?? 0) * Math.PI) / 180;
  let axis: V3 = [Math.sin(t0), 0, Math.cos(t0)];
  if (!f && t0) caveats.push("the magnetic pole's longitude is not published yet, so the tilt is drawn toward longitude 0");
  if (f) {
    const d = dipoleAxis(f.model);
    // The pole where the field enters; ovals ring both poles, so its sense only matters for an offset field.
    axis = d;
    if (Math.abs(d[2]) < 0.9999) caveats.push("the magnetic pole's longitude on the map is not published yet, so the tilt is drawn toward the field's own reference longitude");
  }
  const R = world.radius_m;
  const H = world.scale_height_m && world.scale_height_m > 0 ? world.scale_height_m : null;
  const ox = oxygenShare(world.composition);
  const layers: AuroraLayer[] = [];
  let stand = false;
  let waiting = 0;
  const fromKernel = (kind: AuroraLayer["kind"], driver: string, colat: number, width: number, area: number, em: AuroraEmissionJson) => {
    const rgb = em.colour?.linear_srgb ?? lineRgb(em.wavelength_m);
    // Power spread over the glowing area, sent equally into every direction: radiance looking straight down.
    const L = area > 0 ? em.power_w / (4 * Math.PI * area) : 0;
    layers.push({ kind, driver, line: lineName(em), wavelength_m: em.wavelength_m, colatitude_rad: colat, width_rad: width, height_m: em.peak_height_m, thickness_m: em.thickness_m ?? (H ? 2 * H : em.peak_height_m * 0.2), rgb: [rgb[0], rgb[1], rgb[2]], radiance_w_m2_sr: L, grade: "kernel" });
  };
  for (const o of a.ovals) {
    // Part 2b gives both edges: the oval's centre is half way between them, its width the gap.
    const e = o.edges_deg;
    const lat = e ? (Math.abs(e[0]) + Math.abs(e[1])) / 2 : Math.abs(o.magnetic_latitude_deg);
    const colat = ((90 - lat) * Math.PI) / 180;
    if (o.emissions?.length) {
      const width = (((o.width_deg ?? (e ? Math.abs(Math.abs(e[1]) - Math.abs(e[0])) : OXYGEN_STAND_IN.width_deg)) * Math.PI) / 180);
      const area = 2 * bandArea(R, colat, width);
      for (const em of o.emissions) fromKernel("oval", o.driver, colat, width, area, em);
    } else if (ox >= OXYGEN_STAND_IN.min_share && H) {
      stand = true;
      const width = (OXYGEN_STAND_IN.width_deg * Math.PI) / 180;
      const k = H / OXYGEN_STAND_IN.earth_scale_height_m;
      for (const s of OXYGEN_STAND_IN.lines)
        layers.push({ kind: "oval", driver: o.driver, line: s.line, wavelength_m: s.wavelength_m, colatitude_rad: colat, width_rad: width, height_m: s.height_m * k, thickness_m: s.thickness_m * k, rgb: lineRgb(s.wavelength_m), radiance_w_m2_sr: kiloRayleighRadiance(s.kR, s.wavelength_m), grade: "stand-in" });
    } else waiting++;
  }
  if (a.diffuse) {
    const glow = a.diffuse_glow?.emissions?.length ? a.diffuse_glow.emissions : a.diffuse_emissions;
    if (glow?.length) for (const em of glow) fromKernel("diffuse", "diffuse", 0, Math.PI, 4 * Math.PI * R * R, em);
    else waiting++;
  }
  if (a.footprints.length) caveats.push(`${a.footprints.length === 1 ? "one moon's spot is" : `${a.footprints.length} moons' spots are`} not drawn yet: each needs its moon's place in the sky`);
  const lines = [...new Set(layers.map((l) => l.line))];
  const drivers = [...new Set(layers.map((l) => (l.kind === "diffuse" ? "a diffuse glow over the night side" : `${l.driver === "spin" ? "a spin-driven" : "a wind-driven"} oval at ${Math.round(90 - (l.colatitude_rad * 180) / Math.PI)}° magnetic latitude`)))];
  let note: string;
  if (!layers.length) note = `No aurora drawn yet: the kernel places ${a.diffuse ? "a diffuse glow" : `${a.ovals.length === 1 ? "an oval" : `${a.ovals.length} ovals`}`} here, but its colours wait on the air's own lines (MF9 part 2)${ox < OXYGEN_STAND_IN.min_share ? "; this air has no oxygen, so oxygen's green and red are not borrowed" : ""}.`;
  else {
    const hs = layers.map((l) => `${l.line} at about ${Math.round(l.height_m / 1000)} km`);
    note = `Aurora: ${drivers.join(" and ")}, glowing in ${[...new Set(hs)].join(", ")}.${stand ? ` Stand-in: oxygen's two lines at Earth's heights scaled by this air's ${H ? say(H, "length", undefined, { sig: 2 }).text : "unknown"} scale height, until the kernel gives each line's height and power.` : ""}`;
    if (waiting) note += ` ${waiting === 1 ? "One more glow waits" : `${waiting} more glows wait`} on the air's own lines.`;
    void lines;
  }
  return { layers, axis, stand_in: stand, note, caveats };
}

/** The two exposures (section 16.7): long exposure is the default, eye is the honest single exposure. */
export type AuroraExposure = "long" | "eye";

/** Luminous efficacy at 555 nm, lm/W (CIE). */
const KM = 683;
/**
 * Below about this luminance colour vision fades and faint light looks grey (the low end of the mesopic range,
 * about 0.001 to 0.01 cd/m²; CIE 191:2010). [check the exact limits against the source]
 */
export const COLOUR_VISION_CD_M2: [number, number] = [0.001, 0.01];

export interface AuroraDrawLayer {
  colatitude_rad: number;
  width_rad: number;
  diffuse: boolean;
  /** Shell radius and half-peak thickness in globe radii. */
  radius: number;
  thickness: number;
  /** Linear colour × brightness in the renderer's units (face-on), after the exposure. */
  rgb: [number, number, number];
}

export interface AuroraDraw {
  axis: V3;
  layers: AuroraDrawLayer[];
  /** Highest shell's top, globe radii (the camera keeps it in frame). */
  top: number;
  /** The exposure line for the HUD. */
  says: string;
  /** The lift applied to faint light over the daylight exposure (1 for eye). */
  lift: number;
}

/**
 * TEMPORARY until VP1's shared finish (finish.ts: half-float target, metered exposure, glare, AgX) lands: then the
 * shells emit real radiance into the scene and the finish exposes and lifts them with every other faint layer, and
 * this lift goes (docs/fudges.md, "Aurora exposure, temporary").
 *
 * The plan to draw, under one exposure. `sun` is the star's light at the top of the air in the renderer's units
 * (the atmosphere pass's `sun`), `flux_w_m2` the same light in watts; their ratio turns radiance into the
 * renderer's units, so the ground and the aurora share one exposure. Long exposure lifts faint light by one stated
 * ratio for the view, chosen so the brightest layer shows at `target` of the sunlit ground: ratios between faint
 * things stay true. Eye keeps the daylight exposure, and light below colour vision's limit goes grey.
 */
/** A layer's radiance as the eye weighs it: watts times the photopic curve at its line (1 at 555 nm), so a near-UV line
 * with as many watts as the green one shows faint, as it does to the eye and to a colour camera. */
const seenRadiance = (l: AuroraLayer): number => l.radiance_w_m2_sr * cieXyz(l.wavelength_m * 1e9)[1];

export function exposeAurora(plan: AuroraPlan, radius_m: number, mode: AuroraExposure, sun: number, flux_w_m2: number, target = 0.6): AuroraDraw | null {
  if (!plan.layers.length) return null;
  const toUnits = sun / Math.max(flux_w_m2, 1e-30);
  // The sunlit ground of albedo 0.3 seen face-on, in renderer units (gpu-globe's physical lighting: albedo × sun / π).
  const ground = (0.3 * sun) / Math.PI;
  const peak = Math.max(...plan.layers.map(seenRadiance)) * toUnits;
  const lift = mode === "long" && peak > 0 ? Math.max(1, (target * ground) / peak) : 1;
  const layers = plan.layers.map((l): AuroraDrawLayer => {
    const rgb = lineColour(l, mode, seenRadiance(l) * toUnits * lift);
    return { colatitude_rad: l.colatitude_rad, width_rad: l.width_rad, diffuse: l.kind === "diffuse", radius: 1 + l.height_m / radius_m, thickness: Math.max(1e-5, l.thickness_m / radius_m), rgb };
  });
  const top = Math.max(...layers.map((l) => l.radius + l.thickness / 2));
  const real = lift > 1 ? ` (×${lift.toExponential(1).replace("e+", "×10^")})` : "";
  const says = mode === "long" ? `Long exposure: faint light is lifted${real}, so the aurora shows in full colour; Eye shows it as an eye would.` : "Eye: one exposure for the sunlit world; the aurora is at its true brightness, grey where it is too faint for colour vision.";
  return { axis: plan.axis, layers, top, says, lift };
}

/**
 * A line's colour at a brightness: its chromaticity (greyed under Eye where the line is below colour vision's limit)
 * scaled so the colour's luminance carries `value`.
 */
function lineColour(l: AuroraLayer, mode: AuroraExposure, value: number): [number, number, number] {
  let rgb = l.rgb;
  if (mode === "eye") {
    // Luminance of the line face-on, cd/m², from the CIE photopic curve at its wavelength.
    const lum = KM * cieXyz(l.wavelength_m * 1e9)[1] * l.radiance_w_m2_sr;
    const [lo, hi] = COLOUR_VISION_CD_M2;
    const t = Math.max(0, Math.min(1, Math.log10(Math.max(lum, 1e-30) / lo) / Math.log10(hi / lo)));
    const y = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    rgb = [y + (rgb[0] - y) * t, y + (rgb[1] - y) * t, y + (rgb[2] - y) * t];
  }
  const y = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2] || 1;
  const v = value / y;
  return [rgb[0] * v, rgb[1] * v, rgb[2] * v];
}

/** The aurora seen from a site on the ground (VP-AU7): the same shells, in the site's frame. */
export interface AuroraSky {
  /** The magnetic axis in the site's frame (x east, y north, z up). */
  axis: [number, number, number];
  /** Shells in globe radii (the ground is 1); light per unit of the star's light at the top of the air, lifted. */
  layers: AuroraDrawLayer[];
  /** The exposure line for the HUD. */
  says: string;
  lift: number;
}

/**
 * A direction in the globe's frame (z the spin pole, x at longitude 0) turned into a site's frame (x east, y north, z
 * up) at a latitude and longitude, degrees.
 */
export function toSite(v: readonly number[], lat_deg: number, lon_deg: number): [number, number, number] {
  const la = (lat_deg * Math.PI) / 180;
  const lo = (lon_deg * Math.PI) / 180;
  const up = [Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la)];
  const east = [-Math.sin(lo), Math.cos(lo), 0];
  const north = [-Math.sin(la) * Math.cos(lo), -Math.sin(la) * Math.sin(lo), Math.cos(la)];
  const d = (a: number[]) => a[0]! * v[0]! + a[1]! * v[1]! + a[2]! * v[2]!;
  return [d(east), d(north), d(up)];
}

/**
 * The plan for the ground view at a site, under one exposure. Light is per unit of the star's light at the top of the
 * air (the Land screen's units: its sky is drawn that way), so `flux_w_m2` is the star's light at this world.
 * Long exposure lifts all lines by one ratio so the brightest part of the sky from here shows at `target` after the
 * view's own `exposure`; Eye keeps it as it is (TEMPORARY until VP1's finish, as for the globe).
 */
export function auroraSky(plan: AuroraPlan, radius_m: number, site: { lat_deg: number; lon_deg: number }, mode: AuroraExposure, flux_w_m2: number, exposure: number, target = 0.35): AuroraSky | null {
  if (!plan.layers.length) return null;
  const per = 1 / Math.max(flux_w_m2, 1e-30);
  const axis = toSite(plan.axis, site.lat_deg, site.lon_deg);
  const shells = plan.layers.map((l): AuroraDrawLayer => ({ colatitude_rad: l.colatitude_rad, width_rad: l.width_rad, diffuse: l.kind === "diffuse", radius: 1 + l.height_m / radius_m, thickness: Math.max(1e-6, l.thickness_m / radius_m), rgb: [1, 1, 1] }));
  // The brightest the sky gets from here, where it is above the horizon: each line's face-on light times its path at
  // a coarse grid of directions (the path grows toward the horizon, as all-sky cameras record: the van Rhijn effect).
  let peak = 0;
  for (let el = 3; el <= 90; el += 6)
    for (let az = 0; az < 360; az += 15) {
      const a = (az * Math.PI) / 180;
      const e = (el * Math.PI) / 180;
      const d = [Math.cos(e) * Math.sin(a), Math.cos(e) * Math.cos(a), Math.sin(e)];
      let sum = 0;
      plan.layers.forEach((l, i) => (sum += seenRadiance(l) * skyGlowAlong(shells[i]!, axis, d, 2, radius_m)));
      peak = Math.max(peak, sum);
    }
  const seen = peak * per * Math.max(exposure, 1e-30);
  const lift = mode === "long" && seen > 0 ? Math.max(1, target / seen) : 1;
  const layers = shells.map((sh, i) => ({ ...sh, rgb: lineColour(plan.layers[i]!, mode, seenRadiance(plan.layers[i]!) * per * lift) }));
  const real = lift > 1 ? ` (×${lift.toExponential(1).replace("e+", "×10^")})` : "";
  const says = mode === "long" ? `Long exposure: faint light is lifted${real}, so the aurora shows in full colour.` : "Eye: the aurora at its true brightness, grey where it is too faint for colour vision.";
  return { axis, layers, says, lift };
}

/**
 * Where an aurora layer stands in the sky from a site, for words and tests: the light along a direction (site frame)
 * relative to the layer's face-on light, by the same geometry the shader runs. The eye sits `eye_m` above a ground
 * of `radius_m`; the ray meets the shell once on its way out, at a point whose magnetic colatitude sets the ring's
 * share, and the path through the layer is the thickness over the cosine there, capped at the tangent chord.
 */
export function skyGlowAlong(layer: AuroraDrawLayer, axis: readonly number[], dir: readonly number[], eye_m: number, radius_m: number): number {
  const p0 = 1 + eye_m / radius_m;
  const b = p0 * dir[2]!;
  const c = p0 * p0 - layer.radius * layer.radius;
  const s = -b + Math.sqrt(Math.max(0, b * b - c));
  const q = [s * dir[0]!, s * dir[1]!, p0 + s * dir[2]!];
  const n = Math.hypot(q[0]!, q[1]!, q[2]!) || 1;
  const nq = q.map((x) => x / n);
  const cosn = Math.abs(nq[0]! * dir[0]! + nq[1]! * dir[1]! + nq[2]! * dir[2]!);
  const path = Math.min(1 / Math.max(cosn, 1e-4), 2 * Math.sqrt((2 * layer.radius) / layer.thickness));
  const m = Math.abs(nq[0]! * axis[0]! + nq[1]! * axis[1]! + nq[2]! * axis[2]!);
  const colat = Math.acos(Math.min(1, m));
  const sigma = layer.width_rad / 2.3548;
  const ring = layer.diffuse ? 1 : Math.exp(-0.5 * ((colat - layer.colatitude_rad) / sigma) ** 2);
  return ring * path;
}

export const AURORA_REFERENCES = [
  { text: "Hunten, D. M., Roach, F. E., Chamberlain, J. W. (1956). A photometric unit for the airglow and aurora. Journal of Atmospheric and Terrestrial Physics 8, 345-346.", doi: "10.1016/0021-9169(56)90111-8" },
  { text: "Chamberlain, J. W. (1961). Physics of the Aurora and Airglow. Academic Press, New York.", doi: "10.1029/SP041" },
  { text: "Rees, M. H. (1989). Physics and Chemistry of the Upper Atmosphere. Cambridge University Press.", doi: "10.1017/CBO9780511573118" },
  { text: "Feldstein, Y. I., Starkov, G. V. (1967). Dynamics of auroral belt and polar geomagnetic disturbances. Planetary and Space Science 15, 209-229.", doi: "10.1016/0032-0633(67)90190-0" },
  { text: "Wyman, C., Sloan, P.-P., Shirley, P. (2013). Simple analytic approximations to the CIE XYZ color matching functions. Journal of Computer Graphics Techniques 2(2), 1-11.", url: "https://jcgt.org/published/0002/02/01/" },
  { text: "CIE 191:2010. Recommended system for mesopic photometry based on visual performance.", url: "https://cie.co.at/publications/recommended-system-mesopic-photometry-based-visual-performance" },
];

/** What `loadAurora` asks of the session: the field read and (for a generated world's tilt) the resolved facts. */
export interface AuroraSession {
  readField(ask: { fixture?: string; system?: string }): Promise<{ ok: true; bodies: unknown[] } | { ok: false; code: string; message: string }>;
  resolvedSystem(system: string): Promise<{ ok: true; result: { facts: Record<string, Record<string, unknown>> } } | { ok: false; [k: string]: unknown }>;
}

/**
 * One world's aurora plan, read once: the Solar System's own worlds from read_field's measured fixture (matched by
 * name inside the campaign's Solar System), any other adopted system world by world from its facts, with its MF3
 * `field.tilt`. Null when the kernel has no read for it. Shared by the Body globe and the Land screen.
 */
export async function loadAurora(session: AuroraSession, system: string, body: { address: string; label: string }, world: Omit<AuroraWorld, "tilt_deg">): Promise<{ plan: AuroraPlan; radius_m: number } | null> {
  const sol = /\/sys:sol$/.test(system);
  const r = await session.readField(sol ? { fixture: "solar-system" } : { system });
  const bodies = r.ok ? (r.bodies as (FieldReadJson & { aurora?: AuroraJson | null })[]) : [];
  const read = sol ? bodies.find((b) => b.name === body.label) : bodies.find((b) => b.address === body.address);
  if (!read) return null;
  let tilt: number | undefined;
  if (!sol) {
    const rs = await session.resolvedSystem(system);
    const t = rs.ok ? (rs as { result: { facts: Record<string, Record<string, unknown>> } }).result.facts[body.address]?.["field.tilt"] : undefined;
    if (typeof t === "number") tilt = t;
  }
  const radius = world.radius_m > 0 ? world.radius_m : (read.field?.reference_radius_m ?? 0);
  if (!(radius > 0)) return null;
  return { plan: planAurora(read, { ...world, radius_m: radius, ...(tilt !== undefined ? { tilt_deg: tilt } : {}) }), radius_m: radius };
}
