// Ring structure from the moons (track SB, VP-R3; visual polish design 5.9). Grades (design section 3):
// - fact: where a moon's first-order inner Lindblad resonances fall, r = a_moon ((m − 1)/m)^(2/3), and how hard each
//   pushes against the ring's viscosity, the ratio of the resonance's torque (Tajeddine et al. 2017 eq. 5,
//   T = 8.6 m² μ² Σ n² r⁴) to the ring's viscous torque (eq. 6, 3π ν Σ n r²); Σ cancels, so the ratio needs only the
//   moon's mass, the planet's mass and a viscosity;
// - drawn from the numbers: a resonance that wins (ratio above 1) truncates the ring there (the B ring's outer edge at
//   Mimas 2:1, the A ring's at Janus 7:6); a weaker one is drawn as a thin dip whose depth is its share ratio/(1 + ratio);
//   a moon inside the rings clears a gap whose half-width follows its mass (fudge SB-gap-width, fitted to Pan's Encke
//   gap and Daphnis's Keeler gap).
// Edge waves beside an embedded moon (wavelength 3π times the gap's half-width, Tiscareno 2013) need the ring drawn
// round in two dimensions and are not drawn yet. Stand-ins are in docs/fudges.md (rows SB-ring-viscosity,
// SB-gap-width, SB-resonance-dip).
import { constant } from "../../ui-contract/src/index.ts";
import type { RingBand } from "./rings.ts";

/** A moon of a ringed world, as the structure needs it. */
export interface RingMoon {
  name: string;
  /** Its orbit's semi-major axis round the world, metres. */
  a_m: number;
  mass_kg: number;
}

/** One mark a moon leaves on the rings. */
export interface RingMark {
  /** "edge" (a resonance strong enough to hold the ring's edge), "dip" (a weaker resonance) or "gap" (cleared by a moon inside the rings). */
  kind: "edge" | "dip" | "gap";
  moon: string;
  /** Radius of the resonance or the moon, metres. */
  r_m: number;
  /** The resonance's m (m:m−1), or 0 for a gap. */
  m: number;
  /** Half-width of what is drawn, metres. */
  half_width_m: number;
  /** Resonant torque over the ring's viscous torque (gaps: Infinity). */
  ratio: number;
  /** How much of the ring is cleared there, 0 to 1. */
  depth: number;
}

/** The torque coefficient of a first-order resonance, f ≈ 8.6 m² (Tajeddine et al. 2017 eq. 5, from Goldreich and Tremaine 1982). */
export const RESONANCE_F = 8.6;

/**
 * A ring's effective viscosity when nothing better is known, m²/s: 2 × 10⁻³, inside the 1 to 3 × 10⁻³ Tajeddine et al.
 * 2017 find across the outer A ring and the B ring (fudge SB-ring-viscosity: one number for every ring).
 */
export const RING_VISCOSITY_M2_S = 2e-3;

/** A gap's half-width as a share of its radius per √μ, 415, fitted to the Encke gap (Pan) and the Keeler gap (Daphnis) (fudge SB-gap-width). */
export const GAP_PER_ROOT_MU = 415;

/**
 * How far inside a ring an edge must fall to cut it, as a share of its radius: resonances placed without the planet's
 * oblateness are off by about half a percent (Mimas 2:1 at 116,900 km against the B ring's edge at 117,580 km), so a
 * ring whose recorded edge is that close already is the resonance's edge (fudge SB-resonance-j2).
 */
export const EDGE_SLACK = 0.01;

/** Highest m tried; resonances crowd toward the moon and a moon's 30:29 is already beyond what shows. */
const M_MAX = 30;

/**
 * The marks `moons` leave on rings between `inner_m` and `outer_m` round a world of `planet_kg`, inner edge first.
 * `nu` is the ring's viscosity, m²/s.
 */
export function ringMarks(moons: readonly RingMoon[], planet_kg: number, inner_m: number, outer_m: number, nu = RING_VISCOSITY_M2_S): RingMark[] {
  const gm = constant("G") * planet_kg;
  const out: RingMark[] = [];
  for (const moon of moons) {
    if (!(moon.a_m > 0 && moon.mass_kg > 0 && planet_kg > 0)) continue;
    const mu = moon.mass_kg / planet_kg;
    if (moon.a_m > inner_m && moon.a_m < outer_m) {
      const hw = GAP_PER_ROOT_MU * Math.sqrt(mu) * moon.a_m;
      out.push({ kind: "gap", moon: moon.name, r_m: moon.a_m, m: 0, half_width_m: hw, ratio: Infinity, depth: 1 });
      continue;
    }
    for (let m = 2; m <= M_MAX; m++) {
      const r = moon.a_m * Math.pow((m - 1) / m, 2 / 3);
      if (r < inner_m || r > outer_m) continue;
      const n = Math.sqrt(gm / (r * r * r));
      // T_R / T_ν = f m² μ² n² r⁴ / (3π ν n r²) = f m² μ² n r² / (3π ν).
      const ratio = (RESONANCE_F * m * m * mu * mu * n * r * r) / (3 * Math.PI * nu);
      // A resonance's forced zone is a few times r √(m μ) wide (drawn from the numbers: fudge SB-resonance-dip).
      const hw = Math.max(r * Math.sqrt(m * mu), 1e3);
      out.push({ kind: ratio >= 1 ? "edge" : "dip", moon: moon.name, r_m: r, m, half_width_m: hw, ratio, depth: ratio / (1 + ratio) });
    }
  }
  return out.sort((a, b) => a.r_m - b.r_m);
}

/**
 * The rings with the marks cut into them: a ring an edge falls inside ends at that edge (the part outside it is
 * swept clear), a gap splits the ring around the moon, and a dip is a narrow ring of lower optical depth laid over
 * it (ringProfile lets the narrowest ring holding a radius speak). Only marks with depth of at least `min_depth` are cut.
 */
export function cutRings(rings: readonly RingBand[], marks: readonly RingMark[], min_depth = 0.05): RingBand[] {
  let out = rings.map((r) => ({ ...r }));
  for (const k of marks) {
    if (k.depth < min_depth) continue;
    if (k.kind === "edge") {
      // The ring holding the edge is truncated there; rings wholly outside an edge belong to other moons and stay.
      out = out.map((r) => (k.r_m > r.inner_m && k.r_m < r.outer_m * (1 - EDGE_SLACK) ? { ...r, outer_m: k.r_m } : r));
      continue;
    }
    if (k.kind === "gap") {
      const lo = k.r_m - k.half_width_m, hi = k.r_m + k.half_width_m;
      out = out.flatMap((r) => {
        if (hi <= r.inner_m || lo >= r.outer_m) return [r];
        const parts: RingBand[] = [];
        if (lo > r.inner_m) parts.push({ ...r, outer_m: lo });
        if (hi < r.outer_m) parts.push({ ...r, inner_m: hi });
        return parts;
      });
      continue;
    }
    const host = out.find((r) => k.r_m > r.inner_m && k.r_m < r.outer_m);
    if (!host) continue;
    const tau = (host.tau ?? 0.5) * (1 - k.depth);
    out.push({ ...host, name: `${k.moon} ${k.m}:${k.m - 1}`, inner_m: k.r_m - k.half_width_m, outer_m: k.r_m + k.half_width_m, tau });
  }
  return out.filter((r) => r.outer_m > r.inner_m);
}
