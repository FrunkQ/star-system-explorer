// Screens and views of the Run desk and the players' side (track PB, the Switchboard). The shell spreads these lists
// into its registry and its screens; everything else about them lives in run/.
import { mount, unmount } from "svelte";
import type { ScreenDoc, ViewModule } from "../../../ui-contract/src/index.ts";
import SwitchboardView from "./SwitchboardView.svelte";
import switchboardDoc from "../documents/switchboard.gm.json";

/** The Switchboard (PB2): sources on the left, every role and screen on the right, the lines between. */
export const switchboardView: ViewModule = {
  id: "sse.view.switchboard",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [],
  params: [],
  intents: [],
  mount(el, ctx) {
    const c = mount(SwitchboardView, { target: el, props: { ctx } });
    return {
      update() {},
      dispose() {
        void unmount(c);
      },
    };
  },
};

export const RUN_VIEWS: readonly ViewModule[] = [switchboardView];
export const RUN_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "switchboard", label: "Switchboard", doc: switchboardDoc as unknown as ScreenDoc }];
