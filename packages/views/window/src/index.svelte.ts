import { mount, unmount } from "svelte";
import { flatChart, type ViewModule } from "../../../ui-contract/src/index.ts";
import Window from "./Window.svelte";

/**
 * The Window (V8, design handoff 6.8 "look out the window"): what the ship's
 * sensor picks up from beside a world, now. The kernel's `window_view` says
 * where each object is, how big and how bright, and whether this sensor sees
 * it or makes out its disc; the view points the camera. Nothing the sensor
 * misses is drawn. Read only.
 */
export const windowView: ViewModule = {
  id: "sse.view.window",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  controls: flatChart("The sky is drawn as a flat map for now"),
  inputs: [{ event_kinds: ["authored.set", "adopt.v3", "adopt.formation", "time.aged"] }],
  params: [
    { name: "sensor", type: "enum", default: "eye", options: ["eye", "binoculars", "telescope"], meaning: "The sensor the window opens with (base-nav's sensors).", promoted: true, group: "content" },
    { name: "show_missed", type: "boolean", default: false, meaning: "List what the sensor misses (GM only; players never see it).", group: "behaviour" },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(Window, { target: el, props });
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
