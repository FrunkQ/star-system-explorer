// Screens a game pack's code sheet brings (X3a, X3c: the Traveller track).
// The shell spreads these lists into its registry and its screens, as it does
// screens.more.ts, so the tracks never edit the same lines. Their rail entries
// `need` a stacked pack with `code_sheets`, so they show only while one is.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { sectorView, worldSheetView } from "../../views/sector/src/index.ts";
import sectorDoc from "./documents/sector.gm.json";
import codesDoc from "./documents/codes.gm.json";

export const CODES_VIEWS: readonly ViewModule[] = [sectorView, worldSheetView];

export const CODES_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [
  { value: "sector", label: "Sector map", doc: sectorDoc as unknown as ScreenDoc },
  { value: "codes", label: "Codes", doc: codesDoc as unknown as ScreenDoc },
];
