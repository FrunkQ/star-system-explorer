// True's bare ground from measured reflectance (VP3 part 5, Alex 25 Nov: "the moon is very brown"). The data sheet
// holds laboratory reflectance spectra of real soils (packs/base-reference mixtures, `optical.reflectance`, MP35); the
// kernel's space weathering (SW1) says how mature each sample's soil is. Here the two meet: each material's spectrum is
// sampled at the three bands True draws (650, 550, 450 nm), blended from fresh to mature by the sample's maturity, and
// laid on the tiles as `ground_rgb`, which ui-contract `sampleColour` then uses in place of the typed palette.
// Space weathering darkens and reddens soil (nanophase iron, Hapke 2001): blend in log space, so a mature fraction m
// gives fresh^(1-m) * mature^m, the same as fresh * (mature/fresh)^m.
import type { SurfaceTile } from "../../ui-contract/src/index.ts";
import { softenGround, SURFACE_MATERIALS as MATERIALS, SURFACE_COVERS as COVERS } from "../../ui-contract/src/index.ts";
import ROWS from "./ground-rows.json" with { type: "json" };
import { GRAIN_UM, grainAlbedo, indexAt, indexTableOf, layerReflectance } from "./sheet-optics.ts";

type V3 = [number, number, number];

/** The wavelengths True draws as red, green and blue, nm (the light thread's band centres). */
export const GROUND_BANDS_NM: readonly [number, number, number] = [650, 550, 450];

/** A sheet reflectance table as MP35 writes it. */
export interface ReflectanceTable {
  lambda_nm: number[];
  r: number[];
  maturity?: string;
}

/** Each material's measured reflectance at the three bands, fresh and mature (either may be missing). */
export type GroundOptics = Record<string, { fresh?: V3; mature?: V3 }>;

/** The table's value at a wavelength: linear between points, ends held (as the sheet's note says). */
export function reflectanceAt(t: ReflectanceTable, nm: number): number {
  const l = t.lambda_nm;
  const r = t.r;
  if (!l.length) return NaN;
  if (nm <= l[0]!) return r[0]!;
  for (let i = 1; i < l.length; i++) if (nm <= l[i]!) return r[i - 1]! + ((r[i]! - r[i - 1]!) * (nm - l[i - 1]!)) / (l[i]! - l[i - 1]!);
  return r[l.length - 1]!;
}

export const bandsOf = (t: ReflectanceTable): V3 => [reflectanceAt(t, GROUND_BANDS_NM[0]), reflectanceAt(t, GROUND_BANDS_NM[1]), reflectanceAt(t, GROUND_BANDS_NM[2])];

/** The material-to-row naming (ground-rows.json). */
export const GROUND_ROWS: Record<string, { fresh?: string; mature?: string }> = ROWS.rows;

/** The kernel door this needs (a ViewSession has it). */
export interface MatterReader {
  readMatter(ask: { table?: string; id?: string }): Promise<{ ok: true; value: unknown } | { ok: false; message: string }>;
}

/**
 * A sheet row's reflectance at the three bands: its measured spectrum (`optical.reflectance`) where it has one, else a
 * thick layer of its grains from its refractive index n + ik (Hapke's equivalent slab, sheet-optics.ts, at the same
 * grain size the stains use for regolith, GRAIN_UM), or null when it has neither.
 */
export function rowBands(row: unknown): V3 | null {
  const t = (row as { optical?: { reflectance?: ReflectanceTable } } | null)?.optical?.reflectance;
  if (t?.lambda_nm?.length && t.r?.length === t.lambda_nm.length) return bandsOf(t);
  const ix = indexTableOf(row);
  if (!ix) return null;
  return GROUND_BANDS_NM.map((nm) => {
    const { n, k } = indexAt(ix, nm);
    return layerReflectance(grainAlbedo(n, k, nm, GRAIN_UM));
  }) as V3;
}

let read: Promise<GroundOptics> | null = null;

/** Every named row's bands, read from the sheet once for the page. A row the sheet lacks is simply left out. */
export function readGroundOptics(k: MatterReader): Promise<GroundOptics> {
  read ??= (async () => {
    const out: GroundOptics = {};
    for (const [material, ids] of Object.entries(GROUND_ROWS)) {
      const o: { fresh?: V3; mature?: V3 } = {};
      for (const which of ["fresh", "mature"] as const) {
        const id = ids[which];
        if (!id) continue;
        const r = await k.readMatter({ id });
        const b = r.ok ? rowBands((r.value as { row?: unknown }).row) : null;
        if (b) o[which] = b;
      }
      if (o.fresh || o.mature) out[material] = o;
    }
    return out;
  })();
  return read;
}

/** For tests: forget the page's copy. */
export function forgetGroundOptics(): void {
  read = null;
}

/**
 * A material's reflectance at the three bands for a mature fraction m (0 fresh, 1 fully mature), linear 0..1, or null
 * when the sheet has nothing for it. With one row only, that row is drawn as it is (a fresh soil's spectrum is not
 * guessed from a mature one).
 */
export function groundReflectance(o: GroundOptics, material: string, m: number): V3 | null {
  const e = o[material];
  if (!e) return null;
  const f = e.fresh;
  const a = e.mature;
  if (f && a) {
    const t = Math.min(1, Math.max(0, m));
    return [0, 1, 2].map((i) => Math.exp((1 - t) * Math.log(f[i]!) + t * Math.log(a[i]!))) as V3;
  }
  return (f ?? a)!.slice() as V3;
}

const enc = (v: number) => 255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(Math.max(0, v), 1 / 2.4) - 0.055);

/**
 * The tiles with their measured ground colour laid on (`ground_rgb`, sRGB 0..255): bare ground whose material the
 * sheet has a row for, on every world (MATCH1, 25 Nov: one typed tan palette made Mercury, Mars, Io, Europa and Pluto
 * alike). A tile that carries `maturity` blends fresh to mature by it; one without (no weathering age from the kernel
 * yet) is drawn fully mature, as old exposed ground mostly is (fudge register: 'Ground with no weathering age drawn
 * mature'). Other samples keep the palette. The reflectance is the colour's linear value, so True's physical lighting
 * shows the soil as dark as it is.
 */
export function withGround(tiles: readonly SurfaceTile[], o: GroundOptics): SurfaceTile[] {
  // Then softened over its neighbours (natural outlines, ui-contract softenGround), so ground colour has no square patches.
  if (!Object.keys(o).length) return tiles.map((t) => softenGround(t));
  return tiles.map((t) => softenGround(withRows(t, o)));
}

function withRows(t: SurfaceTile, o: GroundOptics): SurfaceTile {
  const n = t.material.length;
    const rgb = new Float32Array(n * 3).fill(NaN);
    let any = false;
    for (let i = 0; i < n; i++) {
      if ((COVERS[t.cover[i]!] ?? "none") !== "none") continue;
      const r = groundReflectance(o, MATERIALS[t.material[i]!] ?? "", (t.maturity?.[i] ?? 255) / 255);
      if (!r) continue;
      rgb[i * 3] = enc(r[0]);
      rgb[i * 3 + 1] = enc(r[1]);
      rgb[i * 3 + 2] = enc(r[2]);
      any = true;
    }
  return any ? { ...t, ground_rgb: rgb } : t;
}
