import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import GalaxyChart from "./GalaxyChart.svelte";

/**
 * The galaxy chart (U6, galaxy design 11): one galaxy grown from the
 * universe's seed by the kernel's `galaxy_chart` (G1 to G4). Face-on, with
 * its rings shaded by a chosen layer (metals, heavy metals, stars,
 * supernovae), the arm crests and the green band at the chosen time, the
 * history curves on cosmic time with the epochs, and for a picked ring its
 * ledger: what died there and what conditions are like. Read only.
 */
export const galaxyChart: ViewModule = {
  id: "sse.view.galaxy-chart",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: [] }],
  params: [
    { name: "halo", type: "enum", default: "milky-way-like", options: ["dwarf", "milky-way-like", "group"], meaning: "Which reference galaxy to chart first: the median halo of 1e11, 1e12 or 1e13 solar masses grown from the universe's seed.", promoted: true, group: "content" },
    { name: "layer", type: "enum", default: "metals", options: ["metals", "heavy", "stars", "supernovae"], meaning: "What the rings are shaded by.", promoted: true, group: "look" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(GalaxyChart, { target: el, props });
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
