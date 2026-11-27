// Lenses (H8.6): what the GM sees, for the job in hand. A lens is a saved
// preset over the same data. It decides which layers the chart draws, what a
// card shows and in what order, and which tools are on the bar. Switching
// lens never changes data, time or selection: nothing here sends an intent.
// Every player data view is also a lens ("as players see it"), so the two are
// one mechanism. Honesty marks (authored, impossible, fictional, changed
// physics) always show on the GM view, whatever the lens says.
import type { Cost } from "../../contracts/src/index.ts";
import { COSTS, TIER_RUNS_UP_TO } from "./style.ts";
import type { ObserveSetup } from "./data-views.ts";
import type { SeatPlan } from "./seats.ts";
import { EVERYONE } from "./seats.ts";

export const LENS_SCHEMA = "sse.lens/0" as const;

/** What the chart can draw. Each is one switch in a lens. */
export const LAYERS = [
  { id: "orbits", label: "Orbits" },
  { id: "zones", label: "Zones and belts" },
  { id: "labels", label: "Names" },
  { id: "distances", label: "Distance rings" },
  { id: "grid", label: "Survey grid" },
  { id: "unsurveyed", label: "Unsurveyed hatching" },
  { id: "moons", label: "Moon ticks" },
  { id: "stamps", label: "Evidence stamps" },
  { id: "key", label: "Key" },
  { id: "known", label: "What the table knows" },
] as const;
export type LayerId = (typeof LAYERS)[number]["id"];

/** Tools a lens may put on the bar. Typing on a card is allowed in every lens. */
export const TOOLS = [
  { id: "undo", label: "Undo and redo" },
  { id: "drag", label: "Drag orbits" },
  { id: "flows", label: "Open the flow from a why" },
  { id: "clip", label: "Copy and paste bodies" },
] as const;
export type ToolId = (typeof TOOLS)[number]["id"];

export type Density = "line" | "compact" | "full";

export interface LensCard {
  /** Object kind this applies to ("planet", "moon", "star", ...), or "*" for any. */
  subject_kind: string;
  /** Fields in the order shown. Absent: every field the subject has. */
  fields?: string[];
  density: Density;
}

export interface Lens {
  schema: typeof LENS_SCHEMA;
  id: string;
  label: string;
  description?: string;
  /** A player data view to look through: every subject is shown through it. */
  data?: string;
  /** Whose knowledge, for "as players see it" lenses: the audience's own grants (each body's scanned state). */
  view_as?: string;
  cards: LensCard[];
  layers: LayerId[];
  tools: ToolId[];
  /** Cost class of the heaviest layer, so the device edge rule applies. */
  cost: Cost;
  /**
   * Which name layer labels the chart (NM6, naming design 4.3): "designation" for a hard-science chart, "local" for
   * the chart a native would draw, any other layer a pack names. Absent: the common name, else the designation.
   */
  names?: string;
  /** Where it came from: shipped, the GM's own, a pack, or a player data view. Not saved in packs. */
  origin?: "shipped" | "mine" | "pack" | "players";
}

const ALL_LAYERS: LayerId[] = LAYERS.map((l) => l.id);
const ALL_TOOLS: ToolId[] = TOOLS.map((t) => t.id);

const lens = (l: Omit<Lens, "schema" | "origin">): Lens => ({ schema: LENS_SCHEMA, ...l, origin: "shipped" });

/** The shipped defaults (H8.6). */
export const SHIPPED_LENSES: readonly Lens[] = [
  lens({
    id: "basic",
    label: "Basic",
    description: "Name, class and a handful of headline facts. For newcomers and quick looks.",
    cards: [
      { subject_kind: "star", fields: ["mass_sun"], density: "compact" },
      { subject_kind: "*", fields: ["a_au", "mass_earth", "surface_k"], density: "compact" },
    ],
    layers: ["orbits", "zones", "labels"],
    tools: ["undo", "clip"],
    cost: "light",
  }),
  lens({
    id: "advanced",
    label: "Advanced",
    description: "Everything about the system, with every layer of the chart. Player knowledge is in Running and the player lenses.",
    cards: [{ subject_kind: "*", density: "full" }],
    layers: ALL_LAYERS.filter((l) => l !== "known"),
    tools: ALL_TOOLS,
    cost: "light",
  }),
  lens({
    id: "building",
    label: "Building",
    description: "Rich data and every editing tool, with unsurveyed hatching and evidence stamps prominent.",
    cards: [{ subject_kind: "*", density: "full" }],
    layers: ["orbits", "zones", "labels", "distances", "grid", "unsurveyed", "moons", "stamps", "key"],
    tools: ALL_TOOLS,
    cost: "light",
  }),
  lens({
    id: "running",
    label: "Running",
    description: "What matters at the table now: few facts, player knowledge marked, no editing clutter.",
    cards: [
      { subject_kind: "star", fields: ["mass_sun"], density: "line" },
      { subject_kind: "*", fields: ["a_au", "surface_k", "gravity_g", "day_h"], density: "line" },
    ],
    layers: ["orbits", "labels", "moons", "known"],
    tools: [],
    cost: "light",
  }),
];

/**
 * One "as players see it" lens per player data view, and one per audience
 * that keeps its own knowledge (the seats' fog of war). Built from the stack's
 * data views and the seat plan, so a new view or seat brings its lens.
 */
export function playerLenses(setup: ObserveSetup | undefined, plan?: SeatPlan): Lens[] {
  const out: Lens[] = [];
  const base = { schema: LENS_SCHEMA, cards: [{ subject_kind: "*", density: "full" as const }], layers: ["orbits", "zones", "labels", "moons"] as LayerId[], tools: [] as ToolId[], cost: "light" as Cost, origin: "players" as const };
  out.push({ ...base, id: "players", label: "As the table sees it", description: "What everyone at the table has been shown: each body through its own view (the Scanned button).", view_as: EVERYONE });
  for (const a of plan?.audiences ?? []) {
    if (a.id === EVERYONE) continue;
    out.push({ ...base, id: `players.${a.id}`, label: `As ${a.label} sees it`, description: `What ${a.label} has discovered.`, view_as: a.id });
  }
  for (const v of setup?.views ?? []) {
    out.push({ ...base, id: `view.${v.id}`, label: `Through ${v.label}`, description: v.description ?? `Every body through the ${v.label} view.`, data: v.id, view_as: EVERYONE });
  }
  return out;
}

/**
 * See as on the Run desk (three desks design section 3.1): three kinds only, because "What do they see?" is the one lens
 * question asked at the table. The game master (the Running lens, named for who it is), the table, and each role that
 * keeps its own knowledge. Data views, the other job lenses, packs' and your own stay on Explore and Build.
 */
export function tableLenses(all: readonly Lens[]): Lens[] {
  const gm = all.find((l) => l.id === "running");
  return [...(gm ? [{ ...gm, label: "Game master" }] : []), ...all.filter((l) => l.id === "players" || l.id.startsWith("players."))];
}

export function hasLayer(l: Lens | undefined, id: LayerId): boolean {
  return !l || l.layers.includes(id);
}

export function hasTool(l: Lens | undefined, id: ToolId): boolean {
  return !l || l.tools.includes(id);
}

/** True when the lens shows the chart as an audience sees it (the card turns into theirs). */
export function asPlayers(l: Lens | undefined): boolean {
  return !!l && (l.view_as !== undefined || l.data !== undefined);
}

/** The card rule for one kind of subject: its own, then "*", then everything at full density. */
export function cardFor(l: Lens | undefined, kind: string): LensCard {
  return l?.cards.find((c) => c.subject_kind === kind) ?? l?.cards.find((c) => c.subject_kind === "*") ?? { subject_kind: "*", density: "full" };
}

/** Facts in the order a card shows them. A field the lens hides is untouched, never cleared. */
export function cardFacts<F extends { field: string }>(card: LensCard, facts: readonly F[]): F[] {
  if (!card.fields) return [...facts];
  const by = new Map(facts.map((f) => [f.field, f]));
  return card.fields.flatMap((id) => (by.has(id) ? [by.get(id)!] : []));
}

/** Honesty marks: shown on the GM view whatever the lens says. */
export const HONESTY_MARKS: readonly string[] = ["authored", "impossible", "fictional", "changed-physics"];

/** The stamps a GM card shows: every stamp with the stamps layer on, the honesty marks always. */
export function gmStamps(l: Lens | undefined, stamps: readonly string[]): string[] {
  return hasLayer(l, "stamps") ? [...stamps] : stamps.filter((s) => HONESTY_MARKS.includes(s));
}

/** The audience whose scene a lens needs, or null when it needs none. */
export function lensAudience(l: Lens | undefined): string | null {
  if (!l) return null;
  if (l.view_as || l.data) return l.view_as ?? EVERYONE;
  return hasLayer(l, "known") ? EVERYONE : null;
}

/** The lens a device opens with when none was chosen: Basic on a basic device, Advanced otherwise. */
export function defaultLens(tier: string | undefined): string {
  return tier === "basic" ? "basic" : "advanced";
}

/** Whether a lens runs on a device of this tier (the edge rule, as for styles). */
export function lensFits(l: Lens, tier: string): boolean {
  const limit = TIER_RUNS_UP_TO[tier];
  return !limit || COSTS.indexOf(l.cost) <= COSTS.indexOf(limit);
}

const ID = /^[a-z0-9][a-z0-9.-]*$/;

/** Every problem with a lens document, in words a GM can act on; empty when it is good. */
export function checkLens(l: unknown): string[] {
  const e: string[] = [];
  const o = l as Partial<Lens> | null;
  if (!o || typeof o !== "object") return ["a lens is an object"];
  if (o.schema !== LENS_SCHEMA) e.push(`schema must be ${LENS_SCHEMA}`);
  if (typeof o.id !== "string" || !ID.test(o.id)) e.push("id must be lower case letters, digits, dots and dashes");
  if (typeof o.label !== "string" || !o.label.trim()) e.push("a lens needs a label");
  if (!Array.isArray(o.layers)) e.push("layers must be a list");
  else for (const x of o.layers) if (!ALL_LAYERS.includes(x)) e.push(`unknown layer ${String(x)}`);
  if (!Array.isArray(o.tools)) e.push("tools must be a list");
  else for (const x of o.tools) if (!ALL_TOOLS.includes(x)) e.push(`unknown tool ${String(x)}`);
  if (!Array.isArray(o.cards) || !o.cards.length) e.push("a lens needs at least one card rule");
  else
    for (const c of o.cards) {
      if (typeof c?.subject_kind !== "string") e.push("each card rule names a subject kind or *");
      if (!["line", "compact", "full"].includes(c?.density as string)) e.push(`density must be line, compact or full`);
      if (c?.fields !== undefined && (!Array.isArray(c.fields) || c.fields.some((f) => typeof f !== "string"))) e.push("card fields must be a list of field names");
    }
  if (!COSTS.includes(o.cost as Cost)) e.push(`cost must be one of ${COSTS.join(", ")}`);
  if (o.names !== undefined && (typeof o.names !== "string" || !ID.test(o.names))) e.push("names must be a name layer, such as designation, common or local");
  return e;
}

/** A lens as it is saved: no origin, keys in a fixed order (so a round trip is identical). */
export function saved(l: Lens): Lens {
  const out: Lens = { schema: LENS_SCHEMA, id: l.id, label: l.label, cards: l.cards.map((c) => ({ subject_kind: c.subject_kind, ...(c.fields ? { fields: [...c.fields] } : {}), density: c.density })), layers: [...l.layers], tools: [...l.tools], cost: l.cost };
  if (l.description) out.description = l.description;
  if (l.data) out.data = l.data;
  if (l.view_as) out.view_as = l.view_as;
  if (l.names) out.names = l.names;
  return out;
}

/** A new lens of the GM's own, started from another. */
export function copyLens(from: Lens, label: string, taken: readonly string[]): Lens {
  const stem = `mine.${label.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "lens"}`;
  let id = stem;
  for (let i = 2; taken.includes(id); i++) id = `${stem}-${i}`;
  const { description: _d, ...rest } = saved(from);
  return { ...rest, id, label, origin: "mine" };
}

export interface LensPackMeta {
  id: string;
  version?: string;
  description?: string;
}

/** Lenses shipped in a presentation pack: `provides.other.lenses`. */
export function lensPack(lenses: readonly Lens[], meta: LensPackMeta): Record<string, unknown> {
  return {
    format: "sse.pack/1",
    id: meta.id,
    version: meta.version ?? "0.1.0",
    kind: "list",
    kernel_range: "^0.1",
    licence: "Apache-2.0",
    description: meta.description ?? "GM lenses: what the chart room shows for each job.",
    provides: { other: { lenses: lenses.map(saved) } },
  };
}

/** The lenses a pack provides, each checked; a bad lens is reported, never half loaded. */
export function lensesFromPack(doc: unknown): { lenses: Lens[]; problems: string[] } {
  const raw = (doc as { provides?: { other?: { lenses?: unknown } } } | null)?.provides?.other?.lenses;
  if (raw === undefined) return { lenses: [], problems: [] };
  if (!Array.isArray(raw)) return { lenses: [], problems: ["provides.other.lenses must be a list"] };
  const lenses: Lens[] = [];
  const problems: string[] = [];
  for (const l of raw) {
    const e = checkLens(l);
    if (e.length) problems.push(`${(l as { id?: string })?.id ?? "a lens"}: ${e.join("; ")}`);
    else lenses.push({ ...saved(l as Lens), origin: "pack" });
  }
  return { lenses, problems };
}

/**
 * Every lens on offer, in the order the switcher lists them: shipped, the
 * GM's own, those from packs in the stack, then the player lenses. A later
 * lens with an id already taken is left out.
 */
export function allLenses(mine: readonly Lens[], packs: readonly unknown[], setup: ObserveSetup | undefined, plan?: SeatPlan): Lens[] {
  const out: Lens[] = [];
  const seen = new Set<string>();
  const put = (l: Lens) => {
    if (seen.has(l.id)) return;
    seen.add(l.id);
    out.push(l);
  };
  SHIPPED_LENSES.forEach(put);
  mine.forEach((l) => put({ ...l, origin: "mine" }));
  for (const p of packs) lensesFromPack(p).lenses.forEach(put);
  playerLenses(setup, plan).forEach(put);
  return out;
}
