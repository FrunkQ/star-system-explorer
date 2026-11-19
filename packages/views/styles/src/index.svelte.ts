import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Styles from "./Styles.svelte";

export const styles: ViewModule = {
  id: "sse.view.styles",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [
    { name: "view", type: "string", default: "sse.view.chart", group: "content", meaning: "The view whose settings open first." },
    { name: "audience", type: "string", default: "everyone", group: "content", meaning: "Whose scene the preview of a player view shows." },
    { name: "vocab", type: "string", default: "documents/vocab.plain.json", group: "content", meaning: "Vocabulary layer for the preview (URL)." },
    { name: "schema_open", type: "boolean", default: true, group: "look", meaning: "Show every setting of the view (the list Orac works from), not only the promoted ones.", promoted: true },
  ],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Styles, { target: el, props });
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
