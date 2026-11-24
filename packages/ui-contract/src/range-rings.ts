// Range rings (RK12, Atomic Rockets plan section 13 and question 16): where a
// weapon's reading against a target changes, drawn as rings round the one who
// holds it, an overlay under the HUD like the grid. The kernel's
// `effect_at_range` (sse_sense::reach, built by the light thread) does every
// sum: what arrives at a distance, what the target's skin can take, the three
// readings, the odds of a hit, and the distances where each changes. Nothing
// here is fired and nothing is forbidden: a ring is a fact for the GM and dice.

/** What the weapon sends (sse_sense::reach::Weapon). */
export type RangeWeapon =
  | { kind: "beam"; beam_power_w: number; wavelength_m: number; aperture_m: number; spread?: number; jitter_rad?: number; dwell_s?: number; efficiency?: number; reject_k?: number }
  | { kind: "burst"; yield_j: number; xray_share: number }
  | { kind: "projectile"; mass_kg: number; muzzle_m_s: number; face_m2: number };

/** The target: its skin (a sheet material, or a typed figure kept as authored) and how it moves. */
export interface RangeTarget {
  skin: { material?: string; thickness_m: number; effect?: "boil" | "melt"; start_k?: number; absorbed_share?: number; takes_j_m2?: number };
  radius_m: number;
  accel_m_s2?: number;
  closing_m_s?: number;
}

export interface RangeAsk {
  weapon: RangeWeapon;
  target: RangeTarget;
  distance_m: number;
}

/** Distances where the reading changes (none where the energy does not fall with distance). */
export interface RangeRingsRead {
  in_range_m: number | null;
  enough_m: number | null;
  marginal_m: number | null;
  even_odds_m: number | null;
}

/** The kernel's answer for one ask (sse_sense::reach::Effect). */
export interface RangeEffect {
  reading: "in range" | "marginal" | "out of range" | "unknown";
  arrives_j_m2: number;
  takes_j_m2: number | null;
  takes_from: "sheet" | "authored" | null;
  ratio: number | null;
  odds: number | null;
  delay_s: number | null;
  wander_m: number | null;
  spot_radius_m?: number;
  rings: RangeRingsRead;
  load?: { name: string; electric_w: number; reject_k?: number };
  missing: string[];
  why: string[];
  references: { text: string; doi?: string; url?: string }[];
}

/** The HUD layer the rings draw on: shown with the HUD, hidden without it. */
export const RANGE_RING_LAYER = "range";

export type RangeRingKind = "in range" | "marginal" | "even odds";

/** One ring to draw. */
export interface RangeRing {
  radius_m: number;
  kind: RangeRingKind;
  /** Words for its label and tooltip. */
  words: string;
}

/** Metres in the unit a reader would use. */
export function rangeText(m: number): string {
  const km = m / 1000;
  return km >= 1e6 ? `${(km / 1e6).toFixed(2)} million km` : km >= 1000 ? `${Math.round(km).toLocaleString("en-GB")} km` : km >= 1 ? `${km.toFixed(km >= 100 ? 0 : 1)} km` : `${Math.round(m)} m`;
}

/**
 * The rings for one effect, outermost last: inside "in range" it passes what
 * the target can take with the margin to spare; out to "marginal" the GM and
 * the dice decide; "even odds" is where a hit falls to one in two (light lag
 * against how far the target can dodge). An effect the kernel could not read
 * (its skin unknown) draws only the odds ring.
 */
export function rangeRings(e: RangeEffect, name = "the weapon"): RangeRing[] {
  const out: RangeRing[] = [];
  const r = e.rings;
  if (r.in_range_m && r.in_range_m > 0) out.push({ radius_m: r.in_range_m, kind: "in range", words: `${name} in range out to ${rangeText(r.in_range_m)}` });
  if (r.marginal_m && r.marginal_m > 0) out.push({ radius_m: r.marginal_m, kind: "marginal", words: `marginal out to ${rangeText(r.marginal_m)}: the dice decide` });
  if (r.even_odds_m && r.even_odds_m > 0) out.push({ radius_m: r.even_odds_m, kind: "even odds", words: `one hit in two at ${rangeText(r.even_odds_m)}` });
  return out.sort((a, b) => a.radius_m - b.radius_m);
}

/**
 * Points round a ring of `radius_m` about `centre_m` in the orbital plane
 * (x, y; z kept), for a view to project with its own scale. A log-scaled chart
 * bends a ring off-centre, so a view projects each point rather than drawing a
 * circle.
 */
export function ringPoints(centre_m: readonly [number, number, number], radius_m: number, n = 96): [number, number, number][] {
  const out: [number, number, number][] = [];
  for (let i = 0; i < n; i++) {
    const a = (2 * Math.PI * i) / n;
    out.push([centre_m[0] + radius_m * Math.cos(a), centre_m[1] + radius_m * Math.sin(a), centre_m[2]]);
  }
  return out;
}
