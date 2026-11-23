// Voyages between stars (V8 part 3): the pure parts. The kernel's
// `plan_voyage` (sse-nav voyage finder over the campaign's star cells) does
// every sum: legs, fuel carried, refuel stops, relativistic times and the
// refusals. This file words the answer and lays the systems out on a small
// top-down map. It computes no physics.

/** Stock ships that can cross between stars (base-nav pack), by template id. */
export const VOYAGE_SHIPS: readonly { id: string; name: string; drive: string }[] = [
  { id: "jump-courier", name: "Jump courier", drive: "Jump-2, a pack rule (fictional); hydrogen from a giant, ice or a port" },
  { id: "relativistic-probe", name: "Antimatter probe", drive: "1 g sustained, exhaust a third of light; antimatter, only where a port stocks it" },
  { id: "fusion-torch", name: "Fusion torch", drive: "1 g sustained, exhaust 2,000 km/s; deuterium and helium-3, skimmed from giants" },
  { id: "rescue-cutter", name: "Rescue cutter", drive: "2 g sustained, exhaust 3,000 km/s; deuterium from a giant, ice or a port" },
  { id: "chemical-transfer", name: "Chemical transfer ship", drive: "chemical: cannot cross between stars" },
];

export type Minimise = "home_time" | "ship_time" | "fuel" | "legs";

export const MINIMISE_WORDS: readonly { id: Minimise; label: string }[] = [
  { id: "home_time", label: "Least time at home" },
  { id: "ship_time", label: "Least time aboard" },
  { id: "fuel", label: "Least fuel" },
  { id: "legs", label: "Fewest legs" },
];

export interface VoyageAsk {
  from: string;
  to?: string;
  home?: string;
  ship: string | Record<string, unknown>;
  minimise?: Minimise;
  visited?: string[];
  known_fuel?: Record<string, string[]>;
  sensed_fuel?: Record<string, number>;
  surveyed?: string[];
  gambles?: boolean;
  /** In one system (part 3b): a NAV fixture, an inline NAV system or an adopted system; from and to are bodies or belt:<name>. */
  fixture?: string;
  nav?: unknown;
  system?: string;
  at_s?: number;
  depart_s?: number;
}

export interface VoyageLeg {
  from: string;
  to: string;
  means: "jump" | "flight" | "course";
  distance_pc: number;
  /** In-system legs: the NAV course (hohmann, lambert, torch) and the days waited for its window. */
  course?: string;
  wait_days?: number;
  /** In-system legs: speed over escape at the destination before the arrival burn, m/s. */
  arrive_v_inf_m_s?: number;
  counted_pc?: number;
  home_days: number;
  ship_days: number;
  jump_fuel_t: number;
  propellant_t: number;
  peak_beta?: number;
  coast: boolean;
  fictional: boolean;
  gamble: boolean;
  odds?: number;
  odds_from?: string;
  refuel?: string;
  jump_fuel_left_t: number;
  propellant_left_t: number;
  words: string;
}

export interface Voyage {
  from: string;
  to: string;
  minimise: Minimise;
  legs: VoyageLeg[];
  distance_pc: number;
  home_days: number;
  ship_days: number;
  jump_fuel_t: number;
  propellant_t: number;
  refuels: number;
  gambles: number;
  odds: number;
  fictional: boolean;
  needs?: string;
}

export interface VoyageSystem {
  address: string;
  name: string;
  xyz_pc: [number, number, number];
  fuel: string[];
  surveyed: boolean;
  fuel_odds?: number;
  odds_from?: string;
  visited: boolean;
}

export interface VoyagePlan {
  ship: string;
  means: { means: "jump" | "flight"; words: string; rule?: string; fictional: boolean }[];
  searched: number;
  cells: number;
  t_myr: number;
  systems: VoyageSystem[];
  voyage: Voyage | null;
  refusals: string[];
  model: string[];
}

export interface StarCellSystem {
  address: string;
  pinned: boolean;
  mass_msun: number;
  feh: number;
  xyz_pc: [number, number, number];
  companions: unknown[];
}

export interface StarCell {
  galaxy: string;
  cell: [number, number, number];
  frame: number;
  /** The start of the cell's star frame, where every drawn system is still in its cell. */
  frame_start_myr: number;
  t_myr: number;
  cell_pc: number;
  systems: StarCellSystem[];
}

/** A short name for a drawn system ("k3 in 988.0.0"), as the kernel words it. */
export function systemShort(address: string): string {
  const m = /\/cell:(-?\d+)_(-?\d+)_(-?\d+)\/sys:f-?\d+k(\d+)$/.exec(address);
  return m ? `k${m[4]} in ${m[1]}.${m[2]}.${m[3]}` : address;
}

/** Days as words for a voyage: hours, days, years, and thousands of years grouped. */
export function voyageSpan(d: number): string {
  if (!Number.isFinite(d)) return "never";
  const y = d / 365.25;
  if (y >= 100) return `${Math.round(y).toLocaleString("en-GB")} years`;
  if (y >= 2) return `${y.toFixed(1)} years`;
  if (d >= 2) return `${Math.round(d)} days`;
  return `${Math.round(d * 24)} hours`;
}

/** Peak speed in words: a share of light, or km/s when slow. */
export function speedWords(beta: number): string {
  return beta >= 0.01 ? `${(beta * 100).toFixed(1)}% of light` : `${Math.round(beta * 299_792.458).toLocaleString("en-GB")} km/s`;
}

/** The difference the crew feels: how much younger they come home. */
export function agesWords(v: Voyage): string | null {
  const gap = v.home_days - v.ship_days;
  if (gap < 1) return null;
  return `The crew ages ${voyageSpan(v.ship_days)}; ${voyageSpan(gap)} less than the people they left behind.`;
}

export interface MapFrame {
  /** Screen x, y for a position in parsecs (top-down, x right, y up). */
  at(p: readonly number[]): { x: number; y: number };
  scale: number;
}

/** A top-down frame that fits the given points with a margin. */
export function voyageFrame(w: number, h: number, points: readonly (readonly number[])[], pad = 18): MapFrame {
  let [x0, x1, y0, y1] = [Infinity, -Infinity, Infinity, -Infinity];
  for (const p of points) {
    x0 = Math.min(x0, p[0]!);
    x1 = Math.max(x1, p[0]!);
    y0 = Math.min(y0, p[1]!);
    y1 = Math.max(y1, p[1]!);
  }
  if (!Number.isFinite(x0)) [x0, x1, y0, y1] = [-1, 1, -1, 1];
  const span = Math.max(x1 - x0, y1 - y0, 1);
  const scale = Math.min(w - 2 * pad, h - 2 * pad) / span;
  const cx = (x0 + x1) / 2;
  const cy = (y0 + y1) / 2;
  return { scale, at: (p) => ({ x: w / 2 + (p[0]! - cx) * scale, y: h / 2 - (p[1]! - cy) * scale }) };
}

/** Ships for voyages inside one system (the base-nav pack's stock ships). */
export const IN_SYSTEM_SHIPS: readonly { id: string; name: string; drive: string }[] = [
  { id: "chemical-transfer", name: "Chemical transfer ship", drive: "chemical, coasts between burns; hydrogen and oxygen from ice or a port" },
  { id: "nuclear-thermal", name: "Nuclear thermal ship", drive: "nuclear thermal, coasts between burns; hydrogen from ice or a port" },
  { id: "fusion-torch", name: "Fusion torch", drive: "1 g sustained (fictional); deuterium and helium-3, skimmed from giants" },
  { id: "rescue-cutter", name: "Rescue cutter", drive: "2 g sustained (fictional); deuterium from a giant, ice or a port" },
];

/** A place inside a system, as plan_voyage lists it. */
export interface SystemPlace {
  address: string;
  name: string;
  a_au: number;
  fuel: string[];
  belt: boolean;
  visited: boolean;
}

/** plan_voyage's answer inside one system. */
export interface SystemVoyagePlan {
  scale: "system";
  system: string;
  ship: string;
  depart_s: number;
  places: SystemPlace[];
  voyage: Voyage | null;
  refusals: string[];
  model: string[];
}

/** A log-radius axis: x for an orbit's size in au, so inner worlds and the Kuiper belt fit one line. */
export function orbitAxis(w: number, places: readonly { a_au: number }[], pad = 28): (a_au: number) => number {
  const as = places.map((p) => p.a_au).filter((a) => a > 0);
  const lo = Math.log10(Math.min(...as, 0.3) * 0.8);
  const hi = Math.log10(Math.max(...as, 2) * 1.15);
  return (a) => pad + ((Math.log10(Math.max(a, 1e-3)) - lo) / (hi - lo || 1)) * (w - 2 * pad);
}
