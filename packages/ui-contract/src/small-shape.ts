// RW16a: nothing below the size at which a body pulls itself round is drawn as a sphere. Every view that draws a
// scene body (system True, the Body globe) asks the kernel for its lump (`small_shape`) from what the scene knows of
// it: its size, mass and day. The kernel decides whether it is round and what shape it has, from its address and the
// universe's seed, so it is the same lump on every device, at every zoom and on every desk. This file only gathers
// the ask and keeps the answers; it decides no physics.
import type { Json } from "./json.ts";
import type { SceneSubject } from "./scene.ts";
import { M_EARTH, R_EARTH_M } from "./make.ts";

/** What a view hands the renderer to draw the lump (renderer small-body.ts `SmallFigure`). */
export interface SmallShape {
  kind: string;
  axes_m: [number, number, number];
  lobes: { axes_m: [number, number, number]; offset_m: number }[];
  relief: number;
  albedo?: number;
  weathering?: number;
}

export interface ShapeAsk {
  address: string;
  diameter_km?: number;
  mass_kg?: number;
  spin_hours?: number;
}

export interface ShapeReader {
  smallShape(ask: { system: string; address: string; diameter_km?: number; mass_kg?: number; ice?: number; spin_hours?: number; a_au?: number }): Promise<{ ok: true; body: Json } | { ok: false; code: string; message: string }>;
}

/** Kinds that are never a lump: light, a place, a region, or something built. */
const NOT_BODIES = new Set(["star", "belt", "zone", "ring", "station", "ship", "construct", "megaconstruct", "region", "system", "galaxy", "cluster"]);

const num = (s: SceneSubject, field: string): number | null => {
  const f = s.facts.find((x) => x.field === field);
  return typeof f?.value === "number" && Number.isFinite(f.value) && f.value !== 0 ? f.value : null;
};

/** The ask for one scene subject: its size and mass as the scene has them, and its day. Null for what is never a lump. */
export function shapeAsk(s: SceneSubject): ShapeAsk | null {
  if (NOT_BODIES.has(s.kind) || s.ring) return null;
  const re = num(s, "radius_earth");
  const me = num(s, "mass_earth");
  if (re === null && me === null) return null;
  const day = num(s, "day_h");
  return {
    address: s.address,
    ...(re !== null ? { diameter_km: (2 * re * R_EARTH_M) / 1000 } : {}),
    ...(me !== null ? { mass_kg: me * M_EARTH } : {}),
    ...(day !== null ? { spin_hours: Math.abs(day) } : {}),
  };
}

/** The lump in a `small_shape` answer, or null when the kernel says the body is round. */
export function shapeOf(body: Json): SmallShape | null {
  const b = body as { round?: boolean; figure?: SmallShape; albedo?: number; weathering?: number };
  if (b.round !== false || !b.figure) return null;
  return { ...b.figure, ...(typeof b.albedo === "number" ? { albedo: b.albedo } : {}), ...(typeof b.weathering === "number" ? { weathering: b.weathering } : {}) };
}

// Answers kept per kernel session, by system and what was asked: asking again never changes them.
const keptBy = new WeakMap<ShapeReader, Map<string, SmallShape | null>>();

/** The lumps of every subject that has one, by address: round bodies are left out. Asks only what it has not asked. */
export async function readShapes(k: ShapeReader, system: string, subjects: readonly SceneSubject[], stillWanted: () => boolean = () => true): Promise<Map<string, SmallShape>> {
  const out = new Map<string, SmallShape>();
  let kept = keptBy.get(k);
  if (!kept) keptBy.set(k, (kept = new Map()));
  for (const s of subjects) {
    const ask = shapeAsk(s);
    if (!ask) continue;
    const key = `${system}|${JSON.stringify(ask)}`;
    if (!kept.has(key)) {
      if (!stillWanted()) break;
      const r = await k.smallShape({ system, ...ask });
      kept.set(key, r.ok ? shapeOf(r.body) : null);
    }
    const f = kept.get(key);
    if (f) out.set(s.address, f);
  }
  return out;
}

/** The lump of one subject, or null when it is round or not a body. */
export async function readShape(k: ShapeReader, system: string, s: SceneSubject): Promise<SmallShape | null> {
  return (await readShapes(k, system, [s])).get(s.address) ?? null;
}
