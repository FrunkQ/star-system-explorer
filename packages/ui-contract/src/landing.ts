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
