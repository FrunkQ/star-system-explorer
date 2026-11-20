import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import HexMap from "./HexMap.svelte";
import type { HexScene } from "./scene.ts";

/**
 * Hex map (X3c): a sheet of hexes with worlds drawn by glyph rules, all
 * configured by data (grid size, block size, labels, marks), so a game pack
 * makes it its map and any other pack can use it with its own grid. Picking
 * a hex highlights the hexes within the given range (jump range). Text tier:
 * the worlds as a table.
 */
export const hexMap: ViewModule<HexScene> = {
  id: "sse.view.hex-map",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation"] }],
  params: [
    { name: "hex_size", type: "number", default: 22, range: [8, 80], meaning: "Hex size on screen (drawing units).", promoted: true },
    { name: "tier", type: "enum", default: "canvas", options: ["canvas", "text"], meaning: "Draw the map, or list the worlds as a table." },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ scene, params: ctx.params as Record<string, unknown> });
    const c = mount(HexMap, { target: el, props });
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
