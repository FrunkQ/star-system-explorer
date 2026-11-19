import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import RulerView from "./RulerView.svelte";

export const timeRuler: ViewModule = {
  id: "sse.view.time-ruler",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["*"] }],
  params: [
    { name: "scene_events", type: "boolean", default: true, group: "content", meaning: "Show the scene's history and futures, not only edits.", promoted: true },
    { name: "edits", type: "boolean", default: true, group: "content", meaning: "Show the GM's authored edits from the chronicle.", promoted: true },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(RulerView, { target: el, props });
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
