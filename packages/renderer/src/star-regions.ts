// SG7: a star's active regions as the kernel lists them (read_activity's `regions`, sse-activity regions.rs): where
// its field breaks through, each a pair of spots of opposite sign. This file only turns a region into its two spots at
// a time: their share of the region's area now (a straight rise then a steady decay, the kernel's own times), their
// places either side of the region's middle along its tilted axis, carried round at the latitude's rate. Every number
// it uses comes with the region; nothing here decides where a region is.
import type { Spot } from "./star.ts";

export interface Region {
  address: string;
  born_s: number;
  grow_s: number;
  life_s: number;
  /** Peak spot area, share of the star's whole surface. */
  peak_share: number;
  peak_msh: number;
  lat_deg: number;
  /** Longitude at birth in the star's own frame (turning once a spin from age zero). */
  lon_deg: number;
  /** Direction from the following spot to the leading one, degrees from the direction of spin toward north. */
  tilt_deg: number;
  leading_sign: number;
  separation_deg: number;
}

export interface Regions {
  regions: Region[];
  rotation_period_s: number;
  /** The kernel's differential rotation, A, B, C (`sse.activity.differential_rotation`). */
  differential: [number, number, number];
  leader_share: number;
}

/** The region's spot area now, share of the surface: the kernel's rise to its peak, then its steady decay. */
export function regionShare(g: Region, t: number): number {
  const age = t - g.born_s;
  if (age < 0 || age > g.life_s) return 0;
  if (age < g.grow_s) return (g.peak_share * age) / Math.max(1, g.grow_s);
  return (g.peak_share * (g.life_s - age)) / Math.max(1, g.life_s - g.grow_s);
}

/** Surface rate at a latitude over the equator's, from the kernel's coefficients. */
export function rateShareOf(d: [number, number, number], lat_deg: number): number {
  const s2 = Math.sin((lat_deg * Math.PI) / 180) ** 2;
  return (d[0] + d[1] * s2 + d[2] * s2 * s2) / d[0];
}

/** The region's middle now in the star's frame, carried at its latitude's rate. */
export function regionLon(r: Regions, g: Region, t: number): number {
  const p = r.rotation_period_s;
  const lon = p > 0 ? g.lon_deg + (rateShareOf(r.differential, g.lat_deg) - 1) * 360 * ((t - g.born_s) / p) : g.lon_deg;
  return ((lon % 360) + 360) % 360;
}

const capRadius = (share: number) => (Math.acos(1 - 2 * Math.min(0.5, Math.max(0, share))) * 180) / Math.PI;

/** Every region's two spots at time t: the leader ahead in the spin, the follower behind, along the tilted axis. */
export function spotsFromRegions(r: Regions, t: number): Spot[] {
  const out: Spot[] = [];
  for (const g of r.regions) {
    const a = regionShare(g, t);
    if (!(a > 0)) continue;
    const lon = regionLon(r, g, t);
    const tl = (g.tilt_deg * Math.PI) / 180;
    const half = g.separation_deg / 2;
    const cosLat = Math.max(0.05, Math.cos((g.lat_deg * Math.PI) / 180));
    for (const [side, share] of [[1, r.leader_share], [-1, 1 - r.leader_share]] as const) {
      const lat = g.lat_deg + side * half * Math.sin(tl);
      const lo = lon + (side * half * Math.cos(tl)) / cosLat;
      out.push({ lat_deg: Math.max(-89, Math.min(89, lat)), lon_deg: ((lo % 360) + 360) % 360, radius_deg: capRadius(a * share) });
    }
  }
  return out;
}
