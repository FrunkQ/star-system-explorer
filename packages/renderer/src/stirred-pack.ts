// A giant's carry table filed for the GPU (track WS, package WS5): plain arrays, no three.js, so a worker can do it.
//
// The eddies are filed on a grid of 5° cells by bins of days, each row's longitude counted in a frame turning with
// that row's own wind from each bin's start; a place sums its own cell's list (stirred-gpu.ts `eddySum`).

import { bandOf, generations, wrap, zonalWind, type CarryTable } from "./stirred-carry.ts";
import { constant } from "../../ui-contract/src/index.ts";

export const DEG = 180 / Math.PI;
export const DAY_S = constant("day");
/** Width of every packed table texture but the eddy list's (`PackedCarry.listW`). */
export const W = 2048;
/**
 * Where a cell's list starts is kept as two floats, `first = hi × SPLIT + lo`: a float holds whole numbers exactly only
 * to 2^24, and Saturn's list runs past that (about 18.6 million entries, AC-ProArt 28 Nov).
 */
export const SPLIT = 4096;
/**
 * The eddies' grid: cells this many degrees on a side, filed per `GRID_D` days, each row's longitude counted in a frame
 * turning with that row's own wind from the start of each bin. The eddies drift with the bands, so in its row's frame
 * an eddy sweeps only its drift against the row's wind; filed in the ground's frame it swept whole rows and the list
 * outgrew a GPU's largest texture on Jupiter and Saturn (AC-ProArt, 28 Nov).
 */
export const CELL_DEG = 5;
export const ROWS = 180 / CELL_DEG;
export const COLS = 360 / CELL_DEG;
export const GRID_D = 2;
/** Most storms the shader reads (the kernel's lists hold a few dozen). */
export const MAX_STORMS = 64;
/** A date that stands for "always" in float32 (the kernel's never-born and never-ending storms). */
export const FOREVER = 1e30;

/** A carry table laid out for the GPU: Float32 arrays of `W`-wide rows, and the uniforms. */
export interface PackedCarry {
  table: CarryTable;
  /** Eastward wind by latitude, Ψ north and south by ρ: one value a texel. */
  u: Float32Array;
  psiN: Float32Array;
  psiS: Float32Array;
  /** Storms, three texels each: [lat, lon, size, aspect], [strength, sense, born, ends], [day_ref, drift, 0, 0]. */
  storms: Float32Array;
  stormCount: number;
  /** Eddies, two texels each: [lat, lon at birth, size, strength], [sense, born, ends, drift m/s]. */
  eddies: Float32Array;
  /** Per bin and cell, [first ÷ SPLIT, first mod SPLIT, count, 0] into `list`. */
  grid: Float32Array;
  /** Each row's frame: how fast it turns, degrees a day east. */
  rates: Float32Array;
  /** Eddy indices, in rows `listW` wide. */
  list: Float32Array;
  /** The list's texture width: the narrowest of 2048, 4096, 8192, 16384 no taller than it is wide. */
  listW: number;
  bins: number;
  /** Per band, [younger age, older age], days. */
  ages: Float32Array;
}

const rowsOf = (n: number) => Math.max(1, Math.ceil(n / W));
const padded = (n: number, per: number) => new Float32Array(W * rowsOf(n) * per);

/** File a carry table for the GPU (on the processor; a few tens of milliseconds for Jupiter). */
export function packCarry(t: CarryTable): PackedCarry {
  const a = t.radius_m;
  const rel = (d: number) => (d <= -1e300 ? -FOREVER : d >= 1e300 ? FOREVER : d - t.day0);
  const u = padded(t.u_m_s.length, 1);
  u.set(t.u_m_s);
  const psiN = padded(t.psi_north.length, 1);
  psiN.set(t.psi_north);
  const psiS = padded(t.psi_south_table.length, 1);
  psiS.set(t.psi_south_table);
  const ns = Math.min(t.storms.length, MAX_STORMS);
  const storms = padded(ns * 3, 4);
  t.storms.slice(0, ns).forEach((s, i) => {
    storms.set([s.lat_deg, s.lon_deg, s.size_km, s.aspect, s.strength, s.sense, rel(s.born), rel(s.ends), rel(s.day_ref), s.drift_deg_d, 0, 0], i * 12);
  });
  const eddies = padded(t.eddies.length * 2, 4);
  t.eddies.forEach((e, i) => eddies.set([e.lat_deg, e.lon_deg, e.size_km, e.strength, e.sense, rel(e.born), rel(e.ends), e.drift_m_s], i * 8));
  // The grid: each eddy in every cell its reach can touch in each bin it lives, in each row's own turning frame.
  const bins = Math.max(1, Math.ceil((t.day1 - t.day0) / GRID_D));
  const cells = ROWS * COLS;
  const rates = padded(ROWS, 1);
  for (let r = 0; r < ROWS; r++) {
    const lat = -90 + (r + 0.5) * CELL_DEG;
    rates[r] = ((zonalWind(t, lat) * DAY_S) / (a * Math.cos(lat / DEG))) * DEG;
  }
  const lists: number[][] = Array.from({ length: bins * cells }, () => []);
  t.eddies.forEach((e, idx) => {
    const reach = (3 * 0.5 * e.size_km * 1000) / a; // radians
    const reachDeg = reach * DEG;
    const rate = ((e.drift_m_s * DAY_S) / (a * Math.max(Math.cos(e.lat_deg / DEG), 1e-6))) * DEG; // degrees a day
    const lo = Math.max(0, Math.floor((e.born - t.day0) / GRID_D));
    const hi = Math.min(bins - 1, Math.max(0, Math.floor((e.ends - t.day0) / GRID_D)));
    const r0 = Math.max(0, Math.floor((e.lat_deg - reachDeg + 90) / CELL_DEG));
    const r1 = Math.min(ROWS - 1, Math.floor((e.lat_deg + reachDeg + 90) / CELL_DEG));
    // In longitude, as wide as the reach is at its poleward edge.
    const far = Math.min(90, Math.abs(e.lat_deg) + reachDeg);
    const cf = Math.cos(far / DEG);
    const half = cf < 1e-6 || Math.sin(Math.min(reach, Math.PI / 2)) >= cf ? 180 : Math.asin(Math.sin(Math.min(reach, Math.PI / 2)) / cf) * DEG;
    for (let b = lo; b <= hi; b++) {
      const t0 = t.day0 + b * GRID_D;
      // Where it is at the bin's start, in every row's frame alike; then how far it drifts against each row's wind.
      const lonA = e.lon_deg + rate * (t0 - e.born);
      for (let r = r0; r <= r1; r++) {
        const sweep = (rate - rates[r]!) * GRID_D;
        const from = Math.min(lonA, lonA + sweep) - half - CELL_DEG * 0.1;
        const to = Math.max(lonA, lonA + sweep) + half + CELL_DEG * 0.1;
        const all = half >= 180 || to - from >= 360;
        const c0 = all ? 0 : Math.floor((wrap(from) + 180) / CELL_DEG);
        const span = all ? COLS : Math.min(COLS, Math.ceil((to - from) / CELL_DEG) + 1);
        for (let k = 0; k < span; k++) lists[b * cells + r * COLS + ((c0 + k) % COLS)]!.push(idx);
      }
    }
  });
  const grid = padded(bins * cells, 4);
  let total = 0;
  lists.forEach((l, i) => {
    grid.set([Math.floor(total / SPLIT), total % SPLIT, l.length, 0], i * 4);
    total += l.length;
  });
  let listW = W;
  while (listW < 16384 && Math.ceil(total / listW) > listW) listW *= 2;
  const list = new Float32Array(listW * Math.max(1, Math.ceil(total / listW)));
  let at = 0;
  for (const l of lists) for (const x of l) list[at++] = x;
  // The two generations' ages in each band on the picture's date.
  const nb = 2 * Math.max(t.jets, 1);
  const ages = padded(nb, 2);
  for (let b = 0; b < nb; b++) {
    // A latitude inside band b (bands as bandOf files them).
    const j = Math.max(t.jets, 1);
    const k = b % j;
    const lat = Math.asin((k + 0.5) / j) * DEG * (b < j ? 1 : -1);
    const g = generations(t, lat);
    if (bandOf(t, lat) !== b) throw new Error(`band ${b} misfiled`);
    ages.set([g[0][0], g[1][0]], b * 2);
  }
  return { table: t, u, psiN, psiS, storms, stormCount: ns, eddies, grid, rates, list, listW, bins, ages };
}


/** The most steps any place takes in one leg (both legs are at most this). */
export function mostSteps(p: PackedCarry): number {
  const t = p.table;
  let most = 0;
  for (let b = 0; b < p.ages.length / 2 && b < 2 * Math.max(t.jets, 1); b++) most = Math.max(most, p.ages[b * 2]!, p.ages[b * 2 + 1]! - p.ages[b * 2]!);
  return Math.ceil((most * DAY_S) / Math.min(t.dt_s, t.dt_poly_s));
}
