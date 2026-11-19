import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import FlowEditor from "./FlowEditor.svelte";

export const flowEditor: ViewModule = {
  id: "sse.view.flow-editor",
  version: "0.1.0",
  tiers: ["canvas", "text"],
  inputs: [{ quantity_kinds: [] }],
  params: [
    { name: "flow", type: "string", default: "sse.base.planet-climate", meaning: "Flow shown when nothing has been put on the bench." },
  ],
  intents: [{ kind: "adopt_physics", meaning: "Not sent from here: parameter changes become a draft change request, adopted from the change-request panel." }],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(FlowEditor, { target: el, props });
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
