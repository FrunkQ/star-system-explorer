// Roll call (track SH): the screen and its view. The shell spreads these into its registry and its screens; everything
// else about Roll call and In memory lives in rollcall/.
import { mount, unmount } from "svelte";
import type { ScreenDoc, SystemScene, ViewModule } from "../../../ui-contract/src/index.ts";
import RollCall from "./RollCall.svelte";
import rollcallDoc from "../documents/rollcall.gm.json";

/** Who was in space on the day in view, aboard what and where (SH2). Read only. */
export const rollCallView: ViewModule = {
  id: "sse.view.roll-call",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [],
  params: [],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene });
    const c = mount(RollCall, { target: el, props });
    return {
      update(s: SystemScene) {
        props.scene = s;
      },
      dispose() {
        void unmount(c);
      },
    };
  },
};

export const ROLLCALL_VIEWS: readonly ViewModule[] = [rollCallView];
export const ROLLCALL_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "rollcall", label: "Roll call", doc: rollcallDoc as unknown as ScreenDoc }];
