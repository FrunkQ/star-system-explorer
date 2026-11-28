import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { BLACK_HOLE_CONTROLS } from "./controls.ts";
import BlackHole from "./BlackHole.svelte";

/**
 * Black holes in True (XO4; exotic objects design 3.1 to 3.4, 7): the kernel's three holes of design 3.4 drawn by the
 * renderer's light-path stage from the kernel's bend table, disc and feed (`read_light_paths`), in every band. Text
 * tier: the words alone.
 */
export const blackHole: ViewModule = {
  id: "sse.view.black-hole",
  version: "0.1.0",
  tiers: ["gpu-basic", "text"],
  box: "fill",
  controls: BLACK_HOLE_CONTROLS,
  inputs: [{ event_kinds: ["authored.set"] }],
  params: [],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene });
    const c = mount(BlackHole, { target: el, props });
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
