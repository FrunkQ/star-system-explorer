// The cosmic web in 3D (V9w): the kernel's cube of parcels (sse-cosmos `Volume`), each where the truncated
// Zel'dovich step has moved it, sorted for drawing. In 3D they are dots coloured by the place they belong to (void,
// sheet, filament, knot); in True they are light, every parcel the same mass, so crowded matter glows brighter just
// by adding up. Presentation only: positions and kinds are the kernel's.
import type { Vec3 } from "./frames.ts";

export type WebKind = "void" | "sheet" | "filament" | "knot";
export const WEB_KINDS: readonly WebKind[] = ["void", "sheet", "filament", "knot"];

export interface WebCubeData {
  centre: readonly [number, number, number];
  extent: number;
  scale_factor: number;
  position: readonly number[];
  environment: readonly string[];
}

/**
 * The parcels by kind, in display units (Mpc from the cube's centre), shrunk by `shrink` (Real: the scale factor).
 * Parcels the step carried more than `margin` of the side outside the cube are left out: the field beyond the cube
 * was not read, so what would have moved in to take their place is not there either.
 */
export function webParcels(v: WebCubeData, shrink = 1, margin = 0.02): Record<WebKind, Vec3[]> {
  const out: Record<WebKind, Vec3[]> = { void: [], sheet: [], filament: [], knot: [] };
  const half = v.extent / 2;
  const lim = half * (1 + 2 * margin);
  const n = v.environment.length;
  // A parcel stands for the cube of matter round its grid point: each dot is drawn somewhere inside that cube
  // (a fixed scatter by its index), so the starting grid does not show through as a lattice.
  const side = Math.max(1, Math.round(Math.cbrt(n)));
  const cell = v.extent / side;
  for (let i = 0; i < n; i++) {
    const x = v.position[3 * i]! - v.centre[0] + cell * (scatter(i, 1) - 0.5);
    const y = v.position[3 * i + 1]! - v.centre[1] + cell * (scatter(i, 2) - 0.5);
    const z = v.position[3 * i + 2]! - v.centre[2] + cell * (scatter(i, 3) - 0.5);
    if (Math.abs(x) > lim || Math.abs(y) > lim || Math.abs(z) > lim) continue;
    const k = out[v.environment[i] as WebKind];
    if (k) k.push([x * shrink, y * shrink, z * shrink]);
  }
  return out;
}

/** A fixed number in [0, 1) for parcel i and axis k (an integer hash; the same every time). */
function scatter(i: number, k: number): number {
  let h = Math.imul(i + 1, 0x9e3779b1) ^ Math.imul(k, 0x85ebca77);
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d);
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** The cube's twelve edges as closed polylines (two squares and four uprights), half side `h`. */
export function cubeEdges(h: number): Vec3[][] {
  const sq = (z: number): Vec3[] => [
    [-h, -h, z],
    [h, -h, z],
    [h, h, z],
    [-h, h, z],
    [-h, -h, z],
  ];
  const up = (x: number, y: number): Vec3[] => [
    [x, y, -h],
    [x, y, h],
  ];
  return [sq(-h), sq(h), up(-h, -h), up(h, -h), up(h, h), up(-h, h)];
}

/**
 * True: galaxies' light. Each parcel is the same mass, so its light is one fixed share; the colour is old stellar
 * light at about 5,000 K, the colour of a typical galaxy today (g − r near 0.6, Blanton et al. 2003, ApJ 594, 186;
 * one colour for every galaxy is in the fudge register). Strength per dot rises as the cube's parcels get fewer, so a
 * coarse cube is not dimmer than a fine one.
 */
export const WEB_LIGHT_K = 5000;
export function webGlow(parcels: number): number {
  return Math.min(0.9, Math.max(0.08, 0.35 * Math.sqrt(32768 / Math.max(1, parcels))));
}

/** Space behind True, as the GPU path draws it (gpu.ts clears to black for `space`). */
export const SPACE_CSS = "rgb(0, 0, 0)";
