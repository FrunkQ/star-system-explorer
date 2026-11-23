// Screens a game pack brings (the Traveller track: X3a and X3c codes and
// sectors, V3 gap 13 ships and stations). The shell spreads these lists into
// its registry and its screens, as it does screens.more.ts, so the tracks
// never edit the same lines. Their rail entries `need` a stacked pack key
// (`code_sheets`, `templates`), so they show only while a pack gives one.
import type { ScreenDoc, ViewModule } from "../../ui-contract/src/index.ts";
import { sectorView, worldSheetView } from "../../views/sector/src/index.ts";
import { constructsView } from "../../views/constructs/src/index.ts";
import sectorDoc from "./documents/sector.gm.json";
import codesDoc from "./documents/codes.gm.json";
import constructsDoc from "./documents/constructs.gm.json";

export const GAME_VIEWS: readonly ViewModule[] = [sectorView, worldSheetView, constructsView];

export const GAME_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [
  { value: "sector", label: "Sector map", doc: sectorDoc as unknown as ScreenDoc },
  { value: "codes", label: "Codes", doc: codesDoc as unknown as ScreenDoc },
  { value: "constructs", label: "Ships and stations", doc: constructsDoc as unknown as ScreenDoc },
];
