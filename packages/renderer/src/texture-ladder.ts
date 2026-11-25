// The texture ladder (VP3, SSE4-visual-polish-design.md section 5.3): a world's face from space at the size the
// screen can show, from the kernel's own tiles. This is the fact rung of the detail ladder (ladder.ts): nothing here
// invents ground; it only reads the tiles, at whatever levels have been read, and its relief is the tiles' own
// heights at true scale.
//
// - `faceSize` picks the cube face's edge in texels so one texel is about one screen pixel (64 to 512 on the basic
//   tier, to 2048 on high, 4096 at rest).
// - `levelFor` and `tilesFacing` say which kernel tiles are worth reading for that size, nearest the viewer first.
// - `composeFaces` lays any mix of tile levels into one picture per cube face (the deepest read wins; coarser levels
//   fill the gaps), colour from the tiles (the survey colours, as the Body globe and V10a use) and height in metres.
// - `sampleFaces` and `reliefNormal` are the processor twin of the GPU bake (texture-ladder-gpu.ts): bilinear inside
//   a face, and the surface normal tilted by the real slope between neighbouring texels (Horn 1981's idea, on the
//   sphere), so craters and ranges catch the light near the terminator where the sun grazes them.
import { say, SURFACE_COVERS as COVERS, sampleColour, type SurfaceTile, type Tier } from "../../ui-contract/src/index.ts";

/** The kernel's samples along a tile's edge (atlas settings `tile_samples`; the survey uses 32). */
export const TILE_SAMPLES = 32;
/** The smallest face baked. */
export const FACE_MIN = 64;
/** The largest face each tier bakes (design 8.5: B to 512, H to 2048, at rest to 4096). */
export const FACE_CAP: Readonly<Record<Tier | "gpu-ultra", number>> = { text: 0, canvas: 0, "gpu-basic": 512, "gpu-high": 2048, "gpu-ultra": 2048, still: 4096 };
/** The deepest a composed face picture goes (texels along its edge): a picture finer than this is not held on the processor. */
export const COMPOSE_MAX = 1024;

/**
 * The cube face's edge in texels for a disc `disc_px` across: a quarter of the way round the globe spans one face,
 * so the face edge seen square-on covers about π/4 of the disc's width. Rounded up to a power of two and held inside
 * the tier's range; 0 when the tier bakes no faces.
 */
export function faceSize(disc_px: number, tier: Tier | "gpu-ultra"): number {
  const cap = FACE_CAP[tier] ?? 0;
  if (cap <= 0) return 0;
  const want = Math.max(1, (disc_px * Math.PI) / 4);
  const p = Math.pow(2, Math.ceil(Math.log2(want)));
  return Math.max(FACE_MIN, Math.min(cap, p));
}

/** The deepest tile level worth reading for a face of `face_px` texels: its samples then meet the texels. */
export function levelFor(face_px: number, samples = TILE_SAMPLES): number {
  if (!(face_px > samples)) return 0;
  return Math.max(0, Math.ceil(Math.log2(face_px / samples) - 1e-9));
}

type V3 = [number, number, number];
/** Normal, right and up of each cube face (atlas.rs FACES, as ui-contract's globe.ts). */
const FACES: readonly (readonly [V3, V3, V3])[] = [
  [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
  [[-1, 0, 0], [0, -1, 0], [0, 0, 1]],
  [[0, 1, 0], [-1, 0, 0], [0, 0, 1]],
  [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
  [[0, 0, 1], [0, 1, 0], [-1, 0, 0]],
  [[0, 0, -1], [0, 1, 0], [1, 0, 0]],
];
const Q = Math.PI / 4;
/** The direction at cube coordinates (u, v) on a face (equiangular, as the kernel's atlas). */
export function faceDirection(face: number, u: number, v: number): V3 {
  const [n, r, t] = FACES[face]!;
  const a = Math.tan(u * Q);
  const b = Math.tan(v * Q);
  const x = n[0] + a * r[0] + b * t[0];
  const y = n[1] + a * r[1] + b * t[1];
  const z = n[2] + a * r[2] + b * t[2];
  const l = Math.hypot(x, y, z);
  return [x / l, y / l, z / l];
}
/** The face and cube coordinates of a direction (the inverse of faceDirection). */
export function directionFace(d: readonly number[]): { face: number; u: number; v: number } {
  const ax = Math.abs(d[0]!);
  const ay = Math.abs(d[1]!);
  const az = Math.abs(d[2]!);
  const face = ax >= ay && ax >= az ? (d[0]! >= 0 ? 0 : 1) : ay >= az ? (d[1]! >= 0 ? 2 : 3) : d[2]! >= 0 ? 4 : 5;
  const [n, r, t] = FACES[face]!;
  const k = d[0]! * n[0] + d[1]! * n[1] + d[2]! * n[2];
  return { face, u: Math.atan((d[0]! * r[0] + d[1]! * r[1] + d[2]! * r[2]) / k) / Q, v: Math.atan((d[0]! * t[0] + d[1]! * t[1] + d[2]! * t[2]) / k) / Q };
}

export interface TileKey {
  face: number;
  level: number;
  x: number;
  y: number;
}

/**
 * The tiles at `level` whose centres face the viewer (`toward`: unit direction from the body's centre to the camera,
 * in the body's frame, pole on z), nearest the middle of the disc first, at most `max`. A tile just over the limb is
 * kept (its centre up to 0.2 behind the limb), so turning the globe a little shows no hole.
 */
export function tilesFacing(toward: readonly number[], level: number, max: number): TileKey[] {
  const n = 1 << level;
  const out: { k: TileKey; d: number }[] = [];
  for (let face = 0; face < 6; face++) {
    for (let y = 0; y < n; y++) {
      for (let x = 0; x < n; x++) {
        const c = faceDirection(face, -1 + (2 * (x + 0.5)) / n, -1 + (2 * (y + 0.5)) / n);
        const d = c[0] * toward[0]! + c[1] * toward[1]! + c[2] * toward[2]!;
        if (d > -0.2) out.push({ k: { face, level, x, y }, d });
      }
    }
  }
  out.sort((a, b) => b.d - a.d || a.k.face - b.k.face || a.k.y - b.k.y || a.k.x - b.k.x);
  return out.slice(0, Math.max(0, max)).map((o) => o.k);
}

/** One picture per cube face: sRGB colour (rgb per texel) and height in metres, `n` texels along the edge. */
export interface FacePicture {
  n: number;
  rgb: Uint8Array;
  height_m: Float32Array;
  /** Liquid cover (water, magma, hydrocarbon): a flat sea surface, no relief. */
  liquid: Uint8Array;
  /** The deepest tile level laid into it. */
  level: number;
}
export interface Composite {
  faces: (FacePicture | null)[];
  /** The tiles' signature: a new tile read changes it, so a bake can be kept until then. */
  signature: string;
  /** The body's radius from the tiles' own spacing (spacing × samples × 2^level = a quarter of the way round), 0 with no tiles. */
  radius_m: number;
  /** The tiles it was composed from, so finer ones can be laid over it later (withFiner). */
  tiles: readonly SurfaceTile[];
}

/**
 * Lay the tiles read so far into one picture per face. Each face's picture is as fine as its deepest tile (up to
 * COMPOSE_MAX texels); coarser tiles are laid first and finer ones over them, so a part of the face read only at
 * level 0 still shows, at its own coarseness. A face with no tile is null (drawn plain, as today).
 */
export function composeFaces(tiles: readonly SurfaceTile[]): Composite {
  const byFace: SurfaceTile[][] = [[], [], [], [], [], []];
  for (const t of tiles) if (t && t.key.face >= 0 && t.key.face < 6) byFace[t.key.face]!.push(t);
  const sig: string[] = [];
  let radius_m = 0;
  for (const t of tiles) if (t && t.spacing_m > 0 && !radius_m) radius_m = (t.spacing_m * t.samples * (1 << t.key.level)) / (Math.PI / 2);
  const faces = byFace.map((list) => {
    if (!list.length) return null;
    list.sort((a, b) => a.key.level - b.key.level || a.key.y - b.key.y || a.key.x - b.key.x);
    const s = list[0]!.samples;
    let level = 0;
    for (const t of list) if (t.samples * (1 << t.key.level) <= COMPOSE_MAX) level = Math.max(level, t.key.level);
    const n = s * (1 << level);
    const rgb = new Uint8Array(n * n * 3).fill(128);
    const height_m = new Float32Array(n * n);
    const liquid = new Uint8Array(n * n);
    for (const t of list) {
      if (t.key.level > level) continue;
      sig.push(`${t.key.face}/${t.key.level}/${t.key.x}/${t.key.y}:${t.hash}`);
      const ts = t.samples;
      const scale = n / (ts * (1 << t.key.level));
      const x0 = t.key.x * ts * scale;
      const y0 = t.key.y * ts * scale;
      const span = Math.max(1, Math.round(scale));
      for (let j = 0; j < ts; j++) {
        for (let i = 0; i < ts; i++) {
          const k = j * ts + i;
          const c = sampleColour(t, k, 1);
          const h = t.elevation_dm[k]! / 10;
          const wet = (COVERS[t.cover[k]!] ?? "none") !== "none" && COVERS[t.cover[k]!] !== "ice" ? 1 : 0;
          for (let b = 0; b < span; b++) {
            const row = Math.round(y0 + j * scale) + b;
            if (row >= n) break;
            for (let a = 0; a < span; a++) {
              const col = Math.round(x0 + i * scale) + a;
              if (col >= n) break;
              const o = row * n + col;
              rgb[o * 3] = c[0];
              rgb[o * 3 + 1] = c[1];
              rgb[o * 3 + 2] = c[2];
              height_m[o] = wet ? 0 : h;
              liquid[o] = wet;
            }
          }
        }
      }
    }
    return { n, rgb, height_m, liquid, level };
  });
  return { faces, signature: sig.join("|"), radius_m, tiles };
}

/**
 * Finer tiles laid over a composed surface (VP3 part 2): the same faces recomposed with them, deepest first; tiles
 * already in it are not doubled. A surface with no composed faces (a giant's cloud decks, or ground with clouds
 * painted over it) is returned as it is, since its picture is not the ground's alone.
 */
export function withFiner<T extends { faces?: Composite }>(tex: T, fine: readonly SurfaceTile[]): T {
  if (!tex.faces || !fine.length) return tex;
  const k = (t: SurfaceTile) => `${t.key.face}/${t.key.level}/${t.key.x}/${t.key.y}`;
  const have = new Set(tex.faces.tiles.map(k));
  const more = fine.filter((t) => !have.has(k(t)));
  if (!more.length) return tex;
  return { ...tex, faces: composeFaces([...tex.faces.tiles, ...more]) };
}

/** Bilinear inside a face picture at cube coordinates (u, v), clamped to the face (no bleeding across an edge). */
function bilinear(f: FacePicture, u: number, v: number, read: (o: number) => number): number {
  const n = f.n;
  const x = Math.min(n - 1, Math.max(0, ((u + 1) / 2) * n - 0.5));
  const y = Math.min(n - 1, Math.max(0, ((v + 1) / 2) * n - 0.5));
  const i = Math.floor(x);
  const j = Math.floor(y);
  const i1 = Math.min(n - 1, i + 1);
  const j1 = Math.min(n - 1, j + 1);
  const fx = x - i;
  const fy = y - j;
  const a = read(j * n + i) * (1 - fx) + read(j * n + i1) * fx;
  const b = read(j1 * n + i) * (1 - fx) + read(j1 * n + i1) * fx;
  return a * (1 - fy) + b * fy;
}

/** The composed surface at a direction: sRGB colour 0..255 and height in metres, or null where no tile was read. */
export function sampleFaces(c: Composite, d: readonly number[]): { rgb: V3; height_m: number } | null {
  const { face, u, v } = directionFace(d);
  const f = c.faces[face];
  if (!f) return null;
  return {
    rgb: [bilinear(f, u, v, (o) => f.rgb[o * 3]!), bilinear(f, u, v, (o) => f.rgb[o * 3 + 1]!), bilinear(f, u, v, (o) => f.rgb[o * 3 + 2]!)],
    height_m: bilinear(f, u, v, (o) => f.height_m[o]!),
  };
}

/**
 * The surface normal at a direction from the composed heights at true scale: the radial direction tilted against
 * the slope measured between points `step` radians either side (east and north along the surface), on a body of
 * radius `radius_m`. `exaggeration` 1 is the world as it is; any other value is a look and must be said so.
 */
export function reliefNormal(c: Composite, d: readonly number[], radius_m: number, step: number, exaggeration = 1): V3 {
  const r: V3 = [d[0]!, d[1]!, d[2]!];
  // Two tangents: east (round the pole, z) and north; near a pole east is taken round x instead.
  let e: V3 = [-r[1], r[0], 0];
  if (Math.hypot(e[0], e[1]) < 1e-6) e = [0, -r[2], r[1]];
  const el = Math.hypot(e[0], e[1], e[2]);
  e = [e[0] / el, e[1] / el, e[2] / el];
  const nth: V3 = [r[1] * e[2] - r[2] * e[1], r[2] * e[0] - r[0] * e[2], r[0] * e[1] - r[1] * e[0]];
  const at = (a: number, b: number) => {
    const p: V3 = [r[0] + e[0] * a + nth[0] * b, r[1] + e[1] * a + nth[1] * b, r[2] + e[2] * a + nth[2] * b];
    return sampleFaces(c, p)?.height_m ?? 0;
  };
  const span = 2 * step * radius_m;
  const ge = ((at(step, 0) - at(-step, 0)) / span) * exaggeration;
  const gn = ((at(0, step) - at(0, -step)) / span) * exaggeration;
  const n: V3 = [r[0] - ge * e[0] - gn * nth[0], r[1] - ge * e[1] - gn * nth[1], r[2] - ge * e[2] - gn * nth[2]];
  const l = Math.hypot(n[0], n[1], n[2]);
  return [n[0] / l, n[1] / l, n[2] / l];
}

/** The angle one texel of a face `n` texels across spans at the face's middle, radians (the normal's step). */
export const texelAngle = (n: number) => (Math.PI / 2) / n;

/** Bytes a baked cube takes on the GPU: colour and normal, four bytes a texel each, six faces, plus a third for mips. */
export const cubeBytes = (n: number) => Math.round(6 * n * n * 8 * (4 / 3));

/** How much the bakes of one view may hold on the GPU (an LRU evicts beyond it): 48 MB on basic, 256 MB above. */
export const BAKE_BUDGET: Readonly<Record<Tier | "gpu-ultra", number>> = { text: 0, canvas: 0, "gpu-basic": 48e6, "gpu-high": 256e6, "gpu-ultra": 256e6, still: 256e6 };

/** The HUD's words for the face drawn: the finest ground read, in the body's own units. */
export function faceWords(c: Composite, radius_m: number): string {
  let best = Infinity;
  for (const f of c.faces) if (f) best = Math.min(best, (radius_m * (Math.PI / 2)) / f.n);
  if (!Number.isFinite(best)) return "";
  return `Ground read to ${say(best, "length").text}.`;
}

// ------------------------------------------------------------------ the deep patch under a close camera (VP3 part 4)

/** At most this many deep tiles round the point under the camera (3 × 3: about 2 s of kernel time). */
export const PATCH_TILES = 9;
/** A sample to every this many screen pixels at the point under the camera (bilinear between them). */
export const PATCH_PX_PER_SAMPLE = 2;

/** What the camera looks down on (the renderer's close-up GroundView): the body-frame direction and metres a pixel. */
export interface GroundAsk {
  toward: readonly number[];
  m_per_px: number;
}

/**
 * The level the ground under the camera needs: the coarsest whose sample spacing is at or below a sample every
 * PATCH_PX_PER_SAMPLE pixels, from level 0's spacing (a quarter of the way round over `samples`), never past
 * `floor_level` (the kernel's fact floor: below it the detail ladder draws, not the tiles). 0 when the globe's own
 * faces are already fine enough.
 */
export function patchLevel(m_per_px: number, radius_m: number, samples = TILE_SAMPLES, floor_level = 30): number {
  if (!(m_per_px > 0) || !(radius_m > 0)) return 0;
  const spacing0 = (radius_m * (Math.PI / 2)) / samples;
  const want = m_per_px * PATCH_PX_PER_SAMPLE;
  let l = 0;
  while (l < floor_level && spacing0 / 2 ** l > want) l++;
  return l;
}

/**
 * The tiles at `level` nearest a direction, nearest first: found by looking round the point (so a patch that runs
 * over a cube edge takes the neighbour face's tiles), never by walking a whole level.
 */
export function tilesNear(toward: readonly number[], level: number, max = PATCH_TILES): TileKey[] {
  const l = Math.hypot(toward[0]!, toward[1]!, toward[2]!) || 1;
  const t: V3 = [toward[0]! / l, toward[1]! / l, toward[2]! / l];
  // A frame on the sphere at t, and one tile's angular span at this level.
  const a: V3 = Math.abs(t[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0];
  const e = norm([a[1] * t[2] - a[2] * t[1], a[2] * t[0] - a[0] * t[2], a[0] * t[1] - a[1] * t[0]]);
  const nn: V3 = [t[1] * e[2] - t[2] * e[1], t[2] * e[0] - t[0] * e[2], t[0] * e[1] - t[1] * e[0]];
  const span = Math.PI / 2 / (1 << level);
  const found = new Map<string, { k: TileKey; d: number }>();
  const N = 6;
  for (let j = -N; j <= N; j++) {
    for (let i = -N; i <= N; i++) {
      const s = ((1.6 * span) / N);
      const d = norm([t[0] + i * s * e[0] + j * s * nn[0], t[1] + i * s * e[1] + j * s * nn[1], t[2] + i * s * e[2] + j * s * nn[2]]);
      const f = directionFace(d);
      const n = 1 << level;
      const x = Math.min(n - 1, Math.max(0, Math.floor(((f.u + 1) / 2) * n)));
      const y = Math.min(n - 1, Math.max(0, Math.floor(((f.v + 1) / 2) * n)));
      const id = `${f.face}/${x}/${y}`;
      if (found.has(id)) continue;
      const c = faceDirection(f.face, -1 + (2 * (x + 0.5)) / n, -1 + (2 * (y + 0.5)) / n);
      found.set(id, { k: { face: f.face, level, x, y }, d: c[0] * t[0] + c[1] * t[1] + c[2] * t[2] });
    }
  }
  return [...found.values()]
    .sort((p, q) => q.d - p.d || p.k.face - q.k.face || p.k.y - q.k.y || p.k.x - q.k.x)
    .slice(0, Math.max(0, max))
    .map((o) => o.k);
}

/** The deep patch to read for what the camera looks down on: its level and tiles, or none when level 2 will do. */
export function patchFor(g: GroundAsk, radius_m: number, samples = TILE_SAMPLES, floor_level = 30): { level: number; keys: TileKey[] } {
  const level = patchLevel(g.m_per_px, radius_m, samples, floor_level);
  return level <= 2 ? { level, keys: [] } : { level, keys: tilesNear(g.toward, level) };
}

function norm(v: V3): V3 {
  const l = Math.hypot(v[0], v[1], v[2]) || 1;
  return [v[0] / l, v[1] / l, v[2] / l];
}
