// Screens and views added by the small-body designer (RW16a, real catalogues design 6A.1). The shell spreads these
// lists into its registry and its screens; everything else about them lives here, so threads never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { smallDesigner } from "../../views/small-designer/src/index.ts";
import smallDoc from "./documents/small-designer.gm.json";

/** GM view modules for the small-body designer. */
export const SMALL_VIEWS: readonly ViewModule[] = [smallDesigner];

/** The small-body designer (Workshop): one body of any size, designed by its pins and drawn turning in True. */
export const SMALL_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "small-designer", label: "Small bodies", doc: smallDoc as unknown as ScreenDoc }];
