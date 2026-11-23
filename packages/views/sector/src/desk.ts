// The sector desk's pure half (X3c on the chart room): what the stack gives
// (a code sheet, a hex map set-up), the hex map's scene from the kernel's
// `sector_map`, which block to open on, and the file name for an export.
// Nothing here names a game: every word and rule comes from the stacked pack.

import { mapFields, type SectorMap } from "../../../ui-contract/src/index.ts";
import type { HexMapConfig, MapWorld } from "../../hex-map/src/glyphs.ts";
import type { HexScene } from "../../hex-map/src/scene.ts";
import { blockOf, parse } from "../../hex-map/src/grid.ts";

type PackLike = { id?: string; provides?: { other?: Record<string, unknown> } } | null | undefined;
const other = (p: unknown): Record<string, unknown> => ((p as PackLike)?.provides?.other ?? {}) as Record<string, unknown>;

/** The hex map set-up the stack gives (`provides.other.views["sse.view.hex-map"]`; the top pack that has one wins). */
export function hexConfigOf(packs: readonly unknown[]): HexMapConfig | null {
  let found: HexMapConfig | null = null;
  for (const p of packs) {
    const v = (other(p).views as Record<string, unknown> | undefined)?.["sse.view.hex-map"] as HexMapConfig | undefined;
    if (v && v.grid && v.glyphs) found = v;
  }
  return found;
}

/** The hex map's worlds, one per mapped system, carrying the fields the glyph rules read. */
export function mapWorlds(map: SectorMap): MapWorld[] {
  return map.worlds.map((w) => ({ hex: w.hex, name: w.name || w.hex, fields: mapFields(w.reading, w.name || w.hex), address: w.system }));
}

/** Blocks that hold at least one world, in block order, with their names. */
export function blocksInUse(config: HexMapConfig, worlds: readonly MapWorld[]): { index: number; name: string; worlds: number }[] {
  const seen = new Map<number, { index: number; name: string; worlds: number }>();
  for (const w of worlds) {
    const h = parse(config.grid, w.hex);
    if (!h) continue;
    const b = blockOf(config.grid, h);
    const e = seen.get(b.index) ?? { ...b, worlds: 0 };
    e.worlds++;
    seen.set(b.index, e);
  }
  return [...seen.values()].sort((a, b) => a.index - b.index);
}

/** The block to open on: the only one in use, else the whole sheet (undefined). */
export function openingBlock(config: HexMapConfig, worlds: readonly MapWorld[]): number | undefined {
  const used = blocksInUse(config, worlds);
  return used.length === 1 ? used[0]!.index : undefined;
}

export function sceneOf(config: HexMapConfig, worlds: MapWorld[], opts: { block?: number; selected?: string; range?: number; title?: string } = {}): HexScene {
  return { config, worlds, ...opts };
}

/** A file name for an export: the sector's name, else "sector", with the format's extension. */
export function exportName(map: SectorMap | null, ext: string): string {
  const name = map?.worlds.find((w) => w.sector)?.sector ?? "sector";
  return `${name.replace(/[^A-Za-z0-9 _-]+/g, "").trim().replace(/\s+/g, "-") || "sector"}.${ext}`;
}

