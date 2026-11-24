// The cosmic web in 3D (V9w): the kernel's cube of parcels (sse-cosmos `Volume`), each where the truncated
// Zel'dovich step has moved it, sorted for drawing. In 3D they are dots coloured by the place they belong to (void,
// sheet, filament, knot); in True they are light, every parcel the same mass, so crowded matter glows brighter just
// by adding up. Presentation only: positions and kinds are the kernel's.
import type { Vec3 } from "./frames.ts";
import { GALAXY_LIGHT } from "./galaxy-true.ts";

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

/**
 * Light from haloes (V9w): the kernel's galaxies in the same cube (`web_slice` with `cube` and `haloes`), G2's haloes
 * moved by the cube's own step, each with its G3 stars and star formation then. Only galaxies shine; the voids' and
 * sheets' gas and dark matter are dark, as they are in a real survey of the sky.
 */
export interface WebGalaxyData {
  position: readonly [number, number, number];
  halo_msun: number;
  stars_msun: number;
  sfr_msun_yr: number;
  quenched: boolean;
}

/** One group of galaxies drawn alike: a colour and a strength (the GPU path takes one of each per group). */
export interface WebLightGroup {
  points: Vec3[];
  colour_k: number;
  glow: number;
  size: number;
}

/**
 * A galaxy's visible light and colour, from the same laws the True galaxy uses (`GALAXY_LIGHT`): old stars by their
 * mass (Bell and de Jong 2001), young stars by the star formation (Starburst99), and its colour the light-weighted
 * mix of the old (about 4,800 K) and young (about 15,000 K) populations, mixed in temperature (fudge register "web
 * light: colour by mixing temperatures").
 */
export function galaxyLight(g: Pick<WebGalaxyData, "stars_msun" | "sfr_msun_yr">): { l_sun: number; colour_k: number } {
  const old = GALAXY_LIGHT.old_l_per_m * Math.max(0, g.stars_msun);
  const young = GALAXY_LIGHT.young_l_per_sfr * Math.max(0, g.sfr_msun_yr);
  const l = old + young;
  const share = l > 0 ? young / l : 0;
  return { l_sun: l, colour_k: GALAXY_LIGHT.old_k + (GALAXY_LIGHT.young_k - GALAXY_LIGHT.old_k) * share };
}

/** How the True web's light is shown: decades of brightness below the brightest galaxy before a galaxy fades out
 * (as a long photograph would, like the True galaxy's `decades`), and the colour steps drawn. */
export const WEB_LIGHT_SHOW = { decades: 3.5, colours: [4800, 6500, 9000, 15000], steps: 6 } as const;

/**
 * The galaxies as groups of light, in display units (Mpc from the cube's centre, shrunk by `shrink`), each galaxy
 * in the group of its brightness and colour. Galaxies carried more than `margin` of the side outside are left out,
 * as the parcels are.
 */
export function webLights(
  galaxies: readonly WebGalaxyData[],
  cube: { centre: readonly [number, number, number]; extent: number },
  shrink = 1,
  margin = 0.02,
): WebLightGroup[] {
  const lim = (cube.extent / 2) * (1 + 2 * margin);
  const lit = galaxies.map((g) => ({ g, ...galaxyLight(g) })).filter((x) => x.l_sun > 0);
  if (!lit.length) return [];
  const top = Math.log10(Math.max(...lit.map((x) => x.l_sun)));
  const { decades, colours, steps } = WEB_LIGHT_SHOW;
  const groups = new Map<string, WebLightGroup>();
  for (const { g, l_sun, colour_k } of lit) {
    const p: Vec3 = [g.position[0] - cube.centre[0], g.position[1] - cube.centre[1], g.position[2] - cube.centre[2]];
    if (Math.abs(p[0]) > lim || Math.abs(p[1]) > lim || Math.abs(p[2]) > lim) continue;
    // 0 at the faintest shown, 1 at the brightest.
    const b = 1 - (top - Math.log10(l_sun)) / decades;
    if (b <= 0) continue;
    const step = Math.min(steps - 1, Math.floor(b * steps));
    let ci = 0;
    for (let i = 1; i < colours.length; i++) if (Math.abs(colours[i]! - colour_k) < Math.abs(colours[ci]! - colour_k)) ci = i;
    const key = `${step}:${ci}`;
    let grp = groups.get(key);
    if (!grp) {
      const s = (step + 0.5) / steps;
      grp = { points: [], colour_k: colours[ci]!, glow: 0.15 + 0.85 * s, size: 1.2 + 2.3 * s };
      groups.set(key, grp);
    }
    grp.points.push([p[0] * shrink, p[1] * shrink, p[2] * shrink]);
  }
  // Faint first, so the bright are drawn last (the same every time).
  return [...groups.entries()].sort((a, b) => (a[0] < b[0] ? -1 : 1)).map((e) => e[1]);
}

/** The cube's dark matter under True's HUD: the parcels as a faint guide to where the galaxies sit. */
export const WEB_DARK_GLOW = 0.05;
