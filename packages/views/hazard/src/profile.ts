// The Hazard map's Profile strip (exotic objects design 3.8, XO18): pure layout over the kernel's `hazard_map`
// reading. Distance runs along the bottom on a stretched (logarithmic) scale from the surface out; one thin row per
// hazard in three tones; the fixed marks under it and the ship's Park and Dive marks over it. Nothing here is physics:
// every number is the kernel's.

export type Tone = "clear" | "harsh" | "deadly";
export interface HazardBand { from_m: number; to_m: number; tone: Tone }
export interface HazardRow { hazard: string; name: string; bands: HazardBand[]; says: string }
export interface HazardMark { what: string; distance_m: number }
export interface ShipMark { distance_m: number; set_by: string; says: string }
export interface MoonReading { moon: string; distance_m: number; sv_per_day: number; tone: Tone }
export interface HazardPass { closest_m: number; dose_sv: number; peak_hull_k: number; hours_in_belts: number }
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

/** Marks that would overprint (closer than `gap` px) keep their tick but lose their label. */
export function labelled<T extends { x: number }>(marks: T[], gap = 46): (T & { show: boolean })[] {
  let last = -Infinity;
  return [...marks]
    .sort((a, b) => a.x - b.x)
    .map((m) => {
      const show = m.x - last >= gap;
      if (show) last = m.x;
      return { ...m, show };
    });
}
