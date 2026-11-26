// Screens and views added by the Formation screen (formation design section 7, F10). The shell spreads these lists
// into its registry and its screens; everything else about them lives here, so threads never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { formation } from "../../views/formation/src/index.ts";
import formationDoc from "./documents/formation.gm.json";

/** GM view modules for the Formation screen. */
export const FORMATION_VIEWS: readonly ViewModule[] = [formation];

/** How it formed (system scale): the system's history as a chart of distance across and time down. */
export const FORMATION_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "formation", label: "How it formed", doc: formationDoc as unknown as ScreenDoc }];
