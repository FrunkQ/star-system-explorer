import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Cutaway from "./Cutaway.svelte";

/**
 * Cutaway (V6, handoff 8.2): a world's interior and atmosphere shells,
 * regional shells under their region, and features at their heights, to
 * scale or on a log depth scale; beside it the "above, on and within"
 * ladder. Text tier: the ladder alone. Draws only what the world stack says.
 */
export const cutaway: ViewModule = {
  id: "sse.view.cutaway",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set"] }],
  params: [
    { name: "scale", type: "enum", default: "log", options: ["log", "true"], meaning: "Depth to scale (true) or spread on a log scale so thin layers show (log).", promoted: true },
    { name: "wedge_deg", type: "number", default: 140, range: [60, 360], unit: "deg", meaning: "How much of the world the cut shows." },
    { name: "ladder", type: "boolean", default: true, meaning: "Show the above, on and within ladder beside the cut.", promoted: true },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Cutaway, { target: el, props });
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
