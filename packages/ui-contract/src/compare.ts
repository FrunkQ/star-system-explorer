// Size comparison (V14): the pure parts. Items come from scene subjects
// (any system in the universe, by their radius facts) or from a reference
// document (one of each standard planet and star type, measured radii with
// references). The lineup is laid out on one baseline at one true scale; a
// body far larger than the rest is drawn as a limb (an arc entering from the
// left, as on a poster), and anything under a pixel is marked rather than
// hidden. Nothing here computes physics: a subject with no radius fact is
// left out and says so.
import type { SceneSubject } from "./scene.ts";
import { constant } from "./sheet.ts";

export const R_EARTH_KM = constant("R_earth_mean") / 1000;
export const R_JUPITER_KM = constant("R_jup_mean") / 1000;
export const R_SUN_KM = constant("R_sun") / 1000;

export interface ReferenceObject {
  id: string;
  group: string;
  label: string;
  type: string;
  kind: string;
  tags: string[];
  radius_km: number;
  teff_k?: number;
  colour?: string;
  /** A type's typical size, not one measured object. */
  typical?: boolean;
  ref: { cite: string; url: string };
}

export interface ReferenceDoc {
  schema: "sse.reference-objects/0";
  id: string;
  about: string;
  groups: { id: string; label: string }[];
  objects: ReferenceObject[];
}

/** One thing in the lineup. */
export interface CompareItem {
  /** Unique in the lineup: "ref:<id>" or the subject's address. */
  key: string;
  label: string;
  kind: string;
  tags: string[];
  radius_km: number;
  /** Where it comes from: a system's title, or "Reference". */
  where: string;
  /** What the radius rests on: the fact's evidence ("set", "resolved", ...) or "reference". */
  basis: string;
  type?: string;
  teff_k?: number;
  colour?: string;
  ref?: { cite: string; url: string };
}

/** Problems with a reference document, as sentences; empty when it is usable. */
export function checkReferences(doc: unknown): string[] {
  const d = doc as Partial<ReferenceDoc> | null;
  const out: string[] = [];
  if (!d || d.schema !== "sse.reference-objects/0") return ["not an sse.reference-objects/0 document"];
  const groups = new Set((d.groups ?? []).map((g) => g.id));
  const ids = new Set<string>();
  for (const o of d.objects ?? []) {
    if (ids.has(o.id)) out.push(`${o.id}: listed twice`);
    ids.add(o.id);
    if (!groups.has(o.group)) out.push(`${o.id}: unknown group ${o.group}`);
    if (!(o.radius_km > 0) || !Number.isFinite(o.radius_km)) out.push(`${o.id}: radius must be a positive number of km`);
    if (!o.ref?.url?.startsWith("https://")) out.push(`${o.id}: needs a reference with a link`);
  }
  return out;
}

export function itemFromReference(o: ReferenceObject): CompareItem {
  return { key: `ref:${o.id}`, label: o.label, kind: o.kind, tags: o.tags, radius_km: o.radius_km, where: "Reference", basis: o.typical ? "typical" : "reference", type: o.type, teff_k: o.teff_k, colour: o.colour, ref: o.ref };
}

/** A subject's radius in km from its facts (radius_earth or radius_sun), or null: the view never guesses one. */
export function radiusKm(s: SceneSubject): { km: number; basis: string } | null {
  const num = (f: string) => {
    const x = s.facts.find((y) => y.field === f);
    return x && typeof x.value === "number" && x.value > 0 ? { v: x.value, basis: x.source } : null;
  };
  const e = num("radius_earth");
  if (e) return { km: e.v * R_EARTH_KM, basis: e.basis };
  const r = num("radius_sun");
  if (r) return { km: r.v * R_SUN_KM, basis: r.basis };
  return null;
}

export function itemFromSubject(s: SceneSubject, where: string): CompareItem | null {
  const r = radiusKm(s);
  if (!r) return null;
  const t = s.facts.find((f) => f.field === "teff_k")?.value;
  return { key: s.address, label: s.label, kind: s.kind, tags: s.tags, radius_km: r.km, where, basis: r.basis, type: s.subtitle, teff_k: typeof t === "number" ? t : undefined };
}

/** A star's disc colour from its surface temperature: presentation only, a rough blackbody ramp. */
export function starColour(teff_k: number | undefined): string {
  if (teff_k === undefined) return "#ffd27a";
  const stops: [number, [number, number, number]][] = [
    [2500, [255, 140, 70]],
    [3500, [255, 180, 110]],
    [5000, [255, 220, 170]],
    [5800, [255, 240, 215]],
    [7500, [240, 240, 255]],
    [10000, [200, 215, 255]],
    [30000, [160, 185, 255]],
  ];
  const t = Math.min(30000, Math.max(2500, teff_k));
  let i = 0;
  while (i < stops.length - 2 && t > stops[i + 1]![0]) i++;
  const [t0, c0] = stops[i]!;
  const [t1, c1] = stops[i + 1]!;
  const k = (t - t0) / (t1 - t0);
  const c = c0.map((v, j) => Math.round(v + (c1[j]! - v) * k));
  return `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
}

export interface Placed {
  item: CompareItem;
  /** Centre x and radius in px; the disc sits on the baseline. */
  x: number;
  r: number;
  /** Drawn at a minimum size because it is under a pixel at this scale. */
  tiny: boolean;
}

export interface Lineup {
  placed: Placed[];
  /** The body drawn as a limb (an arc from the left edge), when one dwarfs the rest. */
  limb: Placed | null;
  /** km per px. */
  km_per_px: number;
  /** Total width the lineup needs, px. */
  width: number;
  mode: "true" | "log";
}

/**
 * Lay the items out left to right on one baseline. In "true" mode one scale
 * holds for all: the tallest disc fits the height, the row fits `maxWidth`
 * where it can (else it scrolls). When the largest is more than `limbRatio`
 * times the next, it becomes a limb behind the row and the scale is set by
 * the rest. In "log" mode radius grows with log10(km); the view says it is
 * not to scale. `slot` is the least width each item takes (room for its
 * label), so small bodies' labels never overprint.
 */
export function layoutLineup(items: readonly CompareItem[], height: number, maxWidth: number, mode: "true" | "log" = "true", limbRatio = 5, slot = 0): Lineup {
  const gap = 18;
  const minR = 1.5;
  if (!items.length) return { placed: [], limb: null, km_per_px: 1, width: 0, mode };
  if (mode === "log") {
    const lo = Math.min(...items.map((i) => i.radius_km));
    const hi = Math.max(...items.map((i) => i.radius_km));
    const span = Math.max(1e-9, Math.log10(hi) - Math.log10(lo));
    // Radius from 4 px (smallest) to half the height (largest), less if the row would need more than twice the width.
    const raw = (km: number) => 4 + ((Math.log10(km) - Math.log10(lo)) / span) * (height / 2 - 4);
    const need = items.reduce((s, i) => s + Math.max(2 * raw(i.radius_km), slot) + gap, gap);
    const k = need > 1.25 * maxWidth ? Math.max(0.4, (1.25 * maxWidth) / need) : 1;
    let x = gap;
    const placed = items.map((item) => {
      const r = Math.max(3, raw(item.radius_km) * k);
      const wd = Math.max(2 * r, slot);
      const p = { item, x: x + wd / 2, r, tiny: false };
      x += wd + gap;
      return p;
    });
    return { placed, limb: null, km_per_px: NaN, width: x, mode };
  }
  const sorted = [...items].sort((a, b) => b.radius_km - a.radius_km);
  const big = sorted[0]!;
  const next = sorted[1];
  const limbItem = next && big.radius_km > limbRatio * next.radius_km ? big : null;
  const row = limbItem ? items.filter((i) => i !== limbItem) : [...items];
  const rowMax = Math.max(...row.map((i) => i.radius_km));
  const limbBand = limbItem ? Math.min(140, maxWidth * 0.18) : 0;
  const sumD = row.reduce((s, i) => s + 2 * i.radius_km, 0);
  const avail = Math.max(60, maxWidth - limbBand - gap * (row.length + 1));
  // Fit the row's width: small bodies take their slot whatever the scale, so solve for the rest.
  let kmPerPx = Math.max((2 * rowMax) / height, sumD / avail);
  for (let k = 0; k < 4; k++) {
    const fixed = row.filter((i) => (2 * i.radius_km) / kmPerPx < slot);
    const flex = row.filter((i) => (2 * i.radius_km) / kmPerPx >= slot);
    const room = avail - fixed.length * slot;
    const need = flex.reduce((s, i) => s + 2 * i.radius_km, 0);
    if (room <= 0 || need <= 0) break;
    kmPerPx = Math.max((2 * rowMax) / height, need / room);
  }
  let x = limbBand + gap;
  const placed = row.map((item) => {
    const true_r = item.radius_km / kmPerPx;
    const r = Math.max(minR, true_r);
    const wd = Math.max(2 * r, slot);
    const p = { item, x: x + wd / 2, r, tiny: true_r < minR };
    x += wd + gap;
    return p;
  });
  // The limb: its true radius at this scale, its centre left of the view so its edge shows in the band.
  const limb = limbItem ? { item: limbItem, x: limbBand - limbItem.radius_km / kmPerPx, r: limbItem.radius_km / kmPerPx, tiny: false } : null;
  return { placed, limb, km_per_px: kmPerPx, width: x, mode };
}

/** "11.2 × Earth", "0.27 × Earth", "9.7 × Jupiter", "764 × Sun": the most telling yardstick for a radius. */
export function sizeWords(km: number): string {
  const f = (x: number) => (x >= 100 ? Math.round(x).toLocaleString("en-GB") : x >= 10 ? x.toFixed(1) : x >= 0.1 ? x.toFixed(2) : x.toPrecision(2));
  if (km >= 0.5 * R_SUN_KM) return `${f(km / R_SUN_KM)} × Sun`;
  if (km >= 2.5 * R_EARTH_KM && km < 3 * R_JUPITER_KM) return `${f(km / R_JUPITER_KM)} × Jupiter`;
  if (km >= 3 * R_JUPITER_KM) return `${f(km / R_SUN_KM)} × Sun`;
  return `${f(km / R_EARTH_KM)} × Earth`;
}

/** A round scale-bar length (1, 2 or 5 × 10^n km) about `targetPx` long at this scale. */
export function scaleBar(kmPerPx: number, targetPx = 120): { km: number; px: number } {
  const raw = kmPerPx * targetPx;
  const p = Math.pow(10, Math.floor(Math.log10(raw)));
  const m = raw / p;
  const step = (m >= 5 ? 5 : m >= 2 ? 2 : 1) * p;
  return { km: step, px: step / kmPerPx };
}

export function formatKmLength(km: number): string {
  if (km >= 1e6) return `${(km / 1e6).toLocaleString("en-GB", { maximumFractionDigits: 2 })} million km`;
  return `${km.toLocaleString("en-GB", { maximumFractionDigits: km < 10 ? 1 : 0 })} km`;
}
