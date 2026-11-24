import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Reference from "./Reference.svelte";

export const reference: ViewModule = {
  id: "sse.view.reference",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ quantity_kinds: [] }],
  params: [
    { name: "page", type: "enum", options: ["constants", "elements", "units", "names", "classes"], default: "constants", meaning: "The page it opens on: constants, elements, units, names or classes (more pages follow in MP10's later parts)." },
    { name: "names_offer", type: "string", default: "", meaning: "A names pack the Names page offers, read from this address, while no pack on the stack carries names (NM6)." },
    { name: "classes_offer", type: "string", default: "", meaning: "Class atlases the Classes page shows, comma-separated addresses, while no pack on the stack carries classes (the class gallery)." },
  ],
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
