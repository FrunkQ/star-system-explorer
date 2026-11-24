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
  // A system above or below the plane is drawn lifted by its height (y up on the page), so both the foot of its
  // stalk and the system itself must fit.
  const ys = placed.flatMap((p) => [p.position.y, p.position.y + p.position.z]);
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

/** A link the record holds between two places (kernel `links_in`, HW1), as the map reads it. */
export interface RecordLink {
  id: string;
  a: string;
  b: string;
  name?: string;
  kind?: string;
  one_way?: boolean;
}

/** A route on the map: two of the campaign's systems joined by one or more links between places in them. */
export interface SystemRoute {
  a: string;
  b: string;
  /** The links' own names or kinds, in the order the record gave them ("jump-2 lane", "the Spinward Gate"). */
  words: string[];
  /** Every link one way from a to b. */
  one_way: boolean;
  links: string[];
}

/** The system an address sits in: the longest system address that is it or holds it; null when none does. */
export function systemOf(address: string, systems: readonly string[]): string | null {
  let best: string | null = null;
  for (const s of systems) if (s && (address === s || address.startsWith(s + "/")) && (!best || s.length > best.length)) best = s;
  return best;
}

/**
 * Routes between the campaign's systems from the record's links: a link between places in two different systems is a
 * route between them (a link inside one system, or to a place outside every system, is not drawn here). Links between
 * the same pair join into one route; it is one way only if every one of them runs the same way.
 */
export function systemRoutes(links: readonly RecordLink[], systems: readonly string[]): SystemRoute[] {
  const out = new Map<string, SystemRoute & { dirs: Set<string> }>();
  for (const l of links) {
    const a = systemOf(l.a, systems);
    const b = systemOf(l.b, systems);
    if (!a || !b || a === b) continue;
    const [x, y] = a < b ? [a, b] : [b, a];
    const key = `${x}\u0000${y}`;
    const r = out.get(key) ?? { a: x, b: y, words: [], one_way: true, links: [], dirs: new Set<string>() };
    r.links.push(l.id);
    const w = l.name || l.kind;
    if (w && !r.words.includes(w)) r.words.push(w);
    r.dirs.add(l.one_way ? (a === x ? "ab" : "ba") : "both");
    out.set(key, r);
  }
  return [...out.values()].map(({ dirs, ...r }) => {
    const one = dirs.size === 1 && !dirs.has("both");
    return one && dirs.has("ba") ? { ...r, a: r.b, b: r.a, one_way: true } : { ...r, one_way: one };
  });
}

/** Grid lines for a square grid every `step` drawing units across a square drawing `size` wide, centred on `mid`. */
export function gridLines(size: number, step: number, mid = size / 2): number[] {
  if (!(step > 0) || size / step > 200) return [];
  const out: number[] = [];
  for (let v = mid - Math.floor(mid / step) * step; v <= size; v += step) out.push(v);
  return out;
}
