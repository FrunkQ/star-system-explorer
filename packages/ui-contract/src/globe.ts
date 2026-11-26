// The Body globe (V6 part 4): a world's S1 surface tiles (physics track,
// crates/sse-surface/src/atlas.rs) drawn on an orthographic globe. This file
// holds the pure parts: the cube-sphere mapping (the same FACES and tangent
// warp as the kernel's `direction` and `face_of`), sampling a direction, the
// colours, and the numbers the view states in words. The kernel builds the
// tiles; nothing here invents terrain.

/** One S1 tile as `resolved_system` returns it. Elevation in decimetres; layers indexed j * samples + i. */
export interface SurfaceTile {
  address: string;
  key: { face: number; level: number; x: number; y: number };
  samples: number;
  spacing_m: number;
  epoch_myr: number;
  elevation_dm: number[];
  material: number[];
  cover: number[];
  albedo: number[];
  slope_ddeg: number[];
  surface_age_myr: number[];
  pending: string[];
  hash: string;
  /** Space weathering of bare ground (kernel SW1), 0 fresh to 255 fully mature; absent where nothing from space reaches the ground. */
  maturity?: number[];
  /**
   * Presentation only, never from the kernel: bare ground's measured colour, sRGB 0..255 as r, g, b per sample, laid on
   * by the renderer from the data sheet's reflectance rows and `maturity` (renderer ground-optics.ts). NaN where not known.
   */
  ground_rgb?: Float32Array | number[];
}

/** The kernel's names for the material and cover codes (atlas.rs MATERIALS and COVERS). */
export const MATERIALS = ["basalt", "granite", "regolith", "ice", "sediment", "lava", "cloud", "organic sediment", "carbon"] as const;
export const COVERS = ["none", "water", "magma", "hydrocarbon", "ice"] as const;

type V3 = readonly [number, number, number];
/** Normal, right and up of each cube face, as atlas.rs FACES. */
const FACES: readonly (readonly [V3, V3, V3])[] = [
  [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
  [[-1, 0, 0], [0, -1, 0], [0, 0, 1]],
  [[0, 1, 0], [-1, 0, 0], [0, 0, 1]],
  [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
  [[0, 0, 1], [0, 1, 0], [-1, 0, 0]],
  [[0, 0, -1], [0, 1, 0], [1, 0, 0]],
];
const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

/** Unit direction of face coordinates (u, v) in [-1, 1]: the kernel's `direction`. */
export function direction(face: number, u: number, v: number): [number, number, number] {
  const [n, r, t] = FACES[face]!;
  const a = Math.tan((u * Math.PI) / 4);
  const b = Math.tan((v * Math.PI) / 4);
  const d = [n[0] + a * r[0] + b * t[0], n[1] + a * r[1] + b * t[1], n[2] + a * r[2] + b * t[2]];
  const l = Math.hypot(d[0]!, d[1]!, d[2]!);
  return [d[0]! / l, d[1]! / l, d[2]! / l];
}

/** The face and (u, v) a direction falls on: the kernel's `face_of`. */
export function faceOf(d: V3): { face: number; u: number; v: number } {
  let best = 0;
  let bv = -Infinity;
  FACES.forEach((f, i) => {
    const x = dot(d, f[0]);
    if (x > bv) ((bv = x), (best = i));
  });
  const [n, r, t] = FACES[best]!;
  const k = dot(d, n);
  return { face: best, u: (Math.atan(dot(d, r) / k) * 4) / Math.PI, v: (Math.atan(dot(d, t) / k) * 4) / Math.PI };
}

/** The six level-0 tiles of one world, by face. */
export type Globe = readonly (SurfaceTile | undefined)[];

/** Index of the sample a direction falls in, on its face's level-0 tile. */
export function sampleAt(globe: Globe, d: V3): { tile: SurfaceTile; i: number } | null {
  const { face, u, v } = faceOf(d);
  const tile = globe[face];
  if (!tile) return null;
  const s = tile.samples;
  const i = Math.min(s - 1, Math.max(0, Math.floor(((u + 1) / 2) * s)));
  const j = Math.min(s - 1, Math.max(0, Math.floor(((v + 1) / 2) * s)));
  return { tile, i: j * s + i };
}

// ---------------------------------------------------------------- natural outlines

/*
 * Natural outlines (Alex 25 Nov: "they look a bit bitmap"): the kernel's cells drawn with sub-cell coasts, ice edges
 * and patch shapes instead of square steps. One deterministic step, keyed only on the direction (and an optional
 * seed), never on time, so the same place always wiggles the same way and a time-lapse does not shimmer: an edge moves
 * only where the facts move. Presentation only; never a kernel fact. Fudge register: "Natural outlines".
 *
 * The step is a domain warp: the direction a view looks up is nudged sideways by fractal noise, by at most
 * NATURAL_WARP of a cell. A cell's centre is never nudged out of its own cell, so every cell keeps its own fact
 * where the kernel computed it; only the shapes between centres change.
 */

/** The most a direction is nudged, as a share of the cell's angular size (under half, so no centre leaves its cell). */
export const NATURAL_WARP = 0.42;
/** Octaves of the edge noise, from two cells across down to an eighth of a cell. */
const NATURAL_OCTAVES = 4;

/** A lattice hash to -1..1: integer mixing only, the same in every browser. */
function lattice(x: number, y: number, z: number, seed: number): number {
  let h = Math.imul(x | 0, 0x27d4eb2d) ^ Math.imul(y | 0, 0x165667b1) ^ Math.imul(z | 0, 0x1b873593) ^ Math.imul(seed | 0, 0x9e3779b1);
  h = Math.imul(h ^ (h >>> 15), 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  h ^= h >>> 16;
  return ((h >>> 0) / 4294967295) * 2 - 1;
}

/** Smooth 3D value noise at a point, -1..1 (quintic easing, so its slope has no creases). */
function valueNoise(x: number, y: number, z: number, seed: number): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const zi = Math.floor(z);
  const e = (t: number) => t * t * t * (t * (t * 6 - 15) + 10);
  const fx = e(x - xi);
  const fy = e(y - yi);
  const fz = e(z - zi);
  const l = (a: number, b: number, t: number) => a + (b - a) * t;
  const c = (dx: number, dy: number, dz: number) => lattice(xi + dx, yi + dy, zi + dz, seed);
  return l(l(l(c(0, 0, 0), c(1, 0, 0), fx), l(c(0, 1, 0), c(1, 1, 0), fx), fy), l(l(c(0, 0, 1), c(1, 0, 1), fx), l(c(0, 1, 1), c(1, 1, 1), fx), fy), fz);
}

/**
 * The edge noise at a direction, about -1..1: fractal value noise whose largest wiggle is about two cells across for
 * cells `cell_rad` radians wide, keyed only on the direction and `seed`. The time-lapse and every globe share it.
 */
export function edgeNoise(d: readonly number[], cell_rad: number, seed = 0): number {
  const f0 = 1 / (2 * Math.max(1e-9, cell_rad));
  let sum = 0;
  let amp = 0.5;
  let f = f0;
  for (let o = 0; o < NATURAL_OCTAVES; o++) {
    sum += amp * valueNoise(d[0]! * f, d[1]! * f, d[2]! * f, seed + o * 1013);
    f *= 2;
    amp *= 0.5;
  }
  return sum / 0.9375;
}

/**
 * The direction to look a cell up at, for natural outlines: `d` nudged sideways along the sphere by the edge noise,
 * at most NATURAL_WARP of a cell `cell_rad` radians wide. Look up the nearest cell at the result, as before.
 */
export function naturalDirection(d: readonly number[], cell_rad: number, seed = 0): [number, number, number] {
  const r: V3 = [d[0]!, d[1]!, d[2]!];
  const a = edgeNoise(r, cell_rad, seed + 17);
  const b = edgeNoise(r, cell_rad, seed + 7919);
  // Two directions along the sphere at r.
  const ref: V3 = Math.abs(r[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0];
  let e: V3 = [ref[1] * r[2] - ref[2] * r[1], ref[2] * r[0] - ref[0] * r[2], ref[0] * r[1] - ref[1] * r[0]];
  const el = Math.hypot(e[0], e[1], e[2]);
  e = [e[0] / el, e[1] / el, e[2] / el];
  const n: V3 = [r[1] * e[2] - r[2] * e[1], r[2] * e[0] - r[0] * e[2], r[0] * e[1] - r[1] * e[0]];
  let s = Math.hypot(a, b);
  const k = s > 1 ? 1 / s : 1;
  s = NATURAL_WARP * cell_rad * k;
  const o: V3 = [r[0] + s * (a * e[0] + b * n[0]), r[1] + s * (a * e[1] + b * n[1]), r[2] + s * (a * e[2] + b * n[2])];
  const l = Math.hypot(o[0], o[1], o[2]);
  return [o[0] / l, o[1] / l, o[2] / l];
}

/**
 * Which side of an edge a blended field falls once the edge noise is added: true where `value` (blended between
 * cells) is above `threshold`, the noise moving the edge by up to `scale` in the field's own units. Keep `scale`
 * under the field's step between neighbouring cells' values so each cell keeps its own side at its centre.
 */
export function naturalEdge(d: readonly number[], value: number, threshold: number, scale: number, cell_rad: number, seed = 0): boolean {
  return value + scale * edgeNoise(d, cell_rad, seed) > threshold;
}

/** The smallest angle a level-0 sample of this globe spans (the equiangular map's cells are smallest at the corners). */
export function globeCellAngle(globe: Globe): number {
  const t = globe.find((x) => !!x);
  return t ? (Math.PI / 2 / (t.samples * (1 << t.key.level))) * 0.72 : 0;
}

/** How far the edge noise moves an edge in natural outlines, in blended-share units (under half: centres keep their fact). */
export const NATURAL_EDGE = 0.3;

type Kind = 0 | 1 | 2;
/** A sample's kind for outlines: 0 bare ground, 1 ice, 2 liquid (water, magma, hydrocarbon). */
const kindOf = (t: SurfaceTile, k: number): Kind => {
  const c = COVERS[t.cover[k]!] ?? "none";
  return c === "none" ? 0 : c === "ice" ? 1 : 2;
};

/**
 * sampleAt with natural outlines: the sample whose colour a direction shows, with coasts and ice edges drawn as smooth
 * contours between cells instead of steps. The four samples round the point are blended (bilinear); each kind (bare
 * ground, ice, liquid) gets its blended share plus the edge noise (NATURAL_EDGE, keyed on the direction and `seed`),
 * and the largest wins; the answer is the nearest of the four of that kind. At a sample's centre its own share is 1
 * and every other 0, so its own kind always wins: no fact moves. Clamped at a face's edge (no blend across faces).
 */
export function sampleNatural(globe: Globe, d: V3, seed = 0): { tile: SurfaceTile; i: number } | null {
  const { face, u, v } = faceOf(d);
  const t = globe[face];
  if (!t) return null;
  const s = t.samples;
  const x = Math.min(s - 1, Math.max(0, ((u + 1) / 2) * s - 0.5));
  const y = Math.min(s - 1, Math.max(0, ((v + 1) / 2) * s - 0.5));
  const i0 = Math.max(0, Math.min(s - 2, Math.floor(x)));
  const j0 = Math.max(0, Math.min(s - 2, Math.floor(y)));
  const fx = Math.min(1, Math.max(0, x - i0));
  const fy = Math.min(1, Math.max(0, y - j0));
  const i1 = Math.min(s - 1, i0 + 1);
  const j1 = Math.min(s - 1, j0 + 1);
  const ks = [j0 * s + i0, j0 * s + i1, j1 * s + i0, j1 * s + i1];
  const ws = [(1 - fx) * (1 - fy), fx * (1 - fy), (1 - fx) * fy, fx * fy];
  const share: number[] = [0, 0, 0];
  const kinds = ks.map((k) => kindOf(t, k));
  for (let q = 0; q < 4; q++) share[kinds[q]!] = share[kinds[q]!]! + ws[q]!;
  let best: Kind = kinds[ws.indexOf(Math.max(...ws))]!;
  if (share[best]! < 1) {
    const cell = globeCellAngle(globe);
    let top = -Infinity;
    for (const kd of [0, 1, 2] as Kind[]) {
      if (share[kd]! <= 0) continue;
      const v = share[kd]! + NATURAL_EDGE * edgeNoise(d, cell, seed + 101 * (kd + 1));
      if (v > top) ((top = v), (best = kd));
    }
  }
  let pick = -1;
  for (let q = 0; q < 4; q++) if (kinds[q] === best && (pick < 0 || ws[q]! > ws[pick]!)) pick = q;
  return { tile: t, i: ks[pick]! };
}

/** Cells each way the bare ground's colour is softened over (two box passes, about a Gaussian of this width). */
export const SOFT_GROUND_CELLS = 2;

/**
 * Bare ground's colour softened over its neighbours (natural outlines' second half): square patches of ground colour
 * become gradients, as they look from a distance. Only bare samples are averaged, and only with bare samples, so a
 * coast or an ice edge stays where it is. Laid on as `ground_rgb` (presentation only); covers and every kernel value
 * are untouched. Fudge register: "Natural outlines".
 */
export function softenGround(t: SurfaceTile, cells = SOFT_GROUND_CELLS): SurfaceTile {
  const s = t.samples;
  const n = s * s;
  const bare = new Uint8Array(n);
  const rgb = new Float32Array(n * 3);
  let any = false;
  for (let k = 0; k < n; k++) {
    if ((COVERS[t.cover[k]!] ?? "none") !== "none") continue;
    bare[k] = 1;
    any = true;
    const c = sampleColour(t, k, 1);
    rgb[k * 3] = c[0];
    rgb[k * 3 + 1] = c[1];
    rgb[k * 3 + 2] = c[2];
  }
  if (!any || cells < 1) return t;
  const pass = (src: Float32Array, horizontal: boolean): Float32Array => {
    const out = new Float32Array(n * 3);
    for (let a = 0; a < s; a++) {
      for (let b = 0; b < s; b++) {
        const k = horizontal ? a * s + b : b * s + a;
        if (!bare[k]) continue;
        let r = 0;
        let g = 0;
        let bl = 0;
        let w = 0;
        for (let o = -cells; o <= cells; o++) {
          const c = b + o;
          if (c < 0 || c >= s) continue;
          const q = horizontal ? a * s + c : c * s + a;
          if (!bare[q]) continue;
          r += src[q * 3]!;
          g += src[q * 3 + 1]!;
          bl += src[q * 3 + 2]!;
          w++;
        }
        out[k * 3] = r / w;
        out[k * 3 + 1] = g / w;
        out[k * 3 + 2] = bl / w;
      }
    }
    return out;
  };
  let soft: Float32Array = rgb;
  for (let p = 0; p < 2; p++) soft = pass(pass(soft, true), false);
  const ground = new Float32Array(n * 3).fill(NaN);
  for (let k = 0; k < n; k++) if (bare[k]) ground.set([soft[k * 3]!, soft[k * 3 + 1]!, soft[k * 3 + 2]!], k * 3);
  return { ...t, ground_rgb: ground };
}

const MATERIAL_RGB: Record<string, V3> = {
  basalt: [92, 88, 84],
  granite: [150, 132, 112],
  regolith: [158, 148, 132],
  ice: [228, 236, 240],
  sediment: [176, 152, 108],
  lava: [70, 44, 36],
  cloud: [236, 236, 228],
  "organic sediment": [104, 120, 72],
  carbon: [52, 50, 48],
};

/**
 * A sample's colour, presentation only: water by depth, magma glowing,
 * hydrocarbon dark; bare ground by its measured colour where the renderer laid
 * one on (`ground_rgb`), else by its material, lightened with height and by its albedo. Shading (`light`, 0 to 1) comes from the view.
 */
export function sampleColour(t: SurfaceTile, i: number, light = 1): V3 {
  const elev = t.elevation_dm[i]! / 10;
  const cover = COVERS[t.cover[i]!] ?? "none";
  let c: V3;
  if (cover === "water") {
    const depth = Math.min(1, Math.max(0, -elev / 6000));
    c = [Math.round(70 - 50 * depth), Math.round(130 - 80 * depth), Math.round(170 - 60 * depth)];
  } else if (cover === "magma") c = [235, 110, 40];
  else if (cover === "hydrocarbon") c = [60, 46, 30];
  else if (cover === "ice") c = [232, 240, 246];
  else if (t.ground_rgb && t.ground_rgb[i * 3]! >= 0) c = [t.ground_rgb[i * 3]!, t.ground_rgb[i * 3 + 1]!, t.ground_rgb[i * 3 + 2]!];
  else {
    const m = MATERIAL_RGB[MATERIALS[t.material[i]!] ?? "regolith"] ?? MATERIAL_RGB.regolith!;
    const h = Math.min(1, Math.max(0, elev / 8000));
    const a = Math.min(1, Math.max(0, t.albedo[i]! / 1000));
    const k = 0.75 + 0.35 * h + 0.3 * (a - 0.3);
    c = [Math.min(255, m[0] * k), Math.min(255, m[1] * k), Math.min(255, m[2] * k)];
  }
  const g = cover === "magma" ? 1 : light;
  return [Math.round(c[0] * g), Math.round(c[1] * g), Math.round(c[2] * g)];
}

/** What the view says in words: how much is under each cover, the highest and lowest ground, materials. */
export function globeStats(globe: Globe): { covered: Record<string, number>; highest_m: number; lowest_m: number; materials: [string, number][]; samples: number } {
  const covered: Record<string, number> = {};
  const mats = new Map<string, number>();
  let hi = -Infinity;
  let lo = Infinity;
  let n = 0;
  for (const t of globe) {
    if (!t) continue;
    for (let i = 0; i < t.elevation_dm.length; i++) {
      n++;
      const c = COVERS[t.cover[i]!] ?? "none";
      covered[c] = (covered[c] ?? 0) + 1;
      const e = t.elevation_dm[i]! / 10;
      if (e > hi) hi = e;
      if (e < lo) lo = e;
      if (c === "none") {
        const m = MATERIALS[t.material[i]!] ?? "unknown";
        mats.set(m, (mats.get(m) ?? 0) + 1);
      }
    }
  }
  for (const k of Object.keys(covered)) covered[k] = covered[k]! / Math.max(1, n);
  const bare = [...mats.values()].reduce((a, b) => a + b, 0);
  return { covered, highest_m: hi, lowest_m: lo, materials: [...mats].map(([k, v]) => [k, v / Math.max(1, bare)] as [string, number]).sort((a, b) => b[1] - a[1]), samples: n };
}

/** View-space point on the unit disk to a world direction, for a globe turned by `lon_deg` about the pole and tipped by `lat_deg`. */
export function viewToWorld(x: number, y: number, lon_deg: number, lat_deg: number): [number, number, number] | null {
  const r2 = x * x + y * y;
  if (r2 > 1) return null;
  const z = Math.sqrt(1 - r2);
  // View space: x right, y up, z towards the viewer. World: z is the pole.
  const la = (lat_deg * Math.PI) / 180;
  const lo = (lon_deg * Math.PI) / 180;
  // Undo the tip about the view's x axis.
  const vy = y * Math.cos(la) + z * Math.sin(la);
  const vz = -y * Math.sin(la) + z * Math.cos(la);
  // Facing direction (vz) is the world's horizontal toward lon; vy is the pole.
  const wx = vz * Math.cos(lo) - x * Math.sin(lo);
  const wy = vz * Math.sin(lo) + x * Math.cos(lo);
  return [wx, wy, vy];
}
