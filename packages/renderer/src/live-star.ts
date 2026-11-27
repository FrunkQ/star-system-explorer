// A living star (V10c): the star as the kernel says it is at a moment, turning on its own spin, its spots where the
// cycle puts them and its flares rising and fading on their own clock. The Body screen's star disc and the system
// view's True star both read the star this way, so a star looks the same from the system as close up.
//
// Nothing here is invented: spin, spot coverage, cycle phase, X-ray power and the flare list (times, rise, decay,
// peak power, energy, drawn from the light track's flare-frequency law) all come from `read_activity` (O10). This
// file only places them on the disc at a time and draws them with star.ts.
import { constant, type Json } from "../../ui-contract/src/index.ts";
import type { Prominence } from "./star-prominences.ts";
import { spotsFromRegions, type Regions } from "./star-regions.ts";
import { coronaScaleHeight, drawStar, granulesFor, granulesShow, spotsFor, type Convection, type Spot, type StarDraw } from "./star.ts";


export interface Flare {
  at_s: number;
  rise_s: number;
  decay_s: number;
  peak_w: number;
  energy_j: number;
  /** The flare's light at its peak (sse-light LightDescription): a `thermal` part gives its white-light patch. */
  light?: { parts?: Json[] };
  /** A mass ejection with the flare: speed, mass, direction (unit vector, z the spin axis) and cone half-width. */
  ejection?: { speed_m_s: number; mass_kg: number; direction: [number, number, number]; half_width_rad: number };
  /** Where it goes off (SG7): its active region, in the star's own frame at its peak. */
  site?: { region: string; lat_deg: number; lon_deg: number };
}

export interface LiveStar {
  address: string;
  teff: number;
  mass: number;
  radius: number;
  lum: number;
  /** The star's present: the system's age. */
  age_s: number;
  activity: Record<string, Json>;
  moment: Record<string, Json>;
  /** Flares above 10²⁴ J from two days before the present to a year after. */
  flares: Flare[];
  /** Prominences standing from the present to PROMINENCE_WINDOW_S after (SG6), and the field and corona that hold them. */
  prominences?: Prominence[];
  corona?: Record<string, Json>;
  /** The active regions alive over the same span (SG7): the spots, and where flares and prominences sit. */
  regions?: Regions;
}

/** How much of the star's time the drawn star keeps prominences for: about six minutes on the True view's star clock. */
export const PROMINENCE_WINDOW_S = 90 * constant("day");

/** What a session needs to read a star (session.ts `resolvedSystem` and `readActivity`). */
export interface StarReader {
  resolvedSystem(system: string): Promise<{ ok: true; result: { facts: Record<string, Record<string, Json>> } } | { ok: false; message: string }>;
  readActivity(ask: { stars: Json[]; at_s?: number; window?: { from_s: number; to_s: number; min_energy_j?: number }; prominences?: { from_s: number; to_s: number } }): Promise<{ ok: true; stars: Json[] } | { ok: false; message: string }>;
}

/** A star's own numbers from its scene facts (teff_k, mass_sun, radius_sun), for a system the kernel did not resolve. */
export interface GivenStar {
  teff_k?: number;
  mass_sun?: number;
  radius_sun?: number;
}

/**
 * Read a star's facts and activity from the kernel: a LiveStar, or why there is none. A star the resolved system
 * does not describe (an authored system) is taken from its scene facts when they give all three numbers.
 */
/** A world the kernel checks each mass ejection against (read_activity's targets: a circular orbit in the star's equator). */
export interface ActivityTarget {
  address: string;
  distance_m: number;
  period_s?: number;
  /** Angle round the star at age zero, radians. */
  phase_rad: number;
}

/**
 * The worlds circling `star`, as read_activity's targets, from the resolved system's own orbit facts (the same facts
 * the system views place planets by): distance the semi-major axis, period the kernel's (or Kepler's from the host's
 * mass), and the angle at age zero wound back from the mean anomaly at the orbit's epoch.
 */
export function targetsFromFacts(facts: Record<string, Record<string, Json> | undefined>, star: string): ActivityTarget[] {
  const out: ActivityTarget[] = [];
  const hostMass = Number(facts[star]?.["body.mass"]);
  for (const [address, f] of Object.entries(facts)) {
    if (!f || address === star || f["orbit.host"] !== star) continue;
    const a = Number(f["orbit.semi_major_axis"]);
    if (!(a > 0)) continue;
    const given = Number(f["orbit.period"] ?? f["orbit.period_s"]);
    const period = given > 0 ? given : hostMass > 0 ? 2 * Math.PI * Math.sqrt(a ** 3 / (constant("G") * hostMass)) : NaN;
    const m = Number(f["orbit.mean_anomaly_at_epoch"] ?? 0) || 0;
    const epoch = Number(f["orbit.epoch_s"] ?? 0) || 0;
    const turn = 2 * Math.PI;
    const phase = period > 0 ? m - (turn * (epoch % period)) / period : m;
    out.push({ address, distance_m: a, ...(period > 0 ? { period_s: period } : {}), phase_rad: ((phase % turn) + turn) % turn });
  }
  return out;
}

/** `targets`: worlds to check each ejection against (hits and misses); "system" fills them from the system's own orbits. */
export async function readLiveStar(session: StarReader, system: string, address: string, given: GivenStar = {}, targets?: ActivityTarget[] | "system"): Promise<LiveStar | string> {
  const rs = await session.resolvedSystem(system);
  if (!rs.ok) return rs.message;
  const f = rs.result.facts[address] ?? {};
  const pick = (k: number, g: number | undefined) => (k > 0 ? k : Number(g) > 0 ? Number(g) : NaN);
  const teff = pick(Number(f["star.effective_temperature"]), given.teff_k);
  const mass = pick(Number(f["body.mass"]), given.mass_sun === undefined ? undefined : given.mass_sun * constant("M_sun"));
  const radius = pick(Number(f["body.radius"]), given.radius_sun === undefined ? undefined : given.radius_sun * constant("R_sun"));
  const age = Number(rs.result.facts[system]?.["system.age"] ?? 0);
  if (!(teff > 0 && mass > 0 && radius > 0)) return "The kernel has no temperature, mass and radius for this star yet, so there is nothing true to draw.";
  const lum = 4 * Math.PI * radius * radius * constant("sigma_SB") * teff ** 4;
  const age_s = age > 0 ? age : 4.6e9 * constant("yr_julian");
  const aim = targets === "system" ? targetsFromFacts(rs.result.facts as Record<string, Record<string, Json>>, address) : targets;
  const star = { address, mass_kg: mass, radius_m: radius, luminosity_w: lum, teff_k: teff, age_s, ...(aim?.length ? { targets: aim as unknown as Json[] } : {}) };
  const r = await session.readActivity({ stars: [star], at_s: age_s, window: { from_s: age_s - 2 * constant("day"), to_s: age_s + constant("yr_julian"), min_energy_j: 1e24 }, prominences: { from_s: age_s, to_s: age_s + PROMINENCE_WINDOW_S } });
  if (!r.ok) return r.message;
  const one = r.stars[0] as unknown as { activity: Record<string, Json>; moment: Record<string, Json>; flares?: { flares?: Flare[] }; prominences?: { prominences?: Prominence[]; corona?: Record<string, Json> | null }; regions?: Regions };
  return { address, teff, mass, radius, lum, age_s, activity: one.activity, moment: one.moment, flares: one.flares?.flares ?? [], prominences: one.prominences?.prominences ?? [], ...(one.prominences?.corona ? { corona: one.prominences.corona } : {}), ...(one.regions ? { regions: one.regions } : {}) };
}

/** Where the star is in its cycle: progress from minimum and the cycle's number, or null with no clean cycle. */
export function cycleOf(s: LiveStar): { phase: number; number: number } | null {
  const ph = s.moment.cycle_phase;
  const c = s.activity.cycle as { period_s?: number } | undefined;
  if (typeof ph !== "number" || !c?.period_s) return null;
  // The activity factor is 1 + A sin(2 pi phase), lowest at phase 0.75.
  const p = (((ph - 0.75) % 1) + 1) % 1;
  return { phase: p, number: Math.floor(s.age_s / c.period_s + 0.25) };
}

/**
 * The star's spots at its present: its active regions' spots when the kernel gave them (SG7); without them (a star
 * read without a span) a stand-in scatter at the kernel's coverage.
 */
export const spotsOf = (s: LiveStar): Spot[] => (s.regions ? spotsFromRegions(s.regions, s.age_s) : spotsFor(s.address, Number(s.moment.spot_coverage ?? 0), cycleOf(s)));

/**
 * A flare's power at time t over its peak: a straight rise over `rise_s` then an exponential fall with e-folding
 * time `decay_s` (the classic impulsive-gradual light curve, Benz 2017; the kernel gives the two times). Zero before
 * it starts and once it has fallen to a fiftieth.
 */
export function flareProfile(f: Flare, t: number): number {
  const start = f.at_s - f.rise_s;
  if (t < start) return 0;
  if (t <= f.at_s) return f.rise_s > 0 ? (t - start) / f.rise_s : 1;
  const k = Math.exp(-(t - f.at_s) / Math.max(1, f.decay_s));
  return k < 0.02 ? 0 : k;
}

/**
 * Flares going on at time t, each with its strength now (power over the star's X-ray power) and its place. With a
 * `span`, the brightest moment within the span before t: a picture drawn every `span` seconds of star time then shows
 * every flare that peaked between two pictures, however brief.
 */
export function flaresAt(s: LiveStar, spots: Spot[], t: number, span = 0): { lat_deg: number; lon_deg: number; strength: number; flare: Flare }[] {
  const xr = Math.max(1, Number(s.moment.xray_w));
  const out: { lat_deg: number; lon_deg: number; strength: number; flare: Flare }[] = [];
  s.flares.forEach((f, i) => {
    // The light curve has one peak, so its brightest within the span is the peak or one end.
    const k = span > 0 && f.at_s <= t && f.at_s >= t - span ? 1 : Math.max(flareProfile(f, t), span > 0 ? flareProfile(f, t - span) : 0);
    if (k <= 0) return;
    // A flare goes off in its active region (the kernel's site, SG7); one without a site under where its ejection
    // leaves, or over a spot taken in turn.
    const at = f.site ?? flarePlace(s, f);
    const sp = spots[i % Math.max(1, spots.length)];
    out.push({ lat_deg: at?.lat_deg ?? sp?.lat_deg ?? 15, lon_deg: at?.lon_deg ?? sp?.lon_deg ?? 0, strength: (k * f.peak_w) / xr, flare: f });
  });
  return out;
}

/**
 * Where a flare with an ejection goes off, in the star's own turning frame: under the ejection's direction at the
 * flare's peak. The ejection's direction is fixed in space (the kernel aims it for hits and misses), the viewer's frame
 * here, so the place on the star is that direction plus the star's turn at the peak. Null without an ejection.
 */
export function flarePlace(s: LiveStar, f: Flare): { lat_deg: number; lon_deg: number } | null {
  const e = f.ejection;
  if (!e) return null;
  const [dx, dy, dz] = e.direction;
  const lon = (Math.atan2(dy, dx) * 180) / Math.PI + turnAt(s, f.at_s);
  return { lat_deg: (Math.asin(Math.max(-1, Math.min(1, dz))) * 180) / Math.PI, lon_deg: ((lon % 360) + 360) % 360 };
}

/**
 * The star's turn at time t, degrees (its own spin from the kernel): its frame turns once a spin from age zero, the
 * frame the kernel places regions, flares and prominences in.
 */
export function turnAt(s: LiveStar, t: number): number {
  const p = Number(s.activity.rotation_period_s ?? 0);
  if (!(p > 0)) return 0;
  const f = (t / p) % 1;
  return (((f < 0 ? f + 1 : f) * 360) % 360 + 360) % 360;
}

/**
 * The star drawn at time t into a square of RGBA pixels: turned by its spin, spots carried round with it, flares at
 * their strength now, granules when the disc resolves them. `disc` is the share of the square the disc fills.
 */
export function drawLiveStar(out: { width: number; data: Uint8ClampedArray }, s: LiveStar, t: number, light: StarDraw["light"] = "visible", disc = 0.92, lat_deg = 7, span = 0): void {
  const spots = spotsOf(s);
  const conv = (s.activity.convection ?? null) as unknown as Convection | null;
  const discPx = out.width * disc;
  const granules = conv && light === "visible" && granulesShow(conv.granules_across, discPx) ? granulesFor(s.address, conv.granules_across, Math.floor(t / Math.max(1, conv.granule_lifetime_s))) : null;
  drawStar(out, { teff_k: s.teff, spots, lon_deg: turnAt(s, t), lat_deg, light, corona_h: coronaScaleHeight(s.mass, s.radius), flares: flaresAt(s, spots, t, span).map(({ lat_deg, lon_deg, strength }) => ({ lat_deg, lon_deg, strength })), granules }, disc);
}

/**
 * How fast the True view's star runs: six hours of the star's time each second on the screen, stated beside the
 * view. At that rate a Sun-like star turns once in under two minutes, so its spots are seen going round, and a flare
 * shows for a moment; the universe clock is not moved.
 */
export const STAR_SECONDS_PER_SECOND = constant("day") / 4;

/**
 * Differential rotation (VP-S4): a star's surface turns faster at its equator than near its poles. The Sun's rate is
 * ω(φ) = A + B sin²φ + C sin⁴φ with A = 14.713, B = −2.396, C = −1.787 degrees a day (Snodgrass and Ulrich 1990, from
 * Doppler shifts). Other stars take the same shape, scaled to their own spin (a stand-in: Barnes et al. 2005 find the
 * shear depends weakly on temperature): the share of the equator's rate at latitude φ.
 */
export const DIFFERENTIAL_ROTATION = { a: 14.713, b: -2.396, c: -1.787 };
export function rateShare(lat_deg: number): number {
  const s2 = Math.sin((lat_deg * Math.PI) / 180) ** 2;
  const { a, b, c } = DIFFERENTIAL_ROTATION;
  return (a + b * s2 + c * s2 * s2) / a;
}

/**
 * The star's spots at time t, each carried round at its own latitude's rate: placed at the star's present (the
 * kernel's moment), they drift against the equator's turn (which `turnAt` follows) by the difference in rate, so a
 * high-latitude spot falls behind over the days. Without a known spin they stay put.
 */
export function spotsAt(s: LiveStar, t: number): Spot[] {
  if (s.regions) return spotsFromRegions(s.regions, t);
  const p = Number(s.activity.rotation_period_s ?? 0);
  const spots = spotsOf(s);
  if (!(p > 0)) return spots;
  const turns = (t - s.age_s) / p;
  return spots.map((sp) => ({ ...sp, lon_deg: ((((sp.lon_deg + (rateShare(sp.lat_deg) - 1) * 360 * turns) % 360) + 360) % 360) }));
}
