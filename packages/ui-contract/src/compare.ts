// Size comparison (V14): the pure parts. Items come from scene subjects
// (any system in the universe, by their radius facts) or from a reference
// document (one of each standard planet and star type, measured radii with
// references). The lineup is laid out on one baseline at one true scale; a
// body far larger than the rest is drawn as a limb (an arc entering from the
// left, as on a poster), and anything under a pixel is marked rather than
// hidden. Nothing here computes physics: a subject with no radius fact is
// left out and says so.
import type { SceneSubject } from "./scene.ts";
import { constant, numberOf, sheetUnitFactor, type SheetRef } from "./sheet.ts";

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
  /** km, or a sheet reference (`{"sheet": "constants/R_sun", "in": "km"}`) for an object the sheet holds. */
  radius_km: number | SheetRef;
  teff_k?: number | SheetRef;
  /** Its day, hours (a sheet reference for the Sun's); a Solar System body's comes from base-sol instead. */
  day_h?: number | SheetRef;
  /** A Solar System body: its row in the base-sol pack, which holds its measured day, tilt and rings. */
  sol?: string;
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
  /** Its axial tilt to its orbit, radians, when the universe holds it (above a right angle it turns backwards). */
  tilt_rad?: number;
  /** Its day (one turn on its axis), hours, when known; negative turns backwards. */
  day_h?: number;
  /** Its rings, edges from its centre in km, with their depth and brightness when known (the 3D lineups draw them). */
  rings?: { inner_km: number; outer_km: number; tau: number | null; albedo: number | null }[];
}

/** How far out a body reaches, its rings included, km: the room it needs in a lineup that draws its rings. */
export const reachKm = (i: CompareItem): number => Math.max(i.radius_km, ...(i.rings ?? []).map((r) => r.outer_km));

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
    let r = NaN;
    try {
      r = numberOf(o.radius_km);
    } catch {
      /* an unknown sheet row reads as no radius */
    }
    if (!(r > 0) || !Number.isFinite(r)) out.push(`${o.id}: radius must be a positive number of km`);
    if (!o.ref?.url?.startsWith("https://")) out.push(`${o.id}: needs a reference with a link`);
  }
  return out;
}

/** The base-sol pack's Solar System, as far as the size comparison reads it: each body's values and its rings. */
export interface SolDoc {
  bodies: { id: string; values: Record<string, { v: number; unit: string }> }[];
  regions: { id: string; host: string | null; kind?: string; inner_m?: number; outer_m?: number; optical_depth?: number; albedo?: number }[];
}

export function itemFromReference(o: ReferenceObject, sol?: SolDoc | null): CompareItem {
  const i: CompareItem = { key: `ref:${o.id}`, label: o.label, kind: o.kind, tags: o.tags, radius_km: numberOf(o.radius_km), where: "Reference", basis: o.typical ? "typical" : "reference", type: o.type, teff_k: o.teff_k === undefined ? undefined : numberOf(o.teff_k), colour: o.colour, ref: o.ref };
  if (o.day_h !== undefined) i.day_h = numberOf(o.day_h);
  const b = o.sol ? sol?.bodies.find((x) => x.id === o.sol) : undefined;
  if (b) {
    const day = b.values["body.rotation_period"];
    const tilt = b.values["body.obliquity"];
    if (day && day.unit === "s" && day.v) i.day_h = day.v / sheetUnitFactor("h");
    if (tilt && tilt.unit === "deg") i.tilt_rad = (tilt.v * Math.PI) / 180;
    const rings = sol!.regions.filter((r) => r.host === b.id && r.kind === "ring" && r.inner_m && r.outer_m && r.outer_m > r.inner_m);
    if (rings.length) i.rings = rings.map((r) => ({ inner_km: r.inner_m! / 1000, outer_km: r.outer_m! / 1000, tau: r.optical_depth ?? null, albedo: r.albedo ?? null }));
  }
  return i;
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

/**
 * A subject as a lineup item, or null when it has no radius. `all`, the subjects of its scene, gives it its rings (a
 * belt whose host is this body).
 */
export function itemFromSubject(s: SceneSubject, where: string, all: readonly SceneSubject[] = []): CompareItem | null {
  const r = radiusKm(s);
  if (!r) return null;
  const t = s.facts.find((f) => f.field === "teff_k")?.value;
  const day = s.facts.find((f) => f.field === "day_h")?.value;
  const rings = all.filter((x) => x.ring && x.parent === s.address && x.ring.outer_m > x.ring.inner_m).map((x) => ({ inner_km: x.ring!.inner_m / 1000, outer_km: x.ring!.outer_m / 1000, tau: x.ring!.tau, albedo: x.ring!.albedo }));
  return {
    key: s.address,
    label: s.label,
    kind: s.kind,
    tags: s.tags,
    radius_km: r.km,
    where,
    basis: r.basis,
    type: s.subtitle,
    teff_k: typeof t === "number" ? t : undefined,
    ...(s.tilt_rad !== undefined ? { tilt_rad: s.tilt_rad } : {}),
    ...(typeof day === "number" && day !== 0 ? { day_h: day } : {}),
    ...(rings.length ? { rings } : {}),
  };
}

/** A star's disc colour from its surface temperature: presentation only, a rough blackbody ramp. */
export function starColour(teff_k: number | undefined): string {
  if (teff_k === undefined) return "#ffd27a";
  const stops: [number, [number, number, number]][] = [
    [2500, [255, 140, 70]],
    [3500, [255, 180, 110]],
    [5000, [255, 220, 170]],
    [5800, [255, 240, 215]], // not-sheet: an RGB colour stop
    [7500, [240, 240, 255]],
    [10000, [200, 215, 255]], // not-sheet: an RGB colour stop
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
  /** The least width each item took: the asked slot, narrowed when the row would not otherwise fit (labels then stagger). */
  slot: number;
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
export function layoutLineup(items: readonly CompareItem[], height: number, maxWidth: number, mode: "true" | "log" = "true", limbRatio = 5, askedSlot = 0, withRings = false): Lineup {
  // Where rings are drawn, a ringed body takes the room of its rings, at the same scale as its globe.
  const reach = (i: CompareItem) => (withRings ? reachKm(i) / i.radius_km : 1);
  const minR = 1.5;
  if (!items.length) return { placed: [], limb: null, km_per_px: 1, width: 0, mode, slot: askedSlot };
  // A crowded row narrows its gaps and slots (to half the slot room, the rest for the large bodies) before it would
  // need a scroll: a whole system's lineup fits the view, its labels staggering in two rows.
  const crowd = (n: number, room: number) => {
    const g = n * (askedSlot + 18) + 18 > room ? Math.max(6, Math.min(18, room / (8 * (n + 1)))) : 18;
    const s = Math.min(askedSlot, Math.max(28, (0.6 * (room - g * (n + 1))) / Math.max(1, n)));
    return { gap: g, slot: s };
  };
  if (mode === "log") {
    const { gap, slot } = crowd(items.length, maxWidth);
    const lo = Math.min(...items.map((i) => i.radius_km));
    const hi = Math.max(...items.map((i) => i.radius_km));
    const span = Math.max(1e-9, Math.log10(hi) - Math.log10(lo));
    // Radius from 4 px (smallest) to half the height (largest), less if the row would not fit the width.
    const raw = (km: number) => 4 + ((Math.log10(km) - Math.log10(lo)) / span) * (height / 2 - 4);
    // Shrunk to fit the width: slots and gaps hold, so solve for the discs wider than their slot (a few passes).
    const disc = (i: CompareItem, k: number) => 2 * Math.max(3, raw(i.radius_km) * k) * reach(i);
    let k = 1;
    for (let pass = 0; pass < 6; pass++) {
      const need = items.reduce((s, i) => s + Math.max(disc(i, k), slot) + gap, gap);
      if (need <= maxWidth) break;
      const wide = items.filter((i) => disc(i, k) > slot).reduce((s, i) => s + disc(i, k), 0);
      if (wide <= 0) break;
      k = Math.max(0.15, k * Math.min(0.99, Math.max(0.15, (maxWidth - (need - wide)) / wide)));
    }
    let x = gap;
    const placed = items.map((item) => {
      const r = Math.max(3, raw(item.radius_km) * k);
      const wd = Math.max(2 * r * reach(item), slot);
      const p = { item, x: x + wd / 2, r, tiny: false };
      x += wd + gap;
      return p;
    });
    return { placed, limb: null, km_per_px: NaN, width: x, mode, slot };
  }
  const sorted = [...items].sort((a, b) => b.radius_km - a.radius_km);
  const big = sorted[0]!;
  const next = sorted[1];
  const limbItem = next && big.radius_km > limbRatio * next.radius_km ? big : null;
  const row = limbItem ? items.filter((i) => i !== limbItem) : [...items];
  const rowMax = Math.max(...row.map((i) => i.radius_km));
  const limbBand = limbItem ? Math.min(140, maxWidth * 0.18) : 0;
  const { gap, slot } = crowd(row.length, maxWidth - limbBand);
  const sumD = row.reduce((s, i) => s + 2 * i.radius_km * reach(i), 0);
  const avail = Math.max(60, maxWidth - limbBand - gap * (row.length + 1));
  // Fit the row's width: small bodies take their slot whatever the scale, so solve for the rest.
  let kmPerPx = Math.max((2 * rowMax) / height, sumD / avail);
  for (let k = 0; k < 4; k++) {
    const fixed = row.filter((i) => (2 * i.radius_km * reach(i)) / kmPerPx < slot);
    const flex = row.filter((i) => (2 * i.radius_km * reach(i)) / kmPerPx >= slot);
    const room = avail - fixed.length * slot;
    const need = flex.reduce((s, i) => s + 2 * i.radius_km * reach(i), 0);
    if (room <= 0 || need <= 0) break;
    kmPerPx = Math.max((2 * rowMax) / height, need / room);
  }
  let x = limbBand + gap;
  const placed = row.map((item) => {
    const true_r = item.radius_km / kmPerPx;
    const r = Math.max(minR, true_r);
    const wd = Math.max(2 * r * reach(item), slot);
    const p = { item, x: x + wd / 2, r, tiny: true_r < minR };
    x += wd + gap;
    return p;
  });
  // The limb: its true radius at this scale, its centre left of the view so its edge shows in the band.
  const limb = limbItem ? { item: limbItem, x: limbBand - limbItem.radius_km / kmPerPx, r: limbItem.radius_km / kmPerPx, tiny: false } : null;
  return { placed, limb, km_per_px: kmPerPx, width: x, mode, slot };
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
