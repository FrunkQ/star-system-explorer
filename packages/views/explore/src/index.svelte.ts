import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { route } from "../../route/src/index.ts";
import { galaxyChart } from "../../galaxy-chart/src/index.ts";
import { foamMap } from "../../foam-map/src/index.ts";
import { clusterMap } from "../../cluster-map/src/index.ts";
import { windowView } from "../../window/src/index.ts";
import { storyFinder } from "../../story-finder/src/index.ts";
import Explore from "./Explore.svelte";

/** The sections of the Explore screen, in order: each is an ordinary view module. */
export const EXPLORE_SECTIONS: readonly { id: string; label: string; view: ViewModule }[] = [
  { id: "route", label: "Route", view: route },
  { id: "window", label: "Window", view: windowView },
  { id: "galaxy", label: "Galaxy", view: galaxyChart },
  { id: "cluster", label: "Galaxies nearby", view: clusterMap },
  { id: "foam", label: "Cosmic web", view: foamMap },
  { id: "stories", label: "Stories", view: storyFinder },
];

/**
 * Explore: one screen (one tab in the shell) holding this track's views as
 * sections: Route (V8), Galaxy (U6) and, later, the patch, foam and story
 * screens. It only switches between them; each section is a view module
 * mounted with the same context and scene.
 */
export const explore: ViewModule = {
  id: "sse.view.explore",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "section", type: "enum", default: "route", options: EXPLORE_SECTIONS.map((s) => s.id), meaning: "The section the screen opens on.", promoted: true, group: "behaviour" },
    { name: "sections", type: "list", default: [], meaning: "The sections this screen offers, by id; empty offers them all.", group: "content" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const only = Array.isArray(ctx.params.sections) ? ctx.params.sections.map(String) : [];
    const sections = only.length ? EXPLORE_SECTIONS.filter((s) => only.includes(s.id)) : EXPLORE_SECTIONS;
    const props = $state({ ctx, scene, sections });
    const c = mount(Explore, { target: el, props });
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
