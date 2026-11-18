import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Symbols from "./Symbols.svelte";

export const symbols: ViewModule = {
  id: "sse.view.symbols",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [{ name: "all_classes", type: "boolean", default: false, meaning: "List every class the classification pack knows, not only those on the chart." }],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Symbols, { target: el, props });
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
