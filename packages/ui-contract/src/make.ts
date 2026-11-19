// Making a system (M2 screens): a V3 save read by the kernel's importer, or a
// system grown by the formation pipeline, turned into a system scene the
// chart can draw as a preview. Both kernel ops are pure, so nothing here
// touches the open universe; adopting a preview into the campaign is a later
// step (see handover/packages/V13.md).
import type { Json } from "./json.ts";
import { KM_PER_AU, type SceneFact, type SceneSubject, type SystemScene } from "./scene.ts";
import { YEAR_S } from "./intent.ts";

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
  block?: "modern" | "classic" | "classic-corrected";
  now_gyr?: number;
  star_pins?: { primary_mass?: number };
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

const M_EARTH = 5.9722e24;
const M_SUN = 1.98847e30;
const R_EARTH_M = 6_371_000;
const R_SUN_M = 695_700_000;
const AU_M = KM_PER_AU * 1000;

const round = (v: number, d = 3) => Number(v.toPrecision(d));
function f(field: string, label: string, value: number | string, unit?: string, source: SceneFact["source"] = "authored"): SceneFact {
  const v = typeof value === "number" ? round(value) : value;
  return { field, label, value: v, ...(unit ? { unit } : {}), display: unit ? `${v} ${unit}` : String(v), editable: false, source };
}
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
    return {
      address: a,
      kind,
      label: name,
      subtitle: `Imported from V3 · ${role(a)}`,
      ...(a !== primary && parent ? { parent } : {}),
      ...(a !== primary && sma !== undefined ? { orbit: { a_au: sma / AU_M, phase_deg: phase(i++) } } : {}),
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
