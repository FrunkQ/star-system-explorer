import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { GALAXY_CONTROLS, YOUR_SYSTEMS_CONTROLS } from "./controls.ts";
import GalaxyChart from "./GalaxyChart.svelte";
import YourSystems from "./YourSystems.svelte";

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
  tiers: ["text", "canvas", "gpu-basic", "gpu-high"],
  controls: GALAXY_CONTROLS,
  inputs: [{ event_kinds: [] }],
  params: [
    { name: "halo", type: "enum", default: "home", options: ["home", "dwarf", "milky-way-like", "group"], meaning: "Which galaxy to chart first: our own (the universe's home galaxy, one of the cosmic web's), or a reference galaxy, the median halo of 1e11, 1e12 or 1e13 solar masses grown from the universe's seed.", promoted: true, group: "content" },
    { name: "place", type: "string", default: "", meaning: "The galaxy to open on: a reference halo's key or a catalogued halo's address; from a link on the place line.", group: "content" },
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

/**
 * Your systems (V3 gaps 17 and 39, U6 part 2b part 1): the campaign's systems on one map, each where its own data
 * puts it, with a GM-only hover saying what it holds; a press opens the system. Read only.
 */
export const yourSystems: ViewModule = {
  id: "sse.view.your-systems",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  controls: YOUR_SYSTEMS_CONTROLS,
  inputs: [{ event_kinds: [] }],
  params: [],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(YourSystems, { target: el, props });
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
