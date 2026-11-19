// Screens and views added by the second interface track (galaxy charts and
// travel screens: V8, U6, V9, U8). The shell spreads these lists into its
// registry, its screens and its tabs; everything else about them lives here,
// so the two tracks never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { route } from "../../views/route/src/index.ts";
import { galaxyChart } from "../../views/galaxy-chart/src/index.ts";
import { explore } from "../../views/explore/src/index.ts";
import exploreDoc from "./documents/explore.gm.json";
import "./screens.more.css";

/** GM view modules from this track (the sections are registered too, so other screens and Orac can use them alone). */
export const MORE_VIEWS: readonly ViewModule[] = [explore, route, galaxyChart];

/** Screens: one tab in the bar ("Explore"), whose sections switch inside it. */
export const MORE_SCREENS: readonly { value: string; label: string; doc: ScreenDoc; tab?: boolean }[] = [
  { value: "explore", label: "Explore", doc: exploreDoc as unknown as ScreenDoc, tab: true },
];
