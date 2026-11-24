// The alpha note (Alex, 24 Nov 2026): SSE4 is a preview; anyone who wants to use Star System Explorer for real today
// goes to version 3. One address for the start sheet and the players' page.
export const V3_URL = "https://starsystemx.com/";

// How SSE4 is made, by the copyright line (Alex, 24 Nov 2026: "the joke is kinda the point... YES this IS VIBE CODED...
// but that does not mean it is slop"; a placeholder he will fine tune). The support links show only once each has its real address here; none is guessed.
export const MADE_LINE = "Yes, this is vibe-coded, by FrunkQ & Fable/Opus. That doesn't make it slop: it is designed, architected and researched, and every number shows its working through its Why.";
export const SUPPORT_LINKS: readonly { label: string; href: string }[] = [
  { label: "Patreon", href: "" },
  { label: "Ko-fi", href: "" },
  { label: "Discord", href: "" },
].filter((l) => l.href !== "");
