// Air in True at system scale (audit items R1 to R3 and R6, 24 Nov 2026): a world's clouds and haze from the kernel's
// `weather` op, laid over its True surface, and a giant's face made of nothing but its cloud decks. Nothing is
// invented here. The decks (which condensate, how much cover, what optical depth), the bands between the jets, the
// storms and the polar cyclones are all the weather block's, cell by cell; each layer's colour is its condensate's on
// the master data sheet, read by the caller; a haze's colour is the one the weather op sends. This file only stacks
// the layers by the adding method (Bohren 1987), as weather.ts does on the Body globe.
import { constant, sheetUnitFactor } from "../../ui-contract/src/index.ts";
import type { SurfaceTile } from "../../ui-contract/src/index.ts";
import { trueSurface, type TrueTexture } from "./true-look.ts";
import { addLayers, CLOUD_G, HAZE_G, slabReflectance, type WeatherCell, type WeatherTile } from "./weather.ts";

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
interface Stack {
  n: number;
  dirs: Float32Array;
  /** n × (layers + 1) × 4: reflectance, then r, g, b; the last slot is the haze. */
  vals: Float32Array;
  slots: number;
  /** Condensates with no colour on the sheet, drawn as white scatterers. */
  white: string[];
}

function stackOf(cells: readonly WeatherCell[], colourOf: ColourOf): Stack {
  const slots = Math.max(1, ...cells.map((c) => c.layers.length)) + 1;
  const vals = new Float32Array(cells.length * slots * 4);
  const dirs = new Float32Array(cells.length * 3);
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
    // Haze: all aerosols as one slab of their summed depth, tinted by the depth-weighted colour (as weather.ts).
    let tau = 0;
    const rgb = [0, 0, 0];
    for (const x of c.aerosols ?? []) {
      const t = Math.max(0, x.optical_depth);
      tau += t;
      for (let ch = 0; ch < 3; ch++) rgb[ch]! += t * (x.colour[ch] ?? 1);
    }
    const o = (i * slots + slots - 1) * 4;
    vals.set([slabReflectance(tau, HAZE_G), ...(tau > 0 ? rgb.map((v) => v / tau) : [1, 1, 1])], o);
  });
  return { n: cells.length, dirs, vals, slots, white: [...white].sort() };
}

/** The cells' spacing on the unit sphere: the mean nearest-neighbour angle, estimated from the count (radians). */
const spacing = (n: number) => Math.sqrt((4 * Math.PI) / Math.max(1, n));

/**
 * Lay the air over a ground (linear RGB per pixel, or null for a giant's deep atmosphere, which sends back almost
 * none of the light that gets below its decks) and return the picture in the TrueTexture layout (true-look.ts
 * `equirect`). Each pixel blends the nearby cells with a Gaussian of half a cell, so the coarse tiles do not show
 * as blocks; each cell's own values are the kernel's.
 */
function paint(st: Stack, ground: ((o: number) => Rgb) | null, width: number, height: number, tint: Rgb | null): TrueTexture {
  const data = new Uint8Array(width * height * 4);
  const sig = spacing(st.n) / 2;
  const inv = 1 / (2 * sig * sig);
  // Chord-squared cut-off at three sigma.
  const cut = 9 * sig * sig;
  const acc = new Float32Array(st.slots * 4);
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
        const base = p * st.slots * 4;
        for (let q = 0; q < st.slots * 4; q++) acc[q]! += g * st.vals[base + q]!;
      }
      if (ws <= 0 && best >= 0) {
        ws = 1;
        const base = best * st.slots * 4;
        for (let q = 0; q < st.slots * 4; q++) acc[q] = st.vals[base + q]!;
      }
      const o = (j * width + i) * 4;
      const lin: Rgb = ground ? ground(o) : [0, 0, 0];
      // Bottom up: each layer goes on top of what is stacked so far; then the gas above the decks, then the haze.
      for (let k = 0; k < st.slots; k++) {
        if (k === st.slots - 1 && tint) for (let ch = 0; ch < 3; ch++) lin[ch]! *= tint[ch]!;
        const r = acc[k * 4]! / ws;
        if (!(r > 0)) continue;
        for (let ch = 0; ch < 3; ch++) lin[ch] = addLayers(r * (acc[k * 4 + 1 + ch]! / ws), lin[ch]!);
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
  return { width, height, data, mean: [Math.round(mr / mw), Math.round(mg / mw), Math.round(mb / mw)] };
}

function words(st: Stack, giant: boolean): string[] {
  const said: string[] = [];
  if (st.white.length) said.push(`${st.white.join(", ")} ${st.white.length > 1 ? "are" : "is"} drawn white: the data sheet has no colour for ${st.white.length > 1 ? "them" : "it"} yet.`);
  if (giant) said.push("Below its decks a giant is drawn dark: the deep air's own scattering and absorption are not added yet.");
  return said;
}

/**
 * A giant's face from its cloud decks alone (it has no ground to show): the weather op's field tiles, the bands,
 * storms and polar features in them, each deck coloured by its condensate.
 */
export function trueGiant(tiles: readonly WeatherTile[], colourOf: ColourOf, width = 128, height = 64, tint: Rgb | null = null): AirLook | null {
  const cells = tiles.flatMap((t) => t.cells);
  if (!cells.length) return null;
  const st = stackOf(cells, colourOf);
  return { tex: paint(st, null, width, height, tint), said: words(st, true) };
}

/** A world's True surface with its clouds and haze laid over it, at the surface's own size. */
export function withAir(ground: TrueTexture, tiles: readonly WeatherTile[], colourOf: ColourOf, tint: Rgb | null = null): AirLook {
  const cells = tiles.flatMap((t) => t.cells);
  if (!cells.length) return { tex: ground, said: [] };
  const st = stackOf(cells, colourOf);
  const g = (o: number): Rgb => [toLin(ground.data[o]!), toLin(ground.data[o + 1]!), toLin(ground.data[o + 2]!)];
  return { tex: paint(st, g, ground.width, ground.height, tint), said: words(st, false) };
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
 * so a new substance on the sheet is coloured without a list here. Its `condensed_colour_rgb` is linear RGB.
 */
export function sheetColours(rows: readonly SheetSubstance[]): ColourOf {
  const byName = new Map<string, Rgb>();
  for (const r of rows) {
    const v = r.optical?.condensed_colour_rgb?.v;
    if (!v || v.length < 3) continue;
    for (const n of [r.id, r.names?.physics, r.names?.plain, r.names?.story]) if (n) byName.set(nameKey(n), [v[0]!, v[1]!, v[2]!]);
  }
  return (condensate) => byName.get(nameKey(condensate)) ?? null;
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
  surfaceTiles(system: string, body: string): Promise<{ ok: true; tiles: SurfaceTile[]; refused: unknown[] } | { ok: false; message: string }>;
  campaignNow(): Promise<{ ok: true; at_s?: bigint } | { ok: false; message: string }>;
}

/**
 * A world's True face, read from the kernel once: its ground from the level-0 tiles (true-look.ts), today's clouds
 * and haze laid over it; a world with no ground (a giant) is its cloud decks alone. A string says why there is none.
 * The system view and the size comparison both read faces through here, so a world looks the same in each.
 */
export async function readTrueFace(k: FaceReader, system: string, body: string): Promise<AirLook | string> {
  const s = await k.surfaceTiles(system, body);
  const ground = s.ok ? trueSurface(s.tiles) : null;
  const groundWhy = s.ok ? String((s.refused[0] as { why?: string } | undefined)?.why ?? "no surface tiles") : s.message;
  const now = await k.campaignNow();
  const at_s = now.ok && now.at_s !== undefined ? Number(now.at_s) : undefined;
  const air = await readAirTiles(k, system, body, { ...(at_s !== undefined ? { at_s, day: at_s / constant("day") } : {}) });
  if (!air.ok) return ground ? { tex: ground, said: [`Its clouds are not shown: ${air.message}`] } : groundWhy;
  const colours = sheetColours(await readSheetSubstances(k));
  if (ground) return withAir(ground, air.tiles, colours);
  return trueGiant(air.tiles, colours) ?? groundWhy;
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
