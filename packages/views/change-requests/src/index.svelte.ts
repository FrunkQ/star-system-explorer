import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import ChangeRequests from "./ChangeRequests.svelte";

export const changeRequests: ViewModule = {
  id: "sse.view.change-requests",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [{ name: "show_closed", type: "boolean", default: true, group: "content", meaning: "List declined and superseded requests under Closed.", promoted: true }],
  intents: [{ kind: "adopt_physics", meaning: "Adopt or withdraw a change request: the campaign pack is rebuilt and the stack reloaded." }],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(ChangeRequests, { target: el, props });
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
