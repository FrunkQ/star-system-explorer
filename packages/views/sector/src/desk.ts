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


/** What a sector-wide roll did: worlds rolled, codes written, codes kept (already set), worlds refused. */
export interface SectorRollTally {
  worlds: number;
  written: number;
  kept: number;
  refused: { body: string; message: string }[];
}

type RollAsk = { program: string; body: string; apply?: boolean; at_s?: number | bigint };
type RollAnswer = { ok: true; result: { steps: { how: string }[] } } | { ok: false; code: string; message: string };

/**
 * WBH next: roll one program for every mapped world in one ask, writing each
 * world's rolled codes as hand-set codes (the kernel's `roll_profile` with
 * `apply`). Codes already set (by hand, or by an imported profile, bases
 * included) are kept; only the unset ones roll. One world's refusal never stops the others.
 */
export async function rollEveryWorld(roll: (ask: RollAsk) => Promise<RollAnswer>, program: string, bodies: readonly string[], at_s: number | bigint): Promise<SectorRollTally> {
  const tally: SectorRollTally = { worlds: 0, written: 0, kept: 0, refused: [] };
  for (const body of bodies) {
    const r = await roll({ program, body, apply: true, at_s });
    if (!r.ok) {
      tally.refused.push({ body, message: r.message });
      continue;
    }
    tally.worlds++;
    for (const s of r.result.steps) {
      if (s.how === "rolled" || s.how === "zero") tally.written++;
      else if (s.how === "kept") tally.kept++;
    }
  }
  return tally;
}

/** The tally in words. */
export function rollTallyWords(t: SectorRollTally): string {
  const w = (n: number, one: string) => `${n} ${one}${n === 1 ? "" : "s"}`;
  const parts = [`${w(t.worlds, "world")} rolled: ${w(t.written, "code")} written`];
  if (t.kept) parts.push(`${w(t.kept, "code")} already set kept`);
  if (t.refused.length) parts.push(`${w(t.refused.length, "world")} refused (${t.refused[0]!.message})`);
  return `${parts.join("; ")}.${t.written ? " Each written code says how it was rolled; change any of them by hand." : ""}`;
}
