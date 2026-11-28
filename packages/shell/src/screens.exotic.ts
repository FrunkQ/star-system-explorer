// Screens and views added by the exotic objects track (XO18: the Hazard map; XO4: black holes in True). The shell spreads these lists into its
// registry and its screens; everything else about them lives here, so the tracks never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { hazard } from "../../views/hazard/src/index.ts";
import hazardDoc from "./documents/hazard.gm.json";
import { blackHole } from "../../views/black-hole/src/index.ts";
import blackHoleDoc from "./documents/black-hole.gm.json";

/** GM view modules from the exotic objects track. */
export const EXOTIC_VIEWS: readonly ViewModule[] = [hazard, blackHole];

/** Hazard map (world and system scale, travel): how close a ship can go round a dangerous body. */
export const EXOTIC_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [
  { value: "hazard", label: "Hazard map", doc: hazardDoc as unknown as ScreenDoc },
  // Black holes (galaxy scale, look): the kernel's three holes of design 3.4 in True, drawn by the light-path stage.
  { value: "black-hole", label: "Black holes", doc: blackHoleDoc as unknown as ScreenDoc },
];
