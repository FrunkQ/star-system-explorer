// Your systems (V3 gaps 17 and 39, U6 part 2b): the campaign's own systems on one map, each saying what it holds on
// hover. V3's home screen was this map; SSE4 keeps it one step above a system. Part 1 places a system where its own
// data puts it (a V3 starmap's `position`, kept by the importer as `system.position_map_units`); a system with no
// place yet is listed beside the map, never put somewhere made up. Placing systems in the galaxy's own star cells is
// the kernel's G7 step. Presentation only: the kernel's facts, read back with `resolved_system` (no tiles).
import type { ListedSystem, SystemResolved } from "./make.ts";
import { listedTitle, sceneFromResolved } from "./make.ts";
import type { SceneSubject } from "./scene.ts";
import { systemSummary, type SystemSummary } from "./system-summary.ts";

export interface MapPoint {
  x: number;
  y: number;
  z: number;
}

/** One system of the campaign, as the map and the picker read it. */
export interface CampaignSystem {
  system: string;
  title: string;
  source: string;
  /** Where its own data puts it, in the map's units; null when nothing does. */
  position: MapPoint | null;
  /** The map's unit as the save names it (V3's starmap `distanceUnit`: "ly", "pc" or a made-up one), when it does. */
  unit: string | null;
  subjects: SceneSubject[];
  summary: SystemSummary;
}

/** A point from a fact value: {x, y, z} or [x, y, z]; z defaults to 0; anything else is no place. */
export function mapPoint(v: unknown): MapPoint | null {
  const ok = (n: unknown) => typeof n === "number" && Number.isFinite(n);
  if (Array.isArray(v) && v.length >= 2 && ok(v[0]) && ok(v[1])) return { x: v[0], y: v[1], z: ok(v[2]) ? v[2] : 0 };
  if (v && typeof v === "object") {
    const o = v as Record<string, unknown>;
    if (ok(o.x) && ok(o.y)) return { x: o.x as number, y: o.y as number, z: ok(o.z) ? (o.z as number) : 0 };
  }
  return null;
}

/** The starmap's distance unit, from whichever address in the read-back carries V3's `distanceUnit`. */
export function mapUnit(r: Pick<SystemResolved, "facts">): string | null {
  for (const f of Object.values(r.facts)) {
    const u = f?.["v3.distanceUnit"];
    if (typeof u === "string" && u.trim()) return u.trim();
  }
  return null;
}

/** A system read back from the universe as the map holds it. */
export function campaignSystem(r: SystemResolved, title?: string, source?: string): CampaignSystem {
  const scene = sceneFromResolved(r, title);
  return {
    system: r.system,
    title: title ?? scene.title,
    source: source ?? r.source,
    position: mapPoint(r.facts[r.system]?.["system.position_map_units"]),
    unit: mapUnit(r),
    subjects: scene.subjects,
    summary: systemSummary(scene.subjects),
  };
}

/** What the loader needs of a session: the list of systems and one read back each. */
export interface SystemsReader {
  listSystems(): Promise<{ ok: true; systems: ListedSystem[] } | { ok: false; code: string; message: string }>;
  resolvedSystem(system: string): Promise<{ ok: true; result: SystemResolved } | { ok: false; code: string; message: string }>;
}

/**
 * The campaign's systems with what each holds and where its data puts it: one `systems` list, then one read back per
 * system (no tiles, so a V3 system is light; a grown one re-runs formation once and the kernel caches it). A system
 * that cannot be read back is kept by name with nothing counted. The picker and the map share this loader.
 */
export async function readCampaignSystems(s: SystemsReader): Promise<{ ok: true; systems: CampaignSystem[] } | { ok: false; message: string }> {
  const l = await s.listSystems();
  if (!l.ok) return { ok: false, message: l.message };
  const out = await Promise.all(
    l.systems.map(async (x): Promise<CampaignSystem> => {
      const title = listedTitle(x);
      const r = await s.resolvedSystem(x.system);
      if (!r.ok) return { system: x.system, title, source: x.source, position: null, unit: null, subjects: [], summary: systemSummary([]) };
      return campaignSystem(r.result, title, x.source);
    }),
  );
  return { ok: true, systems: out };
}

export interface PlacedMark {
  system: string;
  /** Map position in the drawing's units (0..size), y up on the page. */
  px: number;
  py: number;
  /** Height above (+) or below (−) the map's plane, in the drawing's units, for a depth stalk. */
  pz: number;
}

/**
 * Fit the placed systems into a square drawing `size` across with a margin, seen from above (x across, y up). One
 * scale for both axes, so distances read true; a lone system sits in the middle. Returns the marks and the map units
 * per drawing unit (for a scale bar).
 */
export function layoutSystems(list: readonly Pick<CampaignSystem, "system" | "position">[], size: number, margin = 40): { marks: PlacedMark[]; unitsPerPx: number } {
  const placed = list.filter((x): x is { system: string; position: MapPoint } => x.position !== null);
  if (!placed.length) return { marks: [], unitsPerPx: 1 };
  const xs = placed.map((p) => p.position.x);
  const ys = placed.map((p) => p.position.y);
  const [x0, x1, y0, y1] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const span = Math.max(x1 - x0, y1 - y0);
  const inner = size - 2 * margin;
  const k = span > 0 ? inner / span : 1;
  const cx = (x0 + x1) / 2;
  const cy = (y0 + y1) / 2;
  return {
    marks: placed.map((p) => ({ system: p.system, px: size / 2 + (p.position.x - cx) * k, py: size / 2 - (p.position.y - cy) * k, pz: p.position.z * k })),
    unitsPerPx: span > 0 ? span / inner : 1,
  };
}

/** A round length for a scale bar about `target` drawing units long: 1, 2 or 5 times a power of ten map units. */
export function mapScaleBar(unitsPerPx: number, target = 100): { units: number; px: number } {
  const want = unitsPerPx * target;
  if (!(want > 0) || !Number.isFinite(want)) return { units: 1, px: 1 / (unitsPerPx || 1) };
  const p = 10 ** Math.floor(Math.log10(want));
  const units = [1, 2, 5, 10].map((m) => m * p).reduce((a, b) => (Math.abs(b - want) < Math.abs(a - want) ? b : a));
  return { units, px: units / unitsPerPx };
}
