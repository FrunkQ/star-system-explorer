// Air in True at system scale (audit items R1 to R3 and R6, 24 Nov 2026): a world's clouds and haze from the kernel's
// `weather` op, laid over its True surface, and a giant's face made of nothing but its cloud decks. Nothing is
// invented here. The decks (which condensate, how much cover, what optical depth), the bands between the jets, the
// storms and the polar cyclones are all the weather block's, cell by cell; each layer's colour is its condensate's on
// the master data sheet, read by the caller; a haze's colour is the one the weather op sends. This file only stacks
// the layers by the adding method (Bohren 1987), as weather.ts does on the Body globe.
import { readStains } from "./stain-look.ts";
import CLOUD_ROWS from "./cloud-rows.json" with { type: "json" };
import { constant, sheetUnitFactor } from "../../ui-contract/src/index.ts";
import type { Json, SurfaceTile } from "../../ui-contract/src/index.ts";
import { trueSurface, type TrueTexture } from "./true-look.ts";
import { readGroundOptics, withGround } from "./ground-optics.ts";
import { airShapeOf, type AirShape } from "./air-luts.ts";
import { airFactsOf, airFromOptics, airRequest, KERNEL_GASES } from "./atmosphere.ts";
import { detailGiant, readGiantDetail } from "./giant-detail.ts";
import type { CarryTable } from "./stirred-carry.ts";
import { addLayers, CLOUD_G, HAZE_G, liveSystems, slabReflectance, stormAt, type WeatherCell, type WeatherMap, type WeatherSystem, type WeatherTile } from "./weather.ts";

/** Linear RGB, 0..1. */
export type Rgb = [number, number, number];

/**
 * A cloud deck's colour by the condensate the weather op names ("ammonia ice", "water"), in linear RGB, or null when
 * the sheet holds none. The caller reads it from the sheet (`sheetColours`), so no colour is typed here.
 */
export type ColourOf = (condensate: string) => Rgb | null;

/** What a world's air looks like from afar, with what it could not colour. */
export interface AirLook {
  tex: TrueTexture;
  /** Plain words for the HUD note: decks drawn white because the sheet has no colour for them, and so on. */
  said: string[];
  /** The wind the picture is carried by (VP-A1), pixel for pixel; left out when the cells carry no wind. */
  flow?: AirFlow;
  /** True for a world with no ground, whose face is its cloud decks alone (a giant: `trueGiant`). */
  decks?: boolean;
  /** A world with ground: the ground alone (true-look.ts `trueSurface`'s own picture, with its faces), under its air. */
  ground?: TrueTexture;
  /**
   * A world with ground: its clouds and haze alone (VP-A4), in the same layout. RGB is their own reflected light,
   * premultiplied and sRGB-encoded as every TrueTexture is; alpha is their reflectance, so 1 - alpha reaches the
   * ground. `tex` is the two laid together.
   */
  clouds?: TrueTexture;
  /** The world's air in its own radius, from the kernel's optics (VP-A3), for the rim and terminator. */
  air?: AirShape;
  /**
   * A giant's stirred still (WS5), on its way: the face carried back along the kernel's winds, built on a GPU of its
   * own and kept in memory. The view shows `tex` until it comes, then this; null where it could not be built.
   */
  stirred?: Promise<TrueTexture | null>;
}

/**
 * The wind at the visible deck, from the weather op's cells, at the face's pixels (the same Gaussian blend as the
 * colours). Eastward and northward, m/s, row 0 at the south as the face's rows are.
 */
export interface AirFlow {
  width: number;
  height: number;
  east: Float32Array;
  north: Float32Array;
  /** Each row's mean eastward wind (the zonal jets), m/s. */
  rowEast: Float64Array;
  /** The storms alive when the face was made, where they stood (`flowFace` carries each as one piece). */
  storms?: FlowStorm[];
}

const toLin = (c: number) => {
  const x = c / 255;
  return x <= 0.04045 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
};
const toByte = (x: number) => {
  const v = Math.max(0, Math.min(1, x));
  return Math.round(255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055));
};

/** Per cell: each layer's reflectance (cover times its slab's) and colour, then the haze's, flattened for speed. */
export interface Stack {
  n: number;
  dirs: Float32Array;
  /** n × (layers + 1) × 4: reflectance, then r, g, b; the last slot is the haze. */
  vals: Float32Array;
  slots: number;
  /** Condensates with no colour on the sheet, drawn as white scatterers. */
  white: string[];
  /** n × 2: the wind of the highest deck that shows (east, north), m/s. */
  wind: Float32Array;
  /** Whether any cell has a wind. */
  windy: boolean;
  /** n × 3: the haze's absorption optical depth per channel, its depth times (1 - its colour) summed over aerosols. */
  absorb: Float32Array;
}

/**
 * Diffusivity factor: diffuse light crosses a thin layer as if it were this many times thicker (the two-stream mean
 * of 1/μ over a hemisphere; Elsasser 1942, as used by Goody and Yung 1989, section 6.3).
 */
const DIFFUSIVITY = 1.66;

/**
 * A layer that scatters (reflectance r) and absorbs (absorption depth a) over what is below it (reflectance below),
 * by the adding method: light crossing it both ways loses exp(-D a) each way.
 */
function addAbsorbing(r: number, a: number, below: number): number {
  const t = Math.max(0, 1 - r) * Math.exp(-DIFFUSIVITY * Math.max(0, a));
  return r + (t * t * below) / Math.max(1e-9, 1 - r * below);
}

/**
 * What a giant's deep air sends back from below its decks when its gases are not known: conservative Rayleigh
 * scatterers over thick deeper decks return all the light that reaches it. With its gases, `deepAirOf` (TA4).
 */
export const DEEP_AIR: Rgb = [1, 1, 1];

/**
 * Reflectance of a semi-infinite layer of scatterers whose single-scattering albedo is w, two-stream with isotropic
 * scattering (Bohren 1987): R = (1 - sqrt(1 - w)) / (1 + sqrt(1 - w)).
 */
export function deepReflectance(w: number): number {
  const s = Math.sqrt(Math.max(0, 1 - Math.min(1, w)));
  return (1 - s) / (1 + s);
}

/** The single-scattering albedo whose thick layer looks this colour (deepReflectance inverted): w = 1 - ((1 - R) / (1 + R))^2. */
export function albedoOfColour(r: number): number {
  const c = Math.min(1, Math.max(0, r));
  const q = (1 - c) / (1 + c);
  return 1 - q * q;
}

/** Wavelengths sampled across each channel (red, green, blue), nm: the deep air's colour is each channel's mean reflectance over them. */
export const CHANNEL_NM: readonly (readonly number[])[] = [
  [600, 620, 640, 660, 680, 700],
  [500, 520, 540, 560, 580, 600],
  [420, 440, 460, 480, 500],
];

/**
 * A giant's deep air (TA4): a clear gas, deep beyond seeing, that scatters by Rayleigh (cross-section per molecule
 * from the kernel's air optics) and absorbs by its gases' bands (absorption per molecule of air, the mole fraction
 * times each gas's cross-section, from the kernel's light through gas). Each wavelength's single-scattering albedo
 * is scattering over scattering plus absorption, its reflectance the semi-infinite two-stream one, and each channel
 * the mean over its wavelengths. Methane eating the red is what makes Uranus cyan and Neptune blue (Karkoschka 1994).
 */
export function deepAirOf(rayleigh_m2: readonly number[], absorb_m2: readonly number[]): Rgb {
  let i = 0;
  const out = CHANNEL_NM.map((band) => {
    let sum = 0;
    for (let j = 0; j < band.length; j++, i++) {
      const s = rayleigh_m2[i] ?? 0;
      const a = absorb_m2[i] ?? 0;
      sum += deepReflectance(s + a > 0 ? s / (s + a) : 1);
    }
    return sum / band.length;
  });
  return [out[0]!, out[1]!, out[2]!];
}

/**
 * The chromophore's optical depth comes at 1 µm (Sromovsky et al. 2017); its particles of a few tenths of a µm
 * dim green light more, by an Ångström exponent of about 1.5: (1 µm / 550 nm)^1.5. A stand-in until the data sheet
 * carries the chromophore's optics and TA2's particle block can size it (docs/fudges.md).
 */
const CHROMOPHORE_VISIBLE = (1000 / 550) ** 1.5;

export function stackOf(cells: readonly WeatherCell[], colourOf: ColourOf): Stack {
  const slots = Math.max(1, ...cells.map((c) => c.layers.length)) + 1;
  const vals = new Float32Array(cells.length * slots * 4);
  const dirs = new Float32Array(cells.length * 3);
  const wind = new Float32Array(cells.length * 2);
  const absorb = new Float32Array(cells.length * 3);
  let windy = false;
  const white = new Set<string>();
  const kept = new Map<string, Rgb | null>();
  const colour = (name: string): Rgb | null => {
    if (!kept.has(name)) kept.set(name, colourOf(name));
    return kept.get(name)!;
  };
  cells.forEach((c, i) => {
    const a = (c.lon_deg * Math.PI) / 180;
    const b = (c.lat_deg * Math.PI) / 180;
    dirs.set([Math.cos(b) * Math.cos(a), Math.cos(b) * Math.sin(a), Math.sin(b)], i * 3);
    c.layers.forEach((l, k) => {
      const o = (i * slots + k) * 4;
      const r = l.cloud_cover > 0 && l.optical_depth > 0 ? Math.min(1, l.cloud_cover) * slabReflectance(l.optical_depth, CLOUD_G) : 0;
      // A condensate the sheet cannot colour scatters all colours alike, as big droplets do (weather.ts).
      const rgb = l.condensate ? colour(l.condensate) : null;
      if (r > 0.01 && l.condensate && !rgb) white.add(l.condensate);
      vals.set([r, ...(rgb ?? [1, 1, 1])], o);
    });
    // The wind seen is the highest deck's that shows (the layers are bottom up), else the top layer's.
    const seen = [...c.layers].reverse().find((l) => l.cloud_cover > 0.05 && l.optical_depth > 0.1) ?? c.layers[c.layers.length - 1];
    const w = seen?.wind_m_s ?? [0, 0];
    wind[i * 2] = w[0] ?? 0;
    wind[i * 2 + 1] = w[1] ?? 0;
    if (w[0] || w[1]) windy = true;
    // Haze: all aerosols as one slab of their summed depth, tinted by the depth-weighted colour (as weather.ts).
    let tau = 0;
    const rgb = [0, 0, 0];
    for (const x of c.aerosols ?? []) {
      const t = Math.max(0, x.optical_depth) * (x.kind === "chromophore" ? CHROMOPHORE_VISIBLE : 1);
      tau += t;
      for (let ch = 0; ch < 3; ch++) {
        rgb[ch]! += t * (x.colour[ch] ?? 1);
        // An aerosol's colour is how a thick layer of it looks; the share of light one particle absorbs is read
        // back from that through the semi-infinite two-stream reflectance (albedoOfColour), so a red chromophore
        // or an orange haze takes the blue out of the light below it.
        absorb[i * 3 + ch]! += t * (1 - albedoOfColour(x.colour[ch] ?? 1));
      }
    }
    const o = (i * slots + slots - 1) * 4;
    vals.set([slabReflectance(tau, HAZE_G), ...(tau > 0 ? rgb.map((v) => v / tau) : [1, 1, 1])], o);
  });
  return { n: cells.length, dirs, vals, slots, white: [...white].sort(), wind, windy, absorb };
}

/** The cells' spacing on the unit sphere: the mean nearest-neighbour angle, estimated from the count (radians). */
const spacing = (n: number) => Math.sqrt((4 * Math.PI) / Math.max(1, n));

/**
 * Lay the air over a ground (linear RGB per pixel, or null for a giant's deep atmosphere, DEEP_AIR) and return the picture in the TrueTexture layout (true-look.ts
 * `equirect`). Each pixel blends the nearby cells with a Gaussian of half a cell, so the coarse tiles do not show
 * as blocks; each cell's own values are the kernel's.
 */
function paint(st: Stack, ground: ((o: number) => Rgb) | null, width: number, height: number, tint: Rgb | null, flow: AirFlow | null = null, clouds: Uint8Array | null = null, deep: Rgb = DEEP_AIR): TrueTexture {
  const data = new Uint8Array(width * height * 4);
  const sig = spacing(st.n) / 2;
  const inv = 1 / (2 * sig * sig);
  // Chord-squared cut-off at three sigma.
  const cut = 9 * sig * sig;
  const acc = new Float32Array(st.slots * 4);
  const ab = [0, 0, 0];
  let mr = 0;
  let mg = 0;
  let mb = 0;
  let mw = 0;
  for (let j = 0; j < height; j++) {
    const theta = Math.PI * (1 - (j + 0.5) / height);
    const s = Math.sin(theta);
    const z = Math.cos(theta);
    for (let i = 0; i < width; i++) {
      const u = (i + 0.5) / width;
      // The same direction as true-look.ts `equirect` for pixel (i, j).
      const x = -Math.cos(2 * Math.PI * u) * s;
      const y = -Math.sin(2 * Math.PI * u) * s;
      acc.fill(0);
      ab[0] = ab[1] = ab[2] = 0;
      let we = 0;
      let wn = 0;
      let ws = 0;
      let best = -1;
      let bestD = Infinity;
      for (let p = 0; p < st.n; p++) {
        const dx = x - st.dirs[p * 3]!;
        const dy = y - st.dirs[p * 3 + 1]!;
        const dz = z - st.dirs[p * 3 + 2]!;
        const d2 = dx * dx + dy * dy + dz * dz;
        if (d2 < bestD) (bestD = d2), (best = p);
        if (d2 > cut) continue;
        const g = Math.exp(-d2 * inv);
        ws += g;
        we += g * st.wind[p * 2]!;
        wn += g * st.wind[p * 2 + 1]!;
        const base = p * st.slots * 4;
        for (let q = 0; q < st.slots * 4; q++) acc[q]! += g * st.vals[base + q]!;
        for (let ch = 0; ch < 3; ch++) ab[ch]! += g * st.absorb[p * 3 + ch]!;
      }
      if (ws <= 0 && best >= 0) {
        ws = 1;
        we = st.wind[best * 2]!;
        wn = st.wind[best * 2 + 1]!;
        const base = best * st.slots * 4;
        for (let q = 0; q < st.slots * 4; q++) acc[q] = st.vals[base + q]!;
        for (let ch = 0; ch < 3; ch++) ab[ch] = st.absorb[best * 3 + ch]!;
      }
      const o = (j * width + i) * 4;
      if (flow) {
        flow.east[j * width + i] = we / ws;
        flow.north[j * width + i] = wn / ws;
      }
      const lin = composeAt(st, acc, ab, ws, ground ? ground(o) : [...deep], tint);
      const hazeAt = st.slots - 1;
      if (clouds) {
        // The air alone, over black: its own reflected light (premultiplied) and how much it covers.
        const c: Rgb = [0, 0, 0];
        for (let k = 0; k < st.slots; k++) {
          const r = acc[k * 4]! / ws;
          if (k === hazeAt) {
            // The decks' light below the haze loses its absorption both ways; the ground's is dimmed in the shader by
            // the layer's alpha alone (one alpha, so the haze's colour on the ground is not carried: docs/fudges.md).
            for (let ch = 0; ch < 3; ch++) c[ch] = addAbsorbing(r * (acc[k * 4 + 1 + ch]! / ws), ab[ch]! / ws, c[ch]!);
            continue;
          }
          if (!(r > 0)) continue;
          for (let ch = 0; ch < 3; ch++) c[ch] = addLayers(r * (acc[k * 4 + 1 + ch]! / ws), c[ch]!);
        }
        clouds[o] = toByte(c[0]);
        clouds[o + 1] = toByte(c[1]);
        clouds[o + 2] = toByte(c[2]);
        clouds[o + 3] = Math.round(255 * Math.min(1, (c[0] + c[1] + c[2]) / 3));
      }
      data[o] = toByte(lin[0]);
      data[o + 1] = toByte(lin[1]);
      data[o + 2] = toByte(lin[2]);
      data[o + 3] = 255;
      mr += data[o]! * s;
      mg += data[o + 1]! * s;
      mb += data[o + 2]! * s;
      mw += s;
    }
  }
  if (flow)
    for (let j = 0; j < height; j++) {
      let sum = 0;
      for (let i = 0; i < width; i++) sum += flow.east[j * width + i]!;
      flow.rowEast[j] = sum / width;
    }
  return { width, height, data, mean: [Math.round(mr / mw), Math.round(mg / mw), Math.round(mb / mw)] };
}

/**
 * One pixel's light from its blended cell values (`acc` the weighted sums of each slot's reflectance and colour,
 * `ab` of the haze's absorption, `ws` the weights' sum) over what lies below (`below`, linear RGB, overwritten):
 * bottom up, each layer on top of what is stacked so far; then the gas above the decks, then the haze.
 */
export function composeAt(st: Stack, acc: ArrayLike<number>, ab: ArrayLike<number>, ws: number, below: Rgb, tint: Rgb | null): Rgb {
  const lin = below;
  const hazeAt = st.slots - 1;
  for (let k = 0; k < st.slots; k++) {
    if (k === hazeAt && tint) for (let ch = 0; ch < 3; ch++) lin[ch]! *= tint[ch]!;
    const r = acc[k * 4]! / ws;
    if (k === hazeAt) {
      for (let ch = 0; ch < 3; ch++) lin[ch] = addAbsorbing(r * (acc[k * 4 + 1 + ch]! / ws), ab[ch]! / ws, lin[ch]!);
      continue;
    }
    if (!(r > 0)) continue;
    for (let ch = 0; ch < 3; ch++) lin[ch] = addLayers(r * (acc[k * 4 + 1 + ch]! / ws), lin[ch]!);
  }
  return lin;
}

const newFlow = (st: Stack, width: number, height: number): AirFlow | null =>
  st.windy ? { width, height, east: new Float32Array(width * height), north: new Float32Array(width * height), rowEast: new Float64Array(height) } : null;

function words(st: Stack, giant: boolean, deepKnown = false): string[] {
  const said: string[] = [];
  if (st.white.length) said.push(`${st.white.join(", ")} ${st.white.length > 1 ? "are" : "is"} drawn white: the data sheet has no colour for ${st.white.length > 1 ? "them" : "it"} yet.`);
  if (giant)
    said.push(
      deepKnown
        ? "Below its decks the deep air scatters blue and its gases' bands (methane) take out the red; the gas above the top deck takes out a little more."
        : "Below its decks a giant's deep air sends back all the light that reaches it: its gases are not known here.",
    );
  return said;
}

/**
 * A giant's face from its cloud decks alone (it has no ground to show): the weather op's field tiles, the bands,
 * storms and polar features in them, each deck coloured by its condensate.
 */
export function trueGiant(tiles: readonly WeatherTile[], colourOf: ColourOf, width = 128, height = 64, tint: Rgb | null = null, deep: Rgb | null = null): AirLook | null {
  const cells = tiles.flatMap((t) => t.cells);
  if (!cells.length) return null;
  const st = stackOf(cells, colourOf);
  const flow = newFlow(st, width, height);
  const tex = paint(st, null, width, height, tint, flow, null, deep ?? DEEP_AIR);
  const said = words(st, true, !!deep);
  return flow ? { tex, said, flow, decks: true } : { tex, said, decks: true };
}

/** A world's True surface with its clouds and haze laid over it, at the surface's own size. */
export function withAir(ground: TrueTexture, tiles: readonly WeatherTile[], colourOf: ColourOf, tint: Rgb | null = null): AirLook {
  const cells = tiles.flatMap((t) => t.cells);
  if (!cells.length) return { tex: ground, said: [] };
  const st = stackOf(cells, colourOf);
  const g = (o: number): Rgb => [toLin(ground.data[o]!), toLin(ground.data[o + 1]!), toLin(ground.data[o + 2]!)];
  const c = new Uint8Array(ground.width * ground.height * 4);
  const tex = paint(st, g, ground.width, ground.height, tint, null, c);
  return { tex, said: words(st, false), ground, clouds: { width: ground.width, height: ground.height, data: c, mean: [0, 0, 0] } };
}

// ------------------------------------------------------------------ the picture carried by its winds (VP-A1)

/** A storm as the weather op's `systems` give it, where it stood when the face was made. */
export interface FlowStorm {
  lat_deg: number;
  lon_deg: number;
  size_km: number;
}

const DEG = Math.PI / 180;
const wrap01 = (u: number) => u - Math.floor(u);

/** Bilinear read of a face at a latitude and longitude (degrees), wrapping in longitude. */
function sample(face: TrueTexture, lat: number, lon: number, out: number[]): void {
  const { width: w, height: h, data } = face;
  // Pixel (i, j) is at longitude 360(i + ½)/w - 180 and latitude 180(j + ½)/h - 90 (paint, true-look.ts `equirect`).
  const x = wrap01((lon + 180) / 360) * w - 0.5;
  const y = Math.min(h - 1, Math.max(0, ((Math.max(-90, Math.min(90, lat)) + 90) / 180) * h - 0.5));
  const i0 = Math.floor(x);
  const j0 = Math.floor(y);
  const fx = x - i0;
  const fy = y - j0;
  const j1 = Math.min(h - 1, j0 + 1);
  const a = ((i0 % w) + w) % w;
  const b = (a + 1) % w;
  for (let ch = 0; ch < 4; ch++) {
    const top = data[(j0 * w + a) * 4 + ch]! * (1 - fx) + data[(j0 * w + b) * 4 + ch]! * fx;
    const bot = data[(j1 * w + a) * 4 + ch]! * (1 - fx) + data[(j1 * w + b) * 4 + ch]! * fx;
    out[ch] = top * (1 - fy) + bot * fy;
  }
}

/** The flow's eastward and northward wind nearest a latitude and longitude (degrees). */
function windAt(f: AirFlow, lat: number, lon: number): [number, number] {
  const i = Math.floor(wrap01((lon + 180) / 360) * f.width) % f.width;
  const j = Math.min(f.height - 1, Math.max(0, Math.floor(((lat + 90) / 180) * f.height)));
  return [f.east[j * f.width + i]!, f.north[j * f.width + i]!];
}

const rowOf = (f: AirFlow, lat: number) => Math.min(f.height - 1, Math.max(0, Math.floor(((lat + 90) / 180) * f.height)));

/**
 * The face `dt_s` seconds after it was made, carried by its own winds (visual polish 5.4, VP-A1). Every row slides
 * east at its jet's speed, u / (a cos φ), so the equator laps the temperate bands as Jupiter's does. A storm moves as
 * one piece at its centre's speed (the weather block's drift, Δλ = u Δt / (a cos φ)) and turns at the speed its
 * own winds go round it, read from the cells' winds about its centre. The turning is drawn with two phases blended
 * (the usual flow-map method, a quarter turn a cycle), so the picture never winds itself into a smear. Grade: fact; every speed is the
 * kernel's. The caller reads the face again from the kernel when the day changes, as the storms are born and die.
 */
export function flowFace(face: TrueTexture, flow: AirFlow, opts: { radius_m: number; dt_s: number; storms?: readonly FlowStorm[] }): TrueTexture {
  // The flow's own storms unless the caller names others.
  const { width: w, height: h } = face;
  const a = opts.radius_m;
  const dt = opts.dt_s;
  const data = new Uint8Array(w * h * 4);
  if (!(a > 0) || !Number.isFinite(dt) || flow.width !== w || flow.height !== h) return { ...face, data: Uint8Array.from(face.data) };
  // A row's eastward turn rate, degrees of longitude a second.
  const rate = (lat: number) => {
    const c = Math.max(0.02, Math.cos(lat * DEG));
    return (flow.rowEast[rowOf(flow, lat)]! / (a * c)) / DEG;
  };
  // Each storm: its radius (the weather block's, half its size), its drift, and how fast its winds go round it.
  const storms = (opts.storms ?? flow.storms ?? [])
    .filter((s) => s.size_km > 0)
    .map((s) => {
      const R = 0.5 * s.size_km * 1000;
      const drift = rate(s.lat_deg);
      const u0 = flow.rowEast[rowOf(flow, s.lat_deg)]!;
      // Mean angular speed of the wind round the centre at one radius, anticlockwise positive, from eight points.
      let om = 0;
      for (let k = 0; k < 8; k++) {
        const t = (k * Math.PI) / 4;
        const x = R * Math.cos(t);
        const y = R * Math.sin(t);
        const lat = s.lat_deg + (y / a) / DEG;
        const lon = s.lon_deg + (x / (a * Math.max(0.02, Math.cos(s.lat_deg * DEG)))) / DEG;
        const [ve, vn] = windAt(flow, lat, lon);
        om += (x * vn - y * (ve - u0)) / (R * R) / 8;
      }
      return { ...s, R, drift, om, u0 };
    });
  const A = [0, 0, 0, 0];
  const B = [0, 0, 0, 0];
  const C = [0, 0, 0, 0];
  for (let j = 0; j < h; j++) {
    const lat = ((j + 0.5) / h) * 180 - 90;
    const back = rate(lat) * dt;
    const cl = Math.max(0.02, Math.cos(lat * DEG));
    for (let i = 0; i < w; i++) {
      const lon = ((i + 0.5) / w) * 360 - 180;
      sample(face, lat, lon - back, C);
      for (const s of storms) {
        // Where the storm is now, and this pixel's place from its centre in metres (east, north).
        const lonNow = s.lon_deg + s.drift * dt;
        const dl = (((lon - lonNow + 180) % 360) + 360) % 360 - 180;
        const x = dl * DEG * a * cl;
        const y = (lat - s.lat_deg) * DEG * a;
        const r2 = (x * x + y * y) / (s.R * s.R);
        if (r2 > 9) continue;
        const g = Math.exp(-r2);
        // Turning: the angle the winds carried this point round, with two phases half a turn apart blended.
        const read = (turn: number, out: number[]) => {
          const [ve, vn] = windAt(flow, s.lat_deg + (y / a) / DEG, s.lon_deg + (x / (a * cl)) / DEG);
          const rr = x * x + y * y + 0.01 * s.R * s.R;
          const th = -((x * vn - y * (ve - s.u0)) / rr) * turn;
          const cx = Math.cos(th);
          const sx = Math.sin(th);
          const xs = x * cx - y * sx;
          const ys = x * sx + y * cx;
          sample(face, s.lat_deg + (ys / a) / DEG, s.lon_deg + (xs / (a * cl)) / DEG, out);
        };
        if (Math.abs(s.om) > 1e-12) {
          // A quarter turn a cycle: short enough that the faster core is not wound into a spiral within one.
          const T = Math.PI / 2 / Math.abs(s.om);
          const pa = wrap01(dt / T);
          const pb = wrap01(dt / T + 0.5);
          read(pa * T, A);
          read(pb * T, B);
          const wa = 1 - Math.abs(2 * pa - 1);
          for (let ch = 0; ch < 4; ch++) A[ch] = wa * A[ch]! + (1 - wa) * B[ch]!;
        } else read(0, A);
        for (let ch = 0; ch < 4; ch++) C[ch] = (1 - g) * C[ch]! + g * A[ch]!;
      }
      const o = (j * w + i) * 4;
      for (let ch = 0; ch < 4; ch++) data[o + ch] = Math.round(C[ch]!);
    }
  }
  return { width: w, height: h, data, mean: face.mean };
}

// ------------------------------------------------------------------ colours from the sheet

/** A sheet substance's names and its condensed colour (linear RGB, `optical.condensed_colour_rgb`), as read_matter gives it. */
export interface SheetSubstance {
  id: string;
  names?: { physics?: string; plain?: string; story?: string };
  optical?: { condensed_colour_rgb?: { v: number[] } };
}

/** Words that name a phase, not a substance ("ammonia ice" is the substance ammonia, condensed). */
const PHASE_WORDS = /\b(ice|ices|frost|snow|droplets?|cloud|clouds|haze|liquid|vapour|vapor)\b/g;
/** One spelling for a name: lower case, British "sulph" and American "sulf" alike, phase words dropped. */
export const nameKey = (s: string) =>
  s
    .toLowerCase()
    .replace(/sulph/g, "sulf")
    .replace(PHASE_WORDS, " ")
    .replace(/\s+/g, " ")
    .trim();

/**
 * Match the weather op's condensate names to the sheet's substances by their names (any of physics, plain, story),
 * so a new substance on the sheet is coloured without a list here. Its `condensed_colour_rgb` is linear RGB. `droplets`
 * (from `readCloudRows`) colours condensates the sheet measures as droplets instead, by the weather op's own name.
 */
export function sheetColours(rows: readonly SheetSubstance[], droplets: Readonly<Record<string, Rgb>> = {}): ColourOf {
  const byName = new Map<string, Rgb>();
  for (const r of rows) {
    const v = r.optical?.condensed_colour_rgb?.v;
    if (!v || v.length < 3) continue;
    for (const n of [r.id, r.names?.physics, r.names?.plain, r.names?.story]) if (n) byName.set(nameKey(n), [v[0]!, v[1]!, v[2]!]);
  }
  for (const [n, c] of Object.entries(droplets)) byName.set(nameKey(n), c);
  return (condensate) => byName.get(nameKey(condensate)) ?? null;
}

/** A condensate whose droplets the sheet measures (cloud-rows.json): its row and droplet size. */
export interface CloudRow {
  row: string;
  radius_um: number;
  src?: string;
}

/** The condensate-to-row naming (cloud-rows.json). */
export const CLOUD_ROW_NAMES: Record<string, CloudRow> = CLOUD_ROWS.rows;

/**
 * A deck thick enough to stand for a semi-infinite cloud of it (a condensed colour is a thick layer's reflectance, as
 * the sheet's `condensed_colour_rgb` are; the deck's own optical depth is applied separately by `slabReflectance`).
 */
const THICK_DECK_TAU = 1e4;

let cloudRows: Promise<Record<string, Rgb>> | null = null;

/**
 * The colour of a thick deck of each cloud-rows.json condensate's droplets, linear RGB 0..1, from the row's measured
 * n + ik through the kernel (`light_through_gas` particles: anomalous diffraction per droplet and the adding method,
 * sse-light haze.rs) in sunlight, the eye adapted to it. Read once for the page; a reader without the op, or a row the
 * kernel refuses, leaves that condensate to the sheet's substances (or white, and the note says so).
 */
export function readCloudRows(k: Pick<FaceReader, "lightThroughGas">): Promise<Record<string, Rgb>> {
  cloudRows ??= (async () => {
    const out: Record<string, Rgb> = {};
    if (!k.lightThroughGas) return out;
    for (const [name, c] of Object.entries(CLOUD_ROW_NAMES)) {
      const r = await k.lightThroughGas([
        { address: `cloud:${name}`, t_k: constant("T_sun"), layers: [], particles: [{ material: c.row, radius_m: c.radius_um * 1e-6, optical_depth: THICK_DECK_TAU, at_nm: 550, asymmetry: CLOUD_G }], floor: 0 },
      ]);
      const v = r.ok ? (r.passages[0] as { reflected?: { reflectance_rgb?: number[] } } | undefined)?.reflected?.reflectance_rgb : undefined;
      if (v && v.length >= 3 && v.every((x) => Number.isFinite(x))) out[name] = [v[0]!, v[1]!, v[2]!];
    }
    return out;
  })();
  return cloudRows;
}

/** For tests: forget the page's copy. */
export function forgetCloudRows(): void {
  cloudRows = null;
}

/** The kernel door this needs (a ViewSession has both). */
export interface AirReader {
  weather(ask: Record<string, unknown>): Promise<{ ok: true; data: unknown } | { ok: false; message: string }>;
  readMatter(ask: { table?: string; id?: string }): Promise<{ ok: true; value: unknown } | { ok: false; message: string }>;
}

let sheetRows: Promise<SheetSubstance[]> | null = null;

/** The sheet's substances, read once for the page (cheap: the sheet is in the kernel's memory). */
export function readSheetSubstances(k: AirReader): Promise<SheetSubstance[]> {
  sheetRows ??= (async () => {
    const l = await k.readMatter({});
    const ids = l.ok ? ((l.value as { substances?: string[] }).substances ?? []) : [];
    const rows = await Promise.all(
      ids.map(async (id) => {
        const r = await k.readMatter({ table: "substances", id });
        return r.ok ? ((r.value as { row?: SheetSubstance }).row ?? null) : null;
      }),
    );
    return rows.filter((r): r is SheetSubstance => !!r);
  })();
  return sheetRows;
}

/** Cells per face edge for the system view: enough for a giant's bands (Jupiter's are a few degrees wide). */
export const AIR_N = 24;

/** A world's weather field, all six faces at level 0, on a day (the Body globe asks the same way). */
export async function readAirTiles(k: AirReader, system: string, body: string, opts: { day?: number; at_s?: number; n?: number } = {}): Promise<{ ok: true; tiles: WeatherTile[] } | { ok: false; message: string }> {
  const tiles: WeatherTile[] = [];
  const when = { ...(opts.at_s !== undefined ? { at_s: opts.at_s } : {}), day: opts.day ?? 0 };
  for (const face of [0, 1, 2, 3, 4, 5]) {
    const r = await k.weather({ want: "field", system, body, ...when, tile: [face, 0, 0, 0], n: opts.n ?? AIR_N });
    if (!r.ok) return { ok: false, message: r.message };
    tiles.push(r.data as WeatherTile);
  }
  return { ok: true, tiles };
}

// ------------------------------------------------------------------ one reader for every True view

/** The kernel doors a True face needs (a ViewSession has them all). */
export interface FaceReader extends AirReader {
  surfaceTiles(system: string, body: string): Promise<{ ok: true; tiles: SurfaceTile[]; refused: unknown[]; record?: unknown; kind?: string } | { ok: false; message: string }>;
  campaignNow(): Promise<{ ok: true; at_s?: bigint } | { ok: false; message: string }>;
  /** For the air's rim (VP-A3); a reader without them draws no air shell. */
  resolvedSystem?(system: string): Promise<{ ok: true; result: { surfaces: { address: string }[]; facts: Record<string, Record<string, Json>> } } | { ok: false; message: string }>;
  readAirOptics?(airs: Json[], wavelengths_m?: number[]): Promise<{ ok: true; airs: Json[] } | { ok: false; message: string }>;
  /** For a giant's deep air and the gas over its decks (TA4); a reader without it draws them white. */
  lightThroughGas?(passages: Json[]): Promise<{ ok: true; passages: Json[] } | { ok: false; message: string }>;
}

/**
 * A world's air in its own radius for the rim and terminator in True (VP-A3): its surface pressure, temperature,
 * gravity and gases from `resolved_system`, its optics from `read_air_optics`, as the Body globe reads them. A string
 * says why there is none (no air, or a reader without those doors).
 */
export async function readTrueAir(k: FaceReader, system: string, body: string): Promise<AirShape | string> {
  if (!k.resolvedSystem || !k.readAirOptics) return "this view cannot read the air";
  const rs = await k.resolvedSystem(system);
  if (!rs.ok) return rs.message;
  const surf = rs.result.surfaces.find((x) => x.address === body) as Parameters<typeof airFactsOf>[1];
  const facts = airFactsOf(rs.result.facts[body], surf);
  const req = facts ? airRequest(body, facts) : null;
  if (!facts || !req) return "no air";
  const ao = await k.readAirOptics([req.air]);
  if (!ao.ok || !ao.airs[0]) return ao.ok ? "no optics" : ao.message;
  return airShapeOf(airFromOptics(ao.airs[0] as Parameters<typeof airFromOptics>[0], req), facts.radius_m);
}

/**
 * A giant's deep air and the gas over its decks, from the kernel (TA4): Rayleigh cross-sections from `read_air_optics`
 * and its absorbing gases from `light_through_gas`, at CHANNEL_NM. The deep air is `deepAirOf`; the gas over the decks
 * is `gasAbove` from the world's reference level (its pressure fact) up, lit by the system's star. Null when the reader or the world's air is missing.
 */
export async function readGiantGas(k: FaceReader, system: string, body: string): Promise<{ deep: Rgb; tint: Rgb | null } | null> {
  if (!k.resolvedSystem || !k.readAirOptics || !k.lightThroughGas) return null;
  const rs = await k.resolvedSystem(system);
  if (!rs.ok) return null;
  const surf = rs.result.surfaces.find((x) => x.address === body) as Parameters<typeof airFactsOf>[1];
  const f = rs.result.facts[body];
  // A giant has no surface pressure: its radius and air are quoted at the 1 bar level, the reference used here.
  const facts = airFactsOf(f, surf) ?? airFactsOf({ ...f, "atmosphere.surface_pressure": sheetUnitFactor("bar") }, surf);
  const req = facts ? airRequest(body, facts) : null;
  if (!facts || !req) return null;
  const wl = CHANNEL_NM.flat().map((nm) => nm * 1e-9);
  const ao = await k.readAirOptics([req.air], wl);
  const o = ao.ok ? (ao.airs[0] as { at?: { cross_section_m2: number }[]; mean_molecular_mass_kg?: number } | undefined) : undefined;
  const at = o?.at;
  const mean = o?.mean_molecular_mass_kg ?? 0;
  if (!at || at.length !== wl.length) return null;
  // The light: the system's star where the kernel gives its temperature, else the gas's own (only ratios are used).
  const star = Object.values(rs.result.facts).find((x) => typeof x["star.effective_temperature"] === "number")?.["star.effective_temperature"] as number | undefined;
  const t_k = star ?? facts.temperature_k;
  // Mole fractions by the kernel's gas ids (the composition is by name: CH4 is ch4).
  const byId: Record<string, number> = {};
  for (const [name, x] of Object.entries(facts.composition)) byId[KERNEL_GASES[name] ?? name.toLowerCase()] = x;
  const gases = ABSORBING_GASES.filter((id) => (byId[id] ?? 0) > 0).map((id) => ({ gas: id, number_density_m3: byId[id]! }));
  if (!gases.length) return { deep: deepAirOf(at.map((x) => x.cross_section_m2), []), tint: null };
  // Per molecule of air: each absorbing gas at its mole fraction over one metre of one molecule per cubic metre.
  const unit = { address: body, t_k, wavelengths_m: wl, layers: [{ t_k: facts.temperature_k, path_m: 1, gases }], path_factor: 1 };
  const above = gasAbove(body, { gases: Object.fromEntries(gases.map((g) => [g.gas, g.number_density_m3])), pressure_bar: facts.pressure_pa / sheetUnitFactor("bar"), t_surface_k: facts.temperature_k, gravity_m_s2: facts.gravity_m_s2, mean_molar_kg: mean * constant("N_A") });
  const r = await k.lightThroughGas([unit, ...(above ? [{ ...above, t_k }] : [])]);
  if (!r.ok) return null;
  const tau = (r.passages[0] as { passed?: { optical_depth?: number[] } } | undefined)?.passed?.optical_depth;
  if (!tau || tau.length !== wl.length) return null;
  const tint = above && r.passages[1] ? gasTint(r.passages[1] as unknown as GasColours) : null;
  return { deep: deepAirOf(at.map((x) => x.cross_section_m2), tau), tint };
}

/**
 * A world's True face, read from the kernel once: its ground from the level-0 tiles (true-look.ts), today's clouds
 * and haze laid over it; a world with no ground (a giant) is its cloud decks alone. A string says why there is none.
 * The system view and the size comparison both read faces through here, so a world looks the same in each; `storms`
 * also reads a giant's storms for a view that carries its bands (the size comparison).
 */
export async function readTrueFace(k: FaceReader, system: string, body: string, opts: { storms?: boolean; detail?: boolean; detail_px?: number; stir?: boolean } = {}): Promise<AirLook | string> {
  const s = await k.surfaceTiles(system, body);
  // A giant's tiles are its cloud deck's relief, not a ground: its face is its decks alone (trueGiant).
  const giantDeck = s.ok && s.kind === "cloud-deck";
  // What stains its ground (TRUE4): the kernel's stains, each hued by its sheet row's optics.
  const stains = s.ok && !giantDeck ? await readStains(k, s.record) : [];
  // Bare ground in its measured colour from the data sheet (ground-optics.ts, VP3 part 5), then stained.
  const ground =
    s.ok && !giantDeck ? trueSurface(withGround(s.tiles, await readGroundOptics(k)), 128, 64, stains) : null;
  const groundWhy = s.ok ? String((s.refused[0] as { why?: string } | undefined)?.why ?? "no surface tiles") : s.message;
  const now = await k.campaignNow();
  const at_s = now.ok && now.at_s !== undefined ? Number(now.at_s) : undefined;
  const air = await readAirTiles(k, system, body, { ...(at_s !== undefined ? { at_s, day: at_s / constant("day") } : {}) });
  const gas = ground ? await readTrueAir(k, system, body) : null;
  const withGas = <T extends AirLook>(l: T): T => (gas && typeof gas !== "string" ? { ...l, air: gas } : l);
  if (!air.ok) return ground ? withGas({ tex: ground, ground, said: [`Its clouds are not shown: ${air.message}`] }) : groundWhy;
  const colours = sheetColours(await readSheetSubstances(k), await readCloudRows(k));
  if (ground) return withGas(withAir(ground, air.tiles, colours));
  const gg = await readGiantGas(k, system, body);
  const giant = trueGiant(air.tiles, colours, 128, 64, gg?.tint ?? null, gg?.deep ?? null);
  if (!giant) return groundWhy;
  const day = at_s !== undefined ? at_s / constant("day") : 0;
  // Its storms and poles read closer, turned by their winds, with curls at the band edges (giant-detail.ts).
  const detail = opts.detail === false ? null : await readGiantDetail(k, system, body, { ...(at_s !== undefined ? { at_s } : {}), day }, opts.detail_px);
  const look: AirLook = detail ? (() => {
    const d = detailGiant(giant.tex, giant.flow, detail, colours, { deep: gg?.deep ?? null, tint: gg?.tint ?? null, ...(opts.detail_px ? { width: opts.detail_px } : {}) });
    return { ...giant, tex: d.tex, said: [...giant.said, ...d.said], ...(d.flow ? { flow: d.flow } : {}) };
  })() : giant;
  // The stirred still (WS5): the same face with no winding or curls, carried back along the kernel's winds on a GPU
  // of its own; the view shows the face above until it comes. Only where a page can make a GPU (never in node).
  if (detail && opts.stir !== false && typeof document !== "undefined") {
    const width = opts.detail_px;
    look.stirred = (async () => {
      const key = `${system}|${body}|${at_s ?? "now"}|${width ?? "-"}`;
      const { keptStill, stirFace } = await import("./stirred-runner.ts");
      const have = keptStill(key);
      if (have) return have;
      const ct = await k.weather({ want: "carry", system, body, ...(at_s !== undefined ? { at_s } : {}), day });
      if (!ct.ok) return null;
      const src = detailGiant(giant.tex, giant.flow, detail, colours, { deep: gg?.deep ?? null, tint: gg?.tint ?? null, source: true, ...(width ? { width } : {}) });
      return stirFace(key, src.tex, ct.data as CarryTable);
    })().catch(() => null);
  }
  // Only a view that carries the bands asks for the storms (the size comparison); the rest skip the kernel call.
  if (look.flow && opts.storms) {
    // The storms, so each is carried as one piece as the bands slide (VP-A1).
    const live = detail?.systems ?? (await (async () => {
      const sy = await k.weather({ want: "systems", system, body, ...(at_s !== undefined ? { at_s } : {}), day });
      return sy.ok ? (sy.data as WeatherSystem[]) : [];
    })());
    look.flow.storms = liveSystems({ day, systems: [...live] } as WeatherMap).map((x) => ({ ...stormAt(x, day), size_km: x.size_km }));
  }
  return look;
}

// ------------------------------------------------------------------ the gas above the decks

/** The weather climate's conditions this needs: the world's gases (mole fractions), its reference pressure and temperature. */
export interface AirConditions {
  gases?: Record<string, number>;
  pressure_bar?: number;
  t_surface_k?: number;
  gravity_m_s2?: number;
  mean_molar_kg?: number;
}

/** Gases the sheet gives visible absorption rows for (ND2): only these tint the light. Read by id, so a new row only needs its id here. */
export const ABSORBING_GASES = ["ch4", "na", "k"] as const;

/**
 * The gas over the decks as one passage for `light_through_gas` (ND2), as the light thread set it out: one layer from
 * the reference level up, each absorbing gas at n = p x / (k T) over one scale height H = k T / (m g), so its column is
 * x p / (m g); the light goes down and back up (path factor 2). Null when the world has no absorbing gas or the
 * numbers to place it. Boltzmann's and Avogadro's constants and the bar are the sheet's.
 */
export function gasAbove(address: string, c: AirConditions): Record<string, unknown> | null {
  const k = { boltzmann: constant("k_B"), avogadro: constant("N_A") };
  const p = (c.pressure_bar ?? 0) * sheetUnitFactor("bar");
  const t = c.t_surface_k ?? 0;
  const g = c.gravity_m_s2 ?? 0;
  const m = (c.mean_molar_kg ?? 0) / k.avogadro;
  if (!(p > 0 && t > 0 && g > 0 && m > 0)) return null;
  const h = (k.boltzmann * t) / (m * g);
  const gases = ABSORBING_GASES.filter((id) => (c.gases?.[id] ?? 0) > 0).map((id) => ({ gas: id, number_density_m3: (p * c.gases![id]!) / (k.boltzmann * t) }));
  if (!gases.length) return null;
  return { address, layers: [{ t_k: t, path_m: h, gases }], path_factor: 2 };
}

/** A passage's result as the light thread's op returns it: colours in and out (linear sRGB scaled to its brightest, and XYZ). */
export interface GasColours {
  colour_in: { xyz: number[]; linear_srgb: number[] };
  colour_out: { xyz: number[]; linear_srgb: number[] };
}

/**
 * What the gas does to light, as a factor per channel (linear RGB): the colour's change from the op's colours in and
 * out, scaled so its brightness is the eye's Y out over Y in. Rec. 709 luminance weights, as weather.ts uses.
 */
export function gasTint(r: GasColours): Rgb {
  const yIn = r.colour_in.xyz[1] ?? 0;
  const yOut = r.colour_out.xyz[1] ?? 0;
  if (!(yIn > 0)) return [1, 1, 1];
  const t = [0, 1, 2].map((ch) => {
    const a = r.colour_in.linear_srgb[ch] ?? 0;
    return a > 0 ? (r.colour_out.linear_srgb[ch] ?? 0) / a : 0;
  });
  const lum = 0.2126 * t[0]! + 0.7152 * t[1]! + 0.0722 * t[2]!;
  const s = lum > 0 ? yOut / yIn / lum : 0;
  return [t[0]! * s, t[1]! * s, t[2]! * s];
}

export const AIR_REFS = [
  { what: "Reflectance of a scattering layer, two-stream, and layers combined by the adding method", cite: "Bohren 1987, Am. J. Phys. 55, 524", url: "https://doi.org/10.1119/1.15109" },
] as const;
