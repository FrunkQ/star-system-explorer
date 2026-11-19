import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import StoryFinder from "./StoryFinder.svelte";

/**
 * The story finder (U8, design handoff "story finder" board): sifters run
 * over the whole campaign on the GM's machine (kernel `sift`) and list what
 * is worth a story, best first, each with the facts and events behind it and
 * why it matters. A hit can be picked on the chart or copied as a brief for a
 * storyteller (Minerva, when agents land). Read only.
 */
export const storyFinder: ViewModule = {
  id: "sse.view.story-finder",
  version: "0.1.0",
  tiers: ["text"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [{ name: "min_score", type: "number", default: 0, range: [0, 1], meaning: "Hide hits that matter less than this (0 to 1).", group: "behaviour" }],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(StoryFinder, { target: el, props });
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
