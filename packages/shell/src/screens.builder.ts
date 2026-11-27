// Screens and views added by the ship builder (CM14a, compound ships design 8A.4). The shell spreads these lists into
// its registry and its screens; everything else about them lives here, so threads never edit the same lines.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { shipBuilder } from "../../views/ship-builder/src/index.ts";
import builderDoc from "./documents/ship-builder.gm.json";

/** GM view modules for the ship builder. */
export const BUILDER_VIEWS: readonly ViewModule[] = [shipBuilder];

/** The ship builder (Workshop, Build desk): start from a catalogue ship or nothing, build from parts. */
export const BUILDER_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "ship-builder", label: "Ship builder", doc: builderDoc as unknown as ScreenDoc }];
