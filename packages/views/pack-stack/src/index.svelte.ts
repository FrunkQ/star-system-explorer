import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import PackStack from "./PackStack.svelte";

export const packStack: ViewModule = {
  id: "sse.view.pack-stack",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [
    { name: "samples", type: "list", default: [], meaning: "Sample pack documents (URLs) offered to try." },
    { name: "sample_labels", type: "list", default: [], meaning: "Each sample's button words, in the same order (absent: \"Try the sample pack\")." },
  ],
  intents: [{ kind: "stack_packs", meaning: "Add or remove packs above the campaign's opening stack, after a preview of every value it changes." }],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(PackStack, { target: el, props });
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
