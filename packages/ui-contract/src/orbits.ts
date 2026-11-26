// Orbits by date (U11 part 2): where each body is at the time being viewed,
// from the kernel's `orbits_at` (two-body Kepler, local to each host). The
// views draw from the scene, so the shell lays the phases onto the scene and
// every view (chart, 3D, sound, players' data) moves together. Positions are
// the kernel's (rendering rule 5); this file only builds the request and reads
// the reply.
import type { SceneSubject, SystemScene } from "./scene.ts";
import { constant } from "./sheet.ts";

export const M_SUN = constant("M_sun");
export const M_EARTH = constant("M_earth");
export const AU_M = constant("au");

export interface OrbitPlaced {
  address: string;
  placed: boolean;
  parent?: string;
  period_s?: number;
  /** "measured" when it follows a motion model (RW17: a published series, inside its span). */
  phase_source?: "derived" | "resolved" | "measured";
  missing?: string[];
  /** The motion model it follows and its span (universe seconds), or why the one it names is not used. */
  motion?: { model: string; from_s?: number; to_s?: number; unused?: string };
  /** `motion`: "measured" inside the model's span, "plausible" outside it (a plain orbit fitted at its edge). */
  at?: { at_s: number; r_m: [number, number, number]; phase_deg: number; motion?: "measured" | "plausible" }[];
}

const factValue = (s: SceneSubject, field: string): number | undefined => {
  const v = s.facts.find((f) => f.field === field)?.value;
  return typeof v === "number" && Number.isFinite(v) ? v : undefined;
};

/** A body's mass in kilograms from the scene's facts (solar or Earth masses), if it has one. */
export function massKg(s: SceneSubject): number | undefined {
  const sun = factValue(s, "mass_sun");
  if (sun !== undefined) return sun * M_SUN;
  const earth = factValue(s, "mass_earth");
  if (earth !== undefined) return earth * M_EARTH;
  return undefined;
}

/** Mean anomaly (radians) for a true anomaly (degrees) on an orbit of eccentricity e. */
export function meanFromTrue(true_deg: number, e: number): number {
  const nu = (true_deg * Math.PI) / 180;
  const E = 2 * Math.atan2(Math.sqrt(1 - e) * Math.sin(nu / 2), Math.sqrt(1 + e) * Math.cos(nu / 2));
  const M = E - e * Math.sin(E);
  return ((M % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
}

/**
 * The `bodies` to send for a scene the kernel does not hold (a fixture): each
 * orbiting body's elements, with its drawn phase as its place at `epoch_s`
 * (periapsis along the reference direction, so the phase is the true
 * anomaly), and each host's mass. A system the kernel holds sends no bodies:
 * the kernel reads its own facts and resolves any missing phase itself.
 */
export function orbitBodies(scene: SystemScene, epoch_s: bigint): { address: string; facts: Record<string, number | string | bigint> }[] | undefined {
  if (!scene.fixture) return undefined;
  const primary = scene.subjects.find((s) => !s.orbit && (s.kind === "star" || !s.parent))?.address;
  const out: { address: string; facts: Record<string, number | string | bigint> }[] = [];
  for (const s of scene.subjects) {
    const facts: Record<string, number | string | bigint> = {};
    const m = massKg(s);
    if (m !== undefined) facts["body.mass"] = m;
    if (s.orbit) {
      const e = Math.min(0.99, Math.max(0, factValue(s, "eccentricity") ?? 0));
      const host = s.parent ?? primary;
      if (host) facts["orbit.host"] = host;
      facts["orbit.semi_major_axis"] = s.orbit.a_au * AU_M;
      facts["orbit.eccentricity"] = e;
      facts["orbit.mean_anomaly_at_epoch"] = meanFromTrue(s.orbit.phase_deg, e);
      facts["orbit.epoch_s"] = epoch_s;
    }
    if (Object.keys(facts).length) out.push({ address: s.address, facts });
  }
  return out;
}

/** Each placed body's phase at the reply's first time. */
export function phasesFrom(bodies: readonly OrbitPlaced[]): Map<string, number> {
  const out = new Map<string, number>();
  for (const b of bodies) {
    const p = b.placed ? b.at?.[0]?.phase_deg : undefined;
    if (typeof p === "number" && Number.isFinite(p)) out.set(b.address, p);
  }
  return out;
}

/** Each placed body's place from its host's centre at the reply's first time (metres): eclipses need the third axis. */
export function placesFrom(bodies: readonly OrbitPlaced[]): Map<string, [number, number, number]> {
  const out = new Map<string, [number, number, number]>();
  for (const b of bodies) {
    const r = b.placed ? b.at?.[0]?.r_m : undefined;
    if (r && r.length === 3 && r.every((x) => Number.isFinite(x))) out.set(b.address, [r[0], r[1], r[2]]);
  }
  return out;
}

/** The scene with each body moved to its phase (and its place, when given); bodies the kernel could not place keep theirs. */
export function withPhases(scene: SystemScene, phases: ReadonlyMap<string, number> | null, places?: ReadonlyMap<string, [number, number, number]> | null): SystemScene {
  if (!phases || !phases.size) return scene;
  let moved = false;
  const subjects = scene.subjects.map((s) => {
    const p = s.orbit ? phases.get(s.address) : undefined;
    const r = s.orbit ? places?.get(s.address) : undefined;
    if (p === undefined || !s.orbit || (p === s.orbit.phase_deg && !r)) return s;
    moved = true;
    return { ...s, orbit: { ...s.orbit, phase_deg: p, ...(r ? { r_m: r } : {}) } };
  });
  return moved ? { ...scene, subjects } : scene;
}
