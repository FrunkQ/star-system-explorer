import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Composer from "./Composer.svelte";

export const screenComposer: ViewModule = {
  id: "sse.view.screen-composer",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [
    { name: "screen", type: "string", default: "documents/players.screen.json", meaning: "Screen document to open (URL)." },
    { name: "vocab", type: "string", default: "documents/vocab.plain.json", meaning: "Vocabulary layer for the preview (URL)." },
    { name: "scenes", type: "list", default: ["scenes/tarn-before.json", "scenes/tarn-after.json"], meaning: "Player scenes to preview on: before and after a scan (URLs)." },
    { name: "orac", type: "string", default: "", meaning: "A screen change request to offer (URL); empty for none." },
  ],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Composer, { target: el, props });
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
