import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { HAZARD_CONTROLS } from "./controls.ts";
import HazardMap from "./HazardMap.svelte";

/**
 * Hazard map (XO18; exotic objects design 3.8): for the selected body (or the system's star), how close a ship can
 * go and which hazard stops it, along the equator's plane or over the poles, with the Park and Dive marks. Reads the
 * kernel's `hazard_map`; draws its rows, marks and words and nothing else. Text tier: the words alone.
 */
export const hazard: ViewModule = {
  id: "sse.view.hazard",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  controls: HAZARD_CONTROLS,
  inputs: [{ event_kinds: ["authored.set"] }],
  params: [{ name: "line", type: "enum", default: "equator", options: ["equator", "poles"], meaning: "The line the Profile is read along first: in the equator's plane, or over the poles." }],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(HazardMap, { target: el, props });
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
