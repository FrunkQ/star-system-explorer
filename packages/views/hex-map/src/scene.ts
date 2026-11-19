// The hex map's scene: plain data the kernel (or an import preview) builds.
import type { HexMapConfig, MapWorld } from "./glyphs.ts";

export interface HexRoute {
  from: string;
  to: string;
  kind: string;
}

export interface HexScene {
  config: HexMapConfig;
  worlds: MapWorld[];
  routes?: HexRoute[];
  /** Hex label of the selected world, and a range to highlight from it (in hexes). */
  selected?: string;
  range?: number;
  /** Which block to show (index), or the whole sheet when absent. */
  block?: number;
  /** Words for the title line. */
  title?: string;
}
