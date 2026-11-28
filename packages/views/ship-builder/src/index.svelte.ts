import { mount, unmount } from "svelte";
import { WHY_WIREFRAME_LATER, type ViewControlsSpec, type ViewModule } from "../../../ui-contract/src/index.ts";
import BuilderScreen from "./BuilderScreen.svelte";

/** The builder draws its own plans and 3D model (interface style rule 12: it declares, the shell draws the switch). */
export const SHIP_BUILDER_CONTROLS: ViewControlsSpec = {
  styles: { chart: "The builder draws the ship's plans and model itself", "3d": "The builder draws the ship's plans and model itself", true: "The builder draws the ship's plans and model itself", wireframe: WHY_WIREFRAME_LATER },
};

/**
 * The ship builder (CM14a, compound ships design 8A.4 and rulings 14A.2 to 14A.4): start from a catalogue ship or
 * nothing, clip palette parts at their connection points, stretch and widen them, and either watch what the ship can
 * do as it grows ("Build it") or fill a plan's budgets ("Build to a plan"). The kernel judges it; nothing is stored here.
 */
export const shipBuilder: ViewModule = {
  id: "sse.view.ship-builder",
  version: "0.1.0",
  tiers: ["gpu-basic", "canvas"],
  inputs: [{ addresses: [], quantity_kinds: [] }],
  params: [],
  intents: [],
  controls: SHIP_BUILDER_CONTROLS,
  mount(el, ctx) {
    const props = $state({ ctx });
    const c = mount(BuilderScreen, { target: el, props });
    return {
      update() {},
      dispose() {
        void unmount(c);
      },
    };
  },
};
