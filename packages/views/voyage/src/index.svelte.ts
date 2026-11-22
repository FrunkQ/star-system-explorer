import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import { VOYAGE_SHIPS, MINIMISE_WORDS } from "../../../ui-contract/src/index.ts";
import Voyage from "./Voyage.svelte";

/**
 * Between stars (V8 part 3): the voyage console. Pick a ship and a goal (a
 * star, or the nearest one not yet visited) and the kernel's `plan_voyage`
 * charts the legs: jumps by the pack's rule, or flights there for real with
 * time at home and time aboard, fuel carried and refuel stops where the crew
 * knows of fuel. "Next stop" steps an explorer outward. Nobody hand-pilots:
 * the crew asks, the computer plans. Writes nothing to the universe.
 */
export const voyageView: ViewModule = {
  id: "sse.view.voyage",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set", "time.aged"] }],
  params: [
    { name: "ship", type: "enum", default: "jump-courier", options: VOYAGE_SHIPS.map((s) => s.id), meaning: "The base-nav pack's stock ship the console plans for first.", promoted: true, group: "content" },
    { name: "minimise", type: "enum", default: "home_time", options: MINIMISE_WORDS.map((m) => m.id), meaning: "What the finder keeps smallest: time at home, time aboard, fuel, or the number of legs.", promoted: true, group: "behaviour" },
    { name: "gambles", type: "boolean", default: false, meaning: "Let a plan hope for fuel in systems nobody has surveyed, when the odds are good enough.", group: "behaviour" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Voyage, { target: el, props });
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
