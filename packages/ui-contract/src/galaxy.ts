// The galaxy chart (U6) and the foam map (V9): the pure parts. The kernel's
// `galaxy_chart` and `web_slice` (G1 to G4) grow the galaxy and the web from
// the universe's seed; this file only shapes their numbers for a chart. No
// physics here: radii, abundances, rates, arm crests and green bands are the
// kernel's own.

import { sayNumber, drawn } from "./figures.ts";

export interface GalaxyAsk {
  /** A named halo, or (with `cell`) one of that cell's catalogued haloes, `halo:N`. */
  halo?: string;
  /** A halo cell's index, from `halo_cell` (V9's cluster map). */
  cell?: [number, number, number];
  mass_msun?: number;
  at_gyr?: number;
  place_kpc?: number;
  place_phi?: number;
  /** History edits to preview (G6), applied from their own times on. */
  edits?: GalaxyEdit[];
}

/** An authored change to a galaxy's history (galaxy design 11, kernel G6). */
export type GalaxyEdit =
  | { kind: "merger"; t_gyr: number; ratio: number }
  | { kind: "remove_merger"; index: number }
  | { kind: "starburst"; t_gyr: number; gas_fraction: number }
  | { kind: "quiet_nucleus"; from_gyr: number; to_gyr: number };

/** A galaxy-wide event (G6): mergers, inked starbursts, nucleus episodes. */
export interface GalaxyEventMark { kind: string; id: string; t_myr: number; grid_pc: [number, number, number] }

export interface HistoryCompare {
  first_change_gyr: number | null;
  morphology: [string, string];
  curves: { name: string; t_gyr: number[]; before: number[]; after: number[]; max_change: number }[];
}

export interface GalaxyState {
  t_gyr: number;
  z: number;
  halo: number;
  gas: number;
  stars: number;
  bulge: number;
  sfr: number;
  z_gas: number;
  black_hole: number;
  quenched: boolean;
  v_c: number;
  disc_kpc: number;
}

export type Group = "alpha" | "fe" | "cn" | "s" | "r" | "k40";

export interface GalaxyRing {
  inner_kpc: number;
  outer_kpc: number;
  radius_kpc: number;
  gas: number;
  stars: number;
  /** [X/H] in dex, by element group. */
  abundances: Record<Group, number>;
  deaths: { core_collapse: number; type_ia: number; neutron_star_mergers: number; agb_returned: number };
  cc_per_century: number;
  ia_per_century: number;
}

export interface CosmicEpoch { id: string; name: string; log10_yr: number; z?: number | null; speculative: boolean; why: string }

export interface PlaceLocal {
  place: { r_kpc: number; phi: number; z_pc: number; t_gyr: number };
  gas_pc3: number;
  stars_pc3: number;
  systems_pc3: number;
  feh: number;
  alpha_fe: number;
  c_o: number;
  r_fe: number;
  k40_fe: number;
  sfr_gyr_pc3: number;
  cc_gyr_pc3: number;
  ia_gyr_pc3: number;
  arm: number;
  texture: number;
  sterilising_per_gyr: number;
  encounters_per_myr: number;
  grb_per_gyr: number;
  agn_dose: number;
  rocky_for_gyr?: number | null;
}

export interface PlaceGreen {
  metals_enough: boolean;
  metals_not_too_many: boolean;
  few_supernovae: boolean;
  few_passages: boolean;
  few_bursts: boolean;
  nucleus_quiet: boolean;
  quiet_long_enough: boolean;
  habitable: boolean;
  complex: boolean;
}

export interface GalaxyChart {
  halo: string;
  key: string;
  halo_mass_msun: number;
  morphology: string;
  barred: boolean;
  t_start_gyr: number;
  now_gyr: number;
  at_gyr: number;
  now: GalaxyState;
  history: GalaxyState[];
  active_nuclei: [number, number][];
  sun_radius_kpc: number;
  disc_kpc: number;
  arms: [number, number][][];
  arm_contrast: number;
  rings: GalaxyRing[];
  core_collapse_per_century: number;
  ia_per_century: number;
  green_bands: { t_gyr: number; habitable: [number, number] | null; complex: [number, number] | null }[];
  epochs: CosmicEpoch[];
  place: { local: PlaceLocal; green: PlaceGreen; ring: number } | null;
  /** GZ12: set when the charted galaxy is the universe's home (kernel `home_galaxy`), else null. */
  home?: HomeGalaxy | null;
  /** The halo's own mergers, with whether an edit took each out. */
  mergers: { index: number; t_gyr: number; ratio: number; removed: boolean }[];
  events: GalaxyEventMark[];
  edits: GalaxyEdit[];
  /** Before and after, when history is inked. */
  compare: HistoryCompare | null;
  /** What other galaxies brought (GH3): present when the galaxy is fed by its own assembly tree. */
  assembly?: GalaxyAssembly | null;
  model: string[];
}

/** One scar on a galaxy (GH3, galaxy histories design 9.4): the renderer draws these and nothing else. */
export interface GalaxyMark {
  kind: "warp" | "ripples" | "tilt" | "thick_disc" | "driven_arms" | "ring" | "burst_knots" | "lopsided" | "gas_trail" | "warm_side" | "fresh_gas" | string;
  /** The progenitor that made it (an address). */
  caused_by: string;
  began_gyr: number;
  /** 0 to 1 when it began. */
  strength: number;
  /** It falls by a factor e over this; null lasts for good. */
  fades_gyr: number | null;
  /** A few numbers for its shape: r_kpc (closest approach), prograde (-1 to 1), speed_kms. */
  shape: Record<string, number>;
}

/** GH3's tally of what the galaxy's assembly tree delivered. */
export interface GalaxyAssembly {
  exsitu_msun: number;
  stellar_halo_msun: number;
  stellar_halo_z: number;
  exsitu_z: number;
  thick_disc_msun: number;
  biggest: string;
  biggest_share: number;
  merged: number;
  torn_apart: number;
  members: number;
  marks: GalaxyMark[];
}

export interface SliceAsk {
  z?: number;
  t_gyr?: number;
  centre?: [number, number, number];
  extent_mpc: number;
  n: number;
  /** With the cube's galaxies: the smallest halo drawn, solar masses (never below the pack's floor). */
  galaxy_floor_msun?: number;
}

export interface WebSlice {
  t_gyr: number;
  z: number;
  now_gyr: number;
  slice: {
    scale_factor: number;
    growth: number;
    centre: [number, number, number];
    extent: number;
    n: number;
    octaves: number;
    density: number[];
    environment: ("void" | "sheet" | "filament" | "knot" | string)[];
    displaced: [number, number][];
  };
  model: string[];
}

/** A cube of the cosmic web for the 3D view (kernel `web_slice` with `cube`, sse-cosmos `Volume`). */
export interface WebCube {
  t_gyr: number;
  z: number;
  now_gyr: number;
  volume: {
    scale_factor: number;
    growth: number;
    centre: [number, number, number];
    extent: number;
    n: number;
    /** The octave the motion and environments are read at (truncated Zel'dovich) and its spacing, Mpc. */
    smoothing_octave: number;
    smoothing_mpc: number;
    /** Where each parcel is now, comoving Mpc, x y z per parcel, x fastest over the grid it started on. */
    position: number[];
    /** Density where it started, in units of the mean. */
    density: number[];
    environment: ("void" | "sheet" | "filament" | "knot" | string)[];
    /** The room each parcel takes today as a share of the room it started in: over 1 in voids. */
    room: number[];
  };
  model: string[];
}

/**
 * The galaxies in a cube of the web (kernel `web_slice` with `cube` and `haloes`, V9w light from haloes): G2's haloes
 * from the pack's floor up whose matter started inside the cube, moved by the cube's own step, each with its G3
 * galaxy's stars and star formation then.
 */
export interface WebGalaxies {
  t_gyr: number;
  z: number;
  now_gyr: number;
  centre: [number, number, number];
  extent: number;
  galaxies: { position: [number, number, number]; halo_msun: number; stars_msun: number; sfr_msun_yr: number; quenched: boolean }[];
  /** Smallest halo drawn, solar masses (pack `web_light.floor_msun`). */
  galaxy_floor_msun: number;
  model: string[];
}

/** Shares of space (by each parcel's room) and of matter (by count) in each environment of a cube. */
export function cubeShares(v: WebCube["volume"]): Record<"void" | "sheet" | "filament" | "knot", { space: number; matter: number }> {
  const out = { void: { space: 0, matter: 0 }, sheet: { space: 0, matter: 0 }, filament: { space: 0, matter: 0 }, knot: { space: 0, matter: 0 } };
  let room = 0;
  const n = v.environment.length;
  v.environment.forEach((e, i) => {
    const r = v.room[i] ?? 1;
    room += r;
    const k = out[e as keyof typeof out];
    if (!k) return;
    k.space += r;
    k.matter += 1;
  });
  for (const k of Object.values(out)) {
    k.space /= Math.max(1e-300, room);
    k.matter /= Math.max(1, n);
  }
  return out;
}

/** GZ12: the universe's home galaxy (kernel `home_galaxy`): one of the web's catalogued haloes. */
export interface HomeGalaxy {
  halo: string;
  key: string;
  cell: [number, number, number];
  mass_msun: number;
  position_mpc: [number, number, number];
  /** "web": the kernel's best fit to the home wish; "gm": chosen with "Make this home". */
  by: "web" | "gm";
  fits_wish: boolean;
  why: string[];
}

// GZ12: "home" is the universe's own home galaxy, one of the web's catalogued haloes chosen by the kernel (or the GM);
// the other three are reference galaxies, the median halo of their mass.
export const NAMED_HALOES: readonly { key: "home" | "dwarf" | "milky-way-like" | "group"; label: string }[] = [
  { key: "home", label: "Our galaxy" },
  { key: "dwarf", label: "A dwarf galaxy" },
  { key: "milky-way-like", label: "A Milky-Way-like galaxy" },
  { key: "group", label: "A galaxy group's central" },
];

export const MORPHOLOGY_WORDS: Record<string, string> = {
  dwarf: "dwarf",
  irregular: "irregular",
  disc: "spiral disc",
  barred: "barred spiral",
  lenticular: "lenticular (disc without arms)",
  elliptical: "elliptical",
};

export const GROUP_WORDS: Record<Group, string> = {
  alpha: "Rock-formers (O, Mg, Si)",
  fe: "Iron",
  cn: "Carbon and nitrogen",
  s: "Slow-process metals (barium, lead)",
  r: "Rapid-process metals (gold, uranium, thorium)",
  k40: "Potassium-40 (heat in rocks)",
};

/** The face-on chart frame: centre, pixels per kpc, out to the outermost ring. */
export interface DiscFrame { cx: number; cy: number; k: number; outer_kpc: number }

export function discFrame(w: number, h: number, chart: Pick<GalaxyChart, "rings">, pad = 16): DiscFrame {
  const outer = chart.rings.at(-1)?.outer_kpc ?? 1;
  return { cx: w / 2, cy: h / 2, k: (Math.min(w, h) / 2 - pad) / outer, outer_kpc: outer };
}

/** An arm crest as an SVG path in the frame (phi counter-clockwise from +x, y up). */
export function armPath(arm: readonly [number, number][], f: DiscFrame): string {
  return arm.map(([r, phi], i) => `${i ? "L" : "M"}${drawn(f.cx + r * f.k * Math.cos(phi))} ${drawn(f.cy - r * f.k * Math.sin(phi))}`).join(" ");
}

/** The ring that holds radius r (the kernel's rings tile the disc). */
export function ringAt(rings: readonly GalaxyRing[], r_kpc: number): number {
  const i = rings.findIndex((g) => r_kpc >= g.inner_kpc && r_kpc < g.outer_kpc);
  return i >= 0 ? i : r_kpc < (rings[0]?.inner_kpc ?? 0) ? 0 : rings.length - 1;
}

/** A shade from 0 (light) to 1 (dark) for a ring layer, scaled over the rings shown. */
export type RingLayer = "metals" | "stars" | "supernovae" | "heavy";
export function ringShades(rings: readonly GalaxyRing[], layer: RingLayer): number[] {
  const v = rings.map((g) =>
    layer === "metals" ? g.abundances.fe : layer === "heavy" ? g.abundances.r : layer === "stars" ? Math.log10(Math.max(1, g.stars) / Math.max(1e-30, Math.PI * (g.outer_kpc ** 2 - g.inner_kpc ** 2))) : Math.log10(Math.max(1e-12, g.cc_per_century + g.ia_per_century)),
  );
  const fin = v.filter(Number.isFinite);
  const lo = Math.min(...fin);
  const hi = Math.max(...fin);
  return v.map((x) => (Number.isFinite(x) && hi > lo ? (x - lo) / (hi - lo) : 0));
}

export const LAYER_WORDS: Record<RingLayer, { label: string; legend: string }> = {
  metals: { label: "Metals", legend: "darker: more iron in the gas ([Fe/H])" },
  heavy: { label: "Uranium, thorium, gold", legend: "darker: more rapid-process metals from neutron-star mergers" },
  stars: { label: "Stars", legend: "darker: more stars per square kiloparsec (log)" },
  supernovae: { label: "Supernovae", legend: "darker: more supernovae per century (log)" },
};

/** The green band at a time (nearest half-Gyr row at or before it). */
export function greenAt(chart: Pick<GalaxyChart, "green_bands">, t_gyr: number): { habitable: [number, number] | null; complex: [number, number] | null; t_gyr: number } | null {
  let best: GalaxyChart["green_bands"][number] | null = null;
  for (const b of chart.green_bands) if (b.t_gyr <= t_gyr + 1e-9) best = b;
  return best ?? chart.green_bands[0] ?? null;
}

/** History curves on a cosmic-time axis: x from t, y on a log axis over the series' range. */
export interface CurveAxis { x(t: number): number; t0: number; t1: number; w: number }
export function cosmicAxis(t0: number, t1: number, w: number, pad = 8): CurveAxis {
  return { t0, t1, w, x: (t) => pad + ((t - t0) / (t1 - t0)) * (w - 2 * pad) };
}

export function logCurve(points: readonly { t: number; v: number }[], ax: CurveAxis, top: number, h: number, fix?: { lo: number; hi: number }): { d: string; lo: number; hi: number } {
  const ok = points.filter((p) => p.v > 0 && Number.isFinite(p.v));
  if (!ok.length) return { d: "", lo: 0, hi: 0 };
  const ls = ok.map((p) => Math.log10(p.v));
  const hi = fix ? fix.hi : Math.ceil(Math.max(...ls));
  const lo = fix ? fix.lo : Math.max(Math.floor(Math.min(...ls)), hi - 3);
  const y = (l: number) => top + h - ((Math.max(lo, l) - lo) / Math.max(1e-9, hi - lo)) * h;
  return { d: ok.map((p, i) => `${i ? "L" : "M"}${drawn(ax.x(p.t))} ${drawn(y(Math.log10(p.v)))}`).join(" "), lo, hi };
}

export function sci(x: number, digits = 2): string {
  if (!Number.isFinite(x)) return "–";
  if (x === 0) return "0";
  const a = Math.abs(x);
  if (a >= 1e4 || a < 1e-2) {
    const e = Math.floor(Math.log10(a));
    const m = x / 10 ** e;
    return `${sayNumber(m, 0, { decimals: digits - 1 })} × 10^${e}`;
  }
  return sayNumber(x, digits + 1);
}

/** "1.6 × the Sun" from an [X/H] in dex. */
export function timesSun(dex: number): string {
  const f = 10 ** dex;
  return f >= 0.95 && f <= 1.05 ? "about the Sun's" : `${f < 10 ? sayNumber(f, 0, { decimals: f < 1 ? 2 : 1 }) : sayNumber(Math.round(f), 0, { decimals: 0 })} × the Sun's`;
}

export function dex(x: number): string {
  return `${x >= 0 ? "+" : "−"}${sayNumber(Math.abs(x), 0, { decimals: 2 })} dex`;
}

/** Cosmic epochs placed on a Gyr axis (only those after `t0`). */
export function epochTicks(epochs: readonly CosmicEpoch[], ax: CurveAxis): { x: number; e: CosmicEpoch }[] {
  return epochs.map((e) => ({ e, t: 10 ** e.log10_yr / 1e9 })).filter((x) => x.t >= ax.t0 && x.t <= ax.t1).map((x) => ({ x: ax.x(x.t), e: x.e }));
}

/** The foam slice as cells: density in units of the mean, drawn as darker ink where denser (log). */
export function foamCells(s: WebSlice["slice"]): { i: number; j: number; shade: number; env: string }[] {
  const out: { i: number; j: number; shade: number; env: string }[] = [];
  for (let j = 0; j < s.n; j++)
    for (let i = 0; i < s.n; i++) {
      const d = s.density[j * s.n + i] ?? 1;
      const shade = Math.max(0, Math.min(1, (Math.log10(Math.max(1e-3, d)) + 1) / 2.5));
      out.push({ i, j, shade, env: String(s.environment[j * s.n + i] ?? "") });
    }
  return out;
}

/** An edit in words, for the inked-history list. */
export function editWords(e: GalaxyEdit, mergers: readonly { index: number; t_gyr: number }[] = []): string {
  const g = (t: number) => `${sayNumber(t, 0, { decimals: 1 })} Gyr`;
  switch (e.kind) {
    case "merger":
      return `${e.ratio >= 0.25 ? "Major" : "Minor"} merger at ${g(e.t_gyr)} (1 : ${Math.round(1 / e.ratio)})`;
    case "remove_merger": {
      const m = mergers.find((x) => x.index === e.index);
      return `Took out the merger at ${m ? g(m.t_gyr) : `#${e.index}`}`;
    }
    case "starburst":
      return `Starburst at ${g(e.t_gyr)}: ${Math.round(e.gas_fraction * 100)}% of the gas into stars`;
    case "quiet_nucleus":
      return `Quiet nucleus from ${g(e.from_gyr)} to ${g(e.to_gyr)}`;
  }
}

/** Toggle taking out one of the halo's own mergers. */
export function toggleRemoval(edits: readonly GalaxyEdit[], index: number): GalaxyEdit[] {
  const has = edits.some((e) => e.kind === "remove_merger" && e.index === index);
  return has ? edits.filter((e) => !(e.kind === "remove_merger" && e.index === index)) : [...edits, { kind: "remove_merger", index }];
}

// ---- V9 cluster map: one halo cell (kernel `halo_cell`, G2) ----

export interface HaloCellAsk {
  cell?: [number, number, number];
  min_mass_msun?: number;
  limit?: number;
}

export interface CellHalo {
  address: string;
  key: string;
  mass_msun: number;
  position_mpc: [number, number, number];
  half_mass_gyr: number;
  spin: number;
  mergers: number;
  major_mergers: number;
  last_major_gyr: number | null;
  neighbour: { address: string; distance_mpc: number; bound: boolean; approach_gyr: number | null } | null;
}

export interface HaloCell {
  address: string;
  index: [number, number, number];
  side_mpc: number;
  origin_mpc: [number, number, number];
  delta_l: number;
  mass_msun: number;
  halo_mass_msun: number;
  total: number;
  above_min: number;
  min_mass_msun: number;
  haloes: CellHalo[];
  bins: { log10_mass: number; expected: number; drawn: number }[];
  now_gyr: number;
  model: string[];
}

/** A catalogued halo's address split into its cell and key, or null for a named halo. */
export function cellHalo(address: string | null | undefined): { cell: [number, number, number]; halo: string; key: string } | null {
  const m = /\/cell:(-?\d+)\.(-?\d+)\.(-?\d+)\/halo:(\d+)$/.exec(address ?? "");
  return m ? { cell: [Number(m[1]), Number(m[2]), Number(m[3])], halo: `halo:${m[4]}`, key: m[4]! } : null;
}

/** Which two axes a map shows, and the one it looks along. */
export type Projection = "xy" | "xz" | "yz";
export function projectHalo(p: readonly [number, number, number], origin: readonly [number, number, number], side: number, proj: Projection, size: number, pad = 12): { x: number; y: number } {
  const [a, b] = proj === "xy" ? [0, 1] : proj === "xz" ? [0, 2] : [1, 2];
  const k = (size - 2 * pad) / side;
  return { x: pad + (p[a]! - origin[a]!) * k, y: size - pad - (p[b]! - origin[b]!) * k };
}

/** A dot's radius for a halo: by the cube root of mass (its size), 2 px at the catalogue floor. */
export function haloRadius(mass: number, floor = 1e10): number {
  return 2 + 2.2 * (Math.cbrt(Math.max(1, mass / floor)) - 1);
}

/** The cell's overdensity in words. */
export function overdensityWords(delta_l: number): string {
  const p = Math.round(Math.abs(delta_l) * 100);
  if (Math.abs(delta_l) < 0.15) return "about as dense as the universe on average";
  return delta_l > 0 ? `denser than average (linear overdensity +${p}%)` : `emptier than average (linear underdensity ${p}%)`;
}

/**
 * GH8 (galaxy histories design 9.4, Alex 24 Nov 2026): the leading arm in words. Arms trail as a result of the disc's
 * differential rotation; a leading arm is only ever the mark of a recent pass or merger on a backwards orbit, and it
 * fades as the rotation winds it back. The chart names the latest one made by `t_gyr`: "A leading arm, from a
 * backwards pass 300 Myr ago". `strengthAt` is the renderer's own fade (markStrengthAt), passed in so this stays
 * pure; `showing` is the chart's floor for a mark still visible. Rarity is the kernel's (GH3 draws it from history;
 * the yardstick, at most about 1 in 100 spirals, is a pack parameter), never decided here.
 */
export interface LeadingArmWords {
  /** How many leading arms the history made by this time. */
  made: number;
  /** The latest, in a sentence. */
  line: string;
  /** Whether that one still shows at this time. */
  live: boolean;
  /** Its strength now (0 to 1). */
  strength: number;
}
/**
 * The strength at which a leading arm counts as seen: the kernel's `LeadingArmParams.seen_strength` (GH3, its
 * yardstick against Iye, Tadaki and Fukumoto 2019), not on the wire, so kept in step here.
 */
export const LEADING_ARM_SEEN = 0.25;

export function leadingArmWords(marks: readonly GalaxyMark[], t_gyr: number, strengthAt: (m: GalaxyMark, t: number) => number, showing = LEADING_ARM_SEEN): LeadingArmWords | null {
  // Only arms strong enough to be seen count: weaker ones stay in the history list with the other marks.
  const made = marks.filter((m) => m.kind === "leading_arm" && m.began_gyr <= t_gyr && m.strength >= showing);
  if (!made.length) return null;
  const last = made.reduce((a, b) => (b.began_gyr > a.began_gyr ? b : a));
  const ago = t_gyr - last.began_gyr;
  const strength = strengthAt(last, t_gyr);
  const live = strength >= showing;
  // Once wound back, the line stays for one fading time, then the history list alone keeps it.
  if (!live && ago > (last.fades_gyr ?? 0.9) + Math.log(last.strength / showing) * (last.fades_gyr ?? 0.9)) return null;
  const when = ago < 0.001 ? "just now" : ago < 1 ? `${Math.round(ago * 1000)} Myr ago` : `${sayNumber(ago, 2, { decimals: 1 })} Gyr ago`;
  const pass = typeof last.shape.r_kpc === "number" ? `a backwards pass ${sayNumber(last.shape.r_kpc, 2, { decimals: 0 })} kpc out` : "a backwards merger";
  const line = `A leading arm, from ${pass} ${when}: ${live ? "it still winds against the spin" : "the rotation has wound it back into the trailing sense"}.`;
  return { made: made.length, line, live, strength };
}

/**
 * The cell our galaxy sits in. The sky puts home at (4, 4, 4) Mpc in the web (F37) and a cell is 8 Mpc a side, so home
 * is in cell 0.0.0; if GZ12 moves home, this follows it.
 */
export const HOME_CELL: readonly [number, number, number] = [0, 0, 0];

const NAME_START = ["Ar", "Bel", "Cor", "Dra", "El", "Fen", "Gal", "Hal", "Ir", "Kel", "Lor", "Mor", "Ner", "Or", "Pel", "Quor", "Ras", "Sel", "Tor", "Ul", "Vel", "Wyr", "Xan", "Zer"] as const;
const NAME_MID = ["a", "e", "i", "o", "u", "ae", "ia", "on", "en", "ar"] as const;
const NAME_END = ["dis", "mar", "nor", "thas", "rin", "vex", "lune", "dor", "ris", "nae", "thil", "ra"] as const;

/**
 * A stand-in name for a galaxy nobody has named yet (Alex, 26 Nov: "until something is named we can use a random
 * one"). Keyed on the halo's address, so the same galaxy always gets the same name in the same universe. The naming
 * track replaces it.
 */
export function placeholderGalaxyName(address: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < address.length; i++) h = Math.imul(h ^ address.charCodeAt(i), 0x01000193) >>> 0;
  const a = NAME_START[h % NAME_START.length]!;
  const b = NAME_MID[Math.floor(h / NAME_START.length) % NAME_MID.length]!;
  const c = NAME_END[Math.floor(h / (NAME_START.length * NAME_MID.length)) % NAME_END.length]!;
  return a + b + c;
}

/**
 * A cell is named after the most important galaxy in it: ours where we live ("Milky Way neighbourhood"), otherwise its
 * biggest halo's galaxy, by its stand-in name until it has a real one. A cell with no halo drawn keeps its number.
 */
export function cellName(index: readonly [number, number, number], biggestAddress: string | null | undefined): string {
  if (index.every((v, i) => v === HOME_CELL[i])) return "Milky Way neighbourhood";
  return biggestAddress ? `${placeholderGalaxyName(biggestAddress)} neighbourhood` : `Cell ${index.join(" · ")}`;
}
