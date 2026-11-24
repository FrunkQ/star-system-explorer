// Getting down to a world and walking about (SL1; surfaces design 4.1, Alex 23 Nov 2026). The kernel's three
// pieces, as plain data: the world's landmarks (its named landscape, `survey_surface`'s register), a descent
// planned to any site (`plan_descent`: the site judged on the one height map, then NAV flies a lander down), and
// the ground at the fact floor round the site (`resolved_system` tiles). Any point is a valid site: a rough or wet
// one is offered with its dangers, never refused (danger is drama).

/** A named feature of a world's landscape (sse-surface `NamedFeature`), the usual way to choose where to land. */
export interface Landmark {
  id: string;
  address: string;
  /** ocean, continent, island, range, peak, deep, crater, basin, plain, volcano, ... */
  kind: string;
  lat_deg: number;
  lon_deg: number;
  extent_km: number;
  height_m?: number | null;
  cause: string;
  original_name: string;
  name?: string;
  remarkable: string[];
  authored?: boolean;
}

/** The site as the kernel judged it (sse-surface `SiteReport`). */
export interface LandingSite {
  lat_deg: number;
  lon_deg: number;
  level: number;
  spacing_m: number;
  height_m: number;
  slope_deg: number;
  local_slope_deg: number;
  roughness_m: number;
  relief_m: number;
  under_liquid?: string | null;
  ok: boolean;
  why: string[];
}

export interface DescentHazard {
  kind: string;
  /** ok, caution or danger. */
  level: string;
  source: string;
  why: string;
}

/** One telemetry sample of the kernel's descent (sse-nav `Telemetry`); altitude above the site's ground. */
export interface DescentSample {
  t_s: number;
  altitude_m: number;
  speed_m_s: number;
  vertical_m_s: number;
  downrange_km: number;
  decel_g: number;
  heat_w_cm2: number;
  phase: string;
  thrust: number;
}

export interface DescentEvent {
  t_s: number;
  kind: string;
  text: string;
}

export interface DescentPlan {
  body: string;
  label: string;
  lander: { name: string; mass_kg: number; [k: string]: unknown };
  entry: { altitude_m: number; speed_m_s: number; flight_path_deg: number };
  site: LandingSite;
  air_at_site: { density_kg_m3: number; sound_m_s: number; gas: string | null };
  gravity_m_s2: number;
  hazards: DescentHazard[];
  /** landed, landed-on-a-bad-site, hard-landing, crash, burned-up, out-of-fuel, still-descending. */
  verdict: string;
  descent: {
    outcome: { result: string; touchdown_m_s: number; duration_s: number; peak_g: number; peak_heat_w_cm2: number; propellant_used_kg: number; downrange_km: number; method: string; needs_engines: boolean; why: string[]; chute_terminal_m_s?: number };
    events: DescentEvent[];
    telemetry: DescentSample[];
  };
  at_s?: number;
}

/** The stock landers (base-nav pack `descent.landers`), by id; the kernel's plan names each in full. */
export const STOCK_LANDERS = ["capsule", "mars-lander", "titan-probe", "moon-lander"] as const;
export type StockLander = (typeof STOCK_LANDERS)[number];
export const LANDER_WORDS: Record<StockLander, string> = { capsule: "Crew capsule", "mars-lander": "Heavy Mars lander", "titan-probe": "Titan probe", "moon-lander": "Airless lander" };

/**
 * The stock lander a world's air suggests: none, an airless lander; thin (under a tenth of a bar), the Mars lander's
 * chute and engines; thick and cold enough for a long fall (over a bar at under 150 K), the Titan probe; else the
 * capsule. Only a first suggestion: the GM can pick any.
 */
export function suggestLander(pressure_bar: number, t_surface_k: number): StockLander {
  if (!(pressure_bar > 1e-4)) return "moon-lander";
  if (pressure_bar < 0.1) return "mars-lander";
  if (pressure_bar > 1 && t_surface_k < 150) return "titan-probe";
  return "capsule";
}

/** How a verdict reads. */
export const DESCENT_VERDICT_WORDS: Record<string, string> = {
  landed: "Lands safely",
  "landed-on-a-bad-site": "Gets down, onto bad ground",
  "hard-landing": "Lands hard",
  crash: "Crashes",
  "burned-up": "Burns up on the way in",
  "out-of-fuel": "Runs out of fuel",
  "still-descending": "Cannot get down: this lander has no way to stop",
};

/** Landmarks worth offering first: the remarkable, then the named landscape by size, then craters by size. */
export function landmarkOrder(list: readonly Landmark[]): Landmark[] {
  const tier = (l: Landmark) => (l.remarkable.length ? 0 : l.kind !== "crater" ? 1 : 2);
  return [...list].sort((a, b) => tier(a) - tier(b) || b.extent_km - a.extent_km || a.id.localeCompare(b.id));
}

// ── What the table watches (the landing run-through, 23 Nov) ─────────────────────────────────────────────────────
// The GM's Land screen keeps a LandingShow up to date (`ws.sendLanding`); "Send this view" carries it to the table
// (p2p `Shown.landing`). An ordinary player seat replays it on its own kernel: the same tiles, the same descent
// planned from the same site and lander, so nothing heavy crosses the wire. Only what the GM's own reads found
// about the world (its size, its air's optics, its star) travels with it, so a seat never reads the world's record.

/** The world as the Land screen draws it: plain numbers, the same on every seat. */
export interface LandingWorld {
  radius_m: number;
  pressure_bar: number;
  t_k: number;
  /** The air's optics (renderer `Air`) as the GM's kernel read them; null when the world has none. */
  air: { rayleigh_m: [number, number, number]; scale_height_m: number; density_m3: number; mean_molar_mass: number; zenith_depth: [number, number, number]; unknown_gases: string[]; unknown_share: number } | null;
  star_k: number;
  /** The star's angular radius from the world, in radians. */
  sun_radius: number;
  tilt_deg: number;
  /** Surface gravity, m/s² (the sea's waves run on it). */
  gravity_m_s2?: number;
}

/** The sea at a landing site (kernel weather report `data.sea`, `sse.ocean.wind_waves`), with the tide now. */
export interface LandingSea {
  liquid?: string | null;
  /** Significant wave height, m, and peak period, s. */
  wave_m: number;
  period_s: number;
  /** Wind toward the east, m/s (negative: toward the west). */
  wind_m_s: number;
  fetch_km?: number;
  /** The tide's height now above mean sea level, m. */
  tide_m: number;
  /** The kernel's line about it. */
  line?: string;
  /**
   * The liquid's own numbers for drawing it (W-sea2 to W-sea4: colour, wave trains, froth), as the kernel sent them;
   * the renderer's `readSeaOptics` reads them. Absent until the kernel sends any.
   */
  optics?: Record<string, unknown>;
}

/** The keys of the weather report's `data.sea` that describe the liquid for drawing (handover/status/surface-timelapse.md). */
export const SEA_OPTICS_KEYS = ["bands_nm", "absorb_per_m", "backscatter_per_m", "refractive_index", "deep_rgb", "trains", "ripple_cutoff_m", "superfluid", "liquid_note", "whitecap_fraction", "foam_albedo", "streak_dir_deg"] as const;

/** Where the GM is looking on the ground: metres east and north of the site, facing, and whether flying the drone. */
export interface LandingPose {
  x_m: number;
  y_m: number;
  yaw: number;
  pitch: number;
  drone: boolean;
}

/** One landing as the table watches it. */
export interface LandingShow {
  system: string;
  body: string;
  /** The world's name as the table knows it. */
  label: string;
  site: { lat_deg: number; lon_deg: number; label: string };
  /** A stock lander's id, or a lander object such as `{ ship }` (party.ts `partyLander`). */
  lander: string | Record<string, unknown>;
  /** choose (a site picked, not yet landing), flying (the descent playing) or down (on the ground). */
  phase: "choose" | "flying" | "down";
  /** How long the flown descent takes to play, and how far into it the GM is (0..1). */
  flight_s: number;
  fly_u?: number;
  pose?: LandingPose;
  /** The sun's hour angle in degrees (the Land screen's time of day). */
  hour_deg: number;
  /** The star's declination in the world's sky, in degrees (from its tilt and season). */
  decl_deg?: number;
  /** The sea at the site, when it has one. */
  sea?: LandingSea | null;
  /** How the GM's screen draws the ground (true, 3d or wireframe); a seat draws the same. */
  look?: "true" | "3d" | "wireframe";
  world: LandingWorld;
}

const isNum = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);

/** A LandingShow from what came over the wire, or null when it is not one. */
export function landingShowOf(x: unknown): LandingShow | null {
  if (!x || typeof x !== "object" || Array.isArray(x)) return null;
  const s = x as Partial<LandingShow>;
  if (typeof s.system !== "string" || typeof s.body !== "string" || !s.site || !isNum(s.site.lat_deg) || !isNum(s.site.lon_deg)) return null;
  if (s.phase !== "choose" && s.phase !== "flying" && s.phase !== "down") return null;
  if (typeof s.lander !== "string" && (!s.lander || typeof s.lander !== "object")) return null;
  const w = s.world;
  if (!w || !isNum(w.radius_m) || w.radius_m <= 0 || !isNum(w.star_k) || !isNum(w.sun_radius)) return null;
  return s as LandingShow;
}

/** The pose rounded for the wire: centimetres and milliradians are far finer than a screen shows. */
export function roundPose(p: LandingPose): LandingPose {
  const r = (v: number, k: number) => Math.round(v * k) / k;
  return { x_m: r(p.x_m, 100), y_m: r(p.y_m, 100), yaw: r(p.yaw, 1000), pitch: r(p.pitch, 1000), drone: p.drone };
}

/** What a player seat may read of one world while the table watches its landing (its own kernel, granted for that body only). */
export interface LandingGround {
  surfaceTiles: import("./session.ts").ViewSession["surfaceTiles"];
  surfaceTilesAt: import("./session.ts").ViewSession["surfaceTilesAt"];
  planDescent: import("./session.ts").ViewSession["planDescent"];
  surveyCaves?: import("./session.ts").ViewSession["surveyCaves"];
}

/** The tide's height at a time from the kernel's tide raisers (each a range and a period), all in phase at time 0 (the phase is not on record; fudge register). */
export function tideNow(by: readonly { range_m?: number; period_h?: number }[], t_s: number): number {
  let z = 0;
  for (const b of by) {
    const r = Number(b.range_m ?? 0);
    const p = Number(b.period_h ?? 0) * 3600;
    if (r > 0 && p > 0) z += (r / 2) * Math.cos((2 * Math.PI * (t_s % p)) / p);
  }
  return z;
}

/** The sea from a weather report's `data`, with the tide at `t_s`; null when the site has no sea. */
function opticsOf(sea: Record<string, unknown>): { optics?: Record<string, unknown> } {
  const o: Record<string, unknown> = {};
  for (const k of SEA_OPTICS_KEYS) if (sea[k] !== undefined && sea[k] !== null) o[k] = sea[k];
  return Object.keys(o).length ? { optics: o } : {};
}

export function landingSeaOf(data: unknown, t_s: number): LandingSea | null {
  const sea = (data as { sea?: Record<string, unknown> } | null)?.sea;
  if (!sea || typeof sea !== "object") return null;
  const wave_m = Number(sea.wave_m);
  const period_s = Number(sea.period_s);
  if (!Number.isFinite(wave_m) || !Number.isFinite(period_s)) return null;
  const by = ((sea.tides as { by?: { range_m?: number; period_h?: number }[] } | undefined)?.by ?? []).filter((x) => x && typeof x === "object");
  return {
    liquid: typeof sea.liquid === "string" ? sea.liquid : null,
    wave_m,
    period_s,
    wind_m_s: Number(sea.wind_m_s ?? 0) || 0,
    ...(Number.isFinite(Number(sea.fetch_km)) ? { fetch_km: Number(sea.fetch_km) } : {}),
    tide_m: Math.round(tideNow(by, t_s) * 100) / 100,
    ...(typeof sea.line === "string" ? { line: sea.line } : {}),
    ...opticsOf(sea),
  };
}
