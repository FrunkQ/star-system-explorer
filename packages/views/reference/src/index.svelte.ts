import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Reference from "./Reference.svelte";

export const reference: ViewModule = {
  id: "sse.view.reference",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [{ name: "page", type: "enum", options: ["constants", "units"], default: "constants", meaning: "The page it opens on: constants or units (more pages follow in MP10's later parts)." }],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Reference, { target: el, props });
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
