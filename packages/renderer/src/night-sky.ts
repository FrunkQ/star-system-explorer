// The stars over a landing site, and the eye that adapts to see them (Land, night): the kernel's sky from this system
// (`sky_from_here`, galactic directions, luminosity and distance per star) turned onto the site's sky by the same
// conventions that place the sun there (surface-sky.ts `sunAt`, `sunDirection`), and an exposure that follows the
// light about the eye, so a dark sky shows its stars and a lit one hides them, as an eye's does.
import { constant } from "../../ui-contract/src/index.ts";
import { skyRadiance, sunDirection, sunTransmittance } from "./surface-sky.ts";
import { toSystem, type SystemFrame } from "./sky.ts";

type V3 = [number, number, number];

/** The system's frame from the kernel (`frame` in the sky reply): only its matrix to galactic is read here. */
export type SiteSkyFrame = Pick<SystemFrame, "to_galactic">;

/** A star as the kernel gives it: galactic unit direction, luminosity in suns, distance in parsecs, temperature. */
export interface SiteSkyStar {
  dir: V3;
  luminosity_lsun: number;
  distance_pc: number;
  teff_k: number;
}

export interface SiteStar {
  /** Direction in the site's frame (x east, y north, z up). */
  dir: V3;
  /** Its light at the site as a share of the home star's at the top of the air (bolometric: fudge register). */
  share: number;
  teff_k: number;
}

const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

/**
 * The stars over a site at a moment. The world's pole leans toward the frame's reference direction by its tilt, as
 * the sun's declination assumes (`sunAt`); each star's declination is its angle from the world's equator and its hour
 * angle the sun's plus the difference of their right ascensions, so the stars turn with the sun round the same pole.
 * A world turning backwards (`retrograde`) runs its sky the other way. `host_lsun` and `orbit_m` set the home star's
 * light at the world, against which each star's share is taken.
 */
export function starsOverSite(o: {
  stars: readonly SiteSkyStar[];
  frame: SiteSkyFrame;
  tilt_deg: number;
  orbit_phase_deg: number;
  sun_hour_deg: number;
  lat_deg: number;
  retrograde?: boolean;
  host_lsun: number;
  orbit_m: number;
}): SiteStar[] {
  const eps = (Math.abs(o.tilt_deg > 90 ? 180 - o.tilt_deg : o.tilt_deg) * Math.PI) / 180;
  const pole: V3 = [Math.sin(eps), 0, Math.cos(eps)];
  const e1: V3 = [Math.cos(eps), 0, -Math.sin(eps)];
  const e2: V3 = [0, 1, 0];
  const phi = (o.orbit_phase_deg * Math.PI) / 180;
  const sun: V3 = [-Math.cos(phi), -Math.sin(phi), 0];
  const ra = (v: V3) => Math.atan2(dot(v, e2), dot(v, e1));
  const raSun = ra(sun);
  const pc = constant("pc");
  const sign = o.retrograde ? -1 : 1;
  const home = o.host_lsun / (o.orbit_m * o.orbit_m);
  const out: SiteStar[] = [];
  for (const s of o.stars) {
    const v = toSystem(o.frame as SystemFrame, s.dir) as V3;
    const dec = (Math.asin(Math.max(-1, Math.min(1, dot(v, pole)))) * 180) / Math.PI;
    const hour = o.sun_hour_deg + (sign * (raSun - ra(v)) * 180) / Math.PI;
    const d = s.distance_pc * pc;
    out.push({ dir: sunDirection(o.lat_deg, hour, dec), share: home > 0 && d > 0 ? s.luminosity_lsun / (d * d) / home : 0, teff_k: s.teff_k });
  }
  return out;
}

/** How much brighter than the day the eye lets the night become (a dark-adapted eye gains about this much). */
export const ADAPT_RANGE = 1e5;

/**
 * The exposure for the light about the eye: `day` (the exposure for a sunlit scene) times how much darker the scene is
 * than with the sun 45° up, capped at ADAPT_RANGE. The scene's light is the zenith sky's plus sunlit ground of albedo
 * 0.1, so an airless world in daylight stays at the day's exposure (its ground is bright, its stars too faint to show)
 * and any world at night opens up until its stars show. A stand-in for the eye's adaptation (fudge register).
 */
export function adaptedExposure(day: number, o: { sun_z: number; tau_green: number; radius_m: number; scale_height_m: number }): number {
  const light = (muS: number) => skyRadiance(o.tau_green, 1, muS, muS, o.radius_m, o.scale_height_m) + (Math.max(0, muS) * sunTransmittance(o.tau_green, muS, o.radius_m, o.scale_height_m) * 0.1) / Math.PI;
  const ref = light(Math.SQRT1_2);
  const now = light(o.sun_z);
  return day * Math.min(ADAPT_RANGE, ref / Math.max(now, ref / ADAPT_RANGE));
}

