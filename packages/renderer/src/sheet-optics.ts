// A material's colour from its data-sheet optics (TRUE4): the sheet's refractive index n + ik over wavelength, through
// Hapke's equivalent-slab grains, to the reflectance of a layer of those grains, seen by the eye (CIE 1931) in its star's
// light. No colour is typed: a Mars dust, a tholin or a frost is as red, brown or white as its measured absorption makes
// it. The kernel's tile albedo still sets how bright the ground is; this sets its hue.

import { cmf, linearSrgb, planck } from "../../ui-contract/src/index.ts";

/** A grain size, µm, for surfaces whose grains the kernel does not yet give (fudge register: fine regolith and frost). */
export const GRAIN_UM = 50;

/** The sheet's `optical.refractive_index` table (mixtures and substances share the shape). */
export interface IndexTable {
  lambda_nm: readonly number[];
  n: readonly number[];
  k: readonly number[];
  src?: string;
  check?: string;
}

/** The table read off a sheet row (mixture or substance), or null when it has none. */
export function indexTableOf(row: unknown): IndexTable | null {
  const o = (row as { optical?: Record<string, unknown> } | null)?.optical;
  const t = (o?.refractive_index ?? o?.refractive_index_solid) as Partial<IndexTable> | undefined;
  if (!t || !Array.isArray(t.lambda_nm) || !Array.isArray(t.n) || !Array.isArray(t.k)) return null;
  if (t.lambda_nm.length < 2 || t.n.length !== t.lambda_nm.length || t.k.length !== t.lambda_nm.length) return null;
  return t as IndexTable;
}

/** n and k at a wavelength: linear between the table's rows, held flat past its ends. */
export function indexAt(t: IndexTable, nm: number): { n: number; k: number } {
  const L = t.lambda_nm;
  if (nm <= L[0]!) return { n: t.n[0]!, k: t.k[0]! };
  const last = L.length - 1;
  if (nm >= L[last]!) return { n: t.n[last]!, k: t.k[last]! };
  let i = 1;
  while (L[i]! < nm) i++;
  const u = (nm - L[i - 1]!) / (L[i]! - L[i - 1]!);
  return { n: t.n[i - 1]! + u * (t.n[i]! - t.n[i - 1]!), k: t.k[i - 1]! + u * (t.k[i]! - t.k[i - 1]!) };
}

/**
 * The single-scattering albedo of a grain of diameter `d_um`, by Hapke's equivalent slab (Hapke 2012, Theory of
 * Reflectance and Emittance Spectroscopy, 2nd ed., eqs. 6.20 to 6.23 and 11.13): surface reflection S_e and internal
 * reflection S_i from n and k, the light crossing the grain's mean path ⟨D⟩ = (2/3)(n² − (n² − 1)^1.5 / n) D with
 * absorption α = 4πk/λ, and no internal scatterers.
 */
export function grainAlbedo(n: number, k: number, nm: number, d_um: number): number {
  const se = ((n - 1) ** 2 + k * k) / ((n + 1) ** 2 + k * k) + 0.05;
  const si = 1 - 4 / (n * (n + 1) ** 2);
  const path_m = (2 / 3) * (n * n - Math.pow(Math.max(0, n * n - 1), 1.5) / n) * d_um * 1e-6;
  const theta = Math.exp(-((4 * Math.PI * k) / (nm * 1e-9)) * path_m);
  return Math.min(1, se + ((1 - se) * (1 - si) * theta) / (1 - si * theta));
}

/** A thick layer's diffusive reflectance from its grains' albedo, r0 = (1 − γ)/(1 + γ), γ = √(1 − w) (Hapke 2012 eq. 8.25). */
export const layerReflectance = (w: number): number => {
  const g = Math.sqrt(Math.max(0, 1 - w));
  return (1 - g) / (1 + g);
};

/** One kind of grain in a surface layer: its optics and its share of the layer's grain cross-section. */
export interface GrainShare {
  table: IndexTable;
  share: number;
  d_um?: number;
}

/**
 * The single-scattering albedo of an intimate mixture of grains (Hapke 2012 eq. 10.7): the shares' albedos weighted by
 * their cross-section shares, so a little dark tholin among bright ice grains stains it, as on Pluto and Charon.
 */
export function mixAlbedo(grains: readonly GrainShare[], nm: number, d_um = GRAIN_UM): number {
  let w = 0;
  let total = 0;
  for (const g of grains) {
    if (!(g.share > 0)) continue;
    const { n, k } = indexAt(g.table, nm);
    w += g.share * grainAlbedo(n, k, nm, g.d_um ?? d_um);
    total += g.share;
  }
  return total > 0 ? w / total : 0;
}

/**
 * What the eye sees of a layer of the material's grains in its star's light, the eye adapted to that light (a white
 * layer looks white): linear sRGB per channel over a white layer's, and its visual albedo (the CIE Y share). Grains
 * of `d_um` (the fudge register's grain size until the kernel gives one per surface).
 */
export function seenColour(t: IndexTable | readonly GrainShare[], d_um = GRAIN_UM, star_k = 5772): { rgb: [number, number, number]; albedo: number } {
  const grains: readonly GrainShare[] = "lambda_nm" in t ? [{ table: t, share: 1 }] : t;
  const lit = [0, 0, 0];
  const white = [0, 0, 0];
  for (let nm = 380; nm <= 780; nm += 5) {
    const r = layerReflectance(mixAlbedo(grains, nm, d_um));
    const s = planck(nm * 1e-9, star_k);
    const c = cmf(nm);
    for (let j = 0; j < 3; j++) {
      lit[j] += r * s * c[j]!;
      white[j] += s * c[j]!;
    }
  }
  const a = linearSrgb(lit as [number, number, number]);
  const w = linearSrgb(white as [number, number, number]);
  const rgb = [0, 1, 2].map((j) => Math.max(0, a[j]! / w[j]!)) as [number, number, number];
  return { rgb, albedo: lit[1]! / white[1]! };
}
