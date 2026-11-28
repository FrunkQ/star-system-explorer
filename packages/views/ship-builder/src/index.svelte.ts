import { mount, unmount } from "svelte";
import { type ViewModule } from "../../../ui-contract/src/index.ts";
import BuilderScreen from "./BuilderScreen.svelte";

/**
 * The ship builder (CM14b, to the mock-up; compound ships design 8A.4 and rulings 14A.2 to 14A.4): start from a
 * catalogue ship or nothing, clip palette parts at their connection points, stretch and widen them, and either watch
 * what the ship can do as it grows or fill a plan's shopping list. The kernel judges it; nothing is stored here. It
 * draws its own view controls (View style, Seen from) in its working screen, so it declares none to the shell.
 */
export const shipBuilder: ViewModule = {
  id: "sse.view.ship-builder",
  version: "0.1.0",
  tiers: ["gpu-basic", "canvas"],
  inputs: [{ addresses: [], quantity_kinds: [] }],
  params: [],
  intents: [],
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
