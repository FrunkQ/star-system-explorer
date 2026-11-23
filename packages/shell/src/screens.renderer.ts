// Screens and views added by the renderer track (SL1: Land). The shell spreads these lists into its registry and
// its screens; everything else about them lives here, so the tracks never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { surface } from "../../views/surface/src/index.ts";
import surfaceDoc from "./documents/surface.gm.json";

/** GM view modules from the renderer track. */
export const RENDER_VIEWS: readonly ViewModule[] = [surface];

/** Land (world scale, travel): pick a site, fly down, walk about. */
export const RENDER_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "surface", label: "Land", doc: surfaceDoc as unknown as ScreenDoc }];
