import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import FoamMap from "./FoamMap.svelte";

/**
 * The foam map (V9, galaxy design 2 and 11): a slice of the cosmic web grown
 * from the universe's seed by the kernel's `web_slice` (G1), at any time from
 * the first billion years to now. Scrubbing or playing time animates the foam
 * growing: voids empty, sheets and filaments sharpen, knots gather. Read only.
 */
export const foamMap: ViewModule = {
  id: "sse.view.foam-map",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: [] }],
  params: [
    { name: "extent_mpc", type: "enum", default: "200", options: ["50", "100", "200", "400"], meaning: "Side of the square slice, megaparsecs.", promoted: true, group: "content" },
    { name: "place", type: "string", default: "", meaning: "The slice to open on (web:<side in Mpc>); from a link on the place line.", group: "content" },
    { name: "samples", type: "integer", default: 96, range: [16, 160], meaning: "Samples a side (cost grows with the square).", group: "look", cost: "medium" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(FoamMap, { target: el, props });
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
