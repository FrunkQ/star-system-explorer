// A world's heat books (track HF's screen, the melt design's C6): where the heat inside a world comes from and where
// it goes, at any step of its own thermal history, as rows for the Flow view (RK13). Every amount is a figure the
// kernel's core history publishes (`survey_surface` envelope `core.steps`), the same numbers that drive the world;
// nothing here sums physics. The only arithmetic is presentation: W/m² of surface times the world's area, so bands
// are powers, and comparing two published figures in words.
//
// The fields are the melt design's section 7 (handover/design/SSE4-melt-and-heat-design.md, agreed 26 Nov 2026 with
// the threads "Design melting inside worlds" and "Fix low heat flow in worlds"), W/m² of surface at every step:
//   heat_lost = heat_conducted + heat_erupted
//   heat_made = heat_radiogenic + heat_tidal
//   heat_lost = heat_made + mantle_cooling + lid_cooling + core_heat   (core_freezing and core_cooling make core_heat)
// Stores (mantle_cooling, lid_cooling) are negative while they warm. Until the kernel publishes them (HF4b.1) a step
// carries only today's CoreStep figures (q_core_tw, q_surface_tw, temperatures) and the books say what is missing.

import type { FlowRow } from "./flow.ts";
import { say, sayNumber } from "./figures.ts";
import type { Depth } from "./depth.ts";

/** One step of the core history, as the kernel publishes it. */
export interface HeatStep {
  t_gyr: number;
  t_mantle_k: number;
  t_cmb_k: number;
  q_core_tw: number;
  q_surface_tw: number;
  inner_core_km: number;
  field_ut?: number;
  // Section 7 (HF4b.1 onwards), W/m² of surface.
  heat_lost?: number;
  heat_conducted?: number;
  heat_erupted?: number;
  heat_made?: number;
  heat_radiogenic?: number;
  heat_tidal?: number;
  core_heat?: number;
  core_freezing?: number;
  core_cooling?: number;
  mantle_cooling?: number;
  lid_cooling?: number;
  lid_km?: number;
  crust_km?: number;
}

export interface HeatEvent {
  t_gyr: number;
  kind: string;
  why: string;
}

export interface HeatHistory {
  version: string;
  core_radius_km: number;
  steps: HeatStep[];
  events: HeatEvent[];
  dynamo_now: boolean;
  why_now: string;
}

/** The parts of the survey's envelope the heat view reads. */
export interface HeatEnvelope {
  label: string;
  kind: string;
  regime_now: string;
  regime_label: string;
  reference_radius_m: number;
  tidal_w_m2: number;
  core?: HeatHistory | null;
}

/** The section 7 fields the books need at a step; every one present means the books close. */
export const HEAT_BOOK_FIELDS = ["heat_lost", "heat_made", "heat_conducted", "heat_erupted", "heat_radiogenic", "heat_tidal", "core_heat", "core_freezing", "core_cooling", "mantle_cooling", "lid_cooling"] as const;

/** Does this step carry the full books (section 7), or only today's core history? */
export function booksClose(s: HeatStep): boolean {
  return HEAT_BOOK_FIELDS.every((f) => typeof s[f] === "number" && Number.isFinite(s[f]));
}

/** The world's surface area, m², for turning W/m² into watts. */
export function surfaceArea(env: Pick<HeatEnvelope, "reference_radius_m">): number {
  const r = env.reference_radius_m;
  return 4 * Math.PI * r * r;
}

// Node names, in words a newcomer reads. The world itself is the middle node.
export const HEAT_NODES = {
  coreCooling: "Core cooling",
  freezing: "Inner core freezing",
  core: "Core",
  decay: "Radioactive decay",
  tides: "Tides",
  mantle: "Mantle cooling",
  lid: "Lid cooling",
  conducted: "Through the rock",
  erupted: "Carried up by lava",
  surface: "Out through the surface",
  warmsMantle: "Warms the mantle",
  warmsLid: "Warms the lid",
} as const;

// The ship Power card's words (POWER_COLUMNS), so a world's energy flow reads like a ship's (Alex, 26 Nov: "Heat flow in planets == energy flow").
export const HEAT_COLUMNS = ["", "Comes from", "", "Ends up as"] as const;

export interface HeatBooks {
  rows: FlowRow[];
  /** True when every amount is the kernel's section 7 books; false when only today's core history is published. */
  closes: boolean;
  /** Heat lost through the surface at this step, W (the one heat number). */
  lost_w: number;
  /** What is not published yet, in words (empty when the books close). */
  missing: string;
}

/** The Flow rows at one step: sources, the world, where it goes. Amounts in W. */
export function heatBooks(env: HeatEnvelope, s: HeatStep): HeatBooks {
  const w = env.label || "The world";
  const N = HEAT_NODES;
  if (booksClose(s)) {
    const a = surfaceArea(env);
    const W = (f: (typeof HEAT_BOOK_FIELDS)[number]) => (s[f] as number) * a;
    const rows: FlowRow[] = [];
    const add = (from: string, to: string, amount: number, kind = "heat") => {
      if (amount > 0) rows.push({ from, to, amount, kind });
    };
    add(N.coreCooling, N.core, W("core_cooling"));
    add(N.freezing, N.core, W("core_freezing"));
    add(N.core, w, W("core_heat"));
    add(N.decay, w, W("heat_radiogenic"), "made");
    add(N.tides, w, W("heat_tidal"), "made");
    // A store gives heat up (a source) or takes it in (where some heat goes).
    const mc = W("mantle_cooling");
    const lc = W("lid_cooling");
    if (mc > 0) add(N.mantle, w, mc, "stored");
    else add(w, N.warmsMantle, -mc, "stored");
    if (lc > 0) add(N.lid, w, lc, "stored");
    else add(w, N.warmsLid, -lc, "stored");
    add(w, N.conducted, W("heat_conducted"));
    add(w, N.erupted, W("heat_erupted"), "lava");
    return { rows, closes: true, lost_w: W("heat_lost"), missing: "" };
  }
  // Today's core history: the heat rising out of the core, and the heat out of the mantle's top (with HF1, the heat
  // melting carries out as well). Where the rest comes from is not published, so no band pretends to know.
  const rows: FlowRow[] = [];
  if (s.q_core_tw > 0) rows.push({ from: N.core, to: w, amount: s.q_core_tw * 1e12, kind: "heat" });
  if (s.q_surface_tw > 0) rows.push({ from: w, to: N.surface, amount: s.q_surface_tw * 1e12, kind: "heat" });
  return {
    rows,
    closes: false,
    lost_w: s.q_surface_tw * 1e12,
    missing: "The kernel does not publish the rest of the books yet: how much is made by radioactive decay and by tides, how much is the mantle's own cooling, and how much leaves through the rock or as lava. They come with the melt work (package HF4b.1).",
  };
}

/** Columns for the heat books: the core's own sources, then what feeds the world, the world, where it goes. */
export function heatColumn(env: HeatEnvelope): (id: string) => number | undefined {
  const N = HEAT_NODES;
  return (id) => {
    if (id === N.coreCooling || id === N.freezing) return 0;
    if (id === N.core || id === N.decay || id === N.tides || id === N.mantle || id === N.lid) return 1;
    if (id === (env.label || "The world")) return 2;
    return 3;
  };
}

/** Order inside a column: deep things first, surface things last. */
export function heatRank(id: string): number {
  const order: string[] = Object.values(HEAT_NODES);
  const i = order.indexOf(id);
  return i < 0 ? order.length : i;
}

/** A power in words: "about 35 TW". */
export function heatWatts(w: number, depth: Depth = "science"): string {
  return say(w, "power", undefined, { depth }).text;
}

const GYR_S = 3.15576e16;

/** When a step falls, in words: its age and how long ago (offsets from the world's birth, never a date). */
export function heatWhen(h: HeatHistory, i: number, depth: Depth = "science"): { age: string; since: string; ago: string; now: boolean } {
  const s = h.steps[i]!;
  const last = h.steps[h.steps.length - 1]!;
  const now = i === h.steps.length - 1;
  // The magma ocean's end is within a few million years of birth: "0 s" would read as a fault.
  const age = s.t_gyr < 0.01 ? "just formed" : say(s.t_gyr * GYR_S, "time", undefined, { depth }).text;
  const ago = now ? "now" : `${say((last.t_gyr - s.t_gyr) * GYR_S, "time", undefined, { depth }).text} ago`;
  const since = s.t_gyr < 0.01 ? "just after it formed" : `${age} after it formed`;
  return { age, since, ago, now };
}

/** The step nearest a time since the world formed (Gyr offset). */
export function heatStepAt(h: HeatHistory, t_gyr: number): number {
  let best = 0;
  h.steps.forEach((s, i) => {
    if (Math.abs(s.t_gyr - t_gyr) < Math.abs(h.steps[best]!.t_gyr - t_gyr)) best = i;
  });
  return best;
}

/** The marks (CoreEvent kinds) up to and including a step, newest first: what has happened by then. */
export function heatMarksBy(h: HeatHistory, i: number): HeatEvent[] {
  const t = h.steps[i]!.t_gyr;
  return h.events.filter((e) => e.t_gyr <= t + 1e-9).sort((a, b) => b.t_gyr - a.t_gyr);
}

const youngAt = (h: HeatHistory, depth: Depth) => {
  const a = heatWhen(h, 0, depth).age;
  return a === "just formed" ? "Just after it formed," : `Young, at ${a},`;
};

const temp = (k: number, depth: Depth) => say(k, "temperature", undefined, { depth }).text;

/**
 * Why the world is as hot as it is at a step, in words built only from the published figures: what it loses, where
 * that comes from (the biggest sources named), how hot the mantle and core are, and how its heat loss compares with
 * its youth. Curious depth gets the first two sentences.
 */
export function heatWhy(env: HeatEnvelope, h: HeatHistory, i: number, depth: Depth = "science"): string[] {
  const s = h.steps[i]!;
  const b = heatBooks(env, s);
  const w = env.label || "The world";
  const when = heatWhen(h, i, depth);
  const out: string[] = [];
  const lead = when.now ? `${w} loses ${heatWatts(b.lost_w, depth)} of heat from inside through its surface.` : `${when.ago}, ${w} lost ${heatWatts(b.lost_w, depth)} of heat from inside through its surface.`;
  out.push(lead);
  if (b.closes) {
    const a = surfaceArea(env);
    const made = (s.heat_made ?? 0) * a;
    const parts = b.rows.filter((r) => r.to === w && r.from !== HEAT_NODES.core).concat(b.rows.filter((r) => r.from === HEAT_NODES.core));
    const top = [...parts].sort((x, y) => y.amount - x.amount)[0];
    if (top) {
      const left = top.from === HEAT_NODES.decay || top.from === HEAT_NODES.tides ? "made there and then" : "heat left over from its formation, given up as it cools";
      out.push(`The biggest share, ${heatWatts(top.amount, depth)}, is ${top.from.toLowerCase()}: ${left}.`);
    }
    if (made > 0 && b.lost_w > 0) out.push(made >= b.lost_w ? `It makes as much heat as it loses, or more, so it is not cooling down.` : `It loses more than it makes, so it is cooling down.`);
  } else if (s.q_core_tw > 0) {
    out.push(`${heatWatts(s.q_core_tw * 1e12, depth)} of that rises out of its core.`);
  }
  if (depth === "curious") return out;
  out.push(`Its mantle is at about ${temp(s.t_mantle_k, depth)} and the top of its core at ${temp(s.t_cmb_k, depth)}${s.inner_core_km > 0 ? `, with a solid inner core ${say(s.inner_core_km * 1000, "length", undefined, { depth }).text} in radius` : ""}.`);
  const first = h.steps[0]!;
  const firstLost = heatBooks(env, first).lost_w;
  if (i > 0 && firstLost > 0 && b.lost_w > 0) {
    const r = firstLost / b.lost_w;
    if (r > 1.5) out.push(`${youngAt(h, depth)} it lost ${heatWatts(firstLost, depth)}: about ${sayNumber(r, 2)} times as much. Its heat is running down as it ages.`);
    else if (r < 0.67) out.push(`${youngAt(h, depth)} it lost only ${heatWatts(firstLost, depth)}: it loses more now than then.`);
  }
  if (when.now && h.why_now) out.push(h.why_now);
  return out;
}

/**
 * Known faults in the numbers underneath, said where they apply (honesty marks). Each names the package that fixes
 * it and goes when that package lands. Chosen by the world's conditions, never by its name.
 */
export interface HeatCaveat {
  id: string;
  fixedBy: string;
  words: string;
}

const LID_REGIMES = ["stagnant-lid", "episodic-lid", "heat-pipe", "cold-ice-lid"];

export function heatCaveats(env: HeatEnvelope, s: HeatStep): HeatCaveat[] {
  const out: HeatCaveat[] = [];
  if (LID_REGIMES.includes(env.regime_now) && !booksClose(s)) {
    out.push({ id: "lid-hot", fixedBy: "HF1, then HF4b", words: "Under a lid the kernel's mantles run too hot (above 2,000 K, where published models of Mars give about 1,600 to 1,700 K): the mantle is held at a melting limit instead of melting, and dry rock is made a hundred times stiffer. A known fault, being fixed." });
  }
  // Tides matter where they are a real share of what leaves (a tenth or more).
  if (env.tidal_w_m2 * surfaceArea(env) >= 0.1 * heatBooks(env, s).lost_w && env.tidal_w_m2 > 0) {
    out.push({ id: "tides-one-response", fixedBy: "HF3 and HF6", words: "Tidal heat uses one response for every world, not this world's own interior, and the orbit side of tides is not modelled yet. Strongly heated moons come out one to two orders too cool." });
  }
  if (s.inner_core_km > 0) {
    out.push({ id: "inner-core-early", fixedBy: "the core nucleation fudge (fudge register)", words: "Inner cores freeze too early and grow too big: a stand-in rule for when the core's centre starts to freeze. Earth's comes out about 3,000 km in radius against 1,221 km measured." });
  }
  out.push({ id: "readers-made", fixedBy: "HF2", words: "Landforms on the surface (volcanoes, plates, lava plains) still read the heat made inside, not the heat lost shown here." });
  return out;
}
