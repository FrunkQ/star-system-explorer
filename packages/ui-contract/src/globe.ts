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
}

/** The kernel's names for the material and cover codes (atlas.rs MATERIALS and COVERS). */
export const MATERIALS = ["basalt", "granite", "regolith", "ice", "sediment", "lava", "cloud", "organic sediment", "carbon"] as const;
export const COVERS = ["none", "water", "magma", "hydrocarbon"] as const;

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
 * hydrocarbon dark; bare ground by its material, lightened with height and
 * by its albedo. Shading (`light`, 0 to 1) comes from the view.
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
