// The biosphere view (U7): the pure parts. The kernel's `life_view` (L1d)
// gives one world's life as data: a timeline of production and diversity,
// dated events with reasons, named epochs, guilds, traits, signatures by
// tier and story hooks. Here it becomes things to draw: epoch bands and
// event ticks on a time axis, a production curve on a log axis, a food web
// laid out by feeding level, and what a scan would show at each tier. Nothing
// here computes biology; every number is the kernel's.
import type { Json } from "./json.ts";
import { sayNumber, drawn } from "./figures.ts";

export type Lens3 = { physics: string; ecology: string; history: string };

export interface LifePoint {
  at_myr: number;
  alive: boolean;
  capabilities: string[];
  diversity: number;
  opportunity: number;
  production_gc_yr: number;
}

export interface LifeEvent {
  id: string;
  at_myr: number;
  kind: string;
  subject: string;
  why: string;
  causes: string[];
  dials: string[];
}

export interface LifeEpoch {
  name: string;
  from_myr: number;
  to_myr: number;
  major: boolean;
  significance: number;
  opened_by: string;
  why: Lens3;
}

export interface LifeGuild {
  id: string;
  role: string;
  level: number;
  grade: string;
  habitat_kind: string;
  plain_name: string;
  physics_name: string;
  production_gc_yr: number;
  intake_gc_yr: number;
  share: number;
  can_fly: boolean;
  max_mass_kg: number | null;
}

export interface LifeSignal {
  id: string;
  lr: number;
  value: number;
  why: string;
}

export interface LifeTrace {
  kind: string;
  at_myr: number;
  class: string;
  from_event: string;
  surviving: number;
}

export interface LifeSignatures {
  alive: boolean;
  ever_alive: boolean;
  colour: string | null;
  seasonal: boolean;
  o2: number;
  ch4_ppm: number;
  edge_nm: number | null;
  signals: LifeSignal[];
  candidate: boolean;
  confidence: number;
  blind: string | null;
  summary: string | null;
  guild_cards: string[];
  hazard: string | null;
  biomass_gc: number;
  traces: LifeTrace[];
  evidence: string;
}

export interface LifeHook {
  id: string;
  text: string;
  why: Lens3;
}

/** One set of energy lines (LE2 ledger), W; null where the inputs are missing. */
export interface LifeEnergyLines {
  arriving_w: number | null;
  offered_w: number | null;
  captured_w: number | null;
  keep_alive_w: number | null;
  surplus_w: number | null;
  brought_in_w: number;
}

/** The kernel's energy ledger for a world of the chart (LE2, `life_view` `view.energy`). */
export interface LifeEnergy {
  /** Per habitat: the same lines, flattened in, with where they come from. */
  habitats: (LifeEnergyLines & { habitat: string; kind: string; light_w: number | null; chemical_w: number | null; exergy_factor: number | null; barely?: boolean; closed?: string | null })[];
  world: LifeEnergyLines;
  unknown: string[];
  absorbed_sunlight_w: number | null;
  notes: string[];
}

export interface LifeView {
  /** The energy ledger (LE2); worlds of the chart only, absent for the example fixtures. */
  energy?: LifeEnergy | null;
  body: string;
  evidence: string;
  timeline: LifePoint[];
  events: LifeEvent[];
  epochs: LifeEpoch[];
  guilds: LifeGuild[];
  hooks: LifeHook[];
  signatures: LifeSignatures;
  traits: Record<string, Json> | null;
  dials: Record<string, number>;
  lenses: Record<string, number>;
}

/** The example worlds the kernel carries (L1d fixtures), with what each shows. */
export const LIFE_EXAMPLES: readonly { fixture: string; label: string; shows: string }[] = [
  { fixture: "earth", label: "Earth today", shows: "a teeming biosphere, read from orbit with confidence" },
  { fixture: "earth-archean", label: "Archean Earth", shows: "methane but no oxygen: a candidate, not a certainty" },
  { fixture: "europa", label: "Europa", shows: "alive under the ice, and every scan from outside is blind to it" },
  { fixture: "red-dwarf-desert", label: "Red-dwarf desert", shows: "a false alarm: oxygen with no life" },
  { fixture: "dead-earth", label: "Dead Earth", shows: "life that ended, and the rocks that remember it" },
];

/** Plain words for a capability id ("complex_cells" → "complex cells"). */
export const words = (id: string) => id.replace(/_/g, " ");

export interface TimeAxis {
  /** Myr at the left and right edges. */
  from: number;
  to: number;
  x: (myr: number) => number;
}

export function timeAxis(v: LifeView, width: number, pad = 0): TimeAxis {
  const ends = [...v.timeline.map((p) => p.at_myr), ...v.epochs.map((e) => e.to_myr), ...v.events.map((e) => e.at_myr)];
  const to = ends.length ? Math.max(...ends) : 1;
  const from = 0;
  const span = Math.max(1e-9, to - from);
  return { from, to, x: (m) => pad + ((m - from) / span) * (width - 2 * pad) };
}

/** Epoch bands across the axis; very short epochs keep a sliver so they can still be picked. */
export function epochBands(v: LifeView, axis: TimeAxis): { epoch: LifeEpoch; x0: number; x1: number; index: number }[] {
  return v.epochs.map((epoch, index) => {
    const x0 = axis.x(epoch.from_myr);
    const x1 = Math.max(x0 + 3, axis.x(epoch.to_myr));
    return { epoch, x0, x1, index };
  });
}

/**
 * Production on a log axis from the timeline (g C/yr). Points with no
 * production sit on the floor. Returns an SVG path and the decades shown.
 */
export function productionPath(v: LifeView, axis: TimeAxis, top: number, height: number): { d: string; lo: number; hi: number } {
  const live = v.timeline.filter((p) => p.production_gc_yr > 0).map((p) => p.production_gc_yr);
  if (!live.length) return { d: "", lo: 0, hi: 0 };
  const lo = Math.floor(Math.log10(Math.min(...live)));
  const hi = Math.ceil(Math.log10(Math.max(...live)));
  const span = Math.max(1, hi - lo);
  const y = (g: number) => (g > 0 ? top + height - ((Math.log10(g) - lo) / span) * height : top + height);
  const d = v.timeline.map((p, i) => `${i ? "L" : "M"}${drawn(axis.x(p.at_myr))} ${drawn(y(p.production_gc_yr))}`).join(" ");
  return { d, lo, hi };
}

/** Events worth a tick on the timeline: everything but routine capability gains, which go in the list. */
export const MARKED_KINDS = ["origin", "catastrophe", "capability_lost", "sterilisation", "extinction"] as const;

export interface WebNode {
  guild: LifeGuild;
  x: number;
  y: number;
  r: number;
  /** Makes food (producers), eats (consumers, parasites), recycles (decomposers) or lives with (symbionts). */
  part: "makes" | "eats" | "recycles" | "with";
}

export interface WebLink {
  from: string;
  to: string;
  /** Food flows from `from` to `to`. */
  recycle: boolean;
}

/**
 * The food web by feeding level: producers on the bottom row, each consumer
 * level above the one it eats; decomposers and symbionts in a column to the
 * side. Disc radius follows the order of magnitude of the carbon each guild
 * makes or takes in.
 * Consumers at level n eat level n-1 in the same habitat kind; decomposers
 * take from every guild in their habitat kind.
 */
export function foodWeb(guilds: readonly LifeGuild[], width: number, height: number): { nodes: WebNode[]; links: WebLink[]; levels: number } {
  const partOf = (g: LifeGuild): WebNode["part"] => (g.role === "producer" ? "makes" : g.role === "decomposer" ? "recycles" : g.role === "symbiont" ? "with" : "eats");
  const main = guilds.filter((g) => partOf(g) === "makes" || partOf(g) === "eats");
  const side = guilds.filter((g) => partOf(g) === "recycles" || partOf(g) === "with");
  const levels = Math.max(1, ...main.map((g) => g.level));
  const flux = (g: LifeGuild) => Math.max(g.production_gc_yr, g.intake_gc_yr, 1);
  const maxF = Math.max(1, ...guilds.map(flux));
  const sideW = side.length ? Math.min(180, width * 0.22) : 0;
  const rowH = height / levels;
  const rMax = Math.min(rowH * 0.36, 28);
  // Fluxes span many decades (a vent film to a forest), so disc radius follows log flux: the order of magnitude reads at a glance.
  const minF = Math.min(maxF, ...guilds.map(flux));
  const decades = Math.max(1, Math.log10(maxF) - Math.log10(minF));
  const radius = (g: LifeGuild) => 4 + (rMax - 4) * ((Math.log10(flux(g)) - Math.log10(minF)) / decades);
  const nodes: WebNode[] = [];
  for (let lv = 1; lv <= levels; lv++) {
    const row = main.filter((g) => g.level === lv).sort((a, b) => a.habitat_kind.localeCompare(b.habitat_kind) || a.id.localeCompare(b.id));
    const w = width - sideW;
    row.forEach((g, i) => nodes.push({ guild: g, x: ((i + 0.5) / row.length) * w, y: height - (lv - 0.5) * rowH, r: radius(g), part: partOf(g) }));
  }
  side.forEach((g, i) => nodes.push({ guild: g, x: width - sideW / 2, y: ((i + 0.5) / side.length) * height, r: radius(g), part: partOf(g) }));
  const links: WebLink[] = [];
  for (const n of nodes) {
    if (n.part === "eats") {
      for (const m of nodes) if ((m.part === "makes" || m.part === "eats") && m.guild.level === n.guild.level - 1 && m.guild.habitat_kind === n.guild.habitat_kind) links.push({ from: m.guild.id, to: n.guild.id, recycle: false });
    } else if (n.part === "recycles") {
      for (const m of nodes) if (m !== n && m.part !== "recycles" && m.guild.habitat_kind === n.guild.habitat_kind) links.push({ from: m.guild.id, to: n.guild.id, recycle: true });
    }
  }
  return { nodes, links, levels };
}

/** What a scan would show, tier by tier, in words (what players can learn, and what the GM knows). */
export function scanTiers(s: LifeSignatures): { tier: "afar" | "scanned" | "surveyed" | "past"; label: string; lines: string[] }[] {
  const pct = (x: number) => `${Math.round(x * 100)}%`;
  const afar: string[] = [];
  if (s.colour) afar.push(`${s.colour} where life covers the land${s.seasonal ? ", changing with the seasons" : ""}`);
  else afar.push("no colour that marks life");
  const scanned: string[] = [];
  scanned.push(`oxygen ${s.o2 >= 0.01 ? pct(s.o2) : s.o2 > 0 ? "a trace" : "none"}, methane ${s.ch4_ppm >= 0.1 ? `${sayNumber(s.ch4_ppm, 3)} ppm` : s.ch4_ppm > 0 ? "a trace" : "none"}`);
  if (s.edge_nm) scanned.push(`a red edge near ${Math.round(s.edge_nm)} nm`);
  for (const g of s.signals) scanned.push(g.why);
  scanned.push(s.candidate ? `verdict offered: life candidate, confidence ${confidenceWords(s.confidence)}` : `no candidate (confidence ${confidenceWords(s.confidence)})`);
  if (s.blind) scanned.push(`blind to: ${s.blind}`);
  const surveyed: string[] = [];
  surveyed.push(s.summary ?? "no biosphere found");
  if (s.guild_cards.length) surveyed.push(s.guild_cards.join(", "));
  if (s.hazard) surveyed.push(`to visitors: ${s.hazard}`);
  const past = s.traces.length ? s.traces.map((t) => `${words(t.kind)}: ${t.class} (from ${words(t.from_event)}, ${sayNumber(Math.round(t.at_myr), 0, { decimals: 0 })} Myr)`) : ["no record in the rocks"];
  return [
    { tier: "afar", label: "From afar", lines: afar },
    { tier: "scanned", label: "Scanned", lines: scanned },
    { tier: "surveyed", label: "Surveyed", lines: surveyed },
    { tier: "past", label: "In the rocks", lines: past },
  ];
}

/** "1.2 × 10¹⁷ g C/yr" style, or "none". */
export function carbonWords(g: number): string {
  if (!(g > 0)) return "none";
  const e = Math.floor(Math.log10(g));
  const m = g / 10 ** e;
  const sup = String(e).replace(/[-0-9]/g, (c) => "⁻⁰¹²³⁴⁵⁶⁷⁸⁹"["-0123456789".indexOf(c)]!);
  return `${sayNumber(m, 0, { decimals: 1 })} × 10${sup} g C/yr`;
}

/** A confidence as a whole percentage, rounded down so 0.995 never reads as certain. */
export const confidenceWords = (c: number) => `${Math.floor(c * 100 + 1e-9)}%`;
