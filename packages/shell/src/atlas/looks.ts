// The looks the Atlas and the kits wear (track AT): the player themes, and the looks that live with the Atlas until
// the interface kits move them (AT14). Pure data and strings, so tests and the players' page read it alike.
import { ALL_THEMES } from "../../../skins/player/src/index.ts";
import { ATLAS_LOOK_EXTRAS, GUIDE_GADGET } from "./guide-gadget.theme.ts";

export const ATLAS_LOOKS = [...ALL_THEMES, GUIDE_GADGET];

/** The Atlas's own extra tokens for a look (the Guide's case, keys and cover), as CSS. */
export const ATLAS_LOOK_EXTRA_CSS = (look: string): string => Object.entries(ATLAS_LOOK_EXTRAS[look] ?? {}).map(([k, v]) => ` --sse-${k}: ${v};`).join("");
