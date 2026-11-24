import { mount, unmount } from "svelte";
import type { SystemScene, ViewModule, Vocab } from "../../../ui-contract/src/index.ts";

/** What the shell gives the Atlas: vocabulary layers by id, and a look's tokens as CSS (the view never names a theme). */
export interface AtlasHost {
  vocabs: Readonly<Record<string, Vocab>>;
  lookCss(look: string): string;
  /** The look's paper form for "Save ink": a dark look gives way to a light one (the edition's `save_ink_look`, else paper). */
  paperCss(look: string, saveInkLook?: string): string;
}
import Atlas from "./Atlas.svelte";

/**
 * The Atlas (track AT): a book of the place, read a page at a time, in an edition. Built from the audience's own
 * scene (the kernel's `scene` op), so a page holds only what that audience knows. Text only: it draws on any device.
 * The shell gives it the vocabulary layers and a way to dress a page in an edition's look.
 */
export function atlasView(host: AtlasHost): ViewModule {
  return {
    id: "sse.view.atlas",
    version: "0.1.0",
    tiers: ["text"],
    inputs: [{ event_kinds: ["authored.set", "observe.grant", "adopt.v3", "adopt.formation", "time.aged"] }],
    params: [],
    intents: [],
    mount(el, ctx, scene) {
      const props = $state({ ctx, scene: scene as SystemScene, host });
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
