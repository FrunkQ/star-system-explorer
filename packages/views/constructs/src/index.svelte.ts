import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Constructs from "./Constructs.svelte";

/**
 * Ships and stations (V3 gap 13): the constructs in this system with their
 * cards at the depth in force, and "Add here" from the stacked packs'
 * templates (kernel `templates`, `create`, `retire`). Editing a fact is one
 * set_value. Says plainly when no pack gives templates.
 */
export const constructsView: ViewModule = {
  id: "sse.view.constructs",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ event_kinds: ["authored.set", "template.create", "template.retire"] }],
  params: [],
  intents: [{ kind: "set_value", meaning: "Change one fact of a ship, station or site." }],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Constructs, { target: el, props });
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
