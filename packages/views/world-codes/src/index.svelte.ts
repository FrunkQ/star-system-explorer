import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import WorldCodes from "./WorldCodes.svelte";
import type { WorldCodesInput } from "./sheet.ts";

/**
 * World data sheet (X3a): a body's game-system codes from the kernel's
 * `codes` op, drawn through the pack's code sheet. The code lines big (a UWP,
 * a PBG), then every digit with its plain why (the class and fit, a hand-set
 * code with what physics says beside it, a count with its members), then the
 * rule groups (trade codes, zones, bases) as chips carrying the pack's words.
 * Several bodies show as a list, one row each. Text tier: plain sentences.
 * Nothing in it names a game: the sheet, class names and words are inputs.
 */
export const worldCodes: ViewModule<WorldCodesInput> = {
  id: "sse.view.world-codes",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "mode", type: "enum", default: "auto", options: ["auto", "sheet", "list"], meaning: "One body's sheet, a list of every body read, or auto: the sheet for one body (or the selected one), the list for several.", promoted: true },
    { name: "tier", type: "enum", default: "canvas", options: ["canvas", "text"], meaning: "Draw the sheet, or give it as plain sentences (small screens, screen readers)." },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ scene, params: ctx.params as Record<string, unknown> });
    const c = mount(WorldCodes, { target: el, props });
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
