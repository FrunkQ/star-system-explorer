// Galaxies nearby, centred on home (grand zoom GZ5 part 1, design section 7). Pure: reads the kernel's own galaxies
// as the sky sees them (`sky_from_here` with galaxies and no stars: `galaxies_seen` and `satellites_seen`), and lays
// them out in space around our galaxy. Each galaxy's place is its direction from home times its distance, so this
// view and the night sky agree. Nothing is invented: a galaxy has the type, size, tilt and colour the kernel gave it.
import { constant } from "./sheet.ts";
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
  /** GZ6: every one of home's satellites' paths since infall (asked with `satellite_paths`), empty if not asked. */
  paths: SatellitePath[];
  /** Our galaxy's centre from the observer, Mpc (the observer sits in its disc, about 8 kpc out). */
  centre_mpc: [number, number, number];
  /** The time the sky was read at, Gyr after the Big Bang (the campaign's date). */
  t_gyr: number;
  /** Home's place in the web, Mpc (the web's frame shares the galactic frame's axes; F37). */
  home_mpc: [number, number, number] | null;
}

/** GZ6: one of home's satellites from infall to now, as the kernel followed it in our galaxy's growing pull. */
export interface SatellitePath {
  address: string;
  /** What became of it: it survives, it was torn apart (a stream, shell or mixed remains), or it reached our centre. */
  fate: "survives" | "torn apart" | "merged";
  labels: string[];
  infall_gyr: number;
  merged_gyr: number | null;
  disrupted_gyr: number | null;
  infall_halo_mass_msun: number;
  infall_stars_msun: number;
  /** Times (Gyr) and places (Mpc from the observer, the same frame as each galaxy's `dir`), oldest first. */
  t: number[];
  at: [number, number, number][];
}

/**
 * The reaches (design 7.2): the group, about 1.5 Mpc, and the neighbourhood, the catalogue's own 12 Mpc; and (GZ6)
 * our own satellites, 0.4 Mpc, out past our halo's edge, where their orbits and Play can be read.
 */
export const NEARBY_REACHES = [
  { id: "satellites", label: "Our satellites", mpc: 0.4 },
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
  const paths: SatellitePath[] = [];
  for (const p of Array.isArray(sats.paths) ? (sats.paths as Record<string, unknown>[]) : []) {
    const pts = Array.isArray(p.path) ? (p.path as unknown[]).filter((x): x is number[] => Array.isArray(x) && x.length === 4 && x.every((v) => typeof v === "number")) : [];
    const fate = p.fate === "merged" || p.fate === "torn apart" ? p.fate : "survives";
    if (typeof p.address !== "string" || !pts.length) continue;
    paths.push({
      address: p.address,
      fate,
      labels: Array.isArray(p.labels) ? (p.labels as unknown[]).filter((x): x is string => typeof x === "string") : [],
      infall_gyr: num(p.infall_gyr) ?? pts[0]![0]!,
      merged_gyr: num(p.merged_gyr),
      disrupted_gyr: num(p.disrupted_gyr),
      infall_halo_mass_msun: num(p.infall_halo_mass_msun) ?? 0,
      infall_stars_msun: num(p.infall_stars_msun) ?? 0,
      t: pts.map((x) => x[0]!),
      at: pts.map((x) => [x[1]! / 1000, x[2]! / 1000, x[3]! / 1000]),
    });
  }
  const c = vec(sats.centre_kpc);
  return {
    galaxies,
    home: typeof s!.galaxy === "string" ? s!.galaxy : "",
    home_r200_mpc: r200 === null ? null : r200 / 1000,
    home_halo_msun: num(sats.host_mass_msun),
    reach_mpc: num(walk.reach_mpc) ?? 12,
    remains,
    merged: num(sats.merged) ?? 0,
    paths,
    centre_mpc: c ? [c[0] / 1000, c[1] / 1000, c[2] / 1000] : [0, 0, 0],
    t_gyr: (num(s!.t_myr) ?? 13800) / 1000,
    home_mpc: vec(walk.home_mpc),
  };
}

/**
 * Where a satellite was at a time (Mpc from the observer), read along its kernel path: straight between the
 * kernel's own points, 0.1 Gyr apart at pack defaults (a satellite moves about 25 kpc in that time; the drawing is a
 * fraction of a pixel to a few pixels off the true curve). Null before it fell in and after it reached our centre.
 */
export function pathAt(p: Pick<SatellitePath, "t" | "at" | "merged_gyr">, t_gyr: number): [number, number, number] | null {
  const n = p.t.length;
  if (!n || t_gyr < p.t[0]!) return null;
  if (p.merged_gyr !== null && t_gyr >= p.merged_gyr) return null;
  if (t_gyr >= p.t[n - 1]!) return p.at[n - 1]!;
  let i = 0;
  while (p.t[i + 1]! <= t_gyr) i++;
  const f = (t_gyr - p.t[i]!) / (p.t[i + 1]! - p.t[i]!);
  const a = p.at[i]!;
  const b = p.at[i + 1]!;
  return [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f];
}

/** How long a stream's arc is drawn behind the place its stars have reached: their own path over the last stretch (fudge GZ-F12). */
export const STREAM_ARC_GYR = 1.5;

/**
 * A torn-apart satellite's stream at a time: the stretch of its own path behind where its stars have got to, from
 * when it was torn apart (or half a Gyr before, while it was being stripped) and no longer than `STREAM_ARC_GYR`.
 * Null while it is still whole, or before its path begins. Stripped stars keep to the orbit they left on, spreading
 * ahead and behind (Johnston 1998); one arc along the orbit is the drawing of that (fudge GZ-F12).
 */
export function streamArc(p: Pick<SatellitePath, "t" | "at" | "merged_gyr" | "disrupted_gyr">, t_gyr: number): [number, number, number][] | null {
  if (p.disrupted_gyr === null || t_gyr < p.disrupted_gyr) return null;
  const from = Math.max(p.disrupted_gyr - 0.5, t_gyr - STREAM_ARC_GYR);
  const pts: [number, number, number][] = [];
  for (let i = 0; i < p.t.length; i++) if (p.t[i]! >= from && p.t[i]! <= t_gyr) pts.push(p.at[i]!);
  const end = pathAt(p, t_gyr);
  if (end) pts.push(end);
  return pts.length >= 2 ? pts : null;
}

/** What a satellite is at a time, in words, for Play's caption and the card. */
export function satelliteStateWords(p: Pick<SatellitePath, "fate" | "infall_gyr" | "merged_gyr" | "disrupted_gyr" | "labels">, t_gyr: number): string {
  if (t_gyr < p.infall_gyr) return "not yet fallen in";
  if (p.merged_gyr !== null && t_gyr >= p.merged_gyr) return "merged into our galaxy";
  if (p.disrupted_gyr !== null && t_gyr >= p.disrupted_gyr) return p.labels.includes("shell") ? "torn apart, its stars in shells" : p.labels.includes("mixed remains") ? "torn apart, its stars mixed into our halo" : "torn apart, its stars in a stream";
  return "a satellite on its orbit";
}

/** The first infall among the paths: where Play starts. */
export const firstInfall = (ps: readonly Pick<SatellitePath, "infall_gyr">[]): number | null => (ps.length ? Math.min(...ps.map((p) => p.infall_gyr)) : null);

/** G in Mpc^3 per solar mass per Gyr^2, from the sheet's GM_sun, parsec and Julian year. */
const G_MPC_GYR = (() => {
  const mpc = constant("pc") * 1e6;
  const gyr = constant("yr_julian") * 1e9;
  return (constant("GM_sun") * gyr * gyr) / (mpc * mpc * mpc);
})();

/**
 * The zero-velocity radius of a mass at a time: the sphere inside which matter has stopped moving apart with the
 * expansion and fallen back. Sandage 1986 (ApJ 307, 1, doi:10.1086/164387), the timing argument for a pressureless sphere with no
 * dark energy: R0^3 = 8 G M t^2 / pi^2. Dark energy would widen it by some per cent (fudge GZ-F13).
 */
export function zeroVelocityRadius(mass_msun: number, t_gyr: number): number {
  return Math.cbrt((8 * G_MPC_GYR * Math.max(0, mass_msun) * t_gyr * t_gyr) / (Math.PI * Math.PI));
}

/**
 * The edge of our group (design 7.2): the zero-velocity sphere around home of the mass inside it, found by
 * widening it until it holds what it needs (home's halo, then every catalogued halo inside, which widens it again;
 * the radius only grows, so it settles in a few steps). Satellites are already inside home's halo, so not counted twice.
 */
export function groupEdge(home_halo_msun: number, gs: readonly Pick<NearGalaxy, "halo_mass_msun" | "distance_mpc" | "satellite">[], t_gyr: number): { radius_mpc: number; mass_msun: number; members: number } {
  const others = gs.filter((g) => !g.satellite).sort((a, b) => a.distance_mpc - b.distance_mpc);
  let mass = home_halo_msun;
  let r = zeroVelocityRadius(mass, t_gyr);
  let n = 0;
  while (n < others.length && others[n]!.distance_mpc <= r) {
    mass += others[n]!.halo_mass_msun;
    n++;
    r = zeroVelocityRadius(mass, t_gyr);
  }
  return { radius_mpc: r, mass_msun: mass, members: n };
}

/** A galaxy's place relative to home, Mpc, in the galactic frame (x to our centre, z out of our disc). */
export const nearbyAt = (g: Pick<NearGalaxy, "dir" | "distance_mpc">): [number, number, number] => [g.dir[0] * g.distance_mpc, g.dir[1] * g.distance_mpc, g.dir[2] * g.distance_mpc];

/** The galaxies inside a reach, biggest in stars first. */
export function withinReach(gs: readonly NearGalaxy[], reach_mpc: number): NearGalaxy[] {
  return gs.filter((g) => g.distance_mpc <= reach_mpc).sort((a, b) => b.stellar_mass_msun - a.stellar_mass_msun || (a.address < b.address ? -1 : 1));
}

/** How the sphere is seen: from above our disc, tilted 60 degrees down, edge-on along our disc, or turned freely. */
export type NearbySeen = "above" | "tilted" | "edge" | "free";
const TILT_DEG = 60;

/** A free turn (GZ5 part 2): yaw about our disc's axis and how far the view is tipped from straight above, degrees. */
export interface NearbyTurn {
  yaw_deg: number;
  tilt_deg: number;
}
/** Where Free starts: the Tilted view, so pressing Free changes nothing until it is dragged. */
export const FREE_START: NearbyTurn = { yaw_deg: 0, tilt_deg: TILT_DEG };

/** The turn each fixed view is: Above is no tilt, Tilted 60 degrees, Edge-on 90 (along our disc). */
export function turnOf(seen: NearbySeen, free: NearbyTurn = FREE_START): NearbyTurn {
  if (seen === "free") return { yaw_deg: free.yaw_deg, tilt_deg: Math.max(0, Math.min(90, free.tilt_deg)) };
  return { yaw_deg: 0, tilt_deg: seen === "above" ? 0 : seen === "edge" ? 90 : TILT_DEG };
}

/**
 * Project a place (Mpc, galactic frame) onto the drawing: `size` across, `reach` Mpc from the middle to the edge.
 * Returns the page point, the foot of its stalk on our disc's plane (the same plane Your systems stands its stalks
 * on) and a depth for drawing order (larger is nearer the viewer). The view is turned by `yaw` about our disc's
 * axis, then tipped by `tilt` about the page's across line: 0 is straight above, 90 edge-on.
 */
export function projectNearby(p: readonly [number, number, number], seen: NearbySeen, size: number, reach_mpc: number, pad = 24, free: NearbyTurn = FREE_START): { x: number; y: number; fx: number; fy: number; depth: number } {
  const k = (size / 2 - pad) / reach_mpc;
  const c = size / 2;
  const turn = turnOf(seen, free);
  const yw = (turn.yaw_deg * Math.PI) / 180;
  const t = (turn.tilt_deg * Math.PI) / 180;
  const cy = Math.cos(yw);
  const sy0 = Math.sin(yw);
  const x = p[0] * cy - p[1] * sy0;
  const y = p[0] * sy0 + p[1] * cy;
  const z = p[2];
  // Clean values at the fixed views, so Above and Edge-on keep the exact plane they always had.
  const ct = turn.tilt_deg === 90 ? 0 : Math.cos(t);
  const st = turn.tilt_deg === 0 ? 0 : Math.sin(t);
  // Tipped about x: the plane's y goes up the page shortened by cos(t); height rises by sin(t).
  const up = (v: number, h: number) => v * ct + h * st;
  return { x: c + x * k, y: c - up(y, z) * k, fx: c + x * k, fy: c - up(y, 0) * k, depth: z * ct - y * st };
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

/**
 * The Readable factor for a reach, a round number so the chip can say it: big enough that a disc in the group shows
 * its shape; at our satellites' reach only 2, so our own disc does not cover the satellites' orbits.
 */
export const readableFactor = (reach_mpc: number): number => (reach_mpc <= 0.5 ? 2 : reach_mpc <= 2 ? 8 : 20);

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

/** How many of the biggest neighbours have their real history read in the background (design 7.3: "two or three"). */
export const REAL_FACES = 3;

/**
 * The neighbours whose real charts are read (design 7.3): the biggest in stars within the reach, not home's own
 * satellites (they are not catalogued haloes and have no chart of their own), biggest first.
 */
export function biggestNeighbours(gs: readonly NearGalaxy[], reach_mpc: number, n = REAL_FACES): NearGalaxy[] {
  return withinReach(gs, reach_mpc)
    .filter((g) => !g.satellite)
    .slice(0, n);
}

/** "meet in about 4 Gyr", or "already meeting" when the kernel says now. */
export function meetWords(approach_gyr: number | null): string {
  if (approach_gyr === null) return "bound, falling together";
  if (approach_gyr < 0.05) return "already meeting";
  return `meet in about ${approach_gyr < 1 ? Math.round(approach_gyr * 10) / 10 : Math.round(approach_gyr)} Gyr`;
}
