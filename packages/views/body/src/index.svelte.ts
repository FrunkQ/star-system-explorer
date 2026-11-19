import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { cutaway } from "../../cutaway/src/index.ts";
import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";
import Body from "./Body.svelte";

/**
 * Body view (V6, handoff 8.2): the selected world as a globe drawn from its
 * S1 surface tiles, with the Cutaway of its layers below. The globe is a
 * render recipe (R1, packages/renderer): "gpu-basic" draws it on the GPU
 * (WebGPU, else WebGL2), "canvas" on a 2D canvas, "text" in words. The
 * governor steps quality down when frames miss the budget, and the quality
 * badge says what it did.
 */
export const body: ViewModule = {
  id: "sse.view.body",
  version: "0.1.0",
  tiers: ["text", "canvas", "gpu-basic"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "size_px", type: "number", default: 320, range: [160, 640], unit: "px", meaning: "Globe diameter on screen.", cost: "medium", group: "look" },
    { name: "recipe", type: "enum", default: "survey-relief", options: SHIPPED_RECIPES.map((r) => r.id), meaning: "The render recipe that draws the globe (packages/renderer/src/recipes). A recipe without the tier in use hands over to one of its family that has it.", promoted: true, group: "look" },
    { name: "tier", type: "enum", default: "auto", options: ["auto", "gpu-basic", "canvas", "text"], meaning: "Where the globe is drawn: auto lets this device choose; gpu-basic uses WebGPU or WebGL2; canvas a 2D canvas; text words only.", cost_options: { "gpu-basic": "medium", canvas: "light" }, group: "behaviour" },
    { name: "relief", type: "number", default: 0.5, range: [0, 1], meaning: "The recipe's Relief knob: how strongly the land's shape shows.", promoted: true, group: "look" },
    { name: "detail", type: "number", default: 0.5, range: [0, 1], meaning: "The recipe's Detail knob (mesh fineness on the GPU, contour spacing on a contour sheet).", cost: "medium", group: "look" },
    { name: "budget_ms", type: "number", default: 33.3, range: [8, 200], unit: "ms", meaning: "Frame budget the governor holds (33 ms is 30 frames a second).", group: "behaviour" },
    { name: "governor", type: "boolean", default: true, meaning: "Step quality down when frames miss the budget, and back up when they fit again.", group: "behaviour" },
    { name: "spin", type: "boolean", default: false, meaning: "Turn the globe slowly (a presentation drift, not the world's real rotation).", cost: "medium", group: "behaviour" },
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
