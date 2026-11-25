// Screens and views of postcards (track PC): the wall. The shell spreads these lists into its registry and its
// screens; everything else about postcards lives in postcards/.
import { mount, unmount } from "svelte";
import type { ScreenDoc, ViewModule } from "../../../ui-contract/src/index.ts";
import Wall from "./Wall.svelte";
import postcardsDoc from "../documents/postcards.gm.json";

/** The wall (PC3): this campaign's postcards, visits, and the sets from packs. */
export const postcardsView: ViewModule = {
  id: "sse.view.postcards",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [],
  params: [],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene });
    const c = mount(Wall, { target: el, props });
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

export const POSTCARD_VIEWS: readonly ViewModule[] = [postcardsView];
export const POSTCARD_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "postcards", label: "Postcards", doc: postcardsDoc as unknown as ScreenDoc }];
