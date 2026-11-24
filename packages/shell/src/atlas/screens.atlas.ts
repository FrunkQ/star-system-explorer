// The Atlas's screens and views (track AT). The shell spreads these lists into its registry and its screens;
// everything else about the Atlas lives in atlas/, packages/views/atlas and ui-contract/src/atlas.
import type { ScreenDoc, ViewModule, Vocab } from "../../../ui-contract/src/index.ts";
import { atlasView } from "../../../views/atlas/src/index.ts";
import { themeCss } from "../../../skins/chart-room/src/index.ts";
import { ALL_THEMES, PAPER } from "../../../skins/player/src/index.ts";
import plain from "../documents/vocab.plain.json";
import guide from "../documents/vocab.guide.json";
import atlasDoc from "./atlas.gm.json";
import { ATLAS_LOOK_EXTRAS, GUIDE_GADGET } from "./guide-gadget.theme.ts";

/** The player themes, and the looks that live with the Atlas until the interface kits move them (AT14). */
export const ATLAS_LOOKS = [...ALL_THEMES, GUIDE_GADGET];

/** The looks an edition may wear: the player themes. A look the shell does not know leaves the page in the screen's own. */
const lookCss = (look: string): string => {
  if (!ATLAS_LOOKS.some((t) => t.id === look)) return "";
  const extra = Object.entries(ATLAS_LOOK_EXTRAS[look] ?? {}).map(([k, v]) => ` --sse-${k}: ${v};`).join("");
  return themeCss(look, ATLAS_LOOKS) + extra;
};

/** "Save ink": a dark look prints in its paper form (white ground, dark ink); a light look prints as it is. */
const paperCss = (look: string, saveInkLook?: string): string => {
  const t = ATLAS_LOOKS.find((x) => x.id === look);
  return t?.mode === "dark" ? lookCss(saveInkLook && ATLAS_LOOKS.some((x) => x.id === saveInkLook) ? saveInkLook : PAPER.id) : lookCss(look);
};

export const ATLAS_VIEWS: readonly ViewModule[] = [atlasView({ vocabs: { plain: plain as unknown as Vocab, guide: guide as unknown as Vocab }, lookCss, paperCss })];
export const ATLAS_SCREENS: readonly { value: string; label: string; doc: ScreenDoc }[] = [{ value: "atlas", label: "Atlas", doc: atlasDoc as unknown as ScreenDoc }];
