// The Solar System as catalogued facts (top-level design 3.1, U21 part 2): the start sheet's "Open the Solar
// System" door enters SSE4's own Sol from packs/base-sol (sse.base-sol), every value with its source, as catalogued
// evidence. Nothing here is typed: the values, units and sources are the pack's. The Sun's age becomes system.age in
// seconds; each body goes under its host (planets round the Sun, the Moon round Earth); the belts and the Oort cloud
// are belts round the Sun with their inner and outer edges.
import type { Json } from "./json.ts";
import { YEAR_S } from "./intent.ts";

export interface SolValue {
  v: number | string; // a string only where unit is "substance" (a gas id on the master data sheet)
  unit: string;
  src: string;
  note?: string;
  check?: string;
}
export interface SolBody {
  id: string;
  name: string;
  kind: string;
  host: string | null;
  values: Record<string, SolValue>;
}
export interface SolRegion {
  id: string;
  name: string;
  host: string;
  inner_m: number;
  outer_m: number;
  /** A ring's normal optical depth and its particles' albedo (a region round a planet), when the sheet carries them. */
  tau?: number;
  albedo?: number;
  src: string;
  grade: string;
  note?: string;
  check?: string;
}
export interface SolSource {
  id: string;
  text: string;
  status?: string;
  doi?: string;
  url?: string;
}
export interface SolSheet {
  about: string;
  bodies: SolBody[];
  regions: SolRegion[];
  sources: SolSource[];
}

/** What the system is called on the chart's list. */
export const SOL_TITLE = "The Solar System";

/** The sheet inside a base-sol pack document, or null when the pack does not carry one. */
export function readSol(pack: unknown): SolSheet | null {
  const s = (pack as { provides?: { other?: { sol?: SolSheet } } } | null)?.provides?.other?.sol;
  return s && Array.isArray(s.bodies) && Array.isArray(s.regions) && Array.isArray(s.sources) ? s : null;
}

/** One catalogued batch: every edit from one source, so each fact keeps the source it came from. */
export interface SolBatch {
  source: string;
  edits: { address: string; field: string; value: Json }[];
}

/**
 * The Solar System's facts as batches, one per source, at addresses under `system`: the Sun at star:0, its bodies
 * at body:N in the sheet's order, a body's moons under it, the belts at belt:N. `unverified` lists the facts whose
 * value the sheet marks unverified (address#field), for the GM's honesty marks.
 */
/** SI units the kernel's facts are held in: a value printed in one of these goes in as it is. */
const SI_UNITS: ReadonlySet<string> = new Set(["1", "m", "kg", "s", "K", "W", "Pa", "m/s", "kg/m3", "rad"]);
/**
 * A value as the kernel holds it, in SI. The sheet keeps each value in the unit it was printed in; the door converts
 * by that unit. Angles printed in degrees become radians (a matter of geometry, not a measured constant). A unit the
 * door does not know stops the door rather than entering a value in the wrong unit.
 */
export function toSI(x: SolValue, what = "a value"): number {
  if (typeof x.v !== "number") throw new Error(`sol: ${what} is text, not a number`);
  if (SI_UNITS.has(x.unit)) return x.v;
  if (x.unit === "deg") return (x.v * Math.PI) / 180;
  throw new Error(`sol: ${what} is in "${x.unit}", which the door cannot convert to SI`);
}

/**
 * Units the kernel holds a fact in as it is, though they are not SI: surface water in Earth oceans, and a gas named by
 * its substance id on the master data sheet (text).
 */
const KERNEL_UNITS: ReadonlySet<string> = new Set(["earth_oceans", "substance"]);
function factValue(x: SolValue, what: string): Json {
  return KERNEL_UNITS.has(x.unit) ? x.v : toSI(x, what);
}

export function solBatches(sheet: SolSheet, system: string): { batches: SolBatch[]; addresses: Record<string, string>; unverified: string[]; names: Record<string, string> } {
  const addr: Record<string, string> = {};
  const count = new Map<string, number>();
  const next = (under: string, seg: string) => {
    const k = `${under}|${seg}`;
    const i = count.get(k) ?? 0;
    count.set(k, i + 1);
    return `${under}/${seg}:${i}`;
  };
  // Parents first: the sheet's hosts are other rows, so place rows whose host is placed until all are.
  const left = [...sheet.bodies];
  while (left.length) {
    const i = left.findIndex((b) => b.host === null || addr[b.host] !== undefined);
    if (i < 0) throw new Error(`sol: no host for ${left.map((b) => b.id).join(", ")}`);
    const b = left.splice(i, 1)[0]!;
    addr[b.id] = b.host === null ? next(system, "star") : next(addr[b.host]!.includes("/star:") ? system : addr[b.host]!, "body");
  }
  for (const r of sheet.regions) addr[r.id] = next(system, "belt");

  const by = new Map<string, SolBatch>();
  const put = (src: string, address: string, field: string, value: Json) => {
    if (!by.has(src)) by.set(src, { source: src, edits: [] });
    by.get(src)!.edits.push({ address, field, value });
  };
  const unverified: string[] = [];
  const names: Record<string, string> = {};
  for (const b of sheet.bodies) {
    const a = addr[b.id]!;
    names[a] = b.name;
    // What a body is, its name and its host are the sheet's own structure: they go with its first value's source.
    const first = Object.values(b.values)[0]?.src ?? "sse.base-sol";
    put(first, a, "name", b.name);
    put(first, a, "body.kind", b.kind);
    if (b.host !== null) put(first, a, "orbit.host", addr[b.host]!);
    else put(first, system, "name", SOL_TITLE);
    for (const [field, x] of Object.entries(b.values)) {
      if (field === "system.age_gyr") {
        // Gyr to seconds through the Julian year the time ruler uses.
        put(x.src, system, "system.age", Number(x.v) * 1e9 * Number(YEAR_S));
        continue;
      }
      put(x.src, a, field, factValue(x, `${b.id} ${field}`));
      if (x.check === "unverified") unverified.push(`${a}#${field}`);
    }
  }
  for (const r of sheet.regions) {
    const a = addr[r.id]!;
    names[a] = r.name;
    put(r.src, a, "name", r.name);
    put(r.src, a, "body.kind", "belt");
    put(r.src, a, "orbit.host", addr[r.host]!);
    put(r.src, a, "body.radius_inner", r.inner_m);
    put(r.src, a, "body.radius_outer", r.outer_m);
    put(r.src, a, "belt.shape", /oort/.test(r.id) ? "sphere" : "disc");
    // A ring (a region round a planet): what it is made of, when the sheet carries it (V10f draws it in True).
    if (typeof r.tau === "number") put(r.src, a, "ring.optical_depth", r.tau);
    if (typeof r.albedo === "number") put(r.src, a, "ring.albedo", r.albedo);
    if (r.check === "unverified") unverified.push(`${a}#body.radius_inner`);
  }
  return { batches: [...by.values()], addresses: addr, unverified, names };
}

/** A source's words for the kernel's `source` field: its citation, with the DOI or link when it has one. */
export function solSourceText(sheet: SolSheet, id: string): string {
  const s = sheet.sources.find((x) => x.id === id);
  if (!s) return `sse.base-sol: ${id}`;
  return `${s.text}${s.doi ? ` doi:${s.doi}` : s.url ? ` ${s.url}` : ""}`;
}
