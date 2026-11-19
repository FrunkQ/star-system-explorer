import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Biosphere from "./Biosphere.svelte";

/**
 * Biosphere view and life timeline (U7, life design 9 and 10): one world's
 * life from the kernel's `life_view`. Named epochs and dated events on a time
 * axis with production, the food web by feeding level, traits, what a scan
 * would show at each tier, the dials results lean on, and story hooks. Every
 * reason comes in three lenses (physics, ecology, history). Until adopted
 * systems carry habitat graphs, the kernel's example worlds stand in, and the
 * view says so.
 */
export const biosphere: ViewModule = {
  id: "sse.view.biosphere",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "example", type: "enum", default: "earth", options: ["earth", "earth-archean", "europa", "red-dwarf-desert", "dead-earth"], meaning: "The example world shown when the selected body has no biosphere in the kernel yet.", promoted: true },
    { name: "lens", type: "enum", default: "ecology", options: ["physics", "ecology", "history"], meaning: "The lens reasons are told in first: the physics behind it, what the living world did, or where it sits in the world's history." },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Biosphere, { target: el, props });
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
