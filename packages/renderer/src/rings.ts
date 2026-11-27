// Rings and belts in True (V10f). A world's rings are drawn from what the universe holds for each: its edges, its
// normal optical depth τ and its particles' albedo. Seen through a ring, light behind it is let through by e^(−τ/μ),
// with μ the cosine between the line of sight and the ring's normal; what the ring stops it scatters back, so a
// thick ring is bright and a thin one a veil. The profile here is one row of pixels from the inner edge to the
// outer: colour from the albedo, cover (alpha) from τ seen face on. The draw does not yet thicken it with the slant
// (1/μ) (fudge register); the world's shadow on it, and its shadow on the world, are gpu-shade.ts's (EC1). A ring with no τ or albedo on record is
// drawn at the fallbacks below, also in the register.
// Belts round the star are a haze of points, one per draw slot, never the real count (also in the register).
import { constant } from "../../ui-contract/src/index.ts";

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
export const BELT_SECONDS_PER_SECOND = 30 * constant("day");

// ── One ring per world, from the kernel (VP-R3 structure, VP-R2 light) ──

/** What the kernel's `ring_structure` answer holds that the drawing reads (sse-small ring.rs `RingSystem`). */
export interface KernelRing {
  inner_m: number;
  outer_m: number;
  zones: { name?: string; inner_m: number; outer_m: number; tau: number; albedo: number; rgb: [number, number, number]; dust_share: number; assumed: boolean }[];
  marks: { kind: string; sculptor: string; sculptor_name?: string; m: number; r_m: number; half_width_m: number }[];
  phase: RingPhase;
  profile: { lo_m: number; hi_m: number; tau: number[]; zone: number[] };
  why: string[];
}

/** How a ring's light turns with the angle between sun and eye (the kernel's pack numbers, ring.rs `Phase`). */
export interface RingPhase {
  back_g: number;
  forward_g: number;
  surge_b0: number;
  surge_width_deg: number;
}

/** Per pixel, what the light needs: normal optical depth, the grains' linear colour and the share that is dust. */
export interface RingLookData {
  tau: Float32Array;
  rgb: Float32Array;
  dust: Float32Array;
  phase: RingPhase;
}

const srgbByte = (v: number) => Math.round(255 * Math.min(1, Math.max(0, v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055)));

/**
 * The kernel's ring as a profile: one row of RGBA pixels from its sampled span, colour the grains' own (sRGB bytes of
 * the zone's linear colour) and alpha the light it stops face on, 1 − e^(−τ) (the cover the planet's ring shadow
 * reads). `look` carries the same pixels for the light (ringLook).
 */
export function kernelProfile(k: KernelRing): RingProfile & { look: RingLookData } {
  const n = k.profile.tau.length;
  const data = new Uint8ClampedArray(n * 4);
  const tau = new Float32Array(n);
  const rgb = new Float32Array(n * 3);
  const dust = new Float32Array(n);
  let assumed = false;
  for (let i = 0; i < n; i++) {
    const t = Math.max(0, k.profile.tau[i] ?? 0);
    const z = k.zones[k.profile.zone[i] ?? -1];
    const c = z?.rgb ?? [0, 0, 0];
    if (z?.assumed) assumed = true;
    tau[i] = t;
    rgb[i * 3] = c[0];
    rgb[i * 3 + 1] = c[1];
    rgb[i * 3 + 2] = c[2];
    dust[i] = z?.dust_share ?? 0;
    data[i * 4] = srgbByte(c[0]);
    data[i * 4 + 1] = srgbByte(c[1]);
    data[i * 4 + 2] = srgbByte(c[2]);
    data[i * 4 + 3] = Math.round(255 * (1 - Math.exp(-t)));
  }
  return { inner_m: k.profile.lo_m, outer_m: k.profile.hi_m, width: n, data, assumed, look: { tau, rgb, dust, phase: k.phase } };
}

/** The look's colours are stored over this, so a surge or a glow above 1 is not clipped; the light multiplies it back. */
export const RING_LOOK_SCALE = 2;

/** 4π times the Henyey-Greenstein phase function at phase angle `alpha` (radians): 1 for isotropic scattering. */
function hg(g: number, alpha: number): number {
  const c = -Math.cos(alpha); // the scattering angle is π − α
  return (1 - g * g) / Math.pow(1 + g * g - 2 * g * c, 1.5);
}

/**
 * How bright a ring is per pixel for light from `toLight` seen from `toEye` (`n` its normal): a thin layer of particles
 * scattering once (Chandrasekhar 1960; Cuzzi et al. 1984).
 * - Lit face: I/F = ϖP/4 · μ₀/(μ₀ + μ) · (1 − e^(−τ(1/μ₀ + 1/μ))).
 * - Unlit face, by the light let through: I/F = ϖP/4 · μ₀/(μ₀ − μ) · (e^(−τ/μ₀) − e^(−τ/μ)), which peaks at a middling
 *   depth: the densest bands go dark from below and the thinner ones glow (visual polish design touch 6).
 * - P: metre-sized particles scatter back (Henyey-Greenstein, g < 0) with the opposition surge B₀/(1 + tan(α/2)/h)
 *   (touch 2); the dust share of each zone scatters forward (touch 8).
 * ϖP/4 is normalised so a thick ring at zero phase, without the surge, has I/F equal to its grains' colour (fudge
 * SB-ring-phase-scale). The pixels are premultiplied: the colour is I/F itself over RING_LOOK_SCALE (so a dusty ring too
 * faint to hide anything still shines) and alpha the background it hides along the slant, 1 − e^(−τ/μ). Writes into
 * `out` when given.
 */
export function ringLook(d: RingLookData, n: readonly number[], toLight: readonly number[], toEye: readonly number[], out?: Uint8Array | Uint8ClampedArray): Uint8Array | Uint8ClampedArray {
  const len = (v: readonly number[]) => Math.hypot(v[0]!, v[1]!, v[2]!) || 1;
  const dot = (a: readonly number[], b: readonly number[]) => (a[0]! * b[0]! + a[1]! * b[1]! + a[2]! * b[2]!) / (len(a) * len(b));
  const l = dot(n, toLight);
  const e = dot(n, toEye);
  const mu0 = Math.max(1e-3, Math.abs(l));
  const mu = Math.max(1e-3, Math.abs(e));
  const lit = Math.sign(l) === Math.sign(e);
  const alpha = Math.acos(Math.max(-1, Math.min(1, dot(toLight, toEye))));
  const p = d.phase;
  const h = Math.tan((p.surge_width_deg * Math.PI) / 360);
  const surge = 1 + p.surge_b0 / (1 + Math.tan(alpha / 2) / Math.max(1e-6, h));
  const back = (hg(p.back_g, alpha) * surge) / hg(p.back_g, 0);
  const fwd = hg(p.forward_g, alpha) / hg(p.back_g, 0);
  const N = d.tau.length;
  const o = out ?? new Uint8ClampedArray(N * 4);
  for (let i = 0; i < N; i++) {
    const t = d.tau[i]!;
    const ds = d.dust[i]!;
    const P = (1 - ds) * back + ds * fwd;
    let f: number;
    if (lit) f = ((2 * mu0) / (mu0 + mu)) * (1 - Math.exp(-t * (1 / mu0 + 1 / mu)));
    else if (Math.abs(mu0 - mu) < 1e-4) f = 2 * (t / mu0) * Math.exp(-t / mu0);
    else f = ((2 * mu0) / (mu0 - mu)) * (Math.exp(-t / mu0) - Math.exp(-t / mu));
    const a = 1 - Math.exp(-t / mu);
    const k = (P * f) / RING_LOOK_SCALE;
    o[i * 4] = srgbByte(d.rgb[i * 3]! * k);
    o[i * 4 + 1] = srgbByte(d.rgb[i * 3 + 1]! * k);
    o[i * 4 + 2] = srgbByte(d.rgb[i * 3 + 2]! * k);
    o[i * 4 + 3] = Math.round(255 * Math.min(1, a));
  }
  return o;
}
