// Interface kits (track AT, AT14; atlas design 11A and 12.7): everything a role's screens need, chosen once. A kit is
// data: layouts of parts per situation, a look and its palettes, the view style diagrams start in, the Atlas edition,
// words, how screens arrive and leave, and the weakest device it is for. Like an edition it cannot state a fact or
// widen what a role knows. `checkKit` is pure and DOM-free: the pack loader, the role card and Orac (AT16) call it the
// same way, and it returns plain sentences (empty: the kit loads).
import { typedFigures } from "./loader.ts";
import { sayNumber } from "../figures.ts";
import { ARRANGEMENTS } from "../layouts.ts";
import type { ScreenDoc } from "../screen.ts";
import type { Json } from "../json.ts";
import { checkLookDelivery, PAGE_TURN_MAX_MS, STARTER_DELIVERIES, TURN_MAX_MS, type LookDelivery } from "../delivery.ts";
import type { PartControl } from "../part-controls.ts";
import { VIEW_STYLES, type ViewStyle } from "../view-controls.ts";
import { checkControlSet, STARTER_CONTROL_SETS, type ControlSet } from "./controlset.ts";
import starters from "./kits/starters.json" with { type: "json" };

export const KIT_SCHEMA = "sse.kit/1";
export const SITUATIONS = ["sector", "system", "world", "ship", "landing", "holding"] as const;
export type Situation = (typeof SITUATIONS)[number];
export const TIERS = ["text", "canvas", "gpu-basic", "gpu-high"] as const;

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
  /**
   * How its screens arrive and leave (11A.4a; AT15a's one mechanism): a starter's set by name (hauler, corporate,
   * deck, paper, guide), or a look's own deliveries moment by moment, checked by `checkLookDelivery`.
   */
  delivery?: keyof typeof STARTER_DELIVERIES | LookDelivery;
  needs?: { tier: (typeof TIERS)[number]; falls_back_to?: string };
  layouts?: Partial<Record<Situation, KitLayout>>;
  /** How every kind of control is drawn in this kit (AT15; 11A.4 rule 5): a set by id, or a set of the kit's own. */
  control_set?: string | ControlSet;
  /** One-liners at the top and bottom of the role's screens (SSE3's guide tips), from these note sets by situation. */
  banners?: { at: "top" | "bottom" | "both"; sets: string[] };
}

/** A part as the stack knows it: its declared controls (AT13), and whether it is the GM's alone. */
export interface PartInfo {
  id: string;
  controls: readonly string[];
  gm_only?: boolean;
}

/** The parts a kit may use, read from the views' own declarations (`ViewModule.switchable`, AT13): the players' stack. */
export function partsOf(views: readonly { id: string; switchable?: readonly PartControl[] }[]): PartInfo[] {
  return views.map((v) => ({ id: v.id, controls: (v.switchable ?? []).map((c) => c.id) }));
}

export interface KitStack {
  kits?: readonly KitDoc[];
  parts?: readonly PartInfo[];
  looks?: readonly string[];
  palettes?: readonly Palette[];
  editions?: readonly string[];
  vocabs?: readonly string[];
  noteSets?: readonly string[];
  /** Control sets from the pack stack, beside the starters'. */
  controlSets?: readonly ControlSet[];
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
/** A kit's deliveries moment by moment: a starter's set when named, else as given. */
export function lookDeliveryOf(d: KitDoc["delivery"]): LookDelivery {
  if (d === undefined) return {};
  return typeof d === "string" ? { ...(STARTER_DELIVERIES[d] ?? {}) } : d;
}

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
    // A starter named by its id is read in full; a kit that extends one overrides moment by moment.
    if (d.delivery) out.delivery = { ...lookDeliveryOf(out.delivery), ...lookDeliveryOf(d.delivery) };
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
  if (typeof d.delivery === "string") {
    if (!(d.delivery in STARTER_DELIVERIES)) out.push(`The delivery "${d.delivery}" is not a starter's (${Object.keys(STARTER_DELIVERIES).join(", ")}); name one, or give the deliveries moment by moment.`);
  } else {
    for (const e of checkLookDelivery(d.delivery)) out.push(`The kit's ${e}.`);
    const turn = d.delivery?.turn;
    const cap = turn?.kind === "page_turn" ? PAGE_TURN_MAX_MS : TURN_MAX_MS;
    for (const ms of [turn?.ms, turn?.short_ms]) if (typeof ms === "number" && ms > cap) out.push(`A Turn lasts at most ${sayNumber(cap / 1000, 2, { decimals: 1 })} s, so stepping a book never becomes a wait.`);
  }
  if (d.control_set !== undefined) {
    const sets = [...STARTER_CONTROL_SETS, ...(stack.controlSets ?? [])];
    if (typeof d.control_set === "string") {
      if (!sets.some((x) => x.id === d.control_set)) out.push(`The control set "${d.control_set}" is not loaded (there are ${sets.map((x) => x.id).join(", ")}).`);
    } else out.push(...checkControlSet(d.control_set, sets));
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
    const areas: readonly string[] | null = l.arrangement ? (ARRANGEMENTS as Record<string, { areas: readonly string[] }>)[l.arrangement]?.areas ?? [] : null;
    for (const it of l.items ?? []) {
      const part = stack.parts?.find((p) => p.id === it.view);
      if (stack.parts && !part) out.push(`The ${s} layout uses "${it.view}", which is not a part on a player's screen.`);
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

/** The control set a kit draws its controls in: its own, else by id, else the plain set. */
export function controlSetOf(kit: KitDoc | undefined, sets: readonly ControlSet[] = STARTER_CONTROL_SETS): ControlSet {
  const c = kit?.control_set;
  if (c && typeof c === "object") return c;
  return sets.find((x) => x.id === c) ?? STARTER_CONTROL_SETS[0]!;
}

/** The symbology a kit's screens draw in when they name none: the players' diagrammatic one. */
export const KIT_SYMBOLOGY = "sse.symbology.diagrammatic";

/**
 * A kit's screen for one situation, as an ordinary `sse.screen/1` document (11A.2: layouts are screen documents made
 * of parts), for the players' screen to show when the role has no layout of its own chosen on the Switchboard. The kit
 * is resolved first (its `extends` and each layout's `extends`). Null when the kit has no boxes for the situation (a
 * holding card is a card, not a screen): the caller falls back to the Bridge.
 */
export function kitScreen(kit: KitDoc | string, situation: Situation, docs: readonly KitDoc[] = []): ScreenDoc | null {
  let k: KitDoc;
  try {
    k = resolveKit(typeof kit === "string" ? kit : kit.id, typeof kit === "string" ? docs : [...docs.filter((d) => d.id !== kit.id), kit]);
  } catch {
    return null;
  }
  const l = k.layouts?.[situation];
  if (!l?.items?.length) return null;
  const arrangement = (l.arrangement && l.arrangement in ARRANGEMENTS ? l.arrangement : l.items.length === 1 ? "one" : "report") as keyof typeof ARRANGEMENTS;
  const vocab = k.vocab === "physics" || k.vocab === "story" ? k.vocab : "plain";
  return {
    schema: "sse.screen/1",
    id: `${k.id}.${situation}`,
    label: `${k.name}: ${situation}`,
    description: k.description,
    audience: "players",
    layout: ARRANGEMENTS[arrangement].layout,
    theme: k.look ?? "chart",
    symbology: KIT_SYMBOLOGY,
    vocab,
    items: l.items.map((it) => ({
      view: it.view,
      area: it.area,
      ...(it.style ? { style: it.style } : {}),
      ...(it.params ? { params: it.params as Record<string, Json> } : {}),
      ...(it.controls !== undefined ? { controls: it.controls } : {}),
    })),
  };
}
