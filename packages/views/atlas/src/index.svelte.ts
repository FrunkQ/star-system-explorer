import { ATLAS_CONTROLS } from "./controls.ts";
import { mount, unmount } from "svelte";
import type { SystemScene, ViewModule, Vocab } from "../../../ui-contract/src/index.ts";

/** What the shell gives the Atlas: vocabulary layers by id, and a look's tokens as CSS (the view never names a theme). */
export interface AtlasHost {
  vocabs: Readonly<Record<string, Vocab>>;
  lookCss(look: string): string;
  /** The look's paper form for "Save ink": a dark look gives way to a light one (the edition's `save_ink_look`, else paper). */
  paperCss(look: string, saveInkLook?: string): string;
  /** Black and white (14A note 7): the toner form's tokens, white ground and black ink. Absent, the paper form is used. */
  tonerCss?(): string;
  /** How a look turns a page (AT15a): paper curls, the Guide redraws in bands. Absent, the page just changes. */
  lookDelivery?(look: string): import("../../../ui-contract/src/index.ts").LookDelivery | undefined;
  /** The brand whose paper an audience's copy is on (the role's, AT9); absent, the Atlas draws the placeholder. */
  /** Margin note sets from the pack stack, beside the shipped ones. */
  noteSets?: readonly import("../../../ui-contract/src/index.ts").NoteSet[];
  brand?(audience: string): import("../../../ui-contract/src/index.ts").LetterheadBrand | undefined;
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
    switchable: ATLAS_CONTROLS,
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

import PlayerAtlas from "./PlayerAtlas.svelte";
import type { PlayerContext, PlayerScene } from "../../../ui-contract/src/index.ts";

/**
 * The Atlas on a players' screen (AT15 part 2): the book of what the table was sent, in the edition the layout names
 * (`edition`), its page following the screen's selection (`page: "selected"`). It needs no host: the look is the
 * screen's, the words are the screen's vocabulary.
 */
export const playerAtlas: ViewModule<PlayerScene, PlayerContext> = {
  id: "sse.view.atlas",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: [{ addresses: [], quantity_kinds: [] }],
  params: [
    { name: "edition", type: "string", default: "sse.edition.survey", group: "look", meaning: "The Atlas edition the page is dressed in (a kit names it)." },
    { name: "page", type: "enum", options: ["selected", "contents"], default: "selected", group: "content", meaning: "Which page shows: the selected thing's, or the top of the book." },
  ],
  switchable: ATLAS_CONTROLS,
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene });
    const c = mount(PlayerAtlas, { target: el, props });
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
