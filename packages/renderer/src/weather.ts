// Weather on today's Body globe (R2b part 1): clouds and haze from the kernel's
// `weather` op. The kernel's weather block gives, per coarse atlas cell, three
// height layers (cover, cloud type, condensate, optical depth, wind) and its
// aerosols; `want: "systems"` gives the named storms. This file reads the six
// face tiles, finds the cell under any point, and lays the clouds over the
// ground by the adding method for scattering layers. Nothing is simulated here:
// cover, depth and storms are the weather block's.

import { dirIndex, nearestCell, type V3 } from "./timelapse.ts";

const gkey = (i: number, j: number, k: number) => (i + 512) * 1048576 + (j + 512) * 1024 + (k + 512);

/** One height layer of one cell, as the weather contract sends it. */
export interface WeatherLayer {
  cloud_cover: number;
  cloud_type: string;
  condensate: string;
  optical_depth: number;
  wind_m_s: [number, number];
  turbulence: number;
}

export interface WeatherCell {
  lat_deg: number;
  lon_deg: number;
  /** Bottom up: low, middle, high. */
  layers: WeatherLayer[];
  precip_kind: string;
  precip_mm_day: number;
  temp_offset_k: number;
  lightning_km2_yr: number;
  aerosols: { kind: string; optical_depth: number; colour: [number, number, number] }[];
}

export interface WeatherTile {
  key: { face: number; level: number; x: number; y: number } | Record<string, unknown>;
  n: number;
  cells: WeatherCell[];
}

export interface WeatherSystem {
  id: string;
  kind: string;
  lat_deg: number;
  lon_deg: number;
  /** Positions along its life, oldest first: [day, lat, lon]. */
  track?: [number, number, number][];
  size_km: number;
  strength: number;
  born: number;
  ends: number;
  why: string;
}

export interface WeatherMap {
  day: number;
  cells: WeatherCell[];
  dirs: Float32Array;
  index: { cell: number; grid: Map<number, number[]> };
  /** Per cell: the clouds' reflectance (all layers, cover-weighted) and the haze's. */
  cloudR: Float32Array;
  hazeR: Float32Array;
  hazeRgb: Float32Array;
  systems: WeatherSystem[];
  /** Which block made it (`sse.weather.rules` …) and how (`drawn` or `calculated`). */
  block: string;
  evidence: string;
}

/** Asymmetry parameter of cloud droplets in visible light (Hansen and Travis 1974: about 0.85 for water clouds). */
export const CLOUD_G = 0.85;
/** Asymmetry parameter of haze and dust particles (smaller, less forward-scattering; about 0.7, Hansen and Travis 1974). */
export const HAZE_G = 0.7;

export const WEATHER_REFS = [
  { what: "Reflectance of a non-absorbing scattering layer, two-stream: R = (1-g)τ / (2 + (1-g)τ); layers and ground combine by the adding method", cite: "Bohren 1987, Am. J. Phys. 55, 524", url: "https://doi.org/10.1119/1.15109" },
  { what: "A wet day is one with 1 mm or more of precipitation", cite: "ETCCDI climate change indices (Zhang et al. 2011, WIREs Clim. Change 2, 851)", url: "https://doi.org/10.1002/wcc.147" },
  { what: "Asymmetry parameter of cloud droplets about 0.85 in visible light", cite: "Hansen and Travis 1974, Space Sci. Rev. 16, 527", url: "https://doi.org/10.1007/BF00168069" },
] as const;

/** Two-stream reflectance of a conservative scattering slab of optical depth τ (Bohren 1987). */
export function slabReflectance(tau: number, g: number): number {
  const t = Math.max(0, tau) * (1 - g);
  return t / (2 + t);
}

/** Adding two non-absorbing layers: the upper's reflectance R1 over the lower's R2 (light bouncing between them included). */
export function addLayers(r1: number, r2: number): number {
  const t1 = 1 - r1;
  return r1 + (t1 * t1 * r2) / Math.max(1e-9, 1 - r1 * r2);
}

/** A cell's clouds as one reflectance: each layer's slab reflectance weighted by its cover, stacked high over middle over low. */
export function cloudReflectance(c: WeatherCell): number {
  let r = 0;
  for (const l of c.layers) {
    if (!(l.cloud_cover > 0) || !(l.optical_depth > 0)) continue;
    // Bottom up in the contract, so each later layer goes on top of what is stacked so far.
    r = addLayers(Math.min(1, l.cloud_cover) * slabReflectance(l.optical_depth, CLOUD_G), r);
  }
  return r;
}

const dirOfLatLon = (lat_deg: number, lon_deg: number): V3 => {
  const a = (lon_deg * Math.PI) / 180;
  const b = (lat_deg * Math.PI) / 180;
  return [Math.cos(b) * Math.cos(a), Math.cos(b) * Math.sin(a), Math.sin(b)];
};

/** Read the six face tiles (and the storm list) from the kernel into one map with a direction lookup. */
export function readWeather(tiles: readonly WeatherTile[], systems: readonly WeatherSystem[] = [], meta: { day?: number; block?: string; evidence?: string } = {}): WeatherMap {
  const cells = tiles.flatMap((t) => t.cells);
  if (!cells.length) throw new Error("weather: no cells");
  const dirs = new Float32Array(cells.length * 3);
  const cloudR = new Float32Array(cells.length);
  const hazeR = new Float32Array(cells.length);
  const hazeRgb = new Float32Array(cells.length * 3);
  cells.forEach((c, i) => {
    dirs.set(dirOfLatLon(c.lat_deg, c.lon_deg), i * 3);
    cloudR[i] = cloudReflectance(c);
    // Haze: all aerosols as one slab of their summed depth, tinted by the depth-weighted colour.
    let tau = 0;
    const rgb = [0, 0, 0];
    for (const a of c.aerosols ?? []) {
      const t = Math.max(0, a.optical_depth);
      tau += t;
      for (let k = 0; k < 3; k++) rgb[k]! += t * (a.colour[k] ?? 1);
    }
    hazeR[i] = slabReflectance(tau, HAZE_G);
    for (let k = 0; k < 3; k++) hazeRgb[i * 3 + k] = tau > 0 ? rgb[k]! / tau : 1;
  });
  return { day: meta.day ?? 0, cells, dirs, index: dirIndex(dirs, cells.length), cloudR, hazeR, hazeRgb, systems: [...systems], block: meta.block ?? "", evidence: meta.evidence ?? "" };
}

export function weatherCellAt(w: WeatherMap, d: V3): number {
  return nearestCell(w.index, w.dirs, d[0], d[1], d[2]);
}

/**
 * Cloud and haze reflectance at a direction, blended from the nearby cells with a Gaussian of about half a cell,
 * so the coarse weather tiles do not show as blocks. Presentation only: each cell's own values are the kernel's.
 */
export function blendAt(w: WeatherMap, d: V3): { cloud: number; haze: number; rgb: [number, number, number] } {
  const { cell, grid } = w.index;
  // Bucket width is 1.5 cell spacings (dirIndex); the kernel width is about half a spacing.
  const sig = cell / 1.5 / 2;
  const inv = 1 / (2 * sig * sig);
  const i0 = Math.floor(d[0] / cell);
  const j0 = Math.floor(d[1] / cell);
  const k0 = Math.floor(d[2] / cell);
  let ws = 0;
  let cloud = 0;
  let haze = 0;
  const rgb: [number, number, number] = [0, 0, 0];
  for (let i = i0 - 1; i <= i0 + 1; i++)
    for (let j = j0 - 1; j <= j0 + 1; j++)
      for (let k = k0 - 1; k <= k0 + 1; k++) {
        const b = grid.get(gkey(i, j, k));
        if (!b) continue;
        for (const p of b) {
          const dx = d[0] - w.dirs[p * 3]!;
          const dy = d[1] - w.dirs[p * 3 + 1]!;
          const dz = d[2] - w.dirs[p * 3 + 2]!;
          const g = Math.exp(-(dx * dx + dy * dy + dz * dz) * inv);
          if (g < 1e-4) continue;
          ws += g;
          cloud += g * w.cloudR[p]!;
          haze += g * w.hazeR[p]!;
          for (let ch = 0; ch < 3; ch++) rgb[ch]! += g * w.hazeRgb[p * 3 + ch]!;
        }
      }
  if (ws <= 0) {
    const c = weatherCellAt(w, d);
    if (c < 0) return { cloud: 0, haze: 0, rgb: [1, 1, 1] };
    return { cloud: w.cloudR[c]!, haze: w.hazeR[c]!, rgb: [w.hazeRgb[c * 3]!, w.hazeRgb[c * 3 + 1]!, w.hazeRgb[c * 3 + 2]!] };
  }
  return { cloud: cloud / ws, haze: haze / ws, rgb: [rgb[0] / ws, rgb[1] / ws, rgb[2] / ws] };
}

const toLin = (c: number) => {
  const x = c / 255;
  return x <= 0.04045 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
};
const toSrgb = (x: number) => 255 * (x <= 0.0031308 ? 12.92 * x : 1.055 * Math.pow(Math.max(0, x), 1 / 2.4) - 0.055);

/**
 * The prepared samples with the weather laid over them: both the palette colour
 * (what the chart recipes draw) and the physical albedo (what True lighting
 * reads) become the reflectance of cloud and haze over that ground.
 */
export function withWeather<P extends { faces: ({ s: number; rgb: Uint8ClampedArray; normal: Float32Array; albedo: Float32Array } | null)[] }>(p: P, w: WeatherMap, dirOf: (face: number, u: number, v: number) => V3): P {
  const faces = p.faces.map((f, face) => {
    if (!f) return null;
    const s = f.s;
    const rgb = new Uint8ClampedArray(f.rgb.length);
    const albedo = new Float32Array(f.albedo.length);
    const span = 2 / s;
    for (let j = 0; j < s; j++)
      for (let i = 0; i < s; i++) {
        const k = j * s + i;
        const b = blendAt(w, dirOf(face, -1 + (i + 0.5) * span, -1 + (j + 0.5) * span));
        const rc = b.cloud;
        const rh = b.haze;
        const A = f.albedo[k]!;
        // Clouds over the ground (white: big droplets scatter all colours alike), then haze over both.
        let a = addLayers(rc, A);
        const lin = [0, 1, 2].map((ch) => addLayers(rc, toLin(f.rgb[k * 3 + ch]!)));
        if (rh > 0) {
          a = addLayers(rh * (0.2126 * b.rgb[0] + 0.7152 * b.rgb[1] + 0.0722 * b.rgb[2]), a);
          for (let ch = 0; ch < 3; ch++) lin[ch] = addLayers(rh * b.rgb[ch]!, lin[ch]!);
        }
        albedo[k] = a;
        for (let ch = 0; ch < 3; ch++) rgb[k * 3 + ch] = toSrgb(lin[ch]!);
      }
    return { ...f, rgb, albedo };
  });
  return { ...p, faces };
}

const share = (x: number) => `${Math.round(x * 100)}%`;
const place = (lat: number, lon: number) => `${Math.abs(Math.round(lat))}°${lat >= 0 ? "N" : "S"} ${Math.abs(Math.round(lon))}°${lon >= 0 ? "E" : "W"}`;
const KIND_WORDS: Record<string, string> = { "tropical-cyclone": "tropical cyclone", "dark-spot": "dark spot", "polar-vortex": "polar vortex", "cyclone-ring": "ring of cyclones", "polygon-jet": "polygon jet", "diffuse-cap": "diffuse polar cap", "dust-storm": "dust storm", "substellar-storm": "storm under the star" };

/** Where a storm is on a day: along its track (the last point at or before the day), else its stated place. */
export function stormAt(s: WeatherSystem, day: number): { lat_deg: number; lon_deg: number } {
  let at = { lat_deg: s.lat_deg, lon_deg: s.lon_deg };
  for (const p of s.track ?? []) if (p[0] <= day) at = { lat_deg: p[1], lon_deg: p[2] };
  return at;
}

/** The storms alive on the map's day, largest first. */
export function liveSystems(w: WeatherMap): WeatherSystem[] {
  return w.systems.filter((s) => s.born <= w.day && w.day <= s.ends).sort((a, b) => b.size_km - a.size_km);
}

/** A wet day: 1 mm or more of precipitation (the ETCCDI climate indices' wet-day threshold). */
export const WET_DAY_MM = 1;

/** The weather in words: how much cloud and of what, what falls, and the largest storms. */
export function narrateWeather(w: WeatherMap, label: string): string {
  const n = w.cells.length;
  let cover = 0;
  let wet = 0;
  let mm = 0;
  const types = new Map<string, number>();
  const stuff = new Map<string, number>();
  for (const c of w.cells) {
    let clear = 1;
    for (const l of c.layers) {
      const f = Math.min(1, Math.max(0, l.cloud_cover));
      clear *= 1 - f;
      if (f > 0.05 && l.cloud_type !== "none") {
        types.set(l.cloud_type, (types.get(l.cloud_type) ?? 0) + f);
        if (l.condensate) stuff.set(l.condensate, (stuff.get(l.condensate) ?? 0) + f);
      }
    }
    cover += 1 - clear;
    // Virga evaporates before the ground. A wet day is 1 mm or more (the ETCCDI wet-day mark).
    if (c.precip_kind !== "none" && c.precip_kind !== "virga") {
      mm += Math.max(0, c.precip_mm_day);
      if (c.precip_mm_day >= WET_DAY_MM) wet++;
    }
  }
  const top = [...types].sort((a, b) => b[1] - a[1]).slice(0, 3).map(([t]) => t);
  const made = [...stuff].sort((a, b) => b[1] - a[1]).map(([s]) => s);
  const storms = liveSystems(w).slice(0, 3).map((s) => {
    const at = stormAt(s, w.day);
    return `a ${KIND_WORDS[s.kind] ?? s.kind} ${Math.round(s.size_km).toLocaleString("en-GB")} km across at ${place(at.lat_deg, at.lon_deg)}`;
  });
  const parts = [
    `${label ? `${label}, ` : ""}weather on day ${Math.round(w.day).toLocaleString("en-GB")}: cloud over about ${share(cover / n)} of the globe${top.length ? ` (mostly ${top.join(", ")}${made.length ? `, made of ${made.slice(0, 2).join(" and ")}` : ""})` : ""}`,
    `${(mm / n).toFixed(1)} mm of rain or snow a day on average, a wet day (1 mm or more) over ${share(wet / n)}`,
  ];
  return `${parts.join("; ")}.${storms.length ? ` Storms: ${storms.join("; ")}.` : ""}`;
}

/** What the weather is at one point: the cell's layers, what falls, and any storm over it. */
export function weatherAt(w: WeatherMap, d: V3, radius_km: number): { cell: WeatherCell; cloud_reflectance: number; storm?: WeatherSystem } | null {
  const c = weatherCellAt(w, d);
  if (c < 0) return null;
  const storm = liveSystems(w).find((s) => {
    const at = stormAt(s, w.day);
    const sd = dirOfLatLon(at.lat_deg, at.lon_deg);
    const ang = Math.acos(Math.max(-1, Math.min(1, sd[0] * d[0] + sd[1] * d[1] + sd[2] * d[2])));
    return ang * radius_km <= s.size_km / 2;
  });
  return { cell: w.cells[c]!, cloud_reflectance: w.cloudR[c]!, ...(storm ? { storm } : {}) };
}
