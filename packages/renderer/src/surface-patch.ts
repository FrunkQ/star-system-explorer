// The ground round a landing site (SL1): the kernel's surface atlas tiles at the fact floor, read into one local
// grid in metres about the site, ready to be drawn at eye height or flown over. Nothing here makes terrain: the
// heights, covers, materials and albedos are the kernel's tile samples, interpolated. Finer than the kernel's
// spacing the walk view adds decoration (`decoration`), which nothing in play reads (surface design 3.4).
//
// The local frame: x east, y north, z up, in metres, origin on the datum below the site. The ground falls away
// with the world's curve, (x² + y²) / 2R, so the horizon is the true one for the world's size.
import { faceOf, sampleColour, type DescentSample, type SurfaceTile } from "../../ui-contract/src/index.ts";

export type TileKey = { face: number; level: number; x: number; y: number };

/** The site's frame: up, east and north as unit vectors in the world's own frame. */
export function siteFrame(lat_deg: number, lon_deg: number): { up: [number, number, number]; east: [number, number, number]; north: [number, number, number] } {
  const la = (lat_deg * Math.PI) / 180;
  const lo = (lon_deg * Math.PI) / 180;
  const up: [number, number, number] = [Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la)];
  const east: [number, number, number] = [-Math.sin(lo), Math.cos(lo), 0];
  const north: [number, number, number] = [-Math.sin(la) * Math.cos(lo), -Math.sin(la) * Math.sin(lo), Math.cos(la)];
  return { up, east, north };
}

/** The direction from the world's centre of a point x m east and y m north of the site, on a world of radius R. */
export function localToDir(f: ReturnType<typeof siteFrame>, x: number, y: number, radius_m: number): [number, number, number] {
  const a = x / radius_m;
  const b = y / radius_m;
  const d: [number, number, number] = [f.up[0] + a * f.east[0] + b * f.north[0], f.up[1] + a * f.east[1] + b * f.north[1], f.up[2] + a * f.east[2] + b * f.north[2]];
  const l = Math.hypot(d[0], d[1], d[2]);
  return [d[0] / l, d[1] / l, d[2] / l];
}

/** Where a direction falls on a level's tiles: the tile and its fractional sample position (sample centres at i + 0.5). */
export function tileSpot(d: readonly [number, number, number], level: number, samples: number): { key: TileKey; si: number; sj: number } {
  const { face, u, v } = faceOf(d);
  const n = 2 ** level;
  const fu = ((u + 1) / 2) * n;
  const fv = ((v + 1) / 2) * n;
  const x = Math.min(n - 1, Math.max(0, Math.floor(fu)));
  const y = Math.min(n - 1, Math.max(0, Math.floor(fv)));
  return { key: { face, level, x, y }, si: (fu - x) * samples - 0.5, sj: (fv - y) * samples - 0.5 };
}

const keyText = (k: TileKey) => `${k.face}/${k.level}/${k.x}/${k.y}`;

/** The coarsest level whose spacing is at or below `floor_m`, from a level-0 spacing (the kernel's `level_for`). */
export function levelFor(spacing0_m: number, floor_m: number): number {
  let l = 0;
  while (l < 30 && spacing0_m / 2 ** l > floor_m) l++;
  return l;
}

/** The floor level a kernel refusal names ("BelowFactFloor { level: 12, floor_level: 9 }"), if it names one. */
export function floorFromRefusal(why: string): number | null {
  const m = /floor_level: (\d+)/.exec(why) ?? /max_level: Some\((\d+)\)/.exec(why);
  return m ? Number(m[1]) : null;
}

/** The tiles a square patch of half-width `half_m` about the site needs at a level (checked on a grid finer than a tile). */
export function patchKeys(lat_deg: number, lon_deg: number, half_m: number, level: number, radius_m: number, samples = 256): TileKey[] {
  const f = siteFrame(lat_deg, lon_deg);
  const keys = new Map<string, TileKey>();
  const N = 24;
  for (let j = 0; j <= N; j++)
    for (let i = 0; i <= N; i++) {
      const s = tileSpot(localToDir(f, -half_m + (2 * half_m * i) / N, -half_m + (2 * half_m * j) / N, radius_m), level, samples);
      keys.set(keyText(s.key), s.key);
    }
  return [...keys.values()];
}

/** A square grid of the ground about the site. Index j * n + i, x = -half + i·step east, y = -half + j·step north. */
export interface Patch {
  n: number;
  half_m: number;
  step_m: number;
  /** The kernel's sample spacing the grid was read from (the fact floor at this level). */
  spacing_m: number;
  level: number;
  /** Ground height above the datum, m (cover's surface where there is liquid: the sea's level, not its floor). */
  height_m: Float32Array;
  /** The ground beneath any liquid, m (the same as `height_m` on land). */
  floor_m: Float32Array;
  cover: Uint8Array;
  material: Uint8Array;
  /** Reflectance 0 to 1. */
  albedo: Float32Array;
  /** True where a tile was missing (the grid there is the nearest known height). */
  missing: number;
  /** The tiles read, by their kernel hash. */
  tiles: string[];
}

/**
 * Read the tiles into a grid of n × n points over ±half_m about the site. Liquid covers sit flat at sea level. A body
 * below the round size gives `lump`, its radius along a direction in its mean radius (small-body.ts `smallBodyShape`
 * `mean`): the ground is raised or lowered by the lump under it, so Land stands on the same lump the globe draws.
 */
export function readPatch(tiles: readonly SurfaceTile[], lat_deg: number, lon_deg: number, radius_m: number, half_m: number, n: number, sea_m = 0, lump?: (d: [number, number, number]) => number): Patch {
  const byKey = new Map<string, SurfaceTile>();
  for (const t of tiles) byKey.set(keyText(t.key), t);
  const level = tiles[0]?.key.level ?? 0;
  const samples = tiles[0]?.samples ?? 256;
  const f = siteFrame(lat_deg, lon_deg);
  const step = (2 * half_m) / (n - 1);
  const N = n * n;
  const p: Patch = { n, half_m, step_m: step, spacing_m: tiles[0]?.spacing_m ?? 0, level, height_m: new Float32Array(N), floor_m: new Float32Array(N), cover: new Uint8Array(N), material: new Uint8Array(N), albedo: new Float32Array(N), missing: 0, tiles: tiles.map((t) => t.hash) };
  let last = 0;
  for (let j = 0; j < n; j++)
    for (let i = 0; i < n; i++) {
      const k = j * n + i;
      const dir = localToDir(f, -half_m + i * step, -half_m + j * step, radius_m);
      const s = tileSpot(dir, level, samples);
      const t = byKey.get(keyText(s.key));
      if (!t) {
        p.missing++;
        p.height_m[k] = p.floor_m[k] = last;
        continue;
      }
      const raise = lump ? (lump(dir) - 1) * radius_m : 0;
      const S = t.samples;
      const i0 = Math.min(S - 1, Math.max(0, Math.floor(s.si)));
      const j0 = Math.min(S - 1, Math.max(0, Math.floor(s.sj)));
      const i1 = Math.min(S - 1, i0 + 1);
      const j1 = Math.min(S - 1, j0 + 1);
      const fx = Math.min(1, Math.max(0, s.si - i0));
      const fy = Math.min(1, Math.max(0, s.sj - j0));
      const at = (a: number, b: number) => b * S + a;
      const e = (q: number) => t.elevation_dm[q]! / 10;
      const ground = (e(at(i0, j0)) * (1 - fx) + e(at(i1, j0)) * fx) * (1 - fy) + (e(at(i0, j1)) * (1 - fx) + e(at(i1, j1)) * fx) * fy + raise;
      const near = at(fx < 0.5 ? i0 : i1, fy < 0.5 ? j0 : j1);
      const cover = t.cover[near]!;
      const al = (q: number) => t.albedo[q]! / 1000;
      p.floor_m[k] = ground;
      p.height_m[k] = cover !== 0 ? Math.max(ground, sea_m) : ground;
      p.cover[k] = cover;
      p.material[k] = t.material[near]!;
      p.albedo[k] = (al(at(i0, j0)) * (1 - fx) + al(at(i1, j0)) * fx) * (1 - fy) + (al(at(i0, j1)) * (1 - fx) + al(at(i1, j1)) * fx) * fy;
      last = p.height_m[k]!;
    }
  return p;
}

/** Height at a point of the patch (bilinear), m; the patch's edge value outside it. */
export function heightAt(p: Patch, x: number, y: number): number {
  const fi = Math.min(p.n - 1.0001, Math.max(0, (x + p.half_m) / p.step_m));
  const fj = Math.min(p.n - 1.0001, Math.max(0, (y + p.half_m) / p.step_m));
  const i = Math.floor(fi);
  const j = Math.floor(fj);
  const a = fi - i;
  const b = fj - j;
  const h = (ii: number, jj: number) => p.height_m[jj * p.n + ii]!;
  return (h(i, j) * (1 - a) + h(i + 1, j) * a) * (1 - b) + (h(i, j + 1) * (1 - a) + h(i + 1, j + 1) * a) * b;
}

/** How far the ground has fallen below the site's level plane at a distance d on a world of radius R (the curve). */
export const curveDrop = (d_m: number, radius_m: number): number => radius_m - Math.sqrt(Math.max(0, radius_m * radius_m - d_m * d_m));

/** The true distance to the horizon from an eye h metres above smooth ground on a world of radius R. */
export const horizonDistance = (h_m: number, radius_m: number): number => Math.sqrt(Math.max(0, 2 * radius_m * h_m + h_m * h_m));

/**
 * Decoration finer than the kernel's spacing (surface design 3.4: below the fact floor is decoration, may differ by
 * device, never read by play). Hashed value noise, deterministic by position, sized to the site's measured
 * roughness and fading out at the kernel's spacing so it never moves the kernel's own samples' shape.
 */
export function decoration(x: number, y: number, roughness_m: number, spacing_m: number): number {
  let h = 0;
  let amp = 1;
  let norm = 0;
  let wl = spacing_m / 2;
  for (let o = 0; o < 4; o++) {
    h += amp * valueNoise(x / wl, y / wl, o);
    norm += amp;
    amp *= 0.5;
    wl /= 2.2;
  }
  return (h / norm) * Math.min(roughness_m, spacing_m * 0.05);
}

function hash(i: number, j: number, o: number): number {
  let s = (Math.imul(i, 374761393) + Math.imul(j, 668265263) + Math.imul(o, 1442695041)) | 0;
  s = Math.imul(s ^ (s >>> 13), 1274126177);
  return ((s ^ (s >>> 16)) >>> 0) / 4294967295;
}
function valueNoise(x: number, y: number, o: number): number {
  const i = Math.floor(x);
  const j = Math.floor(y);
  const a = x - i;
  const b = y - j;
  const sa = a * a * (3 - 2 * a);
  const sb = b * b * (3 - 2 * b);
  const v = (ii: number, jj: number) => hash(ii, jj, o) * 2 - 1;
  return (v(i, j) * (1 - sa) + v(i + 1, j) * sa) * (1 - sb) + (v(i, j + 1) * (1 - sa) + v(i + 1, j + 1) * sa) * sb;
}

/**
 * The flown camera's track: where the lander is at each telemetry sample, in the site's frame. The kernel flies the
 * descent along its ground track without a heading; it is laid along `heading_deg` (from north, toward the site),
 * ending at the site, with the ground falling away by the world's curve. Altitude is the kernel's, above the site.
 */
export function descentTrack(tel: readonly DescentSample[], site_height_m: number, radius_m: number, heading_deg = 90): { t_s: number; x: number; y: number; z: number; to_go_km: number }[] {
  if (!tel.length) return [];
  const end = tel[tel.length - 1]!.downrange_km;
  const h = (heading_deg * Math.PI) / 180;
  const dx = Math.sin(h);
  const dy = Math.cos(h);
  return tel.map((s) => {
    const back = (end - s.downrange_km) * 1000;
    // On the curve: `back` along the ground is an angle back / R round the world's centre.
    const th = back / radius_m;
    const r = radius_m + site_height_m + s.altitude_m;
    const along = r * Math.sin(th);
    return { t_s: s.t_s, x: -dx * along, y: -dy * along, z: r * Math.cos(th) - radius_m, to_go_km: back / 1000 };
  });
}

/** One point of ground for a mesh: height above the datum, and what lies there. */
export interface GroundPoint {
  z: number;
  cover: number;
  material: number;
  albedo: number;
  /** Depth of any liquid over the ground, m. */
  depth: number;
  /** The kernel's own height here, before any decoration (contours follow it); `z` when left out. */
  kernel_z?: number;
}

/**
 * A point's two colours: its reflectance (linear, for the True look: the palette's hue at the kernel's albedo, as
 * on the globe) and its chart colour (display values, the globe's diagram palette).
 */
export function groundColours(g: GroundPoint): { albedo: [number, number, number]; chart: [number, number, number] } {
  const t = { elevation_dm: [g.cover !== 0 ? -g.depth * 10 : g.z * 10], cover: [g.cover], material: [g.material], albedo: [g.albedo * 1000] } as unknown as SurfaceTile;
  const c = sampleColour(t, 0);
  const chart: [number, number, number] = [c[0] / 255, c[1] / 255, c[2] / 255];
  const lin = chart.map((v) => Math.pow(v, 2.2)) as [number, number, number];
  const lum = Math.max(0.01, 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2]);
  const k = Math.max(0.01, g.albedo) / lum;
  return { albedo: [Math.min(1, lin[0] * k), Math.min(1, lin[1] * k), Math.min(1, lin[2] * k)], chart };
}

/**
 * A square grid mesh of n × n points over ±half_m about the site, the ground falling away with the world's curve,
 * leaving out the cells wholly inside ±hole_m (where a finer ring lies). `at` gives the ground at a point. A higher
 * `layer` is drawn over lower ones where rings overlap.
 */
export function groundMesh(key: string, n: number, half_m: number, hole_m: number, radius_m: number, at: (x: number, y: number) => GroundPoint, layer = 0): { key: string; positions: Float32Array; albedo: Float32Array; chart: Float32Array; kernel_z: Float32Array; wet: Float32Array; depth: Float32Array; floor: Float32Array; indices: Uint32Array; layer: number } {
  const N = n * n;
  const positions = new Float32Array(N * 3);
  const kernel_z = new Float32Array(N);
  const wet = new Float32Array(N);
  const depth = new Float32Array(N);
  const floor = new Float32Array(N);
  const albedo = new Float32Array(N * 3);
  const chart = new Float32Array(N * 3);
  const step = (2 * half_m) / (n - 1);
  for (let j = 0; j < n; j++)
    for (let i = 0; i < n; i++) {
      const x = -half_m + i * step;
      const y = -half_m + j * step;
      const g = at(x, y);
      const k = j * n + i;
      positions[k * 3] = x;
      positions[k * 3 + 1] = y;
      positions[k * 3 + 2] = g.z - curveDrop(Math.hypot(x, y), radius_m);
      kernel_z[k] = g.kernel_z ?? g.z;
      wet[k] = g.cover !== 0 ? 1 : 0;
      depth[k] = g.cover !== 0 ? g.depth : 0;
      floor[k] = g.albedo;
      const c = groundColours(g);
      albedo.set(c.albedo, k * 3);
      chart.set(c.chart, k * 3);
    }
  const idx: number[] = [];
  for (let j = 0; j < n - 1; j++)
    for (let i = 0; i < n - 1; i++) {
      const x0 = -half_m + i * step;
      const y0 = -half_m + j * step;
      if (hole_m > 0 && x0 >= -hole_m && x0 + step <= hole_m && y0 >= -hole_m && y0 + step <= hole_m) continue;
      const a = j * n + i;
      idx.push(a, a + 1, a + n, a + 1, a + n + 1, a + n);
    }
  return { key, positions, albedo, chart, kernel_z, wet, depth, floor, indices: new Uint32Array(idx), layer };
}

/** A patch's ground at a point (nearest cover and material, bilinear height and albedo). */
export function patchPoint(p: Patch, x: number, y: number): GroundPoint {
  const i = Math.min(p.n - 1, Math.max(0, Math.round((x + p.half_m) / p.step_m)));
  const j = Math.min(p.n - 1, Math.max(0, Math.round((y + p.half_m) / p.step_m)));
  const k = j * p.n + i;
  const z = heightAt(p, x, y);
  return { z, cover: p.cover[k]!, material: p.material[k]!, albedo: p.albedo[k]!, depth: Math.max(0, p.height_m[k]! - p.floor_m[k]!) };
}

/**
 * Where a flown descent is at each playback moment, 0 to 1: the track is played at a steady apparent motion of the
 * ground (how fast it slides and swells in the eye: the distance moved over the height above it), so a long fall
 * under a parachute and a fast pass high up take their fair share of the screen time. Returns, per sample, its
 * place in the playback.
 */
export function playbackKeys(track: readonly { x: number; y: number; z: number }[], ground_z: number): number[] {
  const keys = [0];
  for (let i = 1; i < track.length; i++) {
    const a = track[i - 1]!;
    const b = track[i]!;
    const h = Math.max(0, Math.min(a.z, b.z) - ground_z) + 30;
    const moved = Math.hypot(b.x - a.x, b.y - a.y) + Math.abs(b.z - a.z);
    keys.push(keys[i - 1]! + moved / h);
  }
  const end = keys[keys.length - 1]! || 1;
  return keys.map((k) => k / end);
}
