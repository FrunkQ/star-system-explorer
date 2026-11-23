import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { SECTOR_CONTROLS } from "./controls.ts";
import Sector from "./Sector.svelte";
import WorldSheet from "./WorldSheet.svelte";

/**
 * Sector desk (X3c on the chart room): the campaign's mapped systems on the
 * stacked pack's hex map, the picked world's code sheet, a way into its
 * system, and sector files in (previewed first) and out. Needs a pack with a
 * code sheet and a hex map set-up; says so plainly without one.
 */
export const sectorView: ViewModule = {
  id: "sse.view.sector",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  controls: SECTOR_CONTROLS,
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "hex_size", type: "number", default: 22, range: [8, 80], meaning: "Hex size on screen (drawing units).", promoted: true },
    { name: "reach", type: "number", default: 2, range: [1, 9], meaning: "How far from the picked world to light the hexes, in the map's hex unit (a jump range).", promoted: true },
    { name: "tier", type: "enum", default: "canvas", options: ["canvas", "text"], meaning: "Draw the map, or list the worlds as a table." },
  ],
  intents: [],
  mount(el, ctx) {
    const props = $state({ ctx, params: ctx.params });
    const c = mount(Sector, { target: el, props });
    return {
      update() {},
      dispose() {
        void unmount(c);
      },
    };
  },
};

/** World codes (X3a on the chart room): the world in view read through the stacked code sheet, every digit with its why. */
export const worldSheetView: ViewModule = {
  id: "sse.view.world-sheet",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "mode", type: "enum", default: "auto", options: ["auto", "sheet", "list"], meaning: "The picked world's sheet, a list of every body, or auto: the sheet when a world is picked.", promoted: true },
    { name: "tier", type: "enum", default: "canvas", options: ["canvas", "text"], meaning: "Draw the sheet, or give it as plain sentences." },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(WorldSheet, { target: el, props });
    return {
      update(s) {
        props.scene = s;
      },
      dispose() {
        void unmount(c);
      },
    };
  },
};
