// Making a system (M2 screens): a V3 save read by the kernel's importer, or a
// system grown by the formation pipeline, turned into a system scene the
// chart can draw as a preview. Both kernel ops are pure, so nothing here
// touches the open universe; adopting a preview into the campaign is a later
// step (see handover/packages/V13.md).
import { starClassWords } from "./star-class.ts";
import type { Json } from "./json.ts";
import type { SceneFact, SceneSubject, SystemScene } from "./scene.ts";
import { YEAR_S } from "./intent.ts";
import { constant } from "./sheet.ts";
import { PICTURE_FIELDS } from "./picture.ts";
import { roundTo, sayNumber } from "./figures.ts";

// ── Kernel shapes (the parts the screens read; full types in packages/contracts) ──

export interface ImportedFact {
  value: Json;
  evidence: string;
}
export interface ImportDifference {
  address: string;
  field: string;
  name: string;
  v3: number;
  v4: number;
  relative: number;
  tolerance: number;
  via: string;
  within: boolean;
}
export interface ImportReport {
  format: string;
  source_kind: string;
  v3_app_version: string | null;
  importer_version: string;
  counts: Record<string, number>;
  differences: ImportDifference[];
  derived_not_compared: Record<string, number>;
  unclassified: Record<string, number>;
  dropped: Record<string, number>;
  presentation: Record<string, number>;
  honesty_marks: string[];
  warnings: string[];
  state_hash: string;
}
export interface V3Imported {
  type: "v3_imported";
  universe: string;
  facts: Record<string, Record<string, ImportedFact>>;
  report: ImportReport;
  reconstructed: Json[] | null;
}

export interface FormationRequest {
  seed: string;
  system?: string;
  /** A drawn star of the campaign's galaxy (its `star_cell` address) to grow from: the system takes that address and the star's own birth. */
  from_star?: string;
  block?: "modern" | "classic" | "classic-corrected";
  now_gyr?: number;
  star_pins?: { primary_mass?: number; count?: number };
  /** Stage 1's draws held at these values (sse_formation InheritancePins): a star picked from the gallery keeps its age and metals. */
  inherit_pins?: { feh?: number; age_gyr?: number };
  pins?: { worlds: { label: string; mass_earth: number; a_au: number; e?: number }[]; tries?: number };
}
export interface FormationWorld {
  address: string;
  mass_earth: number;
  radius: number;
  envelope: string;
  envelope_fraction: number;
  water_fraction: number;
  surface_water_oceans: number;
  desiccated: boolean;
  co2_bar: number;
  facts: { kind: string; unit: string; value: Json }[];
}
export interface FormationStarNow {
  phase: string;
  mass: number;
  luminosity: number;
  radius: number;
  teff: number;
}
export interface FormationEvent {
  kind?: string;
  type?: string;
  subject?: string;
  t_myr: number;
  text?: string;
}
export interface FormationRan {
  type: "formation_ran";
  system: string;
  pipeline: string;
  run: {
    block: string;
    worlds: FormationWorld[];
    events: FormationEvent[];
    notes: string[];
    growth: { events: FormationEvent[] };
    /** Stage 7's leftovers: belts, discs and clouds (F5); `around` names the world a ring or swarm belongs to. */
    leftovers?: { clouds: LeftoverCloud[] };
    history: {
      inheritance: { age_gyr: number; feh: number; fuv_g0: number };
      events: FormationEvent[];
      stars: { address: string; mass: number }[];
      slices: { t_myr: number; stars: FormationStarNow[] }[];
      manifest_hash: string;
    };
  };
  pinned?: { fit: number; matches: { how: string; pin: number; world: string }[]; removed: string[] };
  surfaces: { address: string; kind: string; t_surface_now_k: number; regime_label?: string; speculative: boolean }[];
}

// ── Shared helpers ──

export const M_EARTH = constant("M_earth");
export const M_SUN = constant("M_sun");
export const R_EARTH_M = constant("R_earth_mean");
export const R_SUN_M = constant("R_sun");
export const AU_M = constant("au");
/** The Sun's luminosity (IAU 2015 nominal), in watts. */
export const L_SUN_W = constant("L_sun");

const round = (v: number, d = 3) => roundTo(v, d);

/** One of F5's leftover clouds, as the formation run and `resolved_system` give it. */
export interface LeftoverCloud {
  address: string;
  kind: string;
  /** The world a ring or swarm belongs to; null round the star. */
  around: string | null;
  a_min_au: number;
  a_max_au: number;
  /**
   * From F5's tilt model; absent from kernels before it. The kernel thread sends "disc" (Rayleigh tilts) or "sphere"
   * (isotropic); the night-sky design (section 10) names them "ring", "thick-disc" and "shell". Both are read.
   */
  shape?: string;
  i_sigma_deg?: number | null;
}

/** A cloud's shape: the kernel's when it gives one, else the Oort cloud is a sphere and the rest are discs. */
export function cloudShape(c: Pick<LeftoverCloud, "kind" | "shape">): "disc" | "sphere" {
  if (c.shape === "sphere" || c.shape === "shell") return "sphere";
  if (c.shape === "disc" || c.shape === "ring" || c.shape === "thick-disc") return "disc";
  return c.kind === "oort" ? "sphere" : "disc";
}

/**
 * A V3 belt's shape. V3 has no shape, so a belt named for the Oort cloud, or one whose inner edge lies beyond
 * 1,000 AU (where the Galaxy's tides have stirred every orbit's tilt), is a sphere.
 */
export function v3BeltShape(name: string, band: [number, number] | undefined): "disc" | "sphere" {
  return /\boort\b/i.test(name) || (band !== undefined && band[0] > 1000) ? "sphere" : "disc";
}

/** Belts, discs and the Oort cloud round the star from F5's clouds; rings and Trojans belong to their worlds and are not drawn here. */
export function cloudSubjects(clouds: readonly LeftoverCloud[], star: string | undefined): SceneSubject[] {
  const out: SceneSubject[] = [];
  for (const c of clouds) {
    if (c.around || !(c.a_max_au > c.a_min_au) || c.kind === "scattered-disc") continue;
    const name = c.kind === "oort" ? "Oort cloud" : c.kind.replace(/-/g, " ").replace(/^./, (x) => x.toUpperCase());
    const shape = cloudShape(c);
    // A system has one Oort cloud: F5 can fill it from several stirred belts, and those reservoirs are drawn as one shell.
    const same = c.kind === "oort" ? out.find((x) => x.label === name) : undefined;
    if (same) {
      same.band_au = [Math.min(same.band_au![0], c.a_min_au), Math.max(same.band_au![1], c.a_max_au)];
      same.subtitle = `Leftovers of formation · a sphere round the system, ${roughAu(same.band_au[0])} to ${roughAu(same.band_au[1])} AU`;
      continue;
    }
    const span = `${roughAu(c.a_min_au)} to ${roughAu(c.a_max_au)} AU`;
    out.push({
      address: c.address,
      kind: "belt",
      label: name,
      subtitle: `Leftovers of formation · ${shape === "sphere" ? `a sphere round the system, ${span}` : span}`,
      ...(star ? { parent: star } : {}),
      band_au: [c.a_min_au, c.a_max_au],
      shape,
      facts: [{ field: "name", label: "Name", value: name, display: name, editable: false, source: "derived" }],
      stamps: ["generated"],
      tags: [],
    });
  }
  return out;
}

const roughAu = (a: number) => (a >= 100 ? sayNumber(a, 0, { decimals: 0 }) : sayNumber(a, 2));

/**
 * A belt's band on the chart, in AU from its star: its inner and outer edges when the save gives them at orbital scale
 * (V3 belts carry radiusInnerKm and radiusOuterKm), else a band of ±10 % about its orbit. Undefined when neither is known.
 */
export function beltBand(sma_m: number | undefined, inner_m: number | undefined, outer_m: number | undefined): [number, number] | undefined {
  if (inner_m !== undefined && outer_m !== undefined && outer_m > inner_m && outer_m > 1e9) return [inner_m / AU_M, outer_m / AU_M];
  if (sma_m !== undefined && sma_m > 0) return [(sma_m * 0.9) / AU_M, (sma_m * 1.1) / AU_M];
  return undefined;
}
function f(field: string, label: string, value: number | string, unit?: string, source: SceneFact["source"] = "authored", kernel?: SceneFact["kernel"]): SceneFact {
  const v = typeof value === "number" ? round(value) : value;
  // A value the universe holds at a kernel field can be typed over (it becomes the GM's, authored); one the physics
  // works out from others cannot.
  const editable = !!kernel && source !== "derived";
  return { field, label, value: v, ...(unit ? { unit } : {}), display: unit ? `${v} ${unit}` : String(v), editable, source, ...(kernel ? { kernel } : {}) };
}
/** The kernel field a shown value is held at, and how many SI units one shown unit is. */
const at = (field: string, per_unit = 1): SceneFact["kernel"] => ({ field, per_unit });
/** Spread phases so bodies do not stack: a deterministic step of the golden angle. */
const phase = (i: number) => (i * 137.507764) % 360;
const sizeTag = (m_earth: number) => (m_earth < 0.5 ? "size:small" : m_earth < 10 ? "size:medium" : "size:large");

// ── V3 import ──

export interface ImportedSystem {
  address: string;
  name: string;
  bodies: number;
}

/** The systems an import holds, in save order. */
export function importedSystems(r: V3Imported): ImportedSystem[] {
  const keys = Object.keys(r.facts);
  return keys
    .filter((a) => /\/sys:[^/]+$/.test(a))
    .map((a) => ({ address: a, name: String(r.facts[a]?.["v3.name"]?.value ?? a), bodies: keys.filter((k) => k.startsWith(a + "/")).length }));
}

const ROLE_KIND: Record<string, string> = { star: "star", planet: "planet", moon: "moon", belt: "belt", ring: "belt", construct: "station", barycenter: "barycentre", barycentre: "barycentre" };

/** One imported system as a scene the chart can draw. Values are the save's own (authored); honesty marks become stamps. */
export function sceneFromImport(r: V3Imported, system: string): SystemScene {
  const facts = r.facts;
  const num = (a: string, k: string) => {
    const v = facts[a]?.[k]?.value;
    return typeof v === "number" ? v : undefined;
  };
  const str = (a: string, k: string) => {
    const v = facts[a]?.[k]?.value;
    return typeof v === "string" ? v : undefined;
  };
  const inside = Object.keys(facts).filter((a) => a.startsWith(system + "/"));
  const role = (a: string) => str(a, "v3.roleHint") ?? (a.includes("/craft:") ? "construct" : "planet");
  const stars = inside.filter((a) => role(a) === "star");
  const primary = stars.find((a) => !str(a, "parent")) ?? stars[0];
  let i = 0;
  const subjects: SceneSubject[] = inside.map((a) => {
    const kind = ROLE_KIND[role(a)] ?? "planet";
    const name = str(a, "v3.name") ?? a.split("/").pop()!;
    const mass = num(a, "body.mass");
    const radius = num(a, "body.radius");
    const sma = num(a, "orbit.semi_major_axis");
    const parent = str(a, "parent") ?? str(a, "orbit.host");
    const out: SceneFact[] = [f("name", "Name", name)];
    if (kind === "star") {
      if (mass !== undefined) out.push(f("mass_sun", "Mass", mass / M_SUN, "M☉"));
      if (radius !== undefined) out.push(f("radius_sun", "Radius", radius / R_SUN_M, "R☉"));
      const t = num(a, "star.effective_temperature");
      if (t !== undefined) out.push(f("teff_k", "Surface", t, "K"));
    } else {
      if (sma !== undefined) out.push(f("a_au", "Orbit", sma / AU_M, "AU"));
      const e = num(a, "orbit.eccentricity");
      if (e !== undefined) out.push(f("eccentricity", "Eccentricity", e));
      if (mass !== undefined) out.push(f("mass_earth", "Mass", mass / M_EARTH, "M⊕"));
      if (radius !== undefined) out.push(f("radius_earth", "Radius", radius / R_EARTH_M, "R⊕"));
      const day = num(a, "body.rotation_period");
      if (day !== undefined) out.push(f("day_h", "Day", day / 3600, "h"));
    }
    const marked = r.report.honesty_marks.includes(a);
    const band = kind === "belt" && (!parent || parent === primary) ? beltBand(sma, num(a, "body.radius_inner"), num(a, "body.radius_outer")) : undefined;
    return {
      address: a,
      kind,
      label: name,
      subtitle: `Imported from V3 · ${role(a)}`,
      ...(a !== primary && parent ? { parent } : {}),
      ...(band ? { band_au: band, shape: v3BeltShape(name, band) } : a !== primary && sma !== undefined ? { orbit: { a_au: sma / AU_M, phase_deg: phase(i++) } } : {}),
      facts: out,
      stamps: ["authored", "imported", ...(marked ? ["impossible"] : [])],
      tags: mass !== undefined && kind !== "star" ? [sizeTag(mass / M_EARTH)] : [],
    };
  });
  const age = num(system, "system.age");
  const now_yr = age !== undefined ? age / Number(YEAR_S) : 4.6e9;
  const far = Math.max(1, ...subjects.filter((s) => s.orbit && s.parent === primary).map((s) => s.orbit!.a_au));
  return {
    schema: "sse.scene.system/0",
    fixture: false,
    universe: { id: r.universe.replace(/^u:/, ""), seed: str(system, "v3.seed") ?? r.universe },
    system,
    title: str(system, "v3.name") ?? "Imported system",
    subtitle: `Imported from a V3 ${r.report.source_kind}${r.report.v3_app_version ? ` (V3 ${r.report.v3_app_version})` : ""} · preview, not yet in the campaign`,
    now_yr,
    span_yr: [0, Math.max(now_yr * 2, 1e10)],
    surveyed_to_au: Math.ceil(far * 1.5),
    subjects,
    events: [],
  };
}

/** The report in a few plain lines, for the screen's summary. */
export function reportSummary(rep: ImportReport): { line: string; outside: ImportDifference[]; within: number } {
  const outside = rep.differences.filter((d) => !d.within);
  const c = rep.counts;
  const parts = [`${c.systems ?? 0} ${c.systems === 1 ? "system" : "systems"}`, `${c.bodies ?? 0} bodies`, ...(c.constructs ? [`${c.constructs} constructs`] : []), `${c.facts ?? 0} facts`];
  return { line: parts.join(", "), outside, within: rep.differences.length - outside.length };
}

// ── Formation ──

/** The slice nearest a time, in Myr since formation. */
function sliceAt(r: FormationRan, t_myr: number) {
  let best = r.run.history.slices[0];
  for (const s of r.run.history.slices) if (Math.abs(s.t_myr - t_myr) < Math.abs((best?.t_myr ?? 0) - t_myr)) best = s;
  return best;
}

export function worldWords(w: FormationWorld): string {
  const m = w.mass_earth;
  const wet = w.water_fraction > 0.05;
  const kind =
    w.envelope === "giant"
      ? m > 50
        ? "gas giant"
        : "ice giant"
      : w.envelope === "kept"
        ? wet
          ? "water-rich world under a thin hydrogen veil"
          : "world under a thin hydrogen veil"
        : wet
          ? "water-rich world"
          : w.desiccated
            ? "dry rock"
            : w.envelope === "stripped"
              ? "rocky world, its first air stripped"
              : "rocky world";
  return `${round(m, 2)} M⊕ ${kind}`;
}

/** A formation run as a scene: the star at the system's age now, each world with its mass, orbit and surface. */
export function sceneFromFormation(r: FormationRan, seed: string): SystemScene {
  const h = r.run.history;
  const age_myr = h.inheritance.age_gyr * 1000;
  const star = sliceAt(r, age_myr)?.stars[0];
  const starAddr = h.stars[0]?.address ?? `${r.system}/star:0`;
  const surf = new Map(r.surfaces.map((s) => [s.address, s]));
  const value = (w: FormationWorld, k: string) => {
    const v = w.facts.find((x) => x.kind === k)?.value;
    return typeof v === "number" ? v : undefined;
  };
  const subjects: SceneSubject[] = [
    {
      address: starAddr,
      kind: "star",
      label: "Star",
      subtitle: star ? `${star.phase.replace(/-/g, " ")} star, ${round(star.mass, 2)} M☉` : "star",
      facts: [
        f("name", "Name", "Star"),
        ...(star ? [f("mass_sun", "Mass", star.mass, "M☉", "derived"), f("luminosity_sun", "Luminosity now", star.luminosity, "L☉", "derived"), f("radius_sun", "Radius", star.radius, "R☉", "derived"), f("teff_k", "Surface", star.teff, "K", "derived")] : []),
      ],
      stamps: ["generated"],
      tags: [],
    },
  ];
  r.run.worlds.forEach((w, i) => {
    const a = value(w, "orbit.semi_major_axis");
    const s = surf.get(w.address);
    const name = `World ${i + 1}`;
    const pinned = r.pinned?.matches.find((m) => m.world === w.address);
    subjects.push({
      address: w.address,
      kind: "planet",
      label: name,
      subtitle: worldWords(w) + (s?.regime_label ? ` · ${s.regime_label.toLowerCase()}` : ""),
      parent: starAddr,
      ...(a !== undefined ? { orbit: { a_au: a / AU_M, phase_deg: phase(i) } } : {}),
      facts: [
        f("name", "Name", name),
        ...(a !== undefined ? [f("a_au", "Orbit", a / AU_M, "AU", "derived")] : []),
        ...((e) => (e !== undefined ? [f("eccentricity", "Eccentricity", e, undefined, "derived")] : []))(value(w, "orbit.eccentricity")),
        f("mass_earth", "Mass", w.mass_earth, "M⊕", "derived"),
        f("radius_earth", "Radius", w.radius, "R⊕", "derived"),
        ...(s ? [f("surface_k", "Surface", s.t_surface_now_k, "K", "derived")] : []),
      ],
      stamps: ["generated", ...(pinned ? ["authored"] : []), ...(s?.speculative ? ["speculative"] : [])],
      tags: [sizeTag(w.mass_earth)],
    });
  });
  // Belts, discs and the Oort cloud round the star (F5 leftovers).
  subjects.push(...cloudSubjects(r.run.leftovers?.clouds ?? [], starAddr));
  const events = [...h.events, ...r.run.events]
    .filter((e) => e.text)
    .map((e, i) => ({ id: `f${i}`, at_yr: e.t_myr * 1e6, label: e.text!.length > 60 ? e.text!.slice(0, 57) + "…" : e.text!, tone: "past" as const }));
  const far = Math.max(1, ...subjects.filter((s) => s.orbit).map((s) => s.orbit!.a_au));
  return {
    schema: "sse.scene.system/0",
    fixture: false,
    universe: { id: "generated", seed },
    system: r.system,
    title: `Seed "${seed}"`,
    subtitle: `Grown by the ${r.run.block} formation pipeline · preview, not yet in the campaign`,
    now_yr: age_myr * 1e6,
    span_yr: [0, Math.max(age_myr * 2e6, 1e10)],
    surveyed_to_au: Math.ceil(far * 1.5),
    subjects,
    events,
  };
}

/** Why a run made no worlds, in words, when the growth stage says so. */
export function noWorldsWhy(r: FormationRan): string | null {
  if (r.run.worlds.length) return null;
  const gone = r.run.growth.events.find((e) => (e.type ?? e.kind) === "gas-gone");
  const g0 = r.run.history.inheritance.fuv_g0;
  return gone
    ? `No worlds formed: the disc's gas was gone after ${round(gone.t_myr, 2)} Myr${g0 > 1000 ? `, stripped by a bright neighbour (far-UV field ${Math.round(g0)} G₀)` : ""}. Try another seed, or pin the star's mass.`
    : "No worlds formed in this run.";
}

// ── Adopting (V13 part 2) ──

/** `adopt`'s answer: the events written, the systems they hold, how many facts, the new state hash. */
export interface Adopted {
  type: "adopted";
  events: string[];
  systems: string[];
  facts: number;
  state_hash: string;
}
/** One line of the kernel's `systems` list. */
export interface ListedSystem {
  system: string;
  source: "v3" | "formation" | "facts" | string;
  name?: string;
  addresses: string[];
  adopted_event?: string;
}
/** A listed system's title: its own name, else the seed a grown system's address was made from. */
export function listedTitle(x: ListedSystem): string {
  if (x.name) return x.name;
  const tail = x.system.split("/").pop() ?? x.system;
  return x.source === "formation" && tail.startsWith("sys:@") ? `Seed "${tail.slice(5).replace(/-/g, " ")}"` : tail;
}
/** `resolved_system`'s answer: the system as the universe now holds it. */
export interface SystemResolved {
  type: "system_resolved";
  system: string;
  source: "formation" | "v3" | "facts" | string;
  at_s?: number;
  facts: Record<string, Record<string, Json>>;
  /** Per fact: its evidence label and the kind of event that set it (kernel 7c1ed04). */
  evidence?: Record<string, Record<string, { evidence: string; by: string }>>;
  spec?: import("./fate.ts").SystemSpec;
  surfaces: { address: string; kind: string; t_surface_now_k: number; regime_label?: string; speculative: boolean }[];
  tiles: Json[];
  refused: Json[];
  /** F5's leftover clouds for a grown system (kernel PR 64); omitted when empty and for V3 and facts systems. */
  clouds?: LeftoverCloud[];
}

/**
 * A system the universe holds, as a scene: read back with `resolved_system`
 * after `adopt` (or after a look-ahead's write-back), so the chart draws the
 * kernel's facts, not the preview it was made from. Names come from the save
 * (v3.name); a grown system's bodies are "Star", "World 1" and on, by orbit.
 */
export function sceneFromResolved(r: SystemResolved, title?: string): SystemScene {
  const facts = r.facts;
  const val = (a: string, k: string) => facts[a]?.[k];
  const num = (a: string, k: string) => (typeof val(a, k) === "number" ? (val(a, k) as number) : undefined);
  const str = (a: string, k: string) => (typeof val(a, k) === "string" ? (val(a, k) as string) : undefined);
  const system = r.system;
  const inside = Object.keys(facts).filter((a) => a.startsWith(system + "/"));
  const role = (a: string) => str(a, "v3.roleHint") ?? (str(a, "body.kind") ? str(a, "body.kind")! : /\/star:/.test(a) ? "star" : a.includes("/craft:") ? "construct" : a.includes("/cloud:") ? "belt" : "planet");
  const engulfed = (a: string) => val(a, "body.engulfed") === true;
  const bodies = inside.filter((a) => !engulfed(a) && (num(a, "body.mass") !== undefined || role(a) !== "planet"));
  const stars = bodies.filter((a) => role(a) === "star");
  const primary = stars.find((a) => !str(a, "parent") && num(a, "body.mass") !== undefined) ?? stars[0];
  const grown = r.source === "formation";
  const worldOrder = bodies.filter((a) => role(a) !== "star").sort((x, y) => (num(x, "orbit.semi_major_axis") ?? 0) - (num(y, "orbit.semi_major_axis") ?? 0));
  const surf = new Map(r.surfaces.map((s) => [s.address, s]));
  const nameOf = (a: string) => str(a, "v3.name") ?? str(a, "name") ?? (role(a) === "star" ? (stars.length > 1 ? `Star ${stars.indexOf(a) + 1}` : "Star") : `World ${worldOrder.indexOf(a) + 1}`);
  // Each value is marked by the evidence the kernel holds for it: set in the save (authored, or authored against the
  // physics), from a real catalogue, or the physics' own (resolved by formation, derived by ageing).
  const evOf = (a: string, k: string) => r.evidence?.[a]?.[k];
  const srcOf = (a: string, k: string): SceneFact["source"] => {
    const e = evOf(a, k)?.evidence;
    if (!e) return grown ? "derived" : "authored";
    return e === "authored" || e === "authored-impossible" || e === "narrative" ? "authored" : e === "catalogued" ? "catalogued" : e === "resolved" ? "resolved" : "derived";
  };
  const aged = (a: string) => Object.values(r.evidence?.[a] ?? {}).some((e) => e.by === "time.aged");
  // The save's own bookkeeping (v3.name, v3.id and so on) is authored by definition, so only physical values decide the
  // stamps, except V3's overrides and anomalies, which carry the honesty mark (authored-impossible).
  const classes = (a: string) => new Set(Object.entries(r.evidence?.[a] ?? {}).filter(([k, e]) => !k.startsWith("v3.") || e.evidence === "authored-impossible").map(([, e]) => e.evidence));
  let i = 0;
  const subjects: SceneSubject[] = bodies
    .filter((a) => a === primary || num(a, "orbit.semi_major_axis") !== undefined || role(a) === "star" || (ROLE_KIND[role(a)] === "belt" && num(a, "body.radius_outer") !== undefined))
    .map((a) => {
      const kind = ROLE_KIND[role(a)] ?? "planet";
      const name = nameOf(a);
      const mass = num(a, "body.mass");
      const radius = num(a, "body.radius");
      const sma = num(a, "orbit.semi_major_axis");
      const parent = str(a, "parent") ?? str(a, "orbit.host");
      const s = surf.get(a);
      const out: SceneFact[] = [f("name", "Name", name)];
      if (kind === "star") {
        // Its formal class as the kernel reads it at this date (sse_formation::spectral): a label, never physics.
        const sc = str(a, "star.spectral_class");
        if (sc) out.push(f("spectral", "Class", sc, undefined, "derived"));
        if (mass !== undefined) out.push(f("mass_sun", "Mass", mass / M_SUN, "M☉", srcOf(a, "body.mass"), at("body.mass", M_SUN)));
        if (radius !== undefined) out.push(f("radius_sun", "Radius", radius / R_SUN_M, "R☉", srcOf(a, "body.radius"), at("body.radius", R_SUN_M)));
        const t = num(a, "star.effective_temperature");
        if (t !== undefined) out.push(f("teff_k", "Surface", t, "K", srcOf(a, "star.effective_temperature"), at("star.effective_temperature")));
        const l = num(a, "star.luminosity");
        if (l !== undefined) out.push(f("luminosity_sun", "Luminosity", l / L_SUN_W, "L☉", srcOf(a, "star.luminosity"), at("star.luminosity", L_SUN_W)));
        const ph = str(a, "star.phase");
        if (ph) out.push(f("phase", "Phase", ph.replace(/[-_]/g, " "), undefined, srcOf(a, "star.phase")));
      } else {
        if (sma !== undefined) out.push(f("a_au", "Orbit", sma / AU_M, "AU", srcOf(a, "orbit.semi_major_axis"), at("orbit.semi_major_axis", AU_M)));
        const e = num(a, "orbit.eccentricity");
        if (e !== undefined) out.push(f("eccentricity", "Eccentricity", e, undefined, srcOf(a, "orbit.eccentricity"), at("orbit.eccentricity")));
        if (mass !== undefined) out.push(f("mass_earth", "Mass", mass / M_EARTH, "M⊕", srcOf(a, "body.mass"), at("body.mass", M_EARTH)));
        if (radius !== undefined) out.push(f("radius_earth", "Radius", radius / R_EARTH_M, "R⊕", srcOf(a, "body.radius"), at("body.radius", R_EARTH_M)));
        const day = num(a, "body.rotation_period");
        if (day !== undefined) out.push(f("day_h", "Day", day / 3600, "h", srcOf(a, "body.rotation_period"), at("body.rotation_period", 3600)));
        if (s) out.push(f("surface_k", "Surface", s.t_surface_now_k, "K", "derived"));
      }
      // The GM's own pictures and the one it opens on (views/pictures): carried for the card, never shown as facts.
      for (const k of PICTURE_FIELDS) {
        const v = facts[a]?.[k];
        if (v !== undefined && v !== null) out.push({ field: k, label: k, value: v as Json, display: "", editable: false, source: "authored" });
      }
      const band = kind === "belt" && (!parent || parent === primary) ? beltBand(sma, num(a, "body.radius_inner"), num(a, "body.radius_outer")) : undefined;
      // A belt round a world, not the star, is its ring: edges from the world's centre, and what it is made of.
      const rin = num(a, "body.radius_inner");
      const rout = num(a, "body.radius_outer");
      const ring = kind === "belt" && parent && parent !== primary && rin !== undefined && rout !== undefined && rout > rin ? { inner_m: rin, outer_m: rout, tau: num(a, "ring.optical_depth") ?? null, albedo: num(a, "ring.albedo") ?? num(a, "body.albedo") ?? null } : undefined;
      // Axial tilt in radians: the kernel's, else the catalogue's obliquity (radians once through the door; over π/2
      // is a retrograde spin, the same lean).
      const tilt = num(a, "body.axial_tilt") ?? num(a, "body.obliquity");
      return {
        address: a,
        kind,
        label: name,
        subtitle: `${grown ? "Grown" : r.source === "v3" ? "Imported from V3" : "From the universe's facts"} · ${kind === "star" && str(a, "star.spectral_class") ? starClassWords(str(a, "star.spectral_class")!, str(a, "star.class_family")) : kind}${aged(a) ? " · aged by a kept look-ahead" : ""}${s?.regime_label ? ` · ${s.regime_label.toLowerCase()}` : ""}`,
        ...(a !== primary && parent ? { parent } : {}),
        ...(band ? { band_au: band, shape: v3BeltShape(name, band) } : a !== primary && sma !== undefined ? { orbit: { a_au: sma / AU_M, phase_deg: phase(i++) } } : {}),
        ...(ring ? { ring } : {}),
        ...(tilt !== undefined && kind !== "belt" ? { tilt_rad: tilt } : {}),
        facts: out,
        stamps: r.evidence
          ? [...(grown ? ["generated"] : r.source === "v3" ? ["imported"] : []), ...(classes(a).has("authored") ? ["authored"] : []), ...(classes(a).has("catalogued") ? ["catalogued"] : []), ...(classes(a).has("authored-impossible") ? ["authored", "impossible"] : []), ...(s?.speculative ? ["speculative"] : [])].filter((x, j, all) => all.indexOf(x) === j)
          : grown ? ["generated", ...(s?.speculative ? ["speculative"] : [])] : ["authored", "imported"],
        tags: mass !== undefined && kind !== "star" ? [sizeTag(mass / M_EARTH)] : [],
      };
    });
  // A grown system's belts, discs and Oort cloud, from F5's clouds as the universe holds them.
  subjects.push(...cloudSubjects((r.clouds ?? []).filter((c) => !subjects.some((x) => x.address === c.address)), primary));
  const age = num(system, "system.age");
  const now_yr = age !== undefined ? age / Number(YEAR_S) : 4.6e9;
  const far = Math.max(1, ...subjects.filter((s) => s.orbit && s.parent === primary).map((s) => s.orbit!.a_au));
  return {
    schema: "sse.scene.system/0",
    fixture: false,
    universe: { id: system.split("/")[0]!.replace(/^u:/, ""), seed: str(system, "v3.seed") ?? system },
    system,
    title: title ?? str(system, "v3.name") ?? system.split("/").pop()!,
    subtitle: `In the campaign · ${grown ? "grown by the formation pipeline" : r.source === "v3" ? "imported from V3" : "from its facts"} · read back from the universe`,
    now_yr,
    span_yr: [0, Math.max(now_yr * 2, 1e10)],
    surveyed_to_au: Math.ceil(far * 1.5),
    subjects,
    events: [],
  };
}
