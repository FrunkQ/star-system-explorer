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
import type { LifeEnergy, LifeView } from "./life.ts";

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

// The outside half (HV2b; Alex, 26 Nov: "sunlight can heat a core if close enough - we cant miss it... how much is
// absorbed, how much reflected (albedo visually)"). The climate's own sunlight books (`survey_surface` climate
// `sunlight`, HV2a), W/m² averaged over the whole surface: arriving = reflected + absorbed, and the world gives off
// emitted = absorbed + internal. Drawn as its own flow at its own scale beside the inside half, because sunlight is
// usually thousands of times the heat from inside; the inside half's "out through the surface" is what this half
// calls "heat from inside", and the words say how the two compare.

/** The climate's sunlight books, W/m² of surface (sse_climate `Sunlight`). */
export interface Sunlight {
  arriving_w_m2: number;
  absorbed_w_m2: number;
  reflected_w_m2: number;
  bond_albedo: number;
  internal_w_m2: number;
  emitted_w_m2: number;
}

export const SUN_NODES = {
  sun: "Sunlight",
  inside: "Heat from inside",
  skin: "Its surface and air",
  reflected: "Reflected to space",
  emitted: "Given off as warmth",
} as const;

export const SUN_COLUMNS = ["Comes from", "", "Ends up as"] as const;

export interface SunBooks {
  rows: FlowRow[];
  arriving_w: number;
  absorbed_w: number;
  reflected_w: number;
  inside_w: number;
  emitted_w: number;
  /** Share of the arriving sunlight sent straight back to space, 0 to 1 (the Bond albedo the climate uses). */
  albedo: number;
}

/** The outside half's rows, in W: what reaches the surface and air and what leaves it. */
export function sunBooks(env: Pick<HeatEnvelope, "reference_radius_m">, sun: Sunlight): SunBooks {
  const a = surfaceArea(env);
  const N = SUN_NODES;
  const b = { arriving_w: sun.arriving_w_m2 * a, absorbed_w: sun.absorbed_w_m2 * a, reflected_w: sun.reflected_w_m2 * a, inside_w: sun.internal_w_m2 * a, emitted_w: sun.emitted_w_m2 * a };
  const rows: FlowRow[] = [];
  const add = (from: string, to: string, amount: number, kind: string) => {
    if (amount > 0) rows.push({ from, to, amount, kind });
  };
  add(N.sun, N.skin, b.arriving_w, "light");
  add(N.inside, N.skin, b.inside_w, "heat");
  add(N.skin, N.reflected, b.reflected_w, "light");
  add(N.skin, N.emitted, b.emitted_w, "heat");
  return { rows, ...b, albedo: sun.bond_albedo };
}

export function sunColumn(id: string): number {
  if (id === SUN_NODES.sun || id === SUN_NODES.inside) return 0;
  return id === SUN_NODES.skin ? 1 : 2;
}

export function sunRank(id: string): number {
  const order: string[] = Object.values(SUN_NODES);
  const i = order.indexOf(id);
  return i < 0 ? order.length : i;
}

/** A share in whole percent, for words ("30%"). */
export const sharePercent = (x: number) => `${sayNumber(Math.round(x * 100), 3)}%`;

/**
 * The outside half in words, from the climate's sunlight books only: what arrives, what is reflected and soaked
 * up, and how that compares with the heat from inside (which one sets how warm the surface is). Curious gets two.
 */
export function sunWhy(env: HeatEnvelope, sun: Sunlight, depth: Depth = "science", insideLost_w = 0): string[] {
  const b = sunBooks(env, sun);
  const w = env.label || "The world";
  const out: string[] = [];
  if (b.arriving_w <= 0) {
    out.push(`No sunlight reaches ${w} in the climate's books, so all the warmth it gives off comes from inside.`);
    return out;
  }
  out.push(`Sunlight brings ${w} ${heatWatts(b.arriving_w, depth)}. It reflects ${sharePercent(b.albedo)} straight back to space and soaks up the rest, ${heatWatts(b.absorbed_w, depth)}.`);
  // The heat from inside: the climate's own figure where it counts one, else the core history's heat lost (the
  // inside half below), compared in words only; the climate's books are never changed.
  const inside = b.inside_w > 0 ? b.inside_w : insideLost_w;
  if (inside > 0) {
    const r = b.absorbed_w / inside;
    if (r >= 2) out.push(`That is about ${sayNumber(r, 2)} times the heat coming up from inside (${heatWatts(inside, depth)}), so sunlight, not its inside, sets how warm its surface is.`);
    else if (r <= 0.5) out.push(`Its inside gives off about ${sayNumber(1 / r, 2)} times the sunlight it soaks up, so its own heat, not its sun, keeps its surface warm.`);
    else out.push(`Sunlight and its own heat from inside are about as strong as each other at its surface.`);
  } else {
    out.push(`Almost no heat comes up from inside, so sunlight alone sets how warm its surface is.`);
  }
  if (depth === "curious") return out;
  out.push(`In balance it gives off ${heatWatts(b.emitted_w, depth)} as warmth (infrared): what it soaks up${b.inside_w > 0 ? " plus what comes from inside" : ""}.`);
  if (b.inside_w <= 0 && insideLost_w > 0) out.push(`The climate leaves out the heat from inside: next to the sunlight it is too small to change how warm the surface is.`);
  return out;
}

// Life's share (LE-U1; Alex, 26 Nov: energy available "will control the pace of development/evolution as it is the
// "first step" of the energy gradient"). The kernel's energy ledger (LE2, `life_view` `view.energy.world`): usable
// energy reaching places life could live, what its kinds of life could catch, what they catch, what staying alive
// costs, and what is spare. These nest inside one another and span many powers of ten, so they are drawn as a ladder
// of bars on a log scale under the sunlight soaked up, never as bands of the sunlight flow (they would be hairlines).

export interface LifeRung {
  id: "absorbed" | "arriving" | "offered" | "captured" | "keep_alive" | "surplus";
  label: string;
  w: number;
}

/** The ladder, biggest first; a line the kernel leaves null is left out. Surplus only when there is some. */
export function lifeLadder(e: LifeEnergy, absorbed_w: number | null): LifeRung[] {
  const w = e.world;
  const out: LifeRung[] = [];
  const add = (id: LifeRung["id"], label: string, v: number | null | undefined) => {
    if (typeof v === "number" && Number.isFinite(v) && v > 0) out.push({ id, label, w: v });
  };
  add("absorbed", "Sunlight soaked up", absorbed_w);
  add("arriving", "Usable energy where life could live", w.arriving_w);
  add("offered", "What its kinds of life could catch", w.offered_w);
  add("captured", "What life catches", w.captured_w);
  add("keep_alive", "What staying alive costs", w.keep_alive_w);
  add("surplus", "Spare, for growing and spreading", w.surplus_w);
  return out;
}

/** Bar lengths on a log scale, 0 to 1, over the ladder's own span (at least three powers of ten). */
export function ladderLengths(rungs: LifeRung[]): number[] {
  if (!rungs.length) return [];
  const logs = rungs.map((r) => Math.log10(r.w));
  const hi = Math.max(...logs);
  const lo = Math.min(Math.min(...logs), hi - 3) - 0.5;
  return logs.map((l) => (l - lo) / (hi - lo));
}

/**
 * The pace of evolution against Earth's, from the kernel's own opportunity clock (box D: opportunity gathers in
 * Earth-Myr, so Earth-Myr gathered per Myr is the pace against Earth's), over the last two points of the timeline.
 * Null when there is no life now or the clock has stopped.
 */
export function lifePace(v: Pick<LifeView, "timeline">): number | null {
  const t = v.timeline;
  if (t.length < 2) return null;
  const a = t[t.length - 2]!;
  const b = t[t.length - 1]!;
  if (!b.alive || b.at_myr <= a.at_myr) return null;
  const r = (b.opportunity - a.opportunity) / (b.at_myr - a.at_myr);
  return r > 0 && Number.isFinite(r) ? r : null;
}

/** Life's share in words: what it catches against the sunlight, whether it pays its keep, and how fast it evolves. */
export function lifeWhy(e: LifeEnergy, absorbed_w: number | null, pace: number | null, depth: Depth = "science"): string[] {
  const w = e.world;
  const out: string[] = [];
  const cap = w.captured_w;
  if (cap === null) {
    out.push(`How much energy life catches here is not known: ${e.unknown.length ? "some places life could live lack the figures it needs" : "the kernel leaves it out"}.`);
    return out;
  }
  if (cap <= 0) {
    out.push(w.arriving_w && w.arriving_w > 0 ? `No life catches any of the ${heatWatts(w.arriving_w, depth)} of usable energy reaching places it could live.` : `No usable energy reaches anywhere life could live, so nothing catches any.`);
    return out;
  }
  out.push(absorbed_w && absorbed_w > cap ? `Life catches ${heatWatts(cap, depth)}: about 1 part in ${sayNumber(absorbed_w / cap, 2)} of the sunlight it soaks up.` : `Life catches ${heatWatts(cap, depth)}.`);
  const keep = w.keep_alive_w;
  // "Alive, barely" when every habitat where life catches anything is barely alive (LE1).
  const living = e.habitats.filter((h) => !h.closed && (h.captured_w ?? 0) > 0);
  const barely = living.length > 0 && living.every((h) => h.barely);
  if (keep !== null && keep > 0) {
    if (barely) out.push(`Staying alive costs ${heatWatts(keep, depth)} at the thinnest population ever counted, more than it catches: it is alive, barely.`);
    // The kernel's own surplus (summed where life pays its keep), never caught less kept here.
    else if ((w.surplus_w ?? 0) > 0) out.push(`Staying alive costs ${heatWatts(keep, depth)}; where life pays its keep it has ${heatWatts(w.surplus_w!, depth)} spare for growing and spreading.`);
    else out.push(`Staying alive costs ${heatWatts(keep, depth)}, more than it catches: life here is doubtful.`);
  }
  if (depth === "curious") return out;
  if (pace !== null) out.push(pace >= 0.5 && pace <= 2 ? `Its life evolves at about Earth's pace.` : `Its life evolves at about ${sayNumber(pace, 2)} times Earth's pace: more energy caught means more chances to try new things.`);
  return out;
}
