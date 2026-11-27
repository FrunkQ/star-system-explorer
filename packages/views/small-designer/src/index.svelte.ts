import { mount, unmount } from "svelte";
import { WHY_WIREFRAME_LATER, type ViewControlsSpec, type ViewModule } from "../../../ui-contract/src/index.ts";
import SmallDesigner from "./SmallDesigner.svelte";

/** The designer shows the body itself, turning in True (interface style rule 12: it declares, the shell draws the switch). */
export const SMALL_DESIGNER_CONTROLS: ViewControlsSpec = {
  styles: { chart: "The designer shows the body itself, in True", "3d": "The designer shows the body itself, in True", true: true, wireframe: WHY_WIREFRAME_LATER },
};

/**
 * The small-body designer (RW16a, real catalogues design 6A.1): pick a kind (asteroid, comet, contact binary, pair),
 * set its size, what it is made of, its spin, rubble or solid and how battered, and watch it turn in True as the
 * numbers change. Every control is a pin on the kernel's one resolve (`make_small_body`), so what is drawn is what the
 * physics makes of it; an implausible pin is kept and the panel says what it would take. Nothing is stored.
 */
export const smallDesigner: ViewModule = {
  id: "sse.view.small-designer",
  version: "0.1.0",
  tiers: ["gpu-basic", "canvas"],
  inputs: [{ addresses: [], quantity_kinds: [] }],
  params: [],
  intents: [],
  controls: SMALL_DESIGNER_CONTROLS,
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene });
    const c = mount(SmallDesigner, { target: el, props });
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
