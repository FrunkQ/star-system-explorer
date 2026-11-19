// Screens and views added by the second interface track (galaxy charts and
// travel screens: V8, U6, V9, U8). The shell spreads these lists into its
// registry, its screens and its tabs; everything else about them lives here,
// so the two tracks never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { route } from "../../views/route/src/index.ts";
import routeDoc from "./documents/route.gm.json";
import "./screens.more.css";

/** GM view modules from this track. */
export const MORE_VIEWS: readonly ViewModule[] = [route];

/** Screens: one tab in the bar ("Explore"); every screen is in the phone menu. */
export const MORE_SCREENS: readonly { value: string; label: string; doc: ScreenDoc; tab?: boolean }[] = [
  { value: "route", label: "Route", doc: routeDoc as unknown as ScreenDoc, tab: true },
];
