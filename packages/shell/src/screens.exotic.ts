// Screens and views added by the exotic objects track (XO18: the Hazard map). The shell spreads these lists into its
// registry and its screens; everything else about them lives here, so the tracks never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { hazard } from "../../views/hazard/src/index.ts";
import hazardDoc from "./documents/hazard.gm.json";

/** GM view modules from the exotic objects track. */
export const EXOTIC_VIEWS: readonly ViewModule[] = [hazard];

/** Hazard map (world and system scale, travel): how close a ship can go round a dangerous body. */
export const EXOTIC_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "hazard", label: "Hazard map", doc: hazardDoc as unknown as ScreenDoc }];
