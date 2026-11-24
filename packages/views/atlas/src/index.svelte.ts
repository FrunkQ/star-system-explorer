import { mount, unmount } from "svelte";
import type { SystemScene, ViewModule, Vocab } from "../../../ui-contract/src/index.ts";
import Atlas from "./Atlas.svelte";

/**
 * The Atlas (track AT): a book of the place, read a page at a time, in an edition. Built from the audience's own
 * scene (the kernel's `scene` op), so a page holds only what that audience knows. Text only: it draws on any device.
 * `vocab` is the words layer for fields (the shell gives the plain one).
 */
export function atlasView(vocab: Vocab): ViewModule {
  return {
    id: "sse.view.atlas",
    version: "0.1.0",
    tiers: ["text"],
    inputs: [{ event_kinds: ["authored.set", "observe.grant", "adopt.v3", "adopt.formation", "time.aged"] }],
    params: [],
    intents: [],
    mount(el, ctx, scene) {
      const props = $state({ ctx, scene: scene as SystemScene, vocab });
      const c = mount(Atlas, { target: el, props });
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
}
