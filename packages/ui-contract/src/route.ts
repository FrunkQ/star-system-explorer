// The astrogation console (V8): the pure parts. The kernel's `plan_route`
// (T3, sse-nav) does the physics: courses, burns, propellant, dose, hazards
// and difficulty. This file only turns a scene into the system NAV reads,
// words the answer, and lays a course out on a chart in the star's frame.
// It computes no physics: orbits are drawn as the ellipses their elements
// describe, and positions on a course are the kernel's own track.
import type { SceneSubject, SystemScene } from "./scene.ts";
import { YEAR_S } from "./intent.ts";
import { constant } from "./sheet.ts";
import { sayNumber, drawn } from "./figures.ts";

/** A ship as NAV reads it (sse-nav `Ship`), or a template id from the base-nav pack. */
export interface NavShip {
  name: string;
  accel_g: number;
  sustained: boolean;
  exhaust_km_s: number;
  wet_t: number;
  dry_t: number;
  propellant_t?: number;
  reserve?: number;
  g_limit?: number;
  shield_g_cm2?: number;
  fuels?: string[];
  /** Heat shield, shape in air and landing gear (sse-nav `entry`; the renderer's ship designer writes them). */
  shield?: { nose_radius_m: number; max_heat_flux_w_m2: number; max_heat_load_j_m2?: number; kind?: "ablative" | "reusable" | string; mass_t?: number };
  aero?: { ref_area_m2: number; cd: number; lift_area_m2?: number; cl_max?: number; l_over_d?: number };
  landing?: { legs: boolean; max_touchdown_m_s: number; chute?: { area_m2: number; cd: number; max_mach: number; max_q_pa: number; fill_s?: number } };
  /** People aboard (0: an uncrewed probe; missing: crewed). */
  crew?: number;
  /** Cargo aboard, tonnes, inside dry_t. */
  cargo_t?: number;
  /** Each fuel's share of the propellant mass. */
  fuel_shares?: Record<string, number>;
  /** Most deceleration the hull bears, g. */
  structure_g?: number;
}

/** The base-nav pack's stock ships (T3), by template id. Names for the picker; the kernel holds the numbers. */
export const STOCK_SHIPS: readonly { id: string; name: string; drive: string }[] = [
  { id: "chemical-transfer", name: "Chemical transfer ship", drive: "chemical, coasts between burns" },
  { id: "nuclear-thermal", name: "Nuclear thermal ship", drive: "nuclear thermal, coasts between burns" },
  { id: "fusion-torch", name: "Fusion torch", drive: "fusion torch, 1 g sustained (fictional)" },
  { id: "rescue-cutter", name: "Rescue cutter", drive: "torch, 2 g sustained (fictional)" },
];

/**
 * Ships a pack lists for the voyage planner and the ship designer (`provides.other.nav_ships.ships`, each an sse-nav
 * ship the kernel flies as given), in stack order; a higher pack's ship replaces a lower one with the same id.
 */
export function packShips(packs: readonly { id: string; provides?: { other?: Record<string, unknown> } }[]): { id: string; pack: string; ship: NavShip }[] {
  const out: { id: string; pack: string; ship: NavShip }[] = [];
  for (const p of packs) {
    const ships = (p.provides?.other?.nav_ships as { ships?: Record<string, NavShip> } | undefined)?.ships;
    if (!ships || typeof ships !== "object") continue;
    for (const [id, ship] of Object.entries(ships)) {
      if (!ship || typeof ship.name !== "string" || typeof ship.accel_g !== "number") continue;
      const i = out.findIndex((x) => x.id === id);
      const row = { id, pack: p.id, ship };
      if (i >= 0) out[i] = row;
      else out.push(row);
    }
  }
  return out;
}

/** One row of a ship picker: a stock ship (sent by id) or a pack's ship (sent whole, `ship`). */
export interface ShipChoice {
  id: string;
  name: string;
  drive: string;
  /** The pack that lists it; missing for a stock ship. */
  pack?: string;
  ship?: NavShip;
}

/** A pack ship's drive in words, from its own figures. */
export function packDriveWords(ship: NavShip): string {
  const j = (ship as NavShip & { jump?: { rating_pc?: number } }).jump;
  const g = `${ship.accel_g} g${ship.sustained ? " sustained" : ""}`;
  return j && typeof j.rating_pc === "number" ? `Jump-${j.rating_pc}, a pack rule (fictional); ${g} in a system` : g;
}

/**
 * The picker's rows: the stock ships, then every ship the stacked packs list (`packShips`), a pack ship keeping
 * its own id. A pack ship with a stock ship's id replaces it.
 */
export function shipChoices(stock: readonly { id: string; name: string; drive: string }[], packs: readonly { id: string; provides?: { other?: Record<string, unknown> } }[]): ShipChoice[] {
  const out: ShipChoice[] = stock.map((s) => ({ ...s }));
  for (const p of packShips(packs)) {
    const row: ShipChoice = { id: p.id, name: p.ship.name, drive: packDriveWords(p.ship), pack: p.pack, ship: p.ship };
    const i = out.findIndex((x) => x.id === p.id);
    if (i >= 0) out[i] = row;
    else out.push(row);
  }
  return out;
}

/** What to send the kernel for a picked row: the ship itself for a pack ship, else its template id. */
export function shipToSend(choice: ShipChoice): string | NavShip {
  return choice.ship ?? choice.id;
}

export interface NavBody {
  address: string;
  name: string;
  mass_earth: number;
  radius_m: number;
  a_au: number;
  e: number;
  i_deg?: number;
  node_deg?: number;
  varpi_deg?: number;
  mean_longitude_deg?: number;
  port?: boolean;
  /** The body's air (a ship at a world): at the ground, or at 1 bar on a giant. */
  air?: import("./ship-at-world.ts").WorldAir;
}

export interface NavSystem {
  name: string;
  star_mass_msun: number;
  star_luminosity_lsun: number;
  star_radius_rsun: number;
  t0_s: number;
  bodies: NavBody[];
  belts: { name: string; a_in_au: number; a_out_au: number }[];
}

export type Prefer = "fastest" | "save-fuel" | "balanced";

export interface RouteAsk {
  /** An inline system (from a scene), a NAV fixture (`sol`), or an adopted system's address (GM only). */
  nav?: NavSystem;
  fixture?: string;
  system?: string;
  at_s?: number;
  ports?: string[];
  ship: string | NavShip;
  from: string | { r_m: [number, number, number]; v_m_s: [number, number, number] };
  to?: string;
  depart_s?: number;
  window_days?: number;
  prefer?: Prefer;
}

/** Burns and hazards are timed from the course's departure (seconds after `depart_s`), as sse-nav gives them. */
export interface RouteBurn { at_s: number; dv_m_s: number; duration_s: number; what: string }
export interface RouteHazard { kind: string; text: string; at_s?: number }

/** One candidate course (sse-nav `Course`). */
export interface Course {
  id: string;
  kind: string;
  label: string;
  depart_s: number;
  arrive_s: number;
  days: number;
  dv_m_s: number;
  propellant_t: number;
  reserve_left: number;
  peak_g: number;
  burns: RouteBurn[];
  dose_msv: number;
  spe_chance: number;
  peak_flux_w_m2?: number;
  closest_star_au: number;
  delay_home_s: number;
  delay_max_s: number;
  hazards: RouteHazard[];
  difficulty: { score: number; label: string; reasons: string[] };
  why: string[];
  track: [number, number, number][];
  /** Speed over escape at the destination before the arrival burn, m/s (0 for a torch flight). */
  arrive_v_inf_m_s?: number;
}

export interface WayOut { kind: string; label: string; body?: string; dv_m_s: number; at_s: number; days: number; feasible: boolean; why: string[] }

export interface RoutePlan {
  version: string;
  system: string;
  from: string;
  to: string;
  prefer: Prefer;
  ship: NavShip;
  dv_budget_m_s: number;
  usable_dv_m_s: number;
  candidates: Course[];
  recommended?: number | null;
  refused: { kind: string; why: string }[];
  drift?: { fate: string; fate_s?: number; r_au: number; speed_km_s: number; ways_out: WayOut[] } | null;
  model: string[];
}

const AU_M = constant("au");
const R_EARTH_M = constant("R_earth_mean");
const num = (s: SceneSubject, field: string): number | undefined => {
  const v = s.facts.find((f) => f.field === field)?.value;
  return typeof v === "number" && Number.isFinite(v) ? v : undefined;
};

/** What a scene could not give NAV, said plainly (shown beside the console). */
export interface NavFromScene {
  nav: NavSystem | null;
  /** Bodies NAV flies between (planets around the primary), nearest first. */
  bodies: { address: string; label: string }[];
  notes: string[];
}

/**
 * The system NAV reads, from a chart scene: the primary star's mass,
 * luminosity and radius (the base-physics flows derive the last two), and
 * each planet that orbits it directly, with the orbit's size, eccentricity
 * and where it is now (the scene's phase is the mean longitude at the
 * scene's "now"). Moons are left out: NAV does not fly between moons yet.
 * Orbits are taken as coplanar, since the chart carries no inclinations.
 */
export function navFromScene(scene: SystemScene, ports: readonly string[] = []): NavFromScene {
  const notes: string[] = [];
  const star = scene.subjects.find((s) => s.kind === "star" && !s.orbit);
  if (!star) return { nav: null, bodies: [], notes: ["This chart has no primary star, so there is nothing to plot courses around."] };
  const mass = num(star, "mass_sun");
  if (mass === undefined) return { nav: null, bodies: [], notes: [`${star.label} has no mass on the chart yet, so its pull is unknown.`] };
  const lum = num(star, "luminosity_sun");
  const rad = num(star, "radius_sun");
  if (lum === undefined) notes.push(`${star.label}'s luminosity is not derived yet; radiation and heat hazards assume one Sun.`);
  const bodies: NavBody[] = [];
  const skipped: string[] = [];
  for (const s of scene.subjects) {
    if (s.parent !== star.address || !s.orbit || s.kind === "belt" || s.kind === "zone") continue;
    const m = num(s, "mass_earth");
    const r = num(s, "radius_earth");
    if (m === undefined) {
      skipped.push(s.label);
      continue;
    }
    bodies.push({
      address: s.address,
      name: s.label,
      mass_earth: m,
      radius_m: (r ?? 1) * R_EARTH_M,
      a_au: num(s, "a_au") ?? s.orbit.a_au,
      e: num(s, "eccentricity") ?? 0,
      mean_longitude_deg: s.orbit.phase_deg,
      port: ports.includes(s.address),
    });
  }
  bodies.sort((a, b) => a.a_au - b.a_au);
  if (skipped.length) notes.push(`Left out (no mass on the chart): ${skipped.join(", ")}.`);
  const moons = scene.subjects.filter((s) => s.kind === "moon").length;
  if (moons) notes.push(`${moons} moon${moons === 1 ? "" : "s"} not offered: courses run between planets for now, and arrive in a parking orbit around them.`);
  notes.push("Orbits are taken as flat (one plane), since the chart has no inclinations.");
  const belts = scene.subjects.filter((s) => s.kind === "belt" && !!s.band_au).map((s) => ({ name: s.label, a_in_au: s.band_au![0], a_out_au: s.band_au![1] }));
  return {
    nav: {
      name: scene.title,
      star_mass_msun: mass,
      star_luminosity_lsun: lum ?? 1,
      star_radius_rsun: rad ?? 1,
      t0_s: scene.now_yr * Number(YEAR_S),
      bodies,
      belts,
    },
    bodies: bodies.map((b) => ({ address: b.address, label: b.name })),
    notes,
  };
}

/** "Keep us safe" reorders by difficulty, then dose; the kernel's own orders are fastest, save fuel and balanced. */
export function safestFirst(cs: readonly Course[]): Course[] {
  return [...cs].sort((a, b) => a.difficulty.score - b.difficulty.score || a.dose_msv - b.dose_msv || a.days - b.days);
}

/**
 * Up to `n` courses to show side by side: the kernel's order, one per kind of
 * course first (so three torch burns at different throttles don't crowd out
 * a coasting arc), then the rest.
 */
export function shortlist(cs: readonly Course[], n = 3): Course[] {
  const out: Course[] = [];
  const kinds = new Set<string>();
  for (const c of cs) if (!kinds.has(c.kind) && out.length < n) (kinds.add(c.kind), out.push(c));
  for (const c of cs) if (!out.includes(c) && out.length < n) out.push(c);
  return out;
}

export const DIFFICULTY_STEPS = ["routine", "easy", "moderate", "hard", "very hard"] as const;
/** 0 to 4 filled pips for the difficulty bar. */
export function difficultyPips(label: string): number {
  const i = (DIFFICULTY_STEPS as readonly string[]).indexOf(label);
  return i < 0 ? 0 : i;
}

export function daysWords(d: number): string {
  if (!Number.isFinite(d)) return "never";
  if (d < 1) return `${Math.round(d * 24)} h`;
  if (d < 60) return `${d < 10 ? sayNumber(d, 0, { decimals: 1 }) : Math.round(d)} days`;
  if (d < 730) return `${Math.round(d)} days`;
  return `${sayNumber(d / 365.25, 0, { decimals: 1 })} years`;
}

export function dvWords(m_s: number): string {
  return m_s >= 1e5 ? `${sayNumber(Math.round(m_s / 1000), 0, { decimals: 0 })} km/s` : `${sayNumber(m_s / 1000, 0, { decimals: m_s >= 1e4 ? 1 : 2 })} km/s`;
}

export function doseWords(msv: number): string {
  return msv >= 1000 ? `${sayNumber(msv / 1000, 0, { decimals: 1 })} Sv` : `${msv < 10 ? sayNumber(msv, 0, { decimals: 1 }) : Math.round(msv)} mSv`;
}

export function delayWords(s: number): string {
  if (s < 60) return `${Math.round(s)} s`;
  if (s < 3600) return `${Math.round(s / 60)} min`;
  return `${sayNumber(s / 3600, 0, { decimals: 1 })} h`;
}

/** A time on the campaign clock, as days from the scene's now ("in 3 days", "day +12"). */
export function fromNowWords(at_s: number, now_s: number): string {
  const d = (at_s - now_s) / constant("day");
  if (Math.abs(d) < 1 / 24) return "now";
  if (d < 0) return `${daysWords(-d)} ago`;
  return `in ${daysWords(d)}`;
}

/** The share of propellant left on arrival, 0 to 1, and whether it is under the ship's reserve. */
export function fuelLeft(c: Course, ship: NavShip): { left: number; reserve: number; under: boolean } {
  const prop = ship.propellant_t ?? ship.wet_t - ship.dry_t;
  const left = prop > 0 ? Math.max(0, Math.min(1, (prop - c.propellant_t) / prop)) : 0;
  const reserve = ship.reserve ?? 0.2;
  return { left, reserve, under: left < reserve - 1e-9 };
}

/** The chart frame a course is drawn in: the star at the centre, linear metres to pixels, sized to fit the course and both ends' orbits. */
export interface RouteFrame {
  cx: number;
  cy: number;
  /** Pixels per metre. */
  k: number;
  extent_m: number;
}

export function routeFrame(w: number, h: number, nav: NavSystem | null, plan: RoutePlan | null, course: Course | null, pad = 28): RouteFrame {
  let ext = 0;
  if (course) for (const p of course.track) ext = Math.max(ext, Math.hypot(p[0], p[1]));
  if (nav && plan) for (const b of nav.bodies) if (b.address === plan.from || b.address === plan.to || b.name === plan.from || b.name === plan.to) ext = Math.max(ext, b.a_au * (1 + b.e) * AU_M);
  if (!(ext > 0)) ext = (nav?.bodies.at(-1)?.a_au ?? 1) * AU_M;
  const k = (Math.min(w, h) / 2 - pad) / (ext * 1.04);
  return { cx: w / 2, cy: h / 2, k, extent_m: ext };
}

/** A course's track as an SVG path in the frame (y up on the chart). */
export function trackPath(track: readonly [number, number, number][], f: RouteFrame): string {
  return track.map((p, i) => `${i ? "L" : "M"}${drawn(f.cx + p[0] * f.k)} ${drawn(f.cy - p[1] * f.k)}`).join(" ");
}

/** A point of the track in pixels. */
export function trackPoint(p: readonly [number, number, number], f: RouteFrame): { x: number; y: number } {
  return { x: f.cx + p[0] * f.k, y: f.cy - p[1] * f.k };
}

/**
 * The orbit ellipse of a body, from its size, eccentricity and the longitude
 * of perihelion (geometry only). Returns null when it is too small to draw
 * or far outside the frame.
 */
export function orbitEllipse(b: NavBody, f: RouteFrame): { cx: number; cy: number; rx: number; ry: number; rot: number } | null {
  const a = b.a_au * AU_M * f.k;
  if (a < 2 || b.a_au * AU_M * (1 - b.e) > f.extent_m * 3) return null;
  const c = a * b.e;
  const w = ((b.varpi_deg ?? 0) * Math.PI) / 180;
  return { cx: f.cx - c * Math.cos(w), cy: f.cy + c * Math.sin(w), rx: a, ry: a * Math.sqrt(1 - b.e * b.e), rot: -(b.varpi_deg ?? 0) };
}

/** Where along the track (index) a burn or hazard falls; `after_s` is seconds after departure. */
export function trackIndexAt(c: Course, after_s: number): number {
  const n = c.track.length;
  if (n < 2 || c.arrive_s <= c.depart_s) return 0;
  const u = after_s / (c.arrive_s - c.depart_s);
  return Math.max(0, Math.min(n - 1, Math.round(u * (n - 1))));
}
