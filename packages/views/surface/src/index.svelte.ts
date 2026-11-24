import { mount, unmount } from "svelte";
import type { PlayerContext, PlayerScene, ViewModule } from "../../../ui-contract/src/index.ts";
import { SURFACE_CONTROLS } from "./controls.ts";
import Surface from "./Surface.svelte";

/**
 * Surface view (SL1; surfaces design 4.1, Alex 23 Nov 2026): getting down to the selected world and looking about.
 * Pick a landmark or press anywhere on the world's map; the kernel judges the site and flies a lander down
 * (`plan_descent`), with every danger shown and nothing refused; Land plays that descent as a flown camera, which
 * can be skipped; once down, walk or fly a low drone over the few kilometres the kernel knows to its fact floor,
 * under the world's own sky. Ground finer than the fact floor is decoration and says so.
 */
export const surface: ViewModule = {
  id: "sse.view.surface",
  version: "0.1.0",
  tiers: ["canvas", "gpu-basic"],
  controls: SURFACE_CONTROLS,
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "tier", type: "enum", default: "auto", options: ["auto", "gpu-basic", "canvas"], meaning: "Where the ground is drawn: auto lets this device choose; gpu-basic uses WebGPU or WebGL2; canvas draws the ground as lines of its profile.", cost_options: { "gpu-basic": "medium", canvas: "light" }, group: "behaviour" },
    { name: "flight_s", type: "number", default: 24, range: [6, 120], unit: "s", meaning: "How long the flown descent takes to play (the kernel's own clock runs beside it).", group: "behaviour" },
    { name: "eye_m", type: "number", default: 1.7, range: [0.3, 3], unit: "m", meaning: "Eye height when walking.", group: "look" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Surface, { target: el, props });
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

/**
 * The same view on a player's screen (the landing run-through): it follows the landing the GM sent the table, on the
 * seat's own kernel (the same tiles and the same descent, from the same site and lander), with the GM's pose on the
 * ground. It needs the host's `landing` and `ground`; a screen without them says it is waiting. Players watch and
 * never steer. The player scene itself is not read: the ground is not in it.
 */
export const playerSurface: ViewModule<PlayerScene, PlayerContext> = {
  id: "sse.view.surface",
  version: "0.1.0",
  scene: "player",
  box: "fill",
  tiers: ["canvas", "gpu-basic"],
  inputs: [{ addresses: [], quantity_kinds: [] }],
  params: [surface.params![0]!],
  mount(el, ctx) {
    const c = mount(Surface, { target: el, props: { follow: ctx, params: ctx.params } });
    return {
      update() {},
      dispose() {
        void unmount(c);
      },
    };
  },
};
