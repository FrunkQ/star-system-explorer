// "Guide gadget" (track AT, AT7; atlas design 14A note 4): the Guide is cheesy electronics built to a low cost, not
// paper. A night-blue screen with gold and teal and faint scan lines, in a chunky cream plastic case with big coloured
// keys. It lives with the Atlas until the interface kits (AT14, AT15) move the looks into skins/player. The case and
// key tokens are general: any look may give an edition with a `device` a case.
import type { ThemeDoc } from "../../../skins/chart-room/src/index.ts";

export const GUIDE_GADGET: ThemeDoc = {
  schema: "sse.theme/1",
  id: "guide-gadget",
  name: "Guide gadget",
  description: "A cheap, cheerful handheld: night-blue screen, gold and teal, big coloured keys in a cream case.",
  extends: "chart-room",
  mode: "dark",
  supports: { colours: "full", patterns: true, shapes: true },
  vocab: "guide",
  filters: [{ kind: "scanlines", strength: 0.12, spacing: 3 }],
  palette: ["#FFD45C", "#F2545B", "#FF9F43", "#4FB3FF", "#A66CFF", "#4CE38A", "#5FD0C0"],
  tokens: {
    ground: "#0F1726",
    "ground-raised": "#131D30",
    ink: "#EAF0F7",
    "ink-muted": "#A9B8CC",
    "ink-faint": "#6F7F96",
    grid: "#1B2740",
    rule: "#22304A",
    line: "#2C3B58",
    accent: "#FFD45C",
    "accent-ink": "#FFE08A",
    "on-ink": "#0F1726",
    zone: "rgba(95, 208, 192, 0.16)",
    "zone-ink": "#5FD0C0",
    hatch: "#A9B8CC",
    focus: "#5FD0C0",
    "font-display": "'Fredoka', 'Baloo 2', ui-rounded, 'Trebuchet MS', sans-serif",
    "font-body": "ui-rounded, 'Nunito', 'Trebuchet MS', sans-serif",
    "font-mono": "ui-monospace, 'JetBrains Mono', Menlo, Consolas, monospace",
    "card-shadow": "none",
    radius: "14px",
    "chart-ground": "#0F1726",
  },
};

/**
 * What the look adds for the Atlas alone, beyond the theme schema: the device's case and keys, the cover's big
 * letters, and title, label and margin colours. Set as `--sse-<name>` beside the theme's tokens; the Atlas falls
 * back to the theme's own where a look gives none.
 */
export const ATLAS_LOOK_EXTRAS: Readonly<Record<string, Readonly<Record<string, string>>>> = {
  "guide-gadget": {
    "title-ink": "#FFD45C",
    "label-ink": "#5FD0C0",
    "margin-ink": "#FFE08A",
    "case-ground": "#E8E0CF",
    "case-edge": "#CFC4AD",
    "case-ink": "#6A5E49",
    "case-light": "#F2545B",
    "case-radius": "34px",
    "screen-rule": "linear-gradient(90deg, #F2545B, #FF9F43, #FFD45C, #4CE38A, #4FB3FF, #A66CFF, #F25AC8)",
    "key-home": "#D2413A",
    "key-back": "#C9612A",
    "key-main": "#22864A",
    "key-next": "#C9612A",
    "key-print": "#2A6FD0",
    "key-ink": "#FFFFFF",
    "key-shadow": "0 5px 0 rgba(0, 0, 0, 0.28)",
    "cover-big": "#FFD45C",
    "cover-shade": "#B4471F",
  },
};
