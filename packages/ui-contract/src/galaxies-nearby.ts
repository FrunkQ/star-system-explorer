// Galaxies nearby, centred on home (grand zoom GZ5 part 1, design section 7). Pure: reads the kernel's own galaxies
// as the sky sees them (`sky_from_here` with galaxies and no stars: `galaxies_seen` and `satellites_seen`), and lays
// them out in space around our galaxy. Each galaxy's place is its direction from home times its distance, so this
// view and the night sky agree. Nothing is invented: a galaxy has the type, size, tilt and colour the kernel gave it.
import { starColour } from "./compare.ts";
import type { Json } from "./json.ts";

/** One galaxy as the kernel's sky gives it, cut to what this view draws and says. */
export interface NearGalaxy {
  address: string;
  /** dwarf, irregular, disc, barred, lenticular or elliptical: the kernel's `classify` of its state. */
  morphology: string;
  halo_mass_msun: number;
  stellar_mass_msun: number;
  sfr_msun_yr: number;
  distance_mpc: number;
  /** Unit vector from home in the sky's galactic frame (x toward our galaxy's centre, z out of its plane). */
  dir: [number, number, number];
  half_light_kpc: number;
  axis_ratio: number;
  inclination_deg: number;
  position_angle_deg: number;
  /** B-V colour index (Vega), or null when the kernel gave none. */
  b_v: number | null;
  /** Absolute V magnitude, or null. */
  abs_v: number | null;
  /** One of home's own satellites (from its assembly tree), with what its orbit has done to it. */
  satellite: boolean;
  orbit_words: string[];
}

export interface NearbyRead {
  galaxies: NearGalaxy[];
  /** Our galaxy's address. */
  home: string;
  /** Home's dark halo, its radius r200 in Mpc, from the kernel's satellite walk (null if not given). */
  home_r200_mpc: number | null;
  home_halo_msun: number | null;
  /** How far out the catalogue looks (Mpc). */
  reach_mpc: number;
  /** Satellites counted but not drawn: torn into streams or shells, and merged into home. */
  remains: number;
  merged: number;
}

/** The two reaches (design 7.2): the group, about 1.5 Mpc, and the neighbourhood, the catalogue's own 12 Mpc. */
export const NEARBY_REACHES = [
  { id: "group", label: "The group", mpc: 1.5 },
  { id: "neighbourhood", label: "The neighbourhood", mpc: 12 },
] as const;
export type NearbyReach = (typeof NEARBY_REACHES)[number]["id"];

const num = (v: unknown): number | null => (typeof v === "number" && Number.isFinite(v) ? v : null);
const vec = (v: unknown): [number, number, number] | null => (Array.isArray(v) && v.length === 3 && v.every((x) => typeof x === "number") ? (v as [number, number, number]) : null);

/** Read the kernel's sky answer into the galaxies this view draws. Returns null if it holds no galaxy list. */
export function readNearby(sky: Json): NearbyRead | null {
  const s = sky as Record<string, unknown> | null;
  const list = s && Array.isArray(s.galaxies) ? (s.galaxies as Record<string, unknown>[]) : null;
  if (!list) return null;
  const walk = (s!.galaxy_walk ?? {}) as Record<string, unknown>;
  const sats = (walk.satellites ?? {}) as Record<string, unknown>;
  const remains = Object.values((sats.remains ?? {}) as Record<string, unknown>).reduce<number>((a, v) => a + (num(v) ?? 0), 0);
  const galaxies: NearGalaxy[] = [];
  for (const g of list) {
    const dir = vec(g.dir);
    const d = num(g.distance_mpc);
    if (!dir || d === null || typeof g.address !== "string") continue;
    const colour = Array.isArray(g.colours) ? (g.colours as { name?: string; value?: number }[]).find((c) => c.name === "B-V") : undefined;
    const absV = Array.isArray(g.absolute) ? (g.absolute as { band?: string; m_vega?: number }[]).find((m) => m.band === "V") : undefined;
    const orbit = (g.orbit ?? null) as { labels?: unknown } | null;
    galaxies.push({
      address: g.address,
      morphology: typeof g.morphology === "string" ? g.morphology : "galaxy",
      halo_mass_msun: num(g.halo_mass_msun) ?? 0,
      stellar_mass_msun: num(g.stellar_mass_now_msun) ?? num(g.stellar_mass_msun) ?? 0,
      sfr_msun_yr: num(g.sfr_msun_yr) ?? 0,
      distance_mpc: d,
      dir,
      half_light_kpc: num(g.half_light_kpc) ?? 1,
      axis_ratio: num(g.axis_ratio) ?? 1,
      inclination_deg: num(g.inclination_deg) ?? 0,
      position_angle_deg: num(g.position_angle_deg) ?? 0,
      b_v: num(colour?.value) ?? null,
      abs_v: num(absV?.m_vega) ?? null,
      satellite: g.satellite === true,
      orbit_words: Array.isArray(orbit?.labels) ? (orbit!.labels as unknown[]).filter((x): x is string => typeof x === "string") : [],
    });
  }
  const r200 = num(sats.r200_kpc);
  return {
    galaxies,
    home: typeof s!.galaxy === "string" ? s!.galaxy : "",
    home_r200_mpc: r200 === null ? null : r200 / 1000,
    home_halo_msun: num(sats.host_mass_msun),
    reach_mpc: num(walk.reach_mpc) ?? 12,
    remains,
    merged: num(sats.merged) ?? 0,
  };
}

/** A galaxy's place relative to home, Mpc, in the galactic frame (x to our centre, z out of our disc). */
export const nearbyAt = (g: Pick<NearGalaxy, "dir" | "distance_mpc">): [number, number, number] => [g.dir[0] * g.distance_mpc, g.dir[1] * g.distance_mpc, g.dir[2] * g.distance_mpc];

/** The galaxies inside a reach, biggest in stars first. */
export function withinReach(gs: readonly NearGalaxy[], reach_mpc: number): NearGalaxy[] {
  return gs.filter((g) => g.distance_mpc <= reach_mpc).sort((a, b) => b.stellar_mass_msun - a.stellar_mass_msun || (a.address < b.address ? -1 : 1));
}

/** How the sphere is seen: from above our disc, tilted 60 degrees down, or edge-on along our disc. */
export type NearbySeen = "above" | "tilted" | "edge";
const TILT_DEG = 60;

/**
 * Project a place (Mpc, galactic frame) onto the drawing: `size` across, `reach` Mpc from the middle to the edge.
 * Returns the page point, the foot of its stalk on our disc's plane (the same plane Your systems stands its stalks
 * on) and a depth for drawing order (larger is nearer the viewer).
 */
export function projectNearby(p: readonly [number, number, number], seen: NearbySeen, size: number, reach_mpc: number, pad = 24): { x: number; y: number; fx: number; fy: number; depth: number } {
  const k = (size / 2 - pad) / reach_mpc;
  const c = size / 2;
  const [x, y, z] = p;
  if (seen === "above") return { x: c + x * k, y: c - y * k, fx: c + x * k, fy: c - y * k, depth: z };
  if (seen === "edge") return { x: c + x * k, y: c - z * k, fx: c + x * k, fy: c, depth: -y };
  const t = (TILT_DEG * Math.PI) / 180;
  const ct = Math.cos(t);
  const st = Math.sin(t);
  // Turned about x: the plane's y goes up the page shortened by cos(t); height rises by sin(t).
  const sy = (v: number, h: number) => v * ct + h * st;
  return { x: c + x * k, y: c - sy(y, z) * k, fx: c + x * k, fy: c - sy(y, 0) * k, depth: z * ct - y * st };
}

/**
 * How big a galaxy is drawn, in page units. Real: its half-light radius at the drawing's scale (most are then a
 * point of light: that is the truth). Readable: the same radius times one stated factor, so shapes read.
 */
export function nearbyRadius(g: Pick<NearGalaxy, "half_light_kpc">, size: number, reach_mpc: number, scale: "real" | "readable", factor: number, pad = 24): number {
  const k = (size / 2 - pad) / reach_mpc;
  const r = (g.half_light_kpc / 1000) * k * (scale === "readable" ? factor : 1);
  return Math.max(r, 1.1);
}

/** The Readable factor for a reach, a round number so the chip can say it: big enough that a disc in the group shows its shape. */
export const readableFactor = (reach_mpc: number): number => (reach_mpc <= 2 ? 8 : 20);

/** A dark halo's r200 from its mass, scaled from home's own (r200 grows as the cube root of mass at one epoch). */
export function haloR200(mass_msun: number, home: { r200_mpc: number; mass_msun: number }): number {
  return home.r200_mpc * Math.cbrt(mass_msun / home.mass_msun);
}

/**
 * A galaxy's colour from its B-V index: the temperature a star of that colour would have (Ballesteros 2012, EPL 97,
 * 34008), on the chart's star ramp. A galaxy's light is a mix of stars, so this is the colour its light averages to;
 * presentation only.
 */
export function galaxyColour(b_v: number | null): string {
  if (b_v === null) return starColour(undefined);
  const bv = Math.min(2, Math.max(-0.4, b_v));
  return starColour(4600 * (1 / (0.92 * bv + 1.7) + 1 / (0.92 * bv + 0.62)));
}

const TYPE_WORDS: Record<string, string> = {
  dwarf: "a dwarf galaxy",
  irregular: "an irregular galaxy",
  disc: "a disc galaxy",
  barred: "a barred disc galaxy",
  lenticular: "a lenticular galaxy",
  elliptical: "an elliptical galaxy",
};
/** The galaxy's type in words, from the kernel's class label. */
export const galaxyTypeWords = (m: string): string => TYPE_WORDS[m] ?? "a galaxy";

/** Light-travel time from a distance in Mpc, in words (million or billion years). */
export function lightTravelWords(d_mpc: number): string {
  const myr = d_mpc * 3.2616;
  if (myr < 1) return `${Math.round(myr * 1000)} thousand years`;
  if (myr < 1000) return `${Math.round(myr * 10) / 10} million years`;
  return `${Math.round(myr / 100) / 10} billion years`;
}

/** A distance in words: kiloparsecs inside 1 Mpc, else megaparsecs, with light years beside. */
export function galaxyDistanceWords(d_mpc: number): string {
  const ly = d_mpc * 3.2616;
  const lyWords = ly < 1 ? `${Math.round(ly * 1000)} thousand light years` : `${Math.round(ly * 10) / 10} million light years`;
  return d_mpc < 1 ? `${Math.round(d_mpc * 1000)} kpc (${lyWords})` : `${Math.round(d_mpc * 100) / 100} Mpc (${lyWords})`;
}

/** A scale bar for the drawing: a round number of light years about a fifth of the way across. */
export function nearbyScaleBar(size: number, reach_mpc: number, pad = 24): { px: number; words: string } {
  const kPerMly = (size / 2 - pad) / reach_mpc / 3.2616;
  const want = (size / 5) / kPerMly;
  const steps = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50];
  const mly = steps.reduce((a, b) => (Math.abs(b - want) < Math.abs(a - want) ? b : a));
  return { px: mly * kPerMly, words: mly < 1 ? `${Math.round(mly * 1000)} thousand light years` : `${mly} million light years` };
}

/** A pair of galaxies bound to fall together, from the kernel's `halo_cell` (each halo's nearest big neighbour). */
export interface BoundPair {
  a: string;
  b: string;
  distance_mpc: number;
  /** When they first meet, Gyr from now, as the kernel's two-body estimate gives it; null if it gave none. */
  approach_gyr: number | null;
}

/**
 * The bound pairs among the galaxies shown: each halo whose nearest big neighbour is bound to it, both of them on
 * the view, once per pair. Read from the cells' own answers; nothing estimated here.
 */
export function boundPairs(cells: readonly { haloes: readonly { address: string; neighbour: { address: string; distance_mpc: number; bound: boolean; approach_gyr: number | null } | null }[] }[], shown: ReadonlySet<string>): BoundPair[] {
  const out = new Map<string, BoundPair>();
  for (const c of cells)
    for (const h of c.haloes) {
      const n = h.neighbour;
      if (!n?.bound || !shown.has(h.address) || !shown.has(n.address)) continue;
      const [a, b] = h.address < n.address ? [h.address, n.address] : [n.address, h.address];
      if (!out.has(`${a}|${b}`)) out.set(`${a}|${b}`, { a, b, distance_mpc: n.distance_mpc, approach_gyr: n.approach_gyr });
    }
  return [...out.values()].sort((x, y) => x.distance_mpc - y.distance_mpc || (x.a < y.a ? -1 : 1));
}

/** "meet in about 4 Gyr", or "already meeting" when the kernel says now. */
export function meetWords(approach_gyr: number | null): string {
  if (approach_gyr === null) return "bound, falling together";
  if (approach_gyr < 0.05) return "already meeting";
  return `meet in about ${approach_gyr < 1 ? Math.round(approach_gyr * 10) / 10 : Math.round(approach_gyr)} Gyr`;
}
