import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { cutaway } from "../../cutaway/src/index.ts";
import Body from "./Body.svelte";

/**
 * Body view (V6, handoff 8.2): the selected world as a globe drawn from its
 * S1 surface tiles, with the Cutaway of its layers below. Tiers: "canvas"
 * draws the globe on a 2D canvas (no GPU needed); "text" gives the globe's
 * numbers in words and the cutaway's ladder.
 */
export const body: ViewModule = {
  id: "sse.view.body",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "size_px", type: "number", default: 320, range: [160, 640], unit: "px", meaning: "Globe diameter on screen.", cost: "medium" },
    { name: "cutaway", type: "boolean", default: true, meaning: "Show the Cutaway of the world's layers below the globe.", promoted: true },
    ...cutaway.params,
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Body, { target: el, props });
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
