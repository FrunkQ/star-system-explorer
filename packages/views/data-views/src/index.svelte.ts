import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import DataViews from "./DataViews.svelte";

export const dataViews: ViewModule = {
  id: "sse.view.data-views",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [
    { name: "audience", type: "string", default: "everyone", meaning: "The audience the preview shows." },
    { name: "vocab", type: "string", default: "documents/vocab.plain.json", meaning: "Vocabulary layer for the preview (URL)." },
  ],
  intents: [
    { kind: "stack_packs", meaning: "Apply the edited data views as the campaign's own player-data pack." },
    { kind: "scan_toggle", meaning: "The Scanned button for one subject and audience." },
  ],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(DataViews, { target: el, props });
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
