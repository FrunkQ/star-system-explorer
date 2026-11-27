// Screen documents (H8.3). The GM workspace is one of these: a fixed,
// curated layout (H8.8) whose areas hold view modules.
import type { Json } from "./json.ts";
import { resolveParams, type TopicQuery, type ViewModule } from "./view.ts";
import { checkStyle, COSTS, settingsCost, TIER_RUNS_UP_TO, type StyleDoc } from "./style.ts";
import type { ParamSpec } from "./view.ts";
import { checkItemControls, isPicture, resolveControls, type ItemControls } from "./part-controls.ts";
import { checkDesks, shows, type Desk } from "./desk.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyView = ViewModule<any, any>;

export interface ScreenItem {
  view: string;
  area: string;
  params?: Record<string, Json>;
  /** What this item shows (H8.3 `bind`): only these subjects or kinds of the scene; absent shows the whole scene. */
  bind?: TopicQuery;
  /** A style document for this item's view (H8.9); the item's own params go on top. */
  style?: string;
  /** The part's own controls on this screen (AT13): "none" for a pure display, "all", or each by id; unstated take the part's default. */
  controls?: ItemControls;
  /**
   * A box that always shows one place (PB13 part 2: a bookmark put on a layout), drawn from what the role knows of it,
   * whatever is sent; `focus` is the thing in it the box is about. Absent: the box shows whatever is sent.
   */
  place?: BoxPlace;
}

/** The place a layout's box holds: a bookmark's scope, the thing selected there, and its name. */
export interface BoxPlace {
  scope: string;
  focus?: string;
  title?: string;
}

export interface ScreenDoc {
  schema: "sse.screen/1";
  id: string;
  label: string;
  audience: "gm" | "players" | `player:${string}`;
  layout: "chart-room" | "grid" | "flow" | "stack" | "page";
  theme: string;
  symbology: string;
  vocab: "physics" | "plain" | "story";
  /** One line for people and Orac choosing a screen. */
  description?: string;
  /** The data view the screen is seen through; absent uses the audience's own (H8.3). */
  data_view?: string;
  items: ScreenItem[];
  /**
   * Where the screen sits on the GM's instrument rail (top-level design 2.2): one entry per scale it serves. The rail is
   * built from these, so adding a screen adds its entry, and packs, Orac and player screens can read the same list.
   */
  rail?: RailEntry[];
}

/** Scales the rail serves (the place line's steps), plus the Workshop: everything that changes the tool, not the universe. */
export const RAIL_SCALES = ["web", "cluster", "galaxy", "system", "world", "workshop"] as const;
export type RailScale = (typeof RAIL_SCALES)[number];
/** The rail's groups, always in this order; "world" is "This world". */
export const RAIL_GROUPS = ["look", "find", "world", "travel", "understand", "table", "workshop"] as const;
export type RailGroup = (typeof RAIL_GROUPS)[number];
export const RAIL_GROUP_WORDS: Record<RailGroup, string> = { look: "Look", find: "Find", world: "This world", travel: "Travel", understand: "Understand", table: "Table", workshop: "Workshop" };
/**
 * At most this many entries at one scale (design 3.4 rule 4: ten). Eleven since Roll call (track SH): Alex, 25 Nov 2026
 * 01:59 UTC, "we can overpack the UI a bit for now - there will have to be a rationalisation once everything is in
 * place". Back to ten when the rails are rationalised.
 */
export const RAIL_MAX = 11;

export interface RailEntry {
  scale: RailScale;
  group: RailGroup;
  /** In words, as the rail shows it ("Sky from here"). */
  label: string;
  /** Order within the group, smallest first. */
  order: number;
  /** One line on what it shows. */
  hint?: string;
  /**
   * What the stack must give for the entry to show: each a `provides.other` key some stacked pack carries
   * ("code_sheets"). A screen a game pack brings is on the rail only while that pack is stacked, so no entry leads
   * nowhere. Absent: always shown.
   */
  needs?: string[];
  /** The desks the entry is listed on (three desks design section 5); left out: all three. */
  desks?: Desk[];
}

/** One rail line: the screen it opens and how it is shown. */
export interface RailLine extends RailEntry {
  screen: string;
}

function checkRail(rail: unknown): string[] {
  if (rail === undefined) return [];
  if (!Array.isArray(rail)) return ["rail must be a list"];
  const errors: string[] = [];
  const seen = new Set<string>();
  for (const r of rail as RailEntry[]) {
    if (!r || typeof r !== "object") {
      errors.push("a rail entry must be an object");
      continue;
    }
    if (!RAIL_SCALES.includes(r.scale)) errors.push(`rail: unknown scale ${String(r.scale)}`);
    if (!RAIL_GROUPS.includes(r.group)) errors.push(`rail: unknown group ${String(r.group)}`);
    if (typeof r.label !== "string" || !r.label.trim()) errors.push("rail: an entry needs a label");
    if (typeof r.order !== "number" || !Number.isFinite(r.order)) errors.push("rail: an entry needs a numeric order");
    if (r.needs !== undefined && !(Array.isArray(r.needs) && r.needs.every((n) => typeof n === "string" && n))) errors.push("rail: needs must be a list of pack keys");
    errors.push(...checkDesks(r.desks, `rail ${String(r.scale)}`));
    if (seen.has(r.scale)) errors.push(`rail: scale ${r.scale} is listed twice`);
    seen.add(r.scale);
  }
  return errors;
}

/**
 * The rail at one scale, from the screens' own entries: grouped in the fixed group order, ordered within each group.
 * Screens are given as id → document; a screen without an entry for the scale is not on it, nor one whose entry
 * `needs` something `has` says the stack lacks. Given a desk, an entry not listed on that desk is left out too.
 */
export function railOf(screens: Readonly<Record<string, Pick<ScreenDoc, "rail">>>, scale: RailScale, has: (need: string) => boolean = () => true, desk?: Desk): { group: RailGroup; words: string; lines: RailLine[] }[] {
  const lines: RailLine[] = [];
  for (const [screen, doc] of Object.entries(screens)) for (const r of doc.rail ?? []) if (r.scale === scale && (r.needs ?? []).every(has) && (!desk || shows(r, desk))) lines.push({ ...r, screen });
  return RAIL_GROUPS.map((group) => ({
    group,
    words: RAIL_GROUP_WORDS[group],
    lines: lines.filter((l) => l.group === group).sort((a, b) => a.order - b.order || (a.label < b.label ? -1 : a.label > b.label ? 1 : 0)),
  })).filter((g) => g.lines.length);
}

/** Whether a screen is on a desk: some rail entry of it is listed there. A screen with no rail entry is on every desk. */
export function screenOnDesk(doc: Pick<ScreenDoc, "rail">, desk: Desk): boolean {
  return !doc.rail?.length || doc.rail.some((r) => shows(r, desk));
}

/** The scales a screen is on, in rail order. */
export function scalesOf(doc: Pick<ScreenDoc, "rail">): RailScale[] {
  return RAIL_SCALES.filter((s) => (doc.rail ?? []).some((r) => r.scale === s));
}

/** Areas each layout offers. */
export const LAYOUT_AREAS: Record<ScreenDoc["layout"], string[]> = {
  "chart-room": ["chart", "card", "ruler"],
  grid: ["a", "b", "c", "d", "e", "f"],
  // One column in order (stack), wrapping tiles (flow), a printable report (page).
  flow: ["1", "2", "3", "4", "5", "6", "7", "8"],
  stack: ["1", "2", "3", "4", "5", "6", "7", "8"],
  page: ["1", "2", "3", "4", "5", "6", "7", "8"],
};

export interface CheckedScreen {
  doc: ScreenDoc;
  /** `controls`: which of the part's own controls are on; `picture`: every one is off, so nothing in it can be reached. */
  mounts: { module: AnyView; area: string; params: Record<string, Json>; bind?: TopicQuery; controls: Record<string, boolean>; picture: boolean; place?: BoxPlace }[];
}

/**
 * Validate a screen against the registry of view modules; nothing mounts
 * unless all of it is valid. `dataViews`, when given, are the data views the
 * campaign offers, and a screen naming another is refused. `styles` are the
 * style documents items may name, checked for the device tier they will run on.
 */
export function checkScreen(doc: ScreenDoc, registry: readonly AnyView[], dataViews?: readonly string[], styles?: { registry: readonly StyleDoc[]; tier: string }): { ok: true; screen: CheckedScreen } | { ok: false; errors: string[] } {
  const errors: string[] = [];
  if (!doc || typeof doc !== "object") return { ok: false, errors: ["not a screen document"] };
  if (doc.schema !== "sse.screen/1") errors.push(`unknown screen schema ${doc.schema}`);
  if (typeof doc.id !== "string" || !doc.id) errors.push("a screen needs an id");
  if (!Array.isArray(doc.items)) return { ok: false, errors: [...errors, "a screen needs a list of items"] };
  if (doc.data_view !== undefined && dataViews && !dataViews.includes(doc.data_view)) errors.push(`no data view ${doc.data_view} in this campaign (${dataViews.join(", ")})`);
  errors.push(...checkRail(doc.rail));
  const areas = LAYOUT_AREAS[doc.layout];
  if (!areas) errors.push(`unknown layout ${doc.layout}`);
  const used = new Set<string>();
  const mounts: CheckedScreen["mounts"] = [];
  for (const item of doc.items) {
    const module = registry.find((m) => m.id === item.view);
    if (!module) {
      errors.push(`no view module ${item.view}`);
      continue;
    }
    if (areas && !areas.includes(item.area)) errors.push(`layout ${doc.layout} has no area ${item.area}`);
    if (used.has(item.area)) errors.push(`area ${item.area} is used twice`);
    used.add(item.area);
    let r: { params: Record<string, Json>; errors: string[] };
    if (item.style) {
      const sd = styles?.registry.find((d) => d.id === item.style);
      if (!sd) r = { params: {}, errors: [`no style ${item.style}`] };
      else if (sd.view !== item.view) r = { params: {}, errors: [`style ${item.style} is for ${sd.view}`] };
      else {
        const c = checkStyle(item.style, styles!.registry, registry, styles!.tier, item.params ?? {});
        r = c.ok ? { params: c.params, errors: [] } : { params: {}, errors: c.errors.map((e) => `style ${item.style}: ${e}`) };
      }
    } else {
      r = resolveParams(module.params, item.params);
      const over = styles ? overBudget(module.params, r.params, styles.tier) : null;
      if (over) r.errors.push(over);
    }
    errors.push(...r.errors.map((e) => `${item.view}: ${e}`));
    const bad = item.bind ? checkBind(item.bind) : null;
    if (bad) errors.push(`${item.view}: bind ${bad}`);
    errors.push(...checkItemControls(module.switchable, item.controls).map((e) => `${item.view}: ${e}`));
    const controls = resolveControls(module.switchable, item.controls);
    if (item.place !== undefined && !(item.place && typeof item.place.scope === "string" && item.place.scope.startsWith("u:") && (item.place.focus === undefined || (typeof item.place.focus === "string" && item.place.focus.startsWith("u:")))))
      errors.push(`${item.view}: a box's place is a scope (and a focus) starting u:`);
    mounts.push({ module, area: item.area, params: r.params, ...(item.bind ? { bind: item.bind } : {}), controls, picture: isPicture(module.switchable, item.controls), ...(item.place ? { place: item.place } : {}) });
  }
  return errors.length ? { ok: false, errors } : { ok: true, screen: { doc, mounts } };
}

function checkBind(b: TopicQuery): string | null {
  for (const k of Object.keys(b)) if (!["addresses", "quantity_kinds", "event_kinds", "kinds"].includes(k)) return `has an unknown key ${k}`;
  for (const [k, v] of Object.entries(b)) if (!Array.isArray(v) || !v.every((x) => typeof x === "string")) return `${k} must be a list of names`;
  return null;
}

/**
 * The part of a player scene one item shows: its bound subjects (and their
 * children), or kinds; the scope itself stays so a chart keeps its star.
 */
export function boundScene<S extends { scope: string; subjects: { address: string; facts: { field: string; value?: Json }[] }[] }>(scene: S, bind: TopicQuery | undefined): S {
  if (!bind || (!bind.addresses?.length && !bind.kinds?.length)) return scene;
  const kind = (s: S["subjects"][number]) => String(s.facts.find((f) => f.field === "kind")?.value ?? "");
  const keep = (s: S["subjects"][number]) =>
    s.address === scene.scope ||
    (bind.addresses ?? []).some((a) => s.address === a || s.address.startsWith(`${a}/`)) ||
    (bind.kinds ?? []).includes(kind(s));
  return { ...scene, subjects: scene.subjects.filter(keep) };
}

function overBudget(specs: readonly ParamSpec[], params: Record<string, Json>, tier: string): string | null {
  const c = settingsCost(specs, params);
  const limit = TIER_RUNS_UP_TO[tier];
  return limit && COSTS.indexOf(c.cost) > COSTS.indexOf(limit) ? `over budget: ${c.by.join(", ")} cost ${c.cost}, and a ${tier} device runs up to ${limit}` : null;
}
