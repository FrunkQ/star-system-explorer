import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { COMPARE_CONTROLS } from "./controls.ts";
import Compare from "./Compare.svelte";

/**
 * Size comparison (V14; V3's most used tool, kept): bodies side by side on
 * one baseline at one true scale (the view controls' Real, where it opens; Readable is a log scale), from this system, any system in the
 * campaign, or stock reference objects (one of each standard planet and star
 * type, measured radii with references). Text tier: the same lineup as a
 * table in words.
 */
export const compare: ViewModule = {
  id: "sse.view.compare",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  controls: COMPARE_CONTROLS,
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "limb_ratio", type: "number", default: 5, range: [2, 1000], meaning: "When the largest body is this many times the next, draw it as a limb behind the row, as on a poster." },
    { name: "references", type: "string", default: "documents/reference-objects.json", meaning: "Reference objects offered beside the universe's own bodies." },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Compare, { target: el, props });
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
