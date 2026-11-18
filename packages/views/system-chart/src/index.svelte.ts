import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import SystemChart from "./SystemChart.svelte";

export const systemChart: ViewModule = {
  id: "sse.view.system-chart",
  version: "0.1.0",
  tiers: ["canvas"],
  inputs: [{ event_kinds: ["authored.set"] }],
  params: [
    { name: "zoom", type: "number", default: 1, range: [0.5, 6], meaning: "Starting zoom of the radial scale.", promoted: true },
    { name: "centre_x", type: "number", default: 0.4, range: [0.2, 0.8], meaning: "Horizontal position of the star as a fraction of the chart width (wide screens)." },
    { name: "grid_step", type: "integer", default: 160, range: [40, 400], unit: "px", meaning: "Survey grid spacing." },
    { name: "grid_labels", type: "boolean", default: true, meaning: "Letter and number grid references along the edges." },
    { name: "labels", type: "enum", default: "all", options: ["all", "selected", "none"], meaning: "Which bodies carry place names.", promoted: true },
    { name: "unsurveyed_hatch", type: "boolean", default: true, meaning: "Hatch territory beyond the surveyed radius.", promoted: true },
  ],
  intents: [],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(SystemChart, { target: el, props });
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
