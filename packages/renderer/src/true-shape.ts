// A world's shape in True (TA6, audit item R4 oblateness): how much its spin flattens it, from the kernel's
// Darwin-Radau flow `sse.base.body-shape` (packs/base-physics/flows/body-shape.json) on the world's own mass, radius,
// day and moment of inertia factor from `resolved_system`. A result, never a setting: a world whose facts lack any of
// the four is drawn round, and the reason is said. Nothing here is typed in.
import { say, type Json } from "../../ui-contract/src/index.ts";

export interface TrueShape {
  /** Flattening (Re - Rp) / Re. */
  f: number;
  equatorial_radius_m: number;
  polar_radius_m: number;
  /** Plain words for the HUD note. */
  said: string;
}

/** The kernel doors this needs (a ViewSession has them). */
export interface ShapeReader {
  resolvedSystem(system: string): Promise<{ ok: true; result: { facts: Record<string, Record<string, Json>> } } | { ok: false; message: string }>;
  derive(ref: { flow: string; inputs: Record<string, { value: number; unit?: string }> }): Promise<{ ok: true; values: Record<string, { value: number }> } | { ok: false; message: string }>;
}

export const SHAPE_FLOW = "sse.base.body-shape";

/** The facts the flow reads, by its input name. */
const FACTS: Readonly<Record<string, string>> = { mass: "body.mass", radius: "body.radius", rotation_period: "body.rotation_period", inertia_factor: "body.moment_of_inertia_factor" };
const UNITS: Readonly<Record<string, string>> = { mass: "kg", radius: "m", rotation_period: "s", inertia_factor: "one" };

/** A number fact, whether stored bare or as { value }. */
const num = (v: Json | undefined): number | null => (typeof v === "number" ? v : v && typeof v === "object" && !Array.isArray(v) && typeof v.value === "number" ? v.value : null);

/** A world's spin-flattened shape, or why it is drawn round. */
export async function readTrueShape(k: ShapeReader, system: string, body: string): Promise<TrueShape | string> {
  const rs = await k.resolvedSystem(system);
  if (!rs.ok) return rs.message;
  const facts = rs.result.facts[body] ?? {};
  const inputs: Record<string, { value: number; unit: string }> = {};
  const missing: string[] = [];
  for (const [name, key] of Object.entries(FACTS)) {
    const v = num(facts[key]);
    if (v === null || !Number.isFinite(v) || v === 0) missing.push(key);
    // A world turning backwards flattens just the same.
    else inputs[name] = { value: Math.abs(v), unit: UNITS[name]! };
  }
  if (missing.length) return `drawn round: its facts lack ${missing.join(", ")}`;
  const d = await k.derive({ flow: SHAPE_FLOW, inputs });
  if (!d.ok) return `drawn round: ${d.message}`;
  const f = d.values["shape.flattening"]?.value;
  const re = d.values["shape.equatorial_radius"]?.value;
  const rp = d.values["shape.polar_radius"]?.value;
  if (!(typeof f === "number" && f >= 0 && f < 0.5 && re && rp)) return "drawn round: the shape flow gave no flattening";
  return { f, equatorial_radius_m: re, polar_radius_m: rp, said: `flattened by its spin: the pole ${say(f, "share", { class: "planet" }, { sig: 2 }).text} shorter than the equator (Darwin-Radau)` };
}

/**
 * The scale along the equator and along the pole that keeps the drawn volume: a sphere of radius 1 becomes an
 * ellipsoid with a (1 - f) = c, a² c = 1.
 */
export function flattenScale(f: number): { equator: number; pole: number } {
  const a = Math.cbrt(1 / (1 - f));
  return { equator: a, pole: a * (1 - f) };
}
