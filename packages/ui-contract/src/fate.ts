// Looking ahead (U4 part 2, T2 through the kernel API): the system the chart
// shows, as the ageing model's SystemSpec; the kernel ages it on a branch of
// its own; each snapshot comes back as a scene the chart can draw. The main
// line does not move until the GM keeps the branch.
import { primaryOf, type SceneFact, type SceneEvent, type SystemScene } from "./scene.ts";

export interface PlanetSpec {
  name: string;
  address: string;
  mass_me: number;
  a_au: number;
  e: number;
  i_deg: number;
  node_deg: number;
  varpi_deg: number;
  mean_longitude_deg: number;
  oceans?: boolean;
}
export interface SystemSpec {
  name: string;
  system: string;
  star_address: string;
  star_mass: number;
  z: number;
  age_myr: number;
  birth_year: number;
  planets: PlanetSpec[];
  sources?: string;
}
export interface FateEvent {
  kind: string;
  age_myr: number;
  year: number;
  subjects: string[];
  why: string;
  level: string;
  plausible_future: boolean;
  numbers: Record<string, number>;
}
export interface FateStar {
  phase: string;
  luminosity: number;
  radius_rsun: number;
  teff: number;
  mass: number;
  colour: string;
  rgb: [number, number, number];
}
export interface FatePlanet {
  name: string;
  a_au: number;
  e: number;
  i_deg: number;
  engulfed: boolean;
}
export interface FateSnapshot {
  age_myr: number;
  year: number;
  star: FateStar;
  planets: FatePlanet[];
}
export interface LookedAhead {
  type: "looked_ahead";
  branch: string;
  label: string;
  system: string;
  now_myr: number;
  target_myr: number;
  birth_year: number;
  horizon_myr: number;
  events: FateEvent[];
  snapshots: FateSnapshot[];
  work_units: number;
}

/** Hurley, Pols and Tout (2000) fits are calibrated with the Sun at Z = 0.02. */
export const Z_SUN = 0.02;

const num = (facts: readonly SceneFact[], field: string) => {
  const v = facts.find((f) => f.field === field)?.value;
  return typeof v === "number" ? v : undefined;
};

/**
 * The scene as the ageing model reads it, and what had to be assumed. The
 * chart's ruler counts years from the star's birth, so birth_year is 0.
 * Planets (bodies orbiting the star itself) only: moons ride with their planet.
 */
export function specFromScene(scene: SystemScene): { spec: SystemSpec; assumed: string[] } | { error: string } {
  const star = primaryOf(scene);
  if (!star) return { error: "no star in this system" };
  const mass = num(star.facts, "mass_sun");
  if (mass === undefined) return { error: `${star.label} has no mass` };
  const assumed: string[] = [];
  const feh = num(star.facts, "metallicity");
  // [Fe/H] = log10(Z / Z_sun), taking metals to scale together (no alpha enhancement).
  const z = feh === undefined ? Z_SUN : Z_SUN * 10 ** feh;
  if (feh === undefined) assumed.push("solar metallicity (no [Fe/H] given)");
  const planets: PlanetSpec[] = [];
  for (const s of scene.subjects) {
    if (s.kind !== "planet" || !s.orbit || (s.parent && s.parent !== star.address)) continue;
    const m = num(s.facts, "mass_earth");
    if (m === undefined) {
      assumed.push(`${s.label} left out (no mass)`);
      continue;
    }
    planets.push({
      name: s.label,
      address: s.address,
      mass_me: m,
      a_au: s.orbit.a_au,
      e: num(s.facts, "eccentricity") ?? 0,
      i_deg: 0,
      node_deg: 0,
      varpi_deg: 0,
      mean_longitude_deg: s.orbit.phase_deg,
      oceans: s.tags.includes("class:planet/ocean") || (num(s.facts, "ocean_fraction") ?? 0) > 0,
    });
  }
  assumed.push("orbits coplanar, periapses aligned (the chart does not carry inclinations yet)");
  return {
    spec: { name: scene.title, system: scene.system, star_address: star.address, star_mass: mass, z, age_myr: scene.now_yr / 1e6, birth_year: 0, planets, sources: "the chart room's scene" },
    assumed,
  };
}

const sig = (v: number, d = 3) => Number(v.toPrecision(d));
function fact(field: string, label: string, value: number, unit: string): SceneFact {
  const v = sig(value);
  return { field, label, value: v, unit, display: `${v} ${unit}`, editable: false, source: "derived" };
}

/** The nearest snapshot to a star age. */
export function snapshotAt(r: LookedAhead, age_myr: number): FateSnapshot | undefined {
  let best = r.snapshots[0];
  for (const s of r.snapshots) if (Math.abs(s.age_myr - age_myr) < Math.abs((best?.age_myr ?? 0) - age_myr)) best = s;
  return best;
}

export const PHASE_WORDS: Record<string, string> = { "main sequence": "main-sequence star", subgiant: "subgiant", "red giant": "red giant", "white dwarf": "white dwarf" };

/**
 * One snapshot drawn as the scene: the star's new state, each planet's new
 * orbit, engulfed planets stamped and taken off the chart, and the
 * look-ahead's events on the ruler as future events.
 */
export function sceneAt(scene: SystemScene, r: LookedAhead, snap: FateSnapshot): SystemScene {
  const star = primaryOf(scene);
  const byName = new Map(snap.planets.map((p) => [p.name, p]));
  const beyond = snap.age_myr > r.now_myr + r.horizon_myr;
  const subjects = scene.subjects
    .map((s) => {
      if (s === star) {
        const keep = s.facts.filter((f) => !["mass_sun", "luminosity_sun", "radius_sun", "teff_k"].includes(f.field));
        return {
          ...s,
          subtitle: `${PHASE_WORDS[snap.star.phase] ?? snap.star.phase}, ${snap.star.colour}`,
          // The star's own prose describes it now, not then.
          prose: `At ${fmtAge(snap.age_myr)} ${s.label} is a ${PHASE_WORDS[snap.star.phase] ?? snap.star.phase}, ${snap.star.colour}, ${sig(snap.star.luminosity)} times the Sun's luminosity.`,
          facts: [...keep.map((f) => ({ ...f, editable: false })), fact("mass_sun", "Mass", snap.star.mass, "M☉"), fact("luminosity_sun", "Luminosity", snap.star.luminosity, "L☉"), fact("radius_sun", "Radius", snap.star.radius_rsun, "R☉"), fact("teff_k", "Surface", snap.star.teff, "K")],
          stamps: [...s.stamps.filter((x) => x !== "resolved"), "look-ahead", ...(beyond ? ["plausible-future"] : [])],
          tags: s.tags.filter((t) => !t.startsWith("spectral:")),
        };
      }
      const p = byName.get(s.label);
      if (!p || !s.orbit) return s;
      if (p.engulfed) return null;
      const facts = s.facts.map((f) => (f.field === "a_au" ? { ...f, value: sig(p.a_au), display: `${sig(p.a_au)} AU`, editable: false, source: "derived" as const } : f.field === "eccentricity" ? { ...f, value: sig(p.e, 2), display: String(sig(p.e, 2)), editable: false, source: "derived" as const } : { ...f, editable: false }));
      return { ...s, orbit: { ...s.orbit, a_au: p.a_au }, facts, stamps: [...s.stamps, "look-ahead", ...(beyond ? ["plausible-future"] : [])] };
    })
    .filter((s): s is NonNullable<typeof s> => s !== null);
  // Moons go with their planet: a moon of an engulfed planet goes too.
  const kept = new Set(subjects.map((s) => s.address));
  // Nothing in a look-ahead can be edited: it is a view of a branch.
  const live = subjects.filter((s) => !s.parent || s.parent === star?.address || kept.has(s.parent)).map((s) => ({ ...s, facts: s.facts.map((f) => (f.editable ? { ...f, editable: false } : f)) }));
  const far = Math.max(scene.surveyed_to_au, ...live.filter((s) => s.orbit && s.parent === star?.address).map((s) => s.orbit!.a_au * 1.3));
  return {
    ...scene,
    subtitle: `Looking ahead: ${fmtAge(snap.age_myr)}${beyond ? " · one plausible future" : ""}`,
    now_yr: snap.age_myr * 1e6,
    span_yr: [scene.span_yr[0], Math.max(scene.span_yr[1], r.target_myr * 1e6 * 1.05)],
    surveyed_to_au: Math.ceil(far),
    subjects: live,
    events: [...scene.events.filter((e) => e.tone !== "future"), ...fateEvents(r, scene)],
  };
}

export function fateEvents(r: LookedAhead, scene?: SystemScene): SceneEvent[] {
  const name = (a: string) => scene?.subjects.find((s) => s.address === a)?.label ?? a.split("/").pop()!;
  const star = scene ? primaryOf(scene)?.address : undefined;
  const out: SceneEvent[] = [];
  let cooled = false;
  r.events.forEach((e, i) => {
    // The ruler gets the first cooling step only (the panel lists them all); planets engulfed together share one mark.
    if (e.kind === "cooling_colour") {
      if (cooled) return;
      cooled = true;
    }
    const who = e.subjects.filter((a) => a !== star && a !== r.system).map(name);
    const last = out.at(-1);
    if (e.kind === "engulfed" && last?.id.startsWith("ahead-engulfed") && e.age_myr * 1e6 - last.at_yr < 0.01 * last.at_yr) {
      last.label += `, ${who.join(", ")}`;
      return;
    }
    const words = KIND_WORDS[e.kind] ?? e.kind;
    out.push({ id: `ahead-${e.kind === "engulfed" ? "engulfed-" : ""}${i}`, at_yr: e.age_myr * 1e6, label: who.length && (e.kind === "engulfed" || e.kind === "oceans_lost") ? `${words}: ${who.join(", ")}` : words, tone: "future" });
  });
  return out;
}

export const KIND_WORDS: Record<string, string> = {
  oceans_lost: "Oceans lost",
  leaves_main_sequence: "Leaves the main sequence",
  red_giant: "Red giant",
  engulfed: "Engulfed",
  planetary_nebula: "Planetary nebula",
  cooling_colour: "White dwarf cools",
  pollution: "Debris falls on the white dwarf",
};

export function fmtAge(myr: number): string {
  return myr >= 1000 ? `${sig(myr / 1000, 4)} Gyr` : `${sig(myr, 3)} Myr`;
}

/** Names for subjects in event text: the scene's labels, by address. */
export function subjectNames(scene: SystemScene, e: FateEvent): string[] {
  return e.subjects.map((a) => scene.subjects.find((s) => s.address === a)?.label ?? a);
}

