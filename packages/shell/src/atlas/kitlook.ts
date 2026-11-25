// The look a role's kit puts on its screens (track AT, AT15 part 2; atlas design 11A.3 and 11A.5). A role wearing a
// kit sees the kit's look, in the palette the GM chose on the role card (`Seat.palette`), else the kit's first.
//
// One mechanism for every palette: the look's own colours turn by the hue and saturation that take the look's accent
// to the palette's, lightness kept, so its rules, glows, caps and tints follow; then the palette's six checked tokens
// are laid exactly over them. A one-colour look (Phosphor) turns everything, its map and legend slots included,
// since there they are all its one ink; any other look keeps the map, legend and field colours it was given, which
// carry meaning. A kit or look the shell does not know gives null: the screen keeps the player's own theme.
import { resolveKit, SHIPPED_PALETTES, type Palette } from "../../../ui-contract/src/index.ts";
import { themeCss } from "../../../skins/chart-room/src/index.ts";
import { ATLAS_LOOKS, ATLAS_LOOK_EXTRA_CSS } from "./looks.ts";

type ThemeDoc = (typeof ATLAS_LOOKS)[number];

export interface KitLook {
  /** The look's theme id (one of ATLAS_LOOKS). */
  theme: string;
  /** The palette worn, if the kit has one. */
  palette?: string;
  /** The look as worn: its tokens turned to the palette, so marks drawn from the theme follow too. */
  doc: ThemeDoc;
  /** The root's tokens. */
  css: string;
}

/** The palette a kit wears: the one asked for when the kit has it, else the kit's first. */
export function paletteOf(palettes: readonly (string | Palette)[] | undefined, asked?: string): Palette | undefined {
  const all = (palettes ?? []).map((p) => (typeof p === "string" ? SHIPPED_PALETTES.find((x) => x.id === p) : p)).filter((p): p is Palette => !!p);
  return all.find((p) => p.id === asked) ?? all[0];
}

// ---- Colour in HSL, for #rrggbb and rgb()/rgba() inside any token value.
function toHsl(r: number, g: number, b: number): [number, number, number] {
  const [R, G, B] = [r / 255, g / 255, b / 255];
  const max = Math.max(R, G, B);
  const min = Math.min(R, G, B);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  const h = max === R ? (G - B) / d + (G < B ? 6 : 0) : max === G ? (B - R) / d + 2 : (R - G) / d + 4;
  return [h * 60, s, l];
}
function fromHsl(h: number, s: number, l: number): [number, number, number] {
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return [f(0), f(8), f(4)].map((x) => Math.round(Math.max(0, Math.min(1, x)) * 255)) as [number, number, number];
}
const hex2 = (n: number) => n.toString(16).padStart(2, "0").toUpperCase();
function rgbOf(hex: string): [number, number, number] | null {
  const m = /^#([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return null;
  const n = parseInt(m[1]!, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

/** A turn of hue and scale of saturation, from one colour to another. */
export function turnBetween(from: string, to: string): { dh: number; ks: number } | null {
  const a = rgbOf(from);
  const b = rgbOf(to);
  if (!a || !b) return null;
  const [ha, sa] = toHsl(...a);
  const [hb, sb] = toHsl(...b);
  return { dh: hb - ha, ks: sa > 0.02 ? sb / sa : 1 };
}

/** Every colour inside a token value, turned; other text left as it is. */
export function turnColours(value: string, t: { dh: number; ks: number }): string {
  const turn = (r: number, g: number, b: number) => {
    const [h, s, l] = toHsl(r, g, b);
    return fromHsl((((h + t.dh) % 360) + 360) % 360, Math.max(0, Math.min(1, s * t.ks)), l);
  };
  return value
    .replace(/#([0-9a-f]{6})\b/gi, (_, x: string) => {
      const n = parseInt(x, 16);
      const [r, g, b] = turn((n >> 16) & 255, (n >> 8) & 255, n & 255);
      return `#${hex2(r)}${hex2(g)}${hex2(b)}`;
    })
    .replace(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/gi, (m, r: string, g: string, b: string) => {
      const [R, G, B] = turn(Number(r), Number(g), Number(b));
      return `${m.startsWith("rgba") ? "rgba" : "rgb"}(${R}, ${G}, ${B}`;
    });
}

const MEANINGFUL = /^(map-|origin-|field-)/;

/** The look as it is worn in a palette: its own colours turned, then the palette's tokens over them. */
export function wornLook(look: ThemeDoc, p: Palette | undefined, registry: readonly ThemeDoc[] = ATLAS_LOOKS): ThemeDoc {
  if (!p) return look;
  const base = registry.find((t) => t.id === look.extends);
  const all = { ...(base ? resolveFlat(base, registry) : {}), ...look.tokens } as Record<string, string | number>;
  const t = turnBetween(String(all.accent ?? ""), p.tokens.accent);
  const one = look.supports?.colours === 1;
  const tokens: Record<string, string | number> = { ...look.tokens };
  if (t) for (const [k, v] of Object.entries(one ? all : look.tokens)) if (typeof v === "string" && (one || !MEANINGFUL.test(k))) tokens[k] = turnColours(v, t);
  Object.assign(tokens, p.tokens, quietOf(p));
  // A palette may turn a look's night into day (the Guide's Day): the mode follows the palette's ground.
  const g = rgbOf(p.tokens.ground);
  const mode = g ? (toHsl(...g)[2] > 0.5 ? "light" : "dark") : look.mode;
  return { ...look, mode, tokens } as ThemeDoc;
}

/** Mix two #rrggbb colours: t of the way from a to b. */
export function mixHex(a: string, b: string, t: number): string {
  const x = rgbOf(a);
  const y = rgbOf(b);
  if (!x || !y) return a;
  return `#${x.map((c, i) => hex2(Math.round(c + (y[i]! - c) * t))).join("")}`;
}

/**
 * The quiet structure a palette implies, so it reads whichever way round its ground is: text on a pressed control in
 * the ground's colour, the focus ring in the accent, faint lines and rules mixed from the ink toward the ground.
 */
export function quietOf(p: Palette): Record<string, string> {
  const { ground, ink } = p.tokens;
  return {
    "on-ink": ground,
    focus: p.tokens.accent,
    "ink-faint": mixHex(ink, ground, 0.55),
    rule: mixHex(ink, ground, 0.75),
    grid: mixHex(ink, ground, 0.9),
    hatch: p.tokens["ink-muted"],
    "zone-ink": p.tokens["ink-muted"],
  };
}

/** The Atlas's own extras that are text on the screen (titles, labels, margin, the cover's letters) take the palette's. */
const SCREEN_TEXT: Readonly<Record<string, keyof Palette["tokens"]>> = { "title-ink": "accent-ink", "label-ink": "ink-muted", "margin-ink": "accent-ink", "cover-big": "accent" };
function extrasCss(look: string, p: Palette | undefined): string {
  const css = ATLAS_LOOK_EXTRA_CSS(look);
  if (!p) return css;
  return css + Object.entries(SCREEN_TEXT).map(([k, v]) => ` --sse-${k}: ${p.tokens[v] ?? p.tokens.ink};`).join("");
}
function resolveFlat(doc: ThemeDoc, registry: readonly ThemeDoc[]): Record<string, string | number> {
  const up = doc.extends ? registry.find((t) => t.id === doc.extends) : undefined;
  return { ...(up ? resolveFlat(up, registry) : {}), ...doc.tokens };
}

export function kitLookOf(kitId: string | undefined, paletteId?: string): KitLook | null {
  if (!kitId) return null;
  let kit;
  try {
    kit = resolveKit(kitId);
  } catch {
    return null;
  }
  const look = ATLAS_LOOKS.find((t) => t.id === kit.look);
  if (!look) return null;
  const p = paletteOf(kit.palettes, paletteId);
  const doc = wornLook(look, p);
  const registry = ATLAS_LOOKS.map((t) => (t.id === doc.id ? doc : t));
  return { theme: look.id, ...(p ? { palette: p.id } : {}), doc, css: themeCss(look.id, registry) + extrasCss(look.id, p) };
}
