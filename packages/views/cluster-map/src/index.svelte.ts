import { mount, unmount } from "svelte";
import { flatChart, type ViewModule } from "../../../ui-contract/src/index.ts";
import ClusterMap from "./ClusterMap.svelte";

/**
 * The cluster map (V9, galaxy design 2 and 4): one halo cell of the universe
 * (8 Mpc a side at pack defaults), its galaxies' dark-matter haloes drawn by
 * the kernel's `halo_cell` (G2) on the cosmic web through the cell, with the
 * mass function they came from. Pick a galaxy to read its history; "Chart
 * this galaxy" opens it in the galaxy chart. Read only.
 */
export const clusterMap: ViewModule = {
  id: "sse.view.cluster-map",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  controls: flatChart("3D cells come with the galaxy charts' Free view"),
  inputs: [{ event_kinds: [] }],
  params: [
    { name: "min_mass", type: "enum", default: "1e10", options: ["1e10", "1e11", "1e12"], meaning: "Smallest halo shown, solar masses.", promoted: true, group: "content" },
    { name: "web", type: "boolean", default: true, meaning: "Draw the cosmic web through the cell's middle behind the haloes.", group: "look", cost: "medium" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(ClusterMap, { target: el, props });
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
