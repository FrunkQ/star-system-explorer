// A living star (V10c): the star as the kernel says it is at a moment, turning on its own spin, its spots where the
// cycle puts them and its flares rising and fading on their own clock. The Body screen's star disc and the system
// view's True star both read the star this way, so a star looks the same from the system as close up.
//
// Nothing here is invented: spin, spot coverage, cycle phase, X-ray power and the flare list (times, rise, decay,
// peak power, energy, drawn from the light track's flare-frequency law) all come from `read_activity` (O10). This
// file only places them on the disc at a time and draws them with star.ts.
import { constant, type Json } from "../../ui-contract/src/index.ts";
import { coronaScaleHeight, drawStar, granulesFor, granulesShow, spotsFor, type Convection, type Spot, type StarDraw } from "./star.ts";


export interface Flare {
  at_s: number;
  rise_s: number;
  decay_s: number;
  peak_w: number;
  energy_j: number;
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
}

/** What a session needs to read a star (session.ts `resolvedSystem` and `readActivity`). */
export interface StarReader {
  resolvedSystem(system: string): Promise<{ ok: true; result: { facts: Record<string, Record<string, Json>> } } | { ok: false; message: string }>;
  readActivity(ask: { stars: Json[]; at_s?: number; window?: { from_s: number; to_s: number; min_energy_j?: number } }): Promise<{ ok: true; stars: Json[] } | { ok: false; message: string }>;
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
export async function readLiveStar(session: StarReader, system: string, address: string, given: GivenStar = {}): Promise<LiveStar | string> {
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
  const star = { address, mass_kg: mass, radius_m: radius, luminosity_w: lum, teff_k: teff, age_s };
  const r = await session.readActivity({ stars: [star], at_s: age_s, window: { from_s: age_s - 2 * constant("day"), to_s: age_s + constant("yr_julian"), min_energy_j: 1e24 } });
  if (!r.ok) return r.message;
  const one = r.stars[0] as unknown as { activity: Record<string, Json>; moment: Record<string, Json>; flares?: { flares?: Flare[] } };
  return { address, teff, mass, radius, lum, age_s, activity: one.activity, moment: one.moment, flares: one.flares?.flares ?? [] };
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

/** The star's spots at its present (spot places from the star and the cycle; the kernel gives coverage). */
export const spotsOf = (s: LiveStar): Spot[] => spotsFor(s.address, Number(s.moment.spot_coverage ?? 0), cycleOf(s));

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
    // A flare goes off in an active region: over a spot, taken in turn (the kernel gives times, not places).
    const sp = spots[i % Math.max(1, spots.length)];
    out.push({ lat_deg: sp?.lat_deg ?? 15, lon_deg: sp?.lon_deg ?? 0, strength: (k * f.peak_w) / xr, flare: f });
  });
  return out;
}

/** The star's turn under a viewer at time t, degrees of central meridian (its own spin from the kernel). */
export function turnAt(s: LiveStar, t: number): number {
  const p = Number(s.activity.rotation_period_s ?? 0);
  if (!(p > 0)) return 0;
  const f = ((t - s.age_s) / p) % 1;
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
