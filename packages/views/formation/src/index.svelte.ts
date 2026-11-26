import { mount, unmount } from "svelte";
import { flatChart, type ViewModule } from "../../../ui-contract/src/index.ts";
import FormationScreen from "./FormationScreen.svelte";

/** The standard view controls (interface style rule 12): a flat chart of time against distance. */
export const FORMATION_CONTROLS = flatChart("The formation chart is time against distance: it has no 3D");

/**
 * The Formation screen (formation design section 7, package F10, the reading half): how the system in view was made,
 * as a chart of distance from the star across and time down the page, with the nine stages to jump along, a time
 * slice tied to the time ruler, and what the system inherited from its patch. Read from the system's own formation
 * record (kernel `formation_record`); moving the slice recomputes nothing. Read only.
 */
export const formation: ViewModule = {
  id: "sse.view.formation",
  version: "0.1.0",
  tiers: ["canvas"],
  inputs: [{ event_kinds: ["adopt.formation", "adopt.v3"] }],
  params: [],
  intents: [],
  controls: FORMATION_CONTROLS,
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene });
    const c = mount(FormationScreen, { target: el, props });
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
