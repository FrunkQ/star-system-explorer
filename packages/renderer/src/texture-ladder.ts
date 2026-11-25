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
import { groundStains, stainTexel, type StainHue } from "./stain-look.ts";
import { say, SURFACE_COVERS as COVERS, sampleColour, type SurfaceTile, type Tier } from "../../ui-contract/src/index.ts";

/** The kernel's samples along a tile's edge (atlas settings `tile_samples`: 256 on the sample Sol; read `tile.samples` where a tile is at hand). */
export const TILE_SAMPLES = 256;
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
  /** The stains laid on its bare ground (stain-look.ts, TRUE4), kept so finer tiles are stained alike. */
  stains?: readonly StainHue[];
}

/**
 * Lay the tiles read so far into one picture per face. Each face's picture is as fine as its deepest tile (up to
 * COMPOSE_MAX texels); coarser tiles are laid first and finer ones over them, so a part of the face read only at
 * level 0 still shows, at its own coarseness. A face with no tile is null (drawn plain, as today).
 */
export function composeFaces(tiles: readonly SurfaceTile[], stains: readonly StainHue[] = []): Composite {
  const ground = groundStains(stains);
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
      // Each sample's colour (stained) and height first; a coarse tile is then spread over its texels bilinearly
      // between sample centres, not in blocks, so a level-0 tile under finer ones has no stair-stepped edges
      // (MATCH1, 25 Nov: hard pixel edges on basin discs).
      const tc = new Uint8Array(ts * ts * 3);
      const th = new Float32Array(ts * ts);
      const tw = new Uint8Array(ts * ts);
      for (let k = 0; k < ts * ts; k++) {
        let c: readonly [number, number, number] = sampleColour(t, k, 1);
        if (ground.length && t.cover[k] === 0) {
          const one = new Uint8Array(c);
          stainTexel(one, 0, ground);
          c = [one[0]!, one[1]!, one[2]!];
        }
        tc[k * 3] = c[0];
        tc[k * 3 + 1] = c[1];
        tc[k * 3 + 2] = c[2];
        const wet = (COVERS[t.cover[k]!] ?? "none") !== "none" && COVERS[t.cover[k]!] !== "ice" ? 1 : 0;
        tw[k] = wet;
        th[k] = wet ? 0 : t.elevation_dm[k]! / 10;
      }
      const c0 = Math.round(x0);
      const r0 = Math.round(y0);
      const c1 = Math.min(n, Math.round(x0 + ts * scale));
      const r1 = Math.min(n, Math.round(y0 + ts * scale));
      for (let row = r0; row < r1; row++) {
        const y = Math.min(ts - 1, Math.max(0, (row + 0.5 - y0) / scale - 0.5));
        const j = Math.max(0, Math.min(ts - 2, Math.floor(y)));
        const fy = ts > 1 ? y - j : 0;
        for (let col = c0; col < c1; col++) {
          const x = Math.min(ts - 1, Math.max(0, (col + 0.5 - x0) / scale - 0.5));
          const i = Math.max(0, Math.min(ts - 2, Math.floor(x)));
          const fx = ts > 1 ? x - i : 0;
          const i1 = Math.min(ts - 1, i + 1);
          const j1 = Math.min(ts - 1, j + 1);
          const a = j * ts + i;
          const b = j * ts + i1;
          const cc = j1 * ts + i;
          const d = j1 * ts + i1;
          const w00 = (1 - fx) * (1 - fy);
          const w10 = fx * (1 - fy);
          const w01 = (1 - fx) * fy;
          const w11 = fx * fy;
          const o = row * n + col;
          for (let ch = 0; ch < 3; ch++) rgb[o * 3 + ch] = Math.round(tc[a * 3 + ch]! * w00 + tc[b * 3 + ch]! * w10 + tc[cc * 3 + ch]! * w01 + tc[d * 3 + ch]! * w11);
          height_m[o] = th[a]! * w00 + th[b]! * w10 + th[cc]! * w01 + th[d]! * w11;
          // Sea or not is the nearest sample's (a shore is not half wet).
          liquid[o] = tw[(fy < 0.5 ? j : j1) * ts + (fx < 0.5 ? i : i1)]!;
        }
      }
    }
    return { n, rgb, height_m, liquid, level };
  });
  return { faces, signature: sig.join("|"), radius_m, tiles, ...(ground.length ? { stains } : {}) };
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
  return { ...tex, faces: composeFaces([...tex.faces.tiles, ...more], tex.faces.stains) };
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
  const { t, e, n: nn } = frameAt(toward);
  // One tile's angular span at this level.
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

/** A frame on the sphere at a direction: the point, a sideways direction and a third at right angles to both. */
export function frameAt(toward: readonly number[]): { t: V3; e: V3; n: V3 } {
  const t = norm([toward[0]!, toward[1]!, toward[2]!]);
  const a: V3 = Math.abs(t[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0];
  const e = norm([a[1] * t[2] - a[2] * t[1], a[2] * t[0] - a[0] * t[2], a[0] * t[1] - a[1] * t[0]]);
  const n: V3 = [t[1] * e[2] - t[2] * e[1], t[2] * e[0] - t[0] * e[2], t[0] * e[1] - t[1] * e[0]];
  return { t, e, n };
}

/** Texels along the deep patch's edge. */
export const PATCH_N = 512;

/**
 * The deep patch as a picture (VP3 part 4): a square on the sphere about `centre`, laid out gnomonically (texel (i, j)
 * looks along centre + a·east + b·north with a, b from -half to +half, row 0 at b = -half), read from tiles of one
 * level. `rgba` is the sRGB colour with alpha 255 where a tile was read and 0 where none was; `normal` is the relief
 * normal in the body's frame, xyz a texel (seas flat), from the true slopes between neighbouring texels; kept in
 * floats because a byte a component would band gentle slopes (its steps are about half a degree).
 */
export interface PatchPicture {
  n: number;
  rgba: Uint8Array;
  normal: Float32Array;
  centre: V3;
  east: V3;
  north: V3;
  half: number;
  level: number;
  signature: string;
}

/** The direction a patch texel looks along (a, b in the gnomonic plane, -half to +half). */
export function patchDirection(p: Pick<PatchPicture, "centre" | "east" | "north">, a: number, b: number): V3 {
  return norm([p.centre[0] + a * p.east[0] + b * p.north[0], p.centre[1] + a * p.east[1] + b * p.north[1], p.centre[2] + a * p.east[2] + b * p.north[2]]);
}

/** A reader of one level's tiles at any direction: bilinear across tile edges on a face, null where a tile is missing. */
function levelReader(tiles: readonly SurfaceTile[], level: number) {
  const n = 1 << level;
  const byKey = new Map<number, SurfaceTile>();
  let S = 0;
  for (const t of tiles) if (t.key.level === level) (byKey.set((t.key.face * n + t.key.y) * n + t.key.x, t), (S = t.samples));
  // Each tile's samples unpacked once, on first touch: r, g, b, height (seas 0), wet.
  const unpacked = new Map<number, Float32Array>();
  const unpack = (id: number, t: SurfaceTile) => {
    const a = new Float32Array(S * S * 5);
    for (let k = 0; k < S * S; k++) {
      const c = sampleColour(t, k, 1);
      const cov = COVERS[t.cover[k]!] ?? "none";
      const wet = cov !== "none" && cov !== "ice";
      a.set([c[0], c[1], c[2], wet ? 0 : t.elevation_dm[k]! / 10, wet ? 1 : 0], k * 5);
    }
    unpacked.set(id, a);
    return a;
  };
  const N = n * S;
  const sample = (face: number, gx: number, gy: number): number => {
    const ix = Math.min(N - 1, Math.max(0, gx));
    const iy = Math.min(N - 1, Math.max(0, gy));
    const id = (face * n + Math.floor(iy / S)) * n + Math.floor(ix / S);
    let a = unpacked.get(id);
    if (!a) {
      const t = byKey.get(id);
      if (!t) return -1;
      a = unpack(id, t);
    }
    cur = a;
    return ((iy % S) * S + (ix % S)) * 5;
  };
  let cur: Float32Array = new Float32Array(0);
  return (d: readonly number[]) => {
    if (!S) return null;
    const f = directionFace(d);
    const gx = ((f.u + 1) / 2) * n * S - 0.5;
    const gy = ((f.v + 1) / 2) * n * S - 0.5;
    const x0 = Math.floor(gx);
    const y0 = Math.floor(gy);
    const fx = gx - x0;
    const fy = gy - y0;
    const o00 = sample(f.face, x0, y0);
    const a00 = cur;
    const o10 = sample(f.face, x0 + 1, y0);
    const a10 = cur;
    const o01 = sample(f.face, x0, y0 + 1);
    const a01 = cur;
    const o11 = sample(f.face, x0 + 1, y0 + 1);
    const a11 = cur;
    if (o00 < 0 || o10 < 0 || o01 < 0 || o11 < 0) return null;
    const mix = (k: number) => (a00[o00 + k]! * (1 - fx) + a10[o10 + k]! * fx) * (1 - fy) + (a01[o01 + k]! * (1 - fx) + a11[o11 + k]! * fx) * fy;
    return { rgb: [mix(0), mix(1), mix(2)] as V3, h: mix(3), wet: a00[o00 + 4]! + a10[o10 + 4]! + a01[o01 + 4]! + a11[o11 + 4]! === 4 };
  };
}

/**
 * The deep patch from its tiles (all one level, the deepest given is used), about `centre`, `n` texels a side. It
 * spans the tiles read round the centre (a tile and a half each way at that level); texels no tile covers keep
 * alpha 0, so the globe's own faces show there. Null with no tiles.
 */
export function composePatch(tiles: readonly SurfaceTile[], centre: readonly number[], radius_m: number, n = PATCH_N): PatchPicture | null {
  if (!tiles.length) return null;
  const level = Math.max(...tiles.map((t) => t.key.level));
  const read = levelReader(tiles, level);
  const { t, e, n: nn } = frameAt(centre);
  const half = Math.tan((1.5 * Math.PI) / 2 / (1 << level));
  const frame = { centre: t, east: e, north: nn };
  const step = (2 * half) / n;
  // Heights on a grid one texel wider each way, for the slopes at the patch's edge.
  const W = n + 2;
  const hs = new Float64Array(W * W).fill(NaN);
  const rgba = new Uint8Array(n * n * 4);
  const normal = new Float32Array(n * n * 3);
  const wetAt = new Uint8Array(n * n);
  const dirs = new Float64Array(W * W * 3);
  for (let j = -1; j <= n; j++) {
    for (let i = -1; i <= n; i++) {
      const d = patchDirection(frame, -half + (i + 0.5) * step, -half + (j + 0.5) * step);
      dirs.set(d, ((j + 1) * W + (i + 1)) * 3);
      const r = read(d);
      if (!r) continue;
      hs[(j + 1) * W + (i + 1)] = r.h;
      if (i < 0 || j < 0 || i >= n || j >= n) continue;
      const o = (j * n + i) * 4;
      rgba[o] = Math.round(r.rgb[0]);
      rgba[o + 1] = Math.round(r.rgb[1]);
      rgba[o + 2] = Math.round(r.rgb[2]);
      rgba[o + 3] = 255;
      wetAt[j * n + i] = r.wet ? 1 : 0;
    }
  }
  for (let j = 0; j < n; j++) {
    for (let i = 0; i < n; i++) {
      const o = (j * n + i) * 4;
      const at = (di: number, dj: number) => ((j + 1 + dj) * W + (i + 1 + di)) * 3;
      const c = at(0, 0);
      let o0 = dirs[c]!;
      let o1 = dirs[c + 1]!;
      let o2 = dirs[c + 2]!;
      const h = (di: number, dj: number) => hs[(j + 1 + dj) * W + (i + 1 + di)]!;
      if (rgba[o + 3] && !wetAt[j * n + i]) {
        const pa = at(1, 0);
        const ma = at(-1, 0);
        const pb = at(0, 1);
        const mb = at(0, -1);
        const ta0 = dirs[pa]! - dirs[ma]!;
        const ta1 = dirs[pa + 1]! - dirs[ma + 1]!;
        const ta2 = dirs[pa + 2]! - dirs[ma + 2]!;
        const tb0 = dirs[pb]! - dirs[mb]!;
        const tb1 = dirs[pb + 1]! - dirs[mb + 1]!;
        const tb2 = dirs[pb + 2]! - dirs[mb + 2]!;
        const la = Math.hypot(ta0, ta1, ta2);
        const lb = Math.hypot(tb0, tb1, tb2);
        // One-sided at the edge of what was read.
        const ga = slope(h(1, 0), h(0, 0), h(-1, 0), la * radius_m) / la;
        const gb = slope(h(0, 1), h(0, 0), h(0, -1), lb * radius_m) / lb;
        o0 -= ta0 * ga + tb0 * gb;
        o1 -= ta1 * ga + tb1 * gb;
        o2 -= ta2 * ga + tb2 * gb;
        const l = Math.hypot(o0, o1, o2);
        o0 /= l;
        o1 /= l;
        o2 /= l;
      }
      const out: V3 = [o0, o1, o2];
      const q = (j * n + i) * 3;
      normal[q] = out[0];
      normal[q + 1] = out[1];
      normal[q + 2] = out[2];
    }
  }
  const signature = tiles.filter((x) => x.key.level === level).map((x) => `${x.key.face}/${x.key.x}/${x.key.y}:${x.hash}`).sort().join("|") + `@${t.map((x) => Math.round(x * 1e6)).join(",")}`;
  return { n, rgba, normal, ...frame, half, level, signature };
}

/** Slope across a texel either side (a span `l` metres), one-sided where a neighbour was not read. */
function slope(p: number, c: number, m: number, l: number): number {
  if (Number.isFinite(p) && Number.isFinite(m)) return (p - m) / l;
  if (Number.isFinite(p)) return (2 * (p - c)) / l;
  if (Number.isFinite(m)) return (2 * (c - m)) / l;
  return 0;
}

/** The processor twin of the GPU's patch read: the texel a direction falls on (nearest), or null outside it. */
export function samplePatch(p: PatchPicture, d: readonly number[]): { rgba: [number, number, number, number]; normal: V3 } | null {
  const k = p.centre[0] * d[0]! + p.centre[1] * d[1]! + p.centre[2] * d[2]!;
  if (k <= 0) return null;
  const a = (p.east[0] * d[0]! + p.east[1] * d[1]! + p.east[2] * d[2]!) / k;
  const b = (p.north[0] * d[0]! + p.north[1] * d[1]! + p.north[2] * d[2]!) / k;
  const i = Math.floor(((a / p.half + 1) / 2) * p.n);
  const j = Math.floor(((b / p.half + 1) / 2) * p.n);
  if (i < 0 || j < 0 || i >= p.n || j >= p.n) return null;
  const o = (j * p.n + i) * 4;
  const q = (j * p.n + i) * 3;
  return { rgba: [p.rgba[o]!, p.rgba[o + 1]!, p.rgba[o + 2]!, p.rgba[o + 3]!], normal: [p.normal[q]!, p.normal[q + 1]!, p.normal[q + 2]!] };
}

function norm(v: V3): V3 {
  const l = Math.hypot(v[0], v[1], v[2]) || 1;
  return [v[0] / l, v[1] / l, v[2] / l];
}
