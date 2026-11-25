// Small bodies up close (track SB3, VP-R5): the kernel's figure for a resolved
// asteroid, comet nucleus or ring lump (`small_body` / `make_small_body`, crate
// sse-small) turned into a mesh. Grades (visual polish design section 3):
// - fact: the axes, the two lobes of a contact binary and where they sit, the
//   relief's amplitude, the albedo;
// - drawn from the numbers: the lumps themselves, a seeded noise whose
//   amplitude is the kernel's relief share and whose seed is the body's address,
//   so every seat sees the same lumps; never clickable or named.
// Stand-ins are in docs/fudges.md (rows SB-neck, SB-lumps, SB-grey).

import type { Vec3 } from "./frames.ts";

/** One lobe of a two-lobed body, from the kernel. */
export interface SmallLobe {
  axes_m: [number, number, number];
  /** Centre along the long axis from the barycentre, metres. */
  offset_m: number;
}

/** The kernel's figure of a small body (sse-small `Figure`) plus what colours it. */
export interface SmallFigure {
  /** "maclaurin", "jacobi", "irregular", "top-shape" or "two-lobed". */
  kind: string;
  /** Semi-axes a >= b >= c (the whole envelope for two lobes), metres. */
  axes_m: [number, number, number];
  lobes: readonly SmallLobe[];
  /** Relief as a share of the mean radius. */
  relief: number;
  /** Geometric albedo, 0..1. */
  albedo?: number;
  /** Space weathering, 0 fresh to 1 darkened and reddened. */
  weathering?: number;
}

/** A displaced mesh in units of the body's mean radius (the item's `radius` scales it). */
export interface SmallMesh {
  key: string;
  positions: Float32Array;
  normals: Float32Array;
  index: Uint32Array;
}

/** FNV-1a of a string: the lumps' seed from the body's address. */
export function seedOf(text: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

function hash3(x: number, y: number, z: number, seed: number): number {
  let h = seed ^ Math.imul(x, 0x27d4eb2d) ^ Math.imul(y, 0x165667b1) ^ Math.imul(z, 0x9e3779b1);
  h = Math.imul(h ^ (h >>> 15), 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Smooth value noise in 3D, -1..1. */
function noise(px: number, py: number, pz: number, seed: number): number {
  const xi = Math.floor(px), yi = Math.floor(py), zi = Math.floor(pz);
  const fx = px - xi, fy = py - yi, fz = pz - zi;
  const s = (t: number) => t * t * (3 - 2 * t);
  const [u, v, w] = [s(fx), s(fy), s(fz)];
  let acc = 0;
  for (let dz = 0; dz < 2; dz++)
    for (let dy = 0; dy < 2; dy++)
      for (let dx = 0; dx < 2; dx++) {
        const wt = (dx ? u : 1 - u) * (dy ? v : 1 - v) * (dz ? w : 1 - w);
        acc += wt * hash3(xi + dx, yi + dy, zi + dz, seed);
      }
  return acc * 2 - 1;
}

/** Lumps: three octaves, larger ones stronger (an impact-shaped spectrum, fudge SB-lumps). */
function lumps(d: Vec3, seed: number): number {
  let sum = 0, amp = 1, f = 1.5, norm = 0;
  for (let o = 0; o < 3; o++) {
    sum += amp * noise(d[0] * f + 17.1 * o, d[1] * f, d[2] * f, seed + o);
    norm += amp;
    amp *= 0.5;
    f *= 2.1;
  }
  return sum / norm;
}

/** Where a ray from the origin along unit `d` leaves an ellipsoid of semi-axes `ax` centred at `cx` on x (largest root). */
function exitAlong(d: Vec3, ax: readonly number[], cx: number): number {
  const [a, b, c] = ax as [number, number, number];
  const A = (d[0] * d[0]) / (a * a) + (d[1] * d[1]) / (b * b) + (d[2] * d[2]) / (c * c);
  const B = (-2 * d[0] * cx) / (a * a);
  const C = (cx * cx) / (a * a) - 1;
  const disc = B * B - 4 * A * C;
  if (disc < 0) return 0;
  return Math.max(0, (-B + Math.sqrt(disc)) / (2 * A));
}

/** Smooth maximum, so the neck between two lobes is a fillet, not a crease (fudge SB-neck). */
function smoothMax(x: number, y: number, k: number): number {
  const h = Math.max(k - Math.abs(x - y), 0) / k;
  return Math.max(x, y) + (h * h * k) / 4;
}

/** The radius along unit `d`, metres, of the kernel's figure without lumps. */
export function figureRadius(f: SmallFigure, d: Vec3): number {
  if (f.lobes.length === 2) {
    const [l1, l2] = f.lobes as [SmallLobe, SmallLobe];
    const r1 = exitAlong(d, l1.axes_m, l1.offset_m);
    const r2 = exitAlong(d, l2.axes_m, l2.offset_m);
    return smoothMax(r1, r2, 0.25 * Math.min(l1.axes_m[1], l2.axes_m[1]));
  }
  return exitAlong(d, f.axes_m, 0);
}

/** The volume-equivalent mean radius of the figure, metres. */
export function meanRadius(f: SmallFigure): number {
  if (f.lobes.length === 2) return Math.cbrt(f.lobes.reduce((s, l) => s + l.axes_m[0] * l.axes_m[1] * l.axes_m[2], 0));
  return Math.cbrt(f.axes_m[0] * f.axes_m[1] * f.axes_m[2]);
}

const meshes = new Map<string, SmallMesh>();

/**
 * The figure as a mesh in units of its mean radius, the long axis on x and the spin axis on z. `seed` is the body's
 * address. Cached by figure and seed; `segments` sets the grid (64 by 32 when left out).
 */
export function smallBodyMesh(f: SmallFigure, seed: string, segments = 64): SmallMesh {
  const key = `sb|${seed}|${JSON.stringify([f.axes_m, f.lobes, f.relief])}|${segments}`;
  const hit = meshes.get(key);
  if (hit) return hit;
  const nu = segments, nv = Math.max(4, segments >> 1);
  const R = meanRadius(f);
  const s = seedOf(seed);
  const pos = new Float32Array((nu + 1) * (nv + 1) * 3);
  for (let j = 0; j <= nv; j++) {
    const th = (Math.PI * j) / nv;
    for (let i = 0; i <= nu; i++) {
      const ph = (2 * Math.PI * i) / nu;
      const d: Vec3 = [Math.sin(th) * Math.cos(ph), Math.sin(th) * Math.sin(ph), Math.cos(th)];
      const r = (figureRadius(f, d) / R) * (1 + f.relief * lumps(d, s));
      const k = 3 * (j * (nu + 1) + i);
      pos[k] = d[0] * r;
      pos[k + 1] = d[1] * r;
      pos[k + 2] = d[2] * r;
    }
  }
  const idx: number[] = [];
  for (let j = 0; j < nv; j++)
    for (let i = 0; i < nu; i++) {
      const a = j * (nu + 1) + i, b = a + nu + 1;
      if (j > 0) idx.push(a, b, a + 1);
      if (j < nv - 1) idx.push(a + 1, b, b + 1);
    }
  const index = new Uint32Array(idx);
  const normals = new Float32Array(pos.length);
  for (let t = 0; t < index.length; t += 3) {
    const [a, b, c] = [index[t]! * 3, index[t + 1]! * 3, index[t + 2]! * 3];
    const e1 = [pos[b]! - pos[a]!, pos[b + 1]! - pos[a + 1]!, pos[b + 2]! - pos[a + 2]!];
    const e2 = [pos[c]! - pos[a]!, pos[c + 1]! - pos[a + 1]!, pos[c + 2]! - pos[a + 2]!];
    const n = [e1[1]! * e2[2]! - e1[2]! * e2[1]!, e1[2]! * e2[0]! - e1[0]! * e2[2]!, e1[0]! * e2[1]! - e1[1]! * e2[0]!];
    for (const v of [a, b, c]) {
      normals[v] = normals[v]! + n[0]!;
      normals[v + 1] = normals[v + 1]! + n[1]!;
      normals[v + 2] = normals[v + 2]! + n[2]!;
    }
  }
  // The seam (i = 0 and i = nu) shares one normal.
  for (let j = 0; j <= nv; j++) {
    const a = 3 * (j * (nu + 1)), b = 3 * (j * (nu + 1) + nu);
    for (let q = 0; q < 3; q++) normals[a + q] = normals[b + q] = normals[a + q]! + normals[b + q]!;
  }
  for (let v = 0; v < normals.length; v += 3) {
    const l = Math.hypot(normals[v]!, normals[v + 1]!, normals[v + 2]!) || 1;
    // Outward: flip if it points in (the triangle winding above is outward, this guards the poles).
    const out = normals[v]! * pos[v]! + normals[v + 1]! * pos[v + 1]! + normals[v + 2]! * pos[v + 2]! < 0 ? -1 : 1;
    normals[v] = (out * normals[v]!) / l;
    normals[v + 1] = (out * normals[v + 1]!) / l;
    normals[v + 2] = (out * normals[v + 2]!) / l;
  }
  const m = { key, positions: pos, normals, index };
  meshes.set(key, m);
  return m;
}

/**
 * The body's grey from its albedo, as a CSS colour: a body of albedo 0.5 or more is drawn white, darker in proportion
 * (a diffuse surface's brightness goes as its albedo), with weathering's reddening as a slight warm tint
 * (fudge SB-grey: the kernel gives no spectrum for small bodies yet).
 */
export function smallBodyColour(f: SmallFigure): string {
  const a = Math.min(1, Math.max(0.02, (f.albedo ?? 0.1) / 0.5));
  const w = Math.min(1, Math.max(0, f.weathering ?? 0));
  const g = Math.round(255 * a);
  const r = Math.min(255, Math.round(g * (1 + 0.12 * w)));
  const b = Math.round(g * (1 - 0.12 * w));
  return `rgb(${r}, ${g}, ${b})`;
}
