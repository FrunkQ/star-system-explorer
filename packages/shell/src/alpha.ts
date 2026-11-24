// The alpha note (Alex, 24 Nov 2026): SSE4 is a preview; anyone who wants to use Star System Explorer for real today
// goes to version 3. One address for the start sheet and the players' page.
export const V3_URL = "https://starsystemx.com/";

// How SSE4 is made, by the copyright line (Alex, 24 Nov 2026, in his words: "modern-coded, well considered and
// architected and researched"). The support links show only once each has its real address here; none is guessed.
export const MADE_LINE = "Modern-coded: built with AI help, carefully architected and researched, with every number cited and checkable through its Why.";
export const SUPPORT_LINKS: readonly { label: string; href: string }[] = [
  { label: "Patreon", href: "" },
  { label: "Ko-fi", href: "" },
  { label: "Discord", href: "" },
].filter((l) => l.href !== "");
