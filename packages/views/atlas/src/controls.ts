// The Atlas page's own controls, as a part on a player's screen declares them (AT13, atlas design 11A.4). There is no
// Edition control: a player's screen has no Edition switch (14A note 6); the GM chooses the edition by routing.
import type { PartControl } from "../../../ui-contract/src/index.ts";

export const ATLAS_CONTROLS: PartControl[] = [
  { id: "contents", words: "Open the contents and jump to a page", kind: "button", default: true },
  { id: "step", words: "Step to the next or previous page", kind: "step", default: true },
  { id: "print", words: "Print the page, its chapter or the whole Atlas", kind: "button", default: true },
  { id: "links", words: "Follow a link on the page to another page", kind: "link", default: true },
];
