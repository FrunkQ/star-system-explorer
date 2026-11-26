// Shadows and eclipses in True (renderer audit item 1): how much of its star a point on a world sees past the bodies
// between them, and past its own rings. Pure geometry, worked out where the bodies really are (metres, the kernel's
// places) and then turned into the drawing, so a moon's shadow falls where it would even when the view draws distances
// on a log scale and sizes for reading. The GPU does the same sums per pixel (gpu-shade.ts); the functions here are its
// processor twin, for the checks and for the view's own note.
//
// The star is a disc, not a point: a body covering part of it leaves part of the light (a penumbra, soft edges), one
// smaller than the star's disc covers only a ring of it (an annular eclipse), one covering all of it leaves only the
// light its air bends round into its shadow (the kernel's `read_shadow_light`: copper behind an Earth, nothing behind
// an airless world). Rings let light through by e^(−τ/μ₀) along the slant path (the same τ the ring's picture is drawn
// from, rings.ts), so a ringed world wears its rings' shadow across its face and its shadow lies across the rings.
import type { Vec3 } from "./frames.ts";

/** A body between a world and its star, in the world's own radii, in the drawing's frame, centred on the world. */
export interface Caster {
  /** Its centre from the world's centre. */
  at: Vec3;
  /** Its radius. */
  radius: number;
  /** The light its air bends into the middle of its shadow, per channel as a share of the star's own (linear); none, black. */
  umbra?: [number, number, number];
}

/** What a world's shadows need: its star's angular radius seen from it, and the bodies that may stand in the way. */
export interface Shade {
  /** The star's angular radius from the world, radians. */
  star: number;
  casters: Caster[];
}

/** The most bodies whose shadows one world wears at once (the GPU's uniform slots). */
export const MAX_CASTERS = 4;

const dot = (a: readonly number[], b: readonly number[]) => a[0]! * b[0]! + a[1]! * b[1]! + a[2]! * b[2]!;
const len = (a: readonly number[]) => Math.hypot(a[0]!, a[1]!, a[2]!);
const sub = (a: readonly number[], b: readonly number[]): Vec3 => [a[0]! - b[0]!, a[1]! - b[1]!, a[2]! - b[2]!];
const cross = (a: readonly number[], b: readonly number[]): Vec3 => [a[1]! * b[2]! - a[2]! * b[1]!, a[2]! * b[0]! - a[0]! * b[2]!, a[0]! * b[1]! - a[1]! * b[0]!];
const unit = (a: readonly number[]): Vec3 => {
  const l = len(a) || 1;
  return [a[0]! / l, a[1]! / l, a[2]! / l];
};
/** The angle between two directions, accurate when it is tiny (atan2 of the cross and dot, not acos). */
const angle = (a: readonly number[], b: readonly number[]) => Math.atan2(len(cross(a, b)), dot(a, b));

/**
 * The share of a disc of radius 1 covered by a disc of radius `r` whose centre is `d` away (both in units of the first
 * disc's radius): the lens of two overlapping circles over π. Small angles, so the sky is flat across the star.
 */
export function discCover(r: number, d: number): number {
  if (!(r > 0)) return 0;
  if (d >= 1 + r) return 0;
  if (d <= Math.abs(1 - r)) return r >= 1 ? 1 : r * r;
  const a1 = Math.acos(Math.max(-1, Math.min(1, (d * d + 1 - r * r) / (2 * d))));
  const a2 = Math.acos(Math.max(-1, Math.min(1, (d * d + r * r - 1) / (2 * d * r))));
  const k = Math.sqrt(Math.max(0, (-d + 1 + r) * (d + 1 - r) * (d - 1 + r) * (d + 1 + r)));
  return Math.max(0, Math.min(1, (a1 + r * r * a2 - 0.5 * k) / Math.PI));
}

/**
 * The starlight reaching point `p` (in the world's radii from its centre) with the star toward `toStar`, per channel
 * as a share of the unshadowed light: each caster covers part of the star's disc, and where it covers it leaves its
 * umbra's light (bent through its air) in place of the star's.
 */
export function starLightAt(p: readonly number[], toStar: readonly number[], shade: Shade): [number, number, number] {
  const s = unit(toStar);
  const a_s = Math.max(1e-6, shade.star);
  const out: [number, number, number] = [1, 1, 1];
  for (const c of shade.casters.slice(0, MAX_CASTERS)) {
    const d = sub(c.at, p);
    const dist = len(d);
    if (!(dist > c.radius)) continue;
    const a_c = Math.asin(c.radius / dist);
    const f = discCover(a_c / a_s, angle(d, s) / a_s);
    const u = c.umbra ?? [0, 0, 0];
    for (let k = 0; k < 3; k++) out[k] = out[k]! * (1 - f + f * u[k]!);
  }
  return out;
}

/**
 * The share of the star's light that comes through a world's rings to point `p` (its radii, from its centre): the ray
 * toward the star crosses the ring plane (normal `n`) at some radius, and the ring there lets e^(−τ/μ₀) through, μ₀ the
 * cosine of the star's height over the plane. `cover(ρ)` is the ring's face-on cover there, 1 − e^(−τ) (rings.ts's
 * alpha), so the slant passes (1 − cover)^(1/μ₀). One when the ray misses the rings.
 */
export function ringPassAt(p: readonly number[], toStar: readonly number[], n: readonly number[], inner: number, outer: number, cover: (rho: number) => number): number {
  const s = unit(toStar);
  const nn = unit(n);
  const mu0 = dot(s, nn);
  if (Math.abs(mu0) < 1e-6) return 1;
  const t = -dot(p, nn) / mu0;
  if (!(t > 0)) return 1;
  const q: Vec3 = [p[0]! + s[0] * t, p[1]! + s[1] * t, p[2]! + s[2] * t];
  const rho = len(q);
  if (rho < inner || rho > outer) return 1;
  return Math.pow(Math.max(0, 1 - Math.max(0, Math.min(1, cover(rho)))), 1 / Math.abs(mu0));
}

/** A body where it really is: its centre in metres in one frame for the whole system (any origin), and its radius. */
export interface Placed {
  id: string;
  at_m: readonly [number, number, number];
  radius_m: number;
  /** Its umbra's light per channel (`umbraLight`), when its air bends any into its shadow. */
  umbra?: [number, number, number];
}

/** A rotation taking direction `a` onto direction `b` by the shortest turn (Rodrigues); returned as a function. */
export function turnOnto(a: readonly number[], b: readonly number[]): (v: readonly number[]) => Vec3 {
  const u = unit(a);
  const w = unit(b);
  const k = cross(u, w);
  const s = len(k);
  const c = dot(u, w);
  if (s < 1e-12) {
    if (c > 0) return (v) => [v[0]!, v[1]!, v[2]!];
    // Opposite: a half turn about any axis square to a.
    const ax = unit(Math.abs(u[0]) < 0.9 ? cross(u, [1, 0, 0]) : cross(u, [0, 1, 0]));
    return (v) => {
      const d = 2 * dot(ax, v);
      return [ax[0] * d - v[0]!, ax[1] * d - v[1]!, ax[2] * d - v[2]!];
    };
  }
  const kk = unit(k);
  return (v) => {
    const kv = cross(kk, v);
    const kd = dot(kk, v);
    return [v[0]! * c + kv[0] * s + kk[0] * kd * (1 - c), v[1]! * c + kv[1] * s + kk[1] * kd * (1 - c), v[2]! * c + kv[2] * s + kk[2] * kd * (1 - c)];
  };
}

/**
 * The bodies that may shadow `world` from `star`, where they really are, carried into the drawing: centres and radii
 * in the world's radii, turned so the real direction to the star lies along `drawnToStar` (the way the drawing lights
 * the world). A body counts when it is nearer than the star and its disc comes within reach of the star's as seen from
 * somewhere on the world (the world's own size widens the reach by its angle from the caster). At most MAX_CASTERS,
 * nearest to covering first. Null when nothing can stand in the way.
 */
export function shadeFor(world: Placed, others: readonly Placed[], star: Placed, drawnToStar: readonly number[]): Shade | null {
  if (!(world.radius_m > 0)) return null;
  const toStar = sub(star.at_m, world.at_m);
  const d_s = len(toStar);
  if (!(d_s > world.radius_m)) return null;
  const a_s = Math.asin(Math.min(1, star.radius_m / d_s));
  const found: { c: Placed; off: Vec3; reach: number }[] = [];
  for (const c of others) {
    if (c.id === world.id || c.id === star.id || !(c.radius_m > 0)) continue;
    const off = sub(c.at_m, world.at_m);
    const d_c = len(off);
    if (!(d_c > world.radius_m + c.radius_m) || d_c >= d_s) continue;
    const a_c = Math.asin(Math.min(1, c.radius_m / d_c));
    const a_w = Math.asin(Math.min(1, world.radius_m / d_c));
    const theta = angle(off, toStar);
    const reach = theta / (a_c + a_s + a_w);
    if (reach < 1) found.push({ c, off, reach });
  }
  if (!found.length) return null;
  found.sort((x, y) => x.reach - y.reach || (x.c.id < y.c.id ? -1 : x.c.id > y.c.id ? 1 : 0));
  const turn = turnOnto(toStar, drawnToStar);
  const r = world.radius_m;
  return {
    star: a_s,
    casters: found.slice(0, MAX_CASTERS).map(({ c, off }) => {
      const t = turn(off);
      return { at: [t[0] / r, t[1] / r, t[2] / r] as Vec3, radius: c.radius_m / r, ...(c.umbra ? { umbra: c.umbra } : {}) };
    }),
  };
}

/** The part of the kernel's `read_shadow_light` reply the drawing uses (sse_light::shadow::ShadowLight). */
export interface ShadowLightRead {
  lit: boolean;
  colour?: { linear_srgb: [number, number, number] };
  star_colour: { linear_srgb: [number, number, number] };
  /** Its luminance as a share of the star's own light there (absent from a kernel that does not give it). */
  share?: number;
}

/**
 * The light in a body's umbra per channel as a share of the star's own: the kernel's colour over the star's colour,
 * scaled so its luminance is the kernel's share. Null when no light gets in (no air, too thin, or no brightness given:
 * a colour with no brightness is not drawn as if it were daylight).
 */
export function umbraLight(r: ShadowLightRead): [number, number, number] | null {
  if (!r.lit || !r.colour || !(typeof r.share === "number" && r.share > 0)) return null;
  const w = [0.2126, 0.7152, 0.0722];
  const c = r.colour.linear_srgb;
  const s = r.star_colour.linear_srgb;
  const t = c.map((v, k) => (s[k]! > 0 ? v / s[k]! : 0));
  const y = w.reduce((a, wk, k) => a + wk * s[k]! * t[k]!, 0) / w.reduce((a, wk, k) => a + wk * s[k]!, 0);
  if (!(y > 0)) return null;
  const k = r.share / y;
  return [t[0]! * k, t[1]! * k, t[2]! * k];
}

/**
 * The glint's two numbers where the kernel has not answered for a world (fudge register): its answer for water under
 * a 7 m/s wind (`sse.light.sea_glint`, Cox and Munk 1954), the waves' mean square slope and water's reflectance
 * straight back. The law itself is the kernel's; the view only draws it.
 */
export const GLINT_FALLBACK = { slope_variance: 0.03884, normal_reflectance: 0.0204 } as const;

/**
 * The glint's reflectance (a BRDF, per steradian) for light from `l` seen toward `v` (unit vectors from the sea, `n` its
 * level): F(θ) · p(tan β) / (4 μ₀ μ cos⁴β), p the Gaussian slope density e^(−tan²β/σ²)/(πσ²), β the tilt of the
 * facet that mirrors l into v, F Fresnel's in Schlick's form from F₀, the liquid's reflectance straight back (both σ² and
 * F₀ are the kernel's, `sse.light.sea_glint`).
 */
export function glintBrdf(n: readonly number[], l: readonly number[], v: readonly number[], sigma2: number, f0: number = GLINT_FALLBACK.normal_reflectance): number {
  const mu0 = dot(n, l);
  const mu = dot(n, v);
  if (!(mu0 > 0 && mu > 0)) return 0;
  const h = unit([l[0]! + v[0]!, l[1]! + v[1]!, l[2]! + v[2]!]);
  const cb = Math.max(1e-6, dot(n, h));
  const tan2 = (1 - cb * cb) / (cb * cb);
  const p = Math.exp(-tan2 / sigma2) / (Math.PI * sigma2);
  const f = f0 + (1 - f0) * Math.pow(1 - Math.max(0, dot(v, h)), 5);
  return (f * p) / (4 * mu0 * mu * cb * cb * cb * cb);
}
