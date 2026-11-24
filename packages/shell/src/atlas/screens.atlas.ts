// The Atlas's screens and views (track AT). The shell spreads these lists into its registry and its screens;
// everything else about the Atlas lives in atlas/, packages/views/atlas and ui-contract/src/atlas.
import type { ScreenDoc, ViewModule, Vocab } from "../../../ui-contract/src/index.ts";
import { atlasView } from "../../../views/atlas/src/index.ts";
import plain from "../documents/vocab.plain.json";
import atlasDoc from "./atlas.gm.json";

export const ATLAS_VIEWS: readonly ViewModule[] = [atlasView(plain as unknown as Vocab)];
export const ATLAS_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "atlas", label: "Atlas", doc: atlasDoc as unknown as ScreenDoc }];
