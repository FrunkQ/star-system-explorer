// The Hazard map's Profile strip (exotic objects design 3.8, XO18): pure layout over the kernel's `hazard_map`
// reading. Distance runs along the bottom on a stretched (logarithmic) scale from the surface out; one thin row per
// hazard in three tones; the fixed marks under it and the ship's Park and Dive marks over it. Nothing here is physics:
// every number is the kernel's.

export type Tone = "clear" | "harsh" | "deadly";
export interface HazardBand { from_m: number; to_m: number; tone: Tone }
export interface HazardRow { hazard: string; name: string; bands: HazardBand[]; says: string }
export interface HazardMark { what: string; distance_m: number }
export interface ShipMark { distance_m: number; set_by: string; says: string; clock_rate: number; hover_m_s2: number }
export interface MoonReading { moon: string; distance_m: number; sv_per_day: number; tone: Tone }
export interface HazardPass { closest_m: number; dose_sv: number; peak_hull_k: number; hours_in_belts: number; clock_lost_s: number }

/** The push to hold still, in m/s² and in Earth gravities (g0 = 9.80665 m/s², the defined standard). */
export const hoverWords = (a: number): string =>
  !Number.isFinite(a) ? "no push can hold still there" : `holding still there instead takes ${a < 10 ? a.toFixed(2) : a.toFixed(0)} m/s² (${(a / 9.80665).toFixed(a / 9.80665 < 10 ? 2 : 0)} g) of thrust`;
export interface HazardMapRead {
  address: string;
  line: "equator" | "poles";
  ship: { name: string; delta_v_m_s: number; shield_kg_m2: number };
  stock_ship: boolean;
  from_m: number;
  to_m: number;
  rows: HazardRow[];
  quiet: string[];
  marks: HazardMark[];
  moons: MoonReading[];
  safe_bands: [number, number][];
  park?: ShipMark;
  dive?: ShipMark;
  pass?: HazardPass;
  hull_limit_k: number;
  why: string[];
  stand_ins: string[];
}

/** x of a distance on the strip, `x0` to `x1` pixels, logarithmic from the map's start to its end. */
export function xOf(map: Pick<HazardMapRead, "from_m" | "to_m">, d: number, x0: number, x1: number): number {
  const a = Math.log(map.from_m);
  const b = Math.log(map.to_m);
  const t = (Math.log(Math.min(Math.max(d, map.from_m), map.to_m)) - a) / (b - a);
  return x0 + t * (x1 - x0);
}

/** A distance in the body's radii, or in au round a star far out (as the kernel words it). */
export function distanceWords(d: number, radius: number): string {
  const AU_M = 1.495978707e11;
  if (radius > 1e8 && d > 0.05 * AU_M) return `${(d / AU_M).toFixed(2)} au`;
  const r = d / radius;
  return `${r < 10 ? r.toFixed(2) : r.toFixed(1)} radii`;
}

/** Tick distances for the strip: 1, 2, 5 steps of each power of ten radii between the ends. */
export function radiusTicks(map: Pick<HazardMapRead, "from_m" | "to_m">): number[] {
  const out: number[] = [];
  const lo = map.from_m;
  for (let p = 0; p < 8; p++) {
    for (const k of [1, 2, 5]) {
      const d = lo * k * 10 ** p;
      if (d >= lo && d <= map.to_m) out.push(d);
    }
  }
  return out;
}

/** Marks that would overprint keep their tick but lose their label: labels take two rows in turn, each label as wide
 * as its words (`charPx` a letter), and one that would touch its row's last label is left off. */
export function labelled<T extends { x: number; label: string }>(marks: T[], charPx = 5.6, pad = 8): (T & { show: boolean; row: number })[] {
  const end = [-Infinity, -Infinity];
  return [...marks]
    .sort((a, b) => a.x - b.x)
    .map((m) => {
      const half = (m.label.length * charPx) / 2;
      const row = end[0]! <= end[1]! ? 0 : 1;
      const show = m.x - half >= end[row]! + pad;
      if (show) end[row] = m.x + half;
      return { ...m, show, row };
    });
}

/** A mark's short name on the strip ("lane swept by sol/io, 3642 km wide" reads "Io lane"); the full words stay in
 * its tooltip. */
export function shortMark(what: string): string {
  const lane = /^lane swept by ([^,]+)/.exec(what);
  if (lane) return `${lane[1]} lane`;
  const orbit = /^(.+)'s orbit$/.exec(what);
  if (orbit) return orbit[1]!;
  const edge = /^belts' (inner|outer) edge/.exec(what);
  if (edge) return `belts' ${edge[1]} edge`;
  if (what === "edge of the magnetic bubble") return "bubble's edge";
  return what;
}
