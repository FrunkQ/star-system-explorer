// Rings and belts in True (V10f). A world's rings are drawn from what the universe holds for each: its edges, its
// normal optical depth τ and its particles' albedo. Seen through a ring, light behind it is let through by e^(−τ/μ),
// with μ the cosine between the line of sight and the ring's normal; what the ring stops it scatters back, so a
// thick ring is bright and a thin one a veil. The profile here is one row of pixels from the inner edge to the
// outer: colour from the albedo, cover (alpha) from τ seen face on. The draw does not yet thicken it with the slant
// (1/μ), nor cast the world's shadow on it; both are in the fudge register. A ring with no τ or albedo on record is
// drawn at the fallbacks below, also in the register.
// Belts round the star are a haze of points, one per draw slot, never the real count (also in the register).

/** One ring of a world, as Body3D carries it. */
export interface RingBand {
  name: string;
  inner_m: number;
  outer_m: number;
  tau: number | null;
  albedo: number | null;
}

/** τ and albedo used for a ring the universe has no value for (fudge register): a middling ring, half lit. */
export const RING_FALLBACK = { tau: 0.5, albedo: 0.5 } as const;

/** A world's rings as one picture: the span they cover and a row of RGBA pixels across it, inner edge first. */
export interface RingProfile {
  inner_m: number;
  outer_m: number;
  width: number;
  data: Uint8ClampedArray;
  /** True when any ring was drawn at the fallback τ or albedo. */
  assumed: boolean;
}

/**
 * The profile of a world's rings over `width` pixels. `tint` is the colour of the ring material in 0..1 linear
 * (the world's own when nothing better is known); each ring's pixels carry it times the albedo, and an alpha of the
 * light it stops face on, 1 − e^(−τ). Gaps between rings are clear. Null when the world has no ring with real edges.
 */
export function ringProfile(rings: readonly RingBand[], tint: [number, number, number], width = 512): RingProfile | null {
  const real = rings.filter((r) => r.outer_m > r.inner_m && r.inner_m > 0);
  if (!real.length) return null;
  const inner_m = Math.min(...real.map((r) => r.inner_m));
  const outer_m = Math.max(...real.map((r) => r.outer_m));
  const data = new Uint8ClampedArray(width * 4);
  let assumed = false;
  for (let i = 0; i < width; i++) {
    const r = inner_m + ((i + 0.5) / width) * (outer_m - inner_m);
    // Where rings overlap (a gap drawn inside a broader ring), the narrowest one holding r is the one that speaks.
    const here = real.filter((x) => r >= x.inner_m && r < x.outer_m).sort((a, b) => a.outer_m - a.inner_m - (b.outer_m - b.inner_m))[0];
    if (!here) continue;
    if (here.tau === null || here.albedo === null) assumed = true;
    const tau = here.tau ?? RING_FALLBACK.tau;
    const albedo = here.albedo ?? RING_FALLBACK.albedo;
    const k = i * 4;
    data[k] = Math.round(255 * Math.min(1, tint[0] * albedo * 2));
    data[k + 1] = Math.round(255 * Math.min(1, tint[1] * albedo * 2));
    data[k + 2] = Math.round(255 * Math.min(1, tint[2] * albedo * 2));
    data[k + 3] = Math.round(255 * (1 - Math.exp(-Math.max(0, tau))));
  }
  return { inner_m, outer_m, width, data, assumed };
}

/**
 * The pole of a world tilted `tilt_rad` from its orbit's normal (+z). The direction it leans is not on record, so it
 * leans toward +x of the frame (fudge register); a world with no tilt has its rings in its orbit's plane.
 */
export function poleOf(tilt_rad: number | undefined): [number, number, number] {
  const t = tilt_rad ?? 0;
  return [Math.sin(t), 0, Math.cos(t)];
}

/**
 * A belt round the star as a haze of `n` points in its frame's display units: radii spread evenly between the
 * drawn edges (by a low-discrepancy sequence, so the same belt always draws the same haze), and heights from the
 * belt's tilt spread (σ, radians) times the radius. Deterministic and seed-free.
 */
export function beltHaze(inner: number, outer: number, sigma_rad: number, n: number): [number, number, number][] {
  // Roberts' R3 sequence (the generalised golden ratio in three dimensions): radius, angle and height each from their
  // own even sequence, so the haze has no spiral or streak in it.
  const g = 1.2207440846057596;
  const a1 = 1 / g;
  const a2 = 1 / (g * g);
  const a3 = 1 / (g * g * g);
  const out: [number, number, number][] = [];
  for (let i = 0; i < n; i++) {
    const u = (0.5 + a1 * i) % 1;
    const w = (0.5 + a2 * i) % 1;
    const v = (0.5 + a3 * i) % 1;
    // Area-even in radius (a uniform surface density between the edges).
    const r = Math.sqrt(inner * inner + (outer * outer - inner * inner) * u);
    const a = 2 * Math.PI * w;
    // Heights from a Rayleigh spread of tilts σ: |i| = σ √(−2 ln(1 − v)), signed by the angle's half, times r.
    const tilt = sigma_rad * Math.sqrt(-2 * Math.log(1 - v * 0.999)) * (w < 0.5 ? 1 : -1) * Math.sin(2 * Math.PI * u + a);
    out.push([r * Math.cos(a), r * Math.sin(a), r * tilt]);
  }
  return out;
}

/**
 * The haze's colour: a dim neutral grey, the colour of rock dust lit by a Sun-like star, kept dark so the points read
 * as faint dust against black space and never as stars (a display choice, in the fudge register with the count).
 */
export const BELT_DUST = "#4d4b47";
