import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import Route from "./Route.svelte";

/**
 * Route, the astrogation console (V8, design handoff 6.8). The crew tells the
 * ship's computer where to go and what matters most; the kernel's
 * `plan_route` (T3) plots candidate courses from the real orbits, with burns,
 * propellant, dose, hazards and difficulty; the console lays them side by
 * side and draws the chosen one in the star's frame. Nobody hand-pilots: the
 * crew chooses, the computer flies. Writes nothing to the universe.
 */
export const route: ViewModule = {
  id: "sse.view.route",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "ship", type: "enum", default: "fusion-torch", options: ["chemical-transfer", "nuclear-thermal", "fusion-torch", "rescue-cutter"], meaning: "The base-nav pack's stock ship the console plans for first.", promoted: true, group: "content" },
    { name: "prefer", type: "enum", default: "balanced", options: ["fastest", "save-fuel", "balanced", "safest"], meaning: "What matters most when the console orders the courses: the kernel's three orders, or safest (fewest hazards, lowest dose).", promoted: true, group: "behaviour" },
    { name: "window_days", type: "number", default: 120, range: [1, 3650], unit: "days", meaning: "How long the crew can wait for a departure window.", group: "behaviour" },
    { name: "shown", type: "integer", default: 3, range: [1, 6], meaning: "How many courses are laid side by side.", group: "look" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Route, { target: el, props });
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
