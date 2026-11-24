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
 * How bright a ring of particles is, relative to its albedo, for light from `toLight` seen from `toEye` (any lengths;
 * `n` the ring's normal). A layer of many particles scattering once (Chandrasekhar 1960; Cuzzi et al. 1984 for
 * Saturn's rings) gives I/F = ϖ₀ P/4 · μ₀/(μ₀ + μ) on the lit face, so a ring is brighter than a flat surface of the
 * same albedo (μ₀) by 1/(μ₀ + μ), and stays bright when the light is low. Seen from the unlit face it shines only by
 * the light passing through it, drawn as a quarter of the lit face (fudge register). The cover term 1 − e^(−τ(1/μ₀
 * + 1/μ)) is left to the profile's face-on alpha.
 */
export function ringLight(n: readonly number[], toLight: readonly number[], toEye: readonly number[]): number {
  const len = (v: readonly number[]) => Math.hypot(v[0]!, v[1]!, v[2]!) || 1;
  const dot = (a: readonly number[], b: readonly number[]) => (a[0]! * b[0]! + a[1]! * b[1]! + a[2]! * b[2]!) / (len(a) * len(b));
  const l = dot(n, toLight);
  const e = dot(n, toEye);
  const mu0 = Math.max(1e-3, Math.abs(l));
  const mu = Math.max(1e-3, Math.abs(e));
  const lit = mu0 / (mu0 + mu);
  return Math.sign(l) === Math.sign(e) ? lit : 0.25 * lit;
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
 * The haze's colour: a neutral grey, the colour of rock dust lit by a Sun-like star, kept below the worlds so the
 * points read as dust against black space and never as stars (a display choice, in the fudge register with the count).
 */
export const BELT_DUST = "#8a867f";

/** One drawn member of a belt: its orbit (distance from the star, metres), where it was at t = 0, and its tilt. */
export interface BeltParticle {
  a_m: number;
  phase: number;
  incl: number;
  node: number;
}

/** A fixed number in [0, 1) for member i and draw k (an integer hash, the same every time; no lattice). */
function draw(i: number, k: number): number {
  let h = Math.imul(i + 1, 0x9e3779b1) ^ Math.imul(k + 1, 0x85ebca77);
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d);
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/**
 * `n` members of a belt between `a_in_m` and `a_out_m` round the star: spread evenly by area (a uniform surface
 * density), at random angles, each tilted by a Rayleigh spread of σ radians about a random node. Each draw is a hash of
 * the member's number, so the same belt always draws the same members and no spiral or streak shows.
 */
export function beltParticles(a_in_m: number, a_out_m: number, sigma_rad: number, n: number): BeltParticle[] {
  const out: BeltParticle[] = [];
  for (let i = 0; i < n; i++) {
    const a_m = Math.sqrt(a_in_m * a_in_m + (a_out_m * a_out_m - a_in_m * a_in_m) * draw(i, 0));
    out.push({ a_m, phase: 2 * Math.PI * draw(i, 1), incl: sigma_rad * Math.sqrt(-2 * Math.log(1 - 0.999 * draw(i, 2))), node: 2 * Math.PI * draw(i, 3) });
  }
  return out;
}

/**
 * Where the members are at `t_s` seconds: each goes round on its own circular orbit at the Kepler rate √(GM/a³), so
 * the inner edge of a belt runs ahead of the outer. `dist` maps metres to display units; nothing is drawn past `cap`.
 */
export function beltAt(ps: readonly BeltParticle[], t_s: number, gm: number, dist: (a_m: number) => number, cap: number): [number, number, number][] {
  return ps.map((p) => {
    const th = p.phase + Math.sqrt(gm / (p.a_m * p.a_m * p.a_m)) * t_s;
    const r = Math.min(cap, dist(p.a_m));
    const x = r * Math.cos(th);
    const y = r * Math.sin(th);
    // Tilted about its node: the height is the in-plane distance from the node line times the tilt's sine.
    const z = r * Math.sin(p.incl) * Math.sin(th - p.node);
    return [x, y, z];
  });
}

/** The belts' own clock in True: thirty days of their time a second, so the main belt goes round in about a minute. */
export const BELT_SECONDS_PER_SECOND = 30 * 86400;
