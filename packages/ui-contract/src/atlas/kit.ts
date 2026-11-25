// Interface kits (track AT, AT14; atlas design 11A and 12.7): everything a role's screens need, chosen once. A kit is
// data: layouts of parts per situation, a look and its palettes, the view style diagrams start in, the Atlas edition,
// words, how screens arrive and leave, and the weakest device it is for. Like an edition it cannot state a fact or
// widen what a role knows. `checkKit` is pure and DOM-free: the pack loader, the role card and Orac (AT16) call it the
// same way, and it returns plain sentences (empty: the kit loads).
import { typedFigures } from "./loader.ts";
import { sayNumber } from "../figures.ts";
import { VIEW_STYLES, type ViewStyle } from "../view-controls.ts";
import starters from "./kits/starters.json" with { type: "json" };

export const KIT_SCHEMA = "sse.kit/1";
export const SITUATIONS = ["sector", "system", "world", "ship", "landing", "holding"] as const;
export type Situation = (typeof SITUATIONS)[number];
export const TIERS = ["text", "canvas", "gpu-basic", "gpu-high"] as const;
/** The layout editor's arrangements and the areas each has (Playbook 7.4). */
export const ARRANGEMENTS: Readonly<Record<string, readonly string[]>> = {
  one: ["a"],
  "picture-and-side": ["a", "b", "c", "d"],
  "two-by-two": ["a", "b", "c", "d"],
  report: ["a", "b"],
  blank: [],
};
/** The five moments a screen arrives and leaves at (11A.4a). */
export const MOMENTS = ["power_on", "arrive", "turn", "leave", "power_off"] as const;
/** The deliveries in the toolkit (11A.4a); each look owns how they are drawn. */
export const DELIVERIES = ["none", "fade", "wipe", "build-up", "type-on", "scan-in", "warm-up", "collapse-to-dot", "static-burst", "dissolve-blocks", "slide", "page-turn", "stepped-wipe", "iris"] as const;
/** A Turn is always short, so stepping through an Atlas never becomes a wait (11A.4a rule 2). */
export const TURN_MAX_SECONDS = 0.4;

export interface KitItem {
  view: string;
  area: string;
  style?: string;
  params?: Record<string, unknown>;
  /** Per control on or off; "none" makes the part a picture; "all" turns every declared control on. */
  controls?: Record<string, boolean> | "none" | "all";
}

export interface KitLayout {
  extends?: Situation;
  arrangement?: string;
  items?: KitItem[];
  /** A holding card's words, for `holding`. */
  card?: string;
}

export interface KitDelivery {
  use: string;
  seconds?: number;
  then?: string;
  by?: "screen" | "part" | "inside";
  order?: string[];
  chars_per_second?: number;
}

/** A named colour set a look may wear (11A.5): checked for contrast when the kit loads. */
export interface Palette {
  id: string;
  name: string;
  tokens: { ground: string; "ground-raised"?: string; ink: string; "ink-muted": string; accent: string; "accent-ink": string };
}

export interface KitDoc {
  schema: typeof KIT_SCHEMA;
  id: string;
  name: string;
  description: string;
  extends?: string;
  look?: string;
  /** Palette ids, the first the default; or palettes of the kit's own. */
  palettes?: (string | Palette)[];
  /** The view style diagrams start in (the shared view controls' own list). */
  view_style?: ViewStyle;
  vocab?: string;
  atlas?: { edition: string; look?: string };
  delivery?: Partial<Record<(typeof MOMENTS)[number], KitDelivery>>;
  needs?: { tier: (typeof TIERS)[number]; falls_back_to?: string };
  layouts?: Partial<Record<Situation, KitLayout>>;
  /** One-liners at the top and bottom of the role's screens (SSE3's guide tips), from these note sets by situation. */
  banners?: { at: "top" | "bottom" | "both"; sets: string[] };
}

/** A part as the stack knows it: its declared controls (AT13), and whether it is the GM's alone. */
export interface PartInfo {
  id: string;
  controls: readonly string[];
  gm_only?: boolean;
}

export interface KitStack {
  kits?: readonly KitDoc[];
  parts?: readonly PartInfo[];
  looks?: readonly string[];
  palettes?: readonly Palette[];
  editions?: readonly string[];
  vocabs?: readonly string[];
  noteSets?: readonly string[];
}

/** The shipped palettes: the starters' swatches (11A.3), each passing the contrast check (a test holds this). */
export const SHIPPED_PALETTES: readonly Palette[] = (starters as unknown as { palettes: Palette[] }).palettes;
/** The four starters (11A.3): Hauler, Corporate, Deck and The Guide. */
export const STARTER_KITS: readonly KitDoc[] = (starters as unknown as { kits: KitDoc[] }).kits;

// WCAG relative luminance and contrast, for #rrggbb: pure, so the loader needs no skin.
function lum(hex: string): number | null {
  const m = /^#([0-9a-f]{6})$/i.exec(hex);
  if (!m) return null;
  const n = parseInt(m[1]!, 16);
  const ch = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * ch[0]! + 0.7152 * ch[1]! + 0.0722 * ch[2]!;
}
export function contrastOf(a: string, b: string): number | null {
  const la = lum(a);
  const lb = lum(b);
  if (la === null || lb === null) return null;
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

/** Text tokens a palette carries, each checked on the ground at 4.5 to 1. */
const TEXT_TOKENS = ["ink", "ink-muted", "accent-ink"] as const;

/** Why a palette is unreadable, as sentences; empty when every text token reads on its ground. */
export function checkPalette(p: Palette): string[] {
  const out: string[] = [];
  for (const k of ["ground", ...TEXT_TOKENS, "accent"] as const) if (lum(p.tokens[k]) === null) out.push(`The palette ${p.id} gives ${k} as "${p.tokens[k]}"; use a colour as #rrggbb.`);
  if (out.length) return out;
  for (const g of ["ground", "ground-raised"] as const) {
    const ground = p.tokens[g];
    if (!ground) continue;
    for (const k of TEXT_TOKENS) {
      const c = contrastOf(p.tokens[k], ground)!;
      if (c < 4.5) out.push(`In the palette ${p.id}, ${k} on ${g} has a contrast of ${sayNumber(c, 2, { decimals: 1 })} to 1; text needs 4.5. Darken or lighten one of them.`);
    }
  }
  return out;
}

/** The kit `id` with its `extends` chain applied (layouts merge by situation). Throws a sentence on a missing or looping parent. */
export function resolveKit(id: string, docs: readonly KitDoc[] = []): KitDoc {
  const all = [...STARTER_KITS, ...docs];
  const byId = (x: string) => {
    let hit: KitDoc | undefined;
    for (const d of all) if (d.id === x) hit = d;
    return hit;
  };
  const chain: KitDoc[] = [];
  let at: string | undefined = id;
  while (at) {
    if (chain.some((c) => c.id === at)) throw new Error(`the kit ${id} extends itself through ${at}`);
    const d = byId(at);
    if (!d) throw new Error(chain.length ? `the kit ${chain[chain.length - 1]!.id} extends ${at}, which is not here` : `there is no kit ${at}`);
    chain.push(d);
    at = d.extends;
  }
  const out: KitDoc = { schema: KIT_SCHEMA, id: chain[0]!.id, name: chain[0]!.name, description: chain[0]!.description, layouts: {} };
  for (const d of [...chain].reverse()) {
    for (const k of ["look", "palettes", "view_style", "vocab", "atlas", "needs"] as const) if (d[k] !== undefined) (out as unknown as Record<string, unknown>)[k] = d[k];
    if (d.delivery) out.delivery = { ...(out.delivery ?? {}), ...d.delivery };
    for (const [s, l] of Object.entries(d.layouts ?? {})) out.layouts![s as Situation] = l;
  }
  // A layout that extends another situation takes its arrangement and items unless it states its own.
  for (const [s, l] of Object.entries(out.layouts!)) {
    if (l?.extends) {
      const base = out.layouts![l.extends];
      out.layouts![s as Situation] = { ...(base ?? {}), ...l, extends: undefined } as KitLayout;
      delete out.layouts![s as Situation]!.extends;
    }
  }
  return out;
}

/**
 * The loader's checks for a kit (11A.7): the schema; `extends`; every look, palette, edition, vocabulary and part
 * named exists; palettes pass contrast; deliveries are known and a Turn is short; the tier is known and a fallback
 * exists; layouts use known arrangements and areas; a part's controls are ones it declared; no GM-only part; no
 * figure typed in the kit's words. Plain sentences; empty means it loads.
 */
export function checkKit(doc: unknown, stack: KitStack = {}): string[] {
  const d = doc as KitDoc;
  if (!d || typeof d !== "object") return ["A kit is a JSON object."];
  if (d.schema !== KIT_SCHEMA) return [`A kit's schema is "${KIT_SCHEMA}".`];
  const out: string[] = [];
  for (const k of ["id", "name", "description"] as const) if (typeof d[k] !== "string" || !d[k]) out.push(`A kit needs a ${k}.`);
  if (out.length) return out;
  const docs = [...(stack.kits ?? []).filter((k) => k.id !== d.id), d];
  try {
    resolveKit(d.id, docs);
  } catch (e) {
    out.push(`${(e as Error).message[0]!.toUpperCase()}${(e as Error).message.slice(1)}.`);
  }
  for (const t of [d.name, d.description, ...Object.values(d.layouts ?? {}).map((l) => l?.card ?? "")]) for (const f of typedFigures(t)) out.push(`The kit's words state a figure ("${f}"). A kit may not state a fact.`);
  if (d.look && stack.looks && !stack.looks.includes(d.look)) out.push(`The look "${d.look}" is not in the pack stack.`);
  if (d.atlas?.look && stack.looks && !stack.looks.includes(d.atlas.look)) out.push(`The Atlas look "${d.atlas.look}" is not in the pack stack.`);
  if (d.atlas?.edition && stack.editions && !stack.editions.includes(d.atlas.edition)) out.push(`The edition "${d.atlas.edition}" is not in the pack stack.`);
  if (d.vocab && stack.vocabs && !stack.vocabs.includes(d.vocab)) out.push(`The vocabulary "${d.vocab}" is not in the pack stack.`);
  if (d.view_style && !VIEW_STYLES.includes(d.view_style)) out.push(`The view style "${d.view_style}" is not one of ${VIEW_STYLES.join(", ")}.`);
  const known = [...SHIPPED_PALETTES, ...(stack.palettes ?? [])];
  for (const p of d.palettes ?? []) {
    if (typeof p === "string") {
      const hit = known.find((x) => x.id === p);
      if (!hit) out.push(`The palette "${p}" is not known; name a shipped one or give its colours.`);
      else out.push(...checkPalette(hit));
    } else out.push(...checkPalette(p));
  }
  for (const [m, dl] of Object.entries(d.delivery ?? {})) {
    if (!(MOMENTS as readonly string[]).includes(m)) out.push(`"${m}" is not a moment; the moments are ${MOMENTS.join(", ")}.`);
    if (!dl || !(DELIVERIES as readonly string[]).includes(dl.use)) out.push(`The ${m} delivery "${dl?.use}" is not in the toolkit (${DELIVERIES.join(", ")}).`);
    if (dl?.then && !(DELIVERIES as readonly string[]).includes(dl.then)) out.push(`The ${m} delivery goes on to "${dl.then}", which is not in the toolkit.`);
    if (dl?.seconds !== undefined && !(dl.seconds >= 0 && dl.seconds <= 4)) out.push(`The ${m} delivery lasts ${dl.seconds} s; keep a delivery between 0 and 4 seconds.`);
    if (m === "turn" && dl?.seconds !== undefined && dl.seconds > TURN_MAX_SECONDS) out.push(`A Turn lasts at most ${TURN_MAX_SECONDS} s, so stepping a book never becomes a wait.`);
  }
  if (d.banners !== undefined) {
    if (!["top", "bottom", "both"].includes(d.banners?.at)) out.push(`The banners sit at "${String(d.banners?.at)}"; use top, bottom or both.`);
    if (!Array.isArray(d.banners?.sets) || !d.banners.sets.length) out.push("The banners name no note sets to draw their lines from.");
    else for (const set of d.banners.sets) if (stack.noteSets && !stack.noteSets.includes(set)) out.push(`The note set "${set}" is not in the pack stack.`);
  }
  if (d.needs) {
    if (!(TIERS as readonly string[]).includes(d.needs.tier)) out.push(`The tier "${d.needs.tier}" is not one of ${TIERS.join(", ")}.`);
    if (d.needs.falls_back_to && ![...STARTER_KITS, ...(stack.kits ?? [])].some((k) => k.id === d.needs!.falls_back_to)) out.push(`The fallback kit "${d.needs.falls_back_to}" is not here.`);
    if (d.needs.tier !== "text" && !d.needs.falls_back_to) out.push(`A kit for ${d.needs.tier} devices names a kit a weaker device falls back to.`);
  }
  for (const [s, l] of Object.entries(d.layouts ?? {})) {
    if (!(SITUATIONS as readonly string[]).includes(s)) out.push(`"${s}" is not a situation; the situations are ${SITUATIONS.join(", ")}.`);
    if (!l) continue;
    if (l.extends && !(SITUATIONS as readonly string[]).includes(l.extends)) out.push(`The ${s} layout extends "${l.extends}", which is not a situation.`);
    if (l.arrangement && !(l.arrangement in ARRANGEMENTS)) out.push(`The ${s} layout's arrangement "${l.arrangement}" is not one of ${Object.keys(ARRANGEMENTS).join(", ")}.`);
    const areas = l.arrangement ? ARRANGEMENTS[l.arrangement] ?? [] : null;
    for (const it of l.items ?? []) {
      const part = stack.parts?.find((p) => p.id === it.view);
      if (stack.parts && !part) out.push(`The ${s} layout uses "${it.view}", which is not a part.`);
      if (part?.gm_only) out.push(`The ${s} layout uses "${it.view}", which is the GM's alone and never a player's part.`);
      if (areas && !areas.includes(it.area)) out.push(`The ${s} layout puts ${it.view} in area "${it.area}", which ${l.arrangement} does not have (${areas.join(", ") || "none"}).`);
      if (part && it.controls && typeof it.controls === "object") {
        for (const c of Object.keys(it.controls)) if (!part.controls.includes(c)) out.push(`The ${s} layout switches "${c}" on ${it.view}, which does not declare it (it has ${part.controls.join(", ") || "none"}).`);
      }
      if (typeof it.controls === "string" && it.controls !== "none" && it.controls !== "all") out.push(`The ${s} layout gives ${it.view} controls "${it.controls}"; use "none", "all" or a list.`);
    }
  }
  return out;
}

/** Which of a part's declared controls are on in a layout item: unstated ones take the part's default (on). */
export function controlsOn(item: KitItem, part: PartInfo, defaults: Readonly<Record<string, boolean>> = {}): string[] {
  if (item.controls === "none") return [];
  if (item.controls === "all") return [...part.controls];
  const set = item.controls ?? {};
  return part.controls.filter((c) => set[c] ?? defaults[c] ?? true);
}
