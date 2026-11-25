// Finer surface tiles for a world (VP3 part 2), read a few at a time and kept for every screen that draws the world
// (System True now; the Body screen and the body card's globe can read the same map), so nothing is built twice.
//
// The kernel takes about 0.2 s a tile, and every chart click waits behind the worker, so the reader asks for four
// tiles at a time (about 0.8 s), checks between requests that it is still wanted, starts only after the world has
// been chosen and left alone for a while (longer than a camera fly), and pauses again before the finer level.
import type { SurfaceTile } from "../../../ui-contract/src/index.ts";
import { composePatch, patchFor, type GroundAsk, type PatchPicture, tilesFacing, type TileKey } from "../../../renderer/src/texture-ladder.ts";
import { floorFromRefusal } from "../../../renderer/src/surface-patch.ts";

/** Tiles per request: about 0.8 s of kernel time on the sample Sol. */
export const FINER_BATCH = 4;
/** Wait after a world is chosen before reading (a camera fly takes 1 s). */
export const FINER_SETTLE_MS = 1200;
/** Pause before the next level. */
export const FINER_PAUSE_MS = 1500;
/** The levels read: 1 (512 samples a face with the kernel's 256-sample tiles) and 2 (1024, COMPOSE_MAX). */
export const FINER_LEVELS: readonly number[] = [1, 2];

/** Keyed `${system}|${body}`: every finer tile read so far (level 0 stays with each screen's own read). */
const kept = new Map<string, Map<string, SurfaceTile>>();
/** Tiles the kernel refused (no surface there): not asked again this session. */
const refused = new Set<string>();
const tileKey = (k: TileKey) => `${k.face}/${k.level}/${k.x}/${k.y}`;

/** The finer tiles read so far for a world, in a stable order. */
export function finerTiles(key: string): SurfaceTile[] {
  const m = kept.get(key);
  return m ? [...m.values()] : [];
}

export interface FinerReader {
  surfaceTilesAt(system: string, body: string, keys: readonly TileKey[]): Promise<{ ok: true; tiles: SurfaceTile[]; refused?: readonly { why: string }[] } | { ok: false; code: string; message: string }>;
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

/**
 * Read the finer levels for one world, nearest the `toward` direction first (body frame; the whole globe is read in
 * the end), FINER_BATCH tiles a request. `wanted()` is asked before each request and after each wait; once it says
 * no, the reader stops and keeps what it has (a later call carries on from there). `got` is called with all finer
 * tiles kept after each request that brought new ones. Returns how many tiles it read.
 */
export async function readFiner(session: FinerReader, system: string, body: string, wanted: () => boolean, got: (tiles: SurfaceTile[]) => void, toward: readonly number[] = [1, 0, 0]): Promise<number> {
  const key = `${system}|${body}`;
  let m = kept.get(key);
  if (!m) kept.set(key, (m = new Map()));
  let read = 0;
  await sleep(FINER_SETTLE_MS);
  for (const level of FINER_LEVELS) {
    if (!wanted()) return read;
    const have = (k: TileKey) => m!.has(tileKey(k)) || refused.has(`${key}|${tileKey(k)}`);
    const todo = tilesFacing(toward, level, Infinity).filter((k) => !have(k));
    // tilesFacing leaves out the far side; read it last so the whole globe is fine in the end.
    const n = 1 << level;
    for (let face = 0; face < 6; face++) for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
      const k = { face, level, x, y };
      if (!have(k) && !todo.some((t) => tileKey(t) === tileKey(k))) todo.push(k);
    }
    if (!todo.length) continue;
    if (level > FINER_LEVELS[0]!) {
      await sleep(FINER_PAUSE_MS);
      if (!wanted()) return read;
    }
    for (let i = 0; i < todo.length; i += FINER_BATCH) {
      if (!wanted()) return read;
      const r = await session.surfaceTilesAt(system, body, todo.slice(i, i + FINER_BATCH));
      if (!r.ok) return read;
      for (const t of r.tiles) m.set(tileKey(t.key), t);
      read += r.tiles.length;
      if (r.tiles.length && wanted()) got(finerTiles(key));
      for (const k of todo.slice(i, i + FINER_BATCH)) if (!m.has(tileKey(k))) refused.add(`${key}|${tileKey(k)}`);
    }
  }
  return read;
}

// ------------------------------------------------------------------ the deep patch under a close camera (VP3 part 4)

/** Keyed `${system}|${body}`: deep tiles read so far, by tile key (all levels). */
const deep = new Map<string, Map<string, SurfaceTile>>();
/** The kernel's fact floor per world, once a refusal has named it: no tile deeper than this is asked for. */
const floors = new Map<string, number>();

/**
 * Read the deep patch for what the camera looks down on (`ask`: the body-frame direction under it and the ground's
 * metres a screen pixel there), FINER_BATCH tiles a request, nearest first, and hand back its picture. The level
 * is the coarsest fine enough for the pixels (texture-ladder.ts `patchFor`), never past the kernel's fact floor (a
 * refusal names it, and it is kept for the world). `wanted()` is asked before each request; null when there is
 * nothing deeper than the globe's own faces to read, or nothing was read.
 */
export async function readDeep(session: FinerReader, system: string, body: string, ask: GroundAsk, radius_m: number, samples: number, wanted: () => boolean): Promise<PatchPicture | null> {
  const key = `${system}|${body}`;
  let m = deep.get(key);
  if (!m) deep.set(key, (m = new Map()));
  let plan = patchFor(ask, radius_m, samples, floors.get(key) ?? 30);
  if (!plan.keys.length) return null;
  for (let pass = 0; pass < 2; pass++) {
    const todo = plan.keys.filter((k) => !m!.has(tileKey(k)) && !refused.has(`${key}|${tileKey(k)}`));
    let floor: number | null = null;
    for (let i = 0; i < todo.length; i += FINER_BATCH) {
      if (!wanted()) return null;
      const r = await session.surfaceTilesAt(system, body, todo.slice(i, i + FINER_BATCH));
      if (!r.ok) return null;
      for (const t of r.tiles) m.set(tileKey(t.key), t);
      for (const x of r.refused ?? []) floor ??= floorFromRefusal(x.why);
      if (floor !== null) break;
      for (const k of todo.slice(i, i + FINER_BATCH)) if (!m.has(tileKey(k))) refused.add(`${key}|${tileKey(k)}`);
    }
    if (floor === null) break;
    // Below the fact floor: read at the floor instead (the detail ladder draws below it).
    floors.set(key, floor);
    plan = patchFor(ask, radius_m, samples, floor);
    if (!plan.keys.length) return null;
  }
  if (!wanted()) return null;
  const tiles = plan.keys.map((k) => m!.get(tileKey(k))).filter((t): t is SurfaceTile => !!t);
  return composePatch(tiles, ask.toward, radius_m);
}
