// What the 3D view draws, from either scene: bodies with a parent and a
// distance from it (metres, f64), never a flattened system-wide position.
// The view shows one frame at a time (rendering rule 2): a centre body and
// the bodies that orbit it, at log-scaled display distances.
import { KM_PER_AU, constant, fact, parentOf, primaryOf, subjectName, type Mark, type PlayerScene, type PlayerSubject, type SceneSubject, type SystemScene } from "../../../ui-contract/src/index.ts";
import type { FrameNode } from "../../../renderer/src/index.ts";
import type { SmallSample } from "./small-sample.ts";
import type { SmallShape } from "../../../ui-contract/src/index.ts";

export interface Body3D {
  id: string;
  parent: string | null;
  name: string;
  /** Distance from the parent, metres; null when the audience does not have it. */
  a_m: number | null;
  /** Phase angle, degrees; `phase_known` false when it is spread for display (players are not given phases). */
  phase_deg: number;
  phase_known: boolean;
  mark: Mark | null;
  /** The body's radius, metres, for Real scale; null or absent when the audience does not have it. */
  radius_m?: number | null;
  /** Its surface temperature, kelvin (a star's own), for True's infrared; null or absent when not known. */
  t_k?: number | null;
  /** Its rings (a belt whose host is this world), edges from its centre in metres; tau and albedo when known. */
  rings?: Ring3D[];
  /** Its axial tilt, radians, when known: the rings lie in its equator. */
  tilt_rad?: number;
  /** Its mass, kg, when known: a ringed world's moons cut gaps and edges in its rings by it (renderer ring-structure.ts). */
  mass_kg?: number;
  /** A belt member the kernel resolved for True (small-sample.ts): drawn in its own shape, with its coma and tails when active. */
  small?: SmallSample;
  /** RW16a: a scene body below the round size, drawn as its own lump (kernel `small_shape`, ui-contract small-shape.ts). */
  figure?: SmallShape;
  /** Where the kernel places it from its parent's centre (metres, the system's frame), when placed by date: shadows in True. */
  r_m?: [number, number, number];
}

/** A resolved belt member as a body round the star, where its orbit puts it now (True only). */
export function fromSmallSample(s: SmallSample, star: string): Body3D {
  return { id: s.address, parent: star, name: s.name, a_m: s.r_m, phase_deg: s.longitude_deg, phase_known: true, mark: null, radius_m: s.radius_m, small: s };
}

export interface Ring3D {
  name: string;
  inner_m: number;
  outer_m: number;
  tau: number | null;
  albedo: number | null;
}

export function fromSystemScene(scene: SystemScene, mark: (s: SceneSubject) => Mark): Body3D[] {
  const star = primaryOf(scene);
  return scene.subjects
    .filter((s) => s === star || s.orbit)
    .map((s) => {
      const rings = scene.subjects.filter((r) => r.ring && r.parent === s.address).map((r) => ({ name: r.label, ...r.ring! }));
      return { id: s.address, parent: s === star ? null : (s.parent ?? star?.address ?? null), name: s.label, a_m: s.orbit ? s.orbit.a_au * KM_PER_AU * 1000 : null, phase_deg: s.orbit?.phase_deg ?? 0, phase_known: true, ...(s.orbit?.r_m ? { r_m: s.orbit.r_m } : {}), mark: mark(s), radius_m: radiusOf(s), t_k: tempOf(s), ...(rings.length ? { rings } : {}), ...(s.tilt_rad !== undefined ? { tilt_rad: s.tilt_rad } : {}), ...massOf(s) };
    });
}

/** A subject's mass in kg from its facts (Earth masses for worlds, Sun masses for stars), when known. */
function massOf(s: SceneSubject): { mass_kg?: number } {
  const v = (field: string) => {
    const f = s.facts.find((x) => x.field === field);
    return typeof f?.value === "number" && f.value > 0 ? f.value : null;
  };
  const me = v("mass_earth");
  if (me !== null) return { mass_kg: me * constant("M_earth") };
  const ms = v("mass_sun");
  return ms !== null ? { mass_kg: ms * constant("M_sun") } : {};
}

/** A world's moons as the rings need them: name, distance and mass, for those known. */
export function moonsOf(bodies: readonly Body3D[], id: string): { name: string; a_m: number; mass_kg: number }[] {
  return bodies.filter((b) => b.parent === id && b.a_m !== null && b.mass_kg !== undefined).map((b) => ({ name: b.name, a_m: b.a_m!, mass_kg: b.mass_kg! }));
}

/** A subject's surface temperature in kelvin: a star's effective, a world's surface, else its equilibrium. */
function tempOf(s: SceneSubject): number | null {
  for (const field of ["teff_k", "surface_k", "teq_k"]) {
    const f = s.facts.find((x) => x.field === field);
    if (typeof f?.value === "number" && f.value > 0) return f.value;
  }
  return null;
}

/** A subject's radius in metres from its facts (Earth radii for worlds, Sun radii for stars), else null. */
function radiusOf(s: SceneSubject): number | null {
  const v = (field: string) => {
    const f = s.facts.find((x) => x.field === field);
    return typeof f?.value === "number" && f.value > 0 ? f.value : null;
  };
  const re = v("radius_earth");
  if (re !== null) return re * constant("R_earth_mean");
  const rs = v("radius_sun");
  return rs !== null ? rs * constant("R_sun") : null;
}

/** A belt, disc or cloud round the star: drawn flat (a disc) or as a shell all round (a sphere, the Oort cloud). */
export interface Band3D {
  id: string;
  name: string;
  a_in_m: number;
  a_out_m: number;
  shape: "disc" | "sphere";
  /** The star's GM, m³/s², when its mass is known: the belt's members go round at the Kepler rate. */
  gm?: number;
}

export function bandsFromScene(scene: SystemScene): Band3D[] {
  const star = primaryOf(scene);
  const ms = star?.facts.find((x) => x.field === "mass_sun")?.value;
  const gm = typeof ms === "number" && ms > 0 ? constant("G") * ms * constant("M_sun") : undefined;
  return scene.subjects
    .filter((s) => s.kind === "belt" && s.band_au && (!s.parent || s.parent === star?.address))
    .map((s) => ({ id: s.address, name: s.label, a_in_m: s.band_au![0] * KM_PER_AU * 1000, a_out_m: s.band_au![1] * KM_PER_AU * 1000, shape: s.shape ?? "disc", ...(gm ? { gm } : {}) }));
}

/**
 * The lines that draw a flat band in its frame (display units, centre at the origin): its inner and outer edge in
 * the plane. `dist` maps metres to display units; nothing is drawn past `cap`.
 */
export function bandLines(b: Band3D, dist: (a_m: number) => number, cap: number, seg = 96): [number, number, number][][] {
  if (b.shape !== "disc") return [];
  const circle = (r: number): [number, number, number][] => {
    const out: [number, number, number][] = [];
    for (let i = 0; i <= seg; i++) {
      const a = (i / seg) * 2 * Math.PI;
      out.push([r * Math.cos(a), r * Math.sin(a), 0]);
    }
    return out;
  };
  return [circle(Math.min(cap, dist(b.a_in_m))), circle(Math.min(cap, dist(b.a_out_m)))];
}

/**
 * A shell (the Oort cloud) as a sparse round haze of points (night-sky design section 10): spread evenly over the
 * sphere by the golden angle, at radii from its inner edge outward to `cap`. Deterministic: the same shell always
 * draws the same points.
 */
export function shellDots(b: Band3D, dist: (a_m: number) => number, cap: number, n = 420): [number, number, number][] {
  if (b.shape !== "sphere") return [];
  const r0 = Math.min(cap - 1, dist(b.a_in_m));
  const golden = Math.PI * (3 - Math.sqrt(5));
  const out: [number, number, number][] = [];
  for (let i = 0; i < n; i++) {
    const z = 1 - (2 * (i + 0.5)) / n;
    const ring = Math.sqrt(1 - z * z);
    const t = i * golden;
    // Depth through the shell from a second low-discrepancy sequence, so the haze has thickness without randomness.
    const r = r0 + (cap - r0) * ((i * 0.618033988749895) % 1);
    out.push([r * ring * Math.cos(t), r * ring * Math.sin(t), r * z]);
  }
  return out;
}

export function fromPlayerScene(scene: PlayerScene, field: string, mark?: (s: PlayerSubject) => Mark): Body3D[] {
  const out: Body3D[] = [];
  const kids = new Map<string, number>();
  for (const s of scene.subjects) {
    const parent = s.address === scene.scope ? null : (parentOf(s.address, scene.scope) ?? scene.scope);
    const f = fact(s, field);
    const a = typeof f?.value === "number" ? f.value : f?.band ? (f.band[0] + f.band[1]) / 2 : null;
    const i = parent ? (kids.get(parent) ?? 0) : 0;
    if (parent) kids.set(parent, i + 1);
    out.push({ id: s.address, parent, name: subjectName(s), a_m: a, phase_deg: 25 + i * 67, phase_known: false, mark: mark ? mark(s) : null });
  }
  return out;
}

export const AU = KM_PER_AU * 1000;

export interface FrameLayout {
  centre: Body3D;
  placed: Body3D[];
  unplaced: Body3D[];
  nodes: FrameNode[];
  /** Display radius of each placed body's orbit (display units), by id. */
  ring: Map<string, number>;
  inner_m: number;
  outer_m: number;
  /** Metres from the centre to display units, the scale the rings use. */
  dist: (a_m: number) => number;
}

/** Display units: orbits between 1.5 and 10, log-spaced. */
export function layout(bodies: readonly Body3D[], centreId: string): FrameLayout | null {
  const centre = bodies.find((b) => b.id === centreId);
  if (!centre) return null;
  const kids = bodies.filter((b) => b.parent === centreId);
  const placed = kids.filter((b) => b.a_m !== null && b.a_m > 0).sort((a, b) => a.a_m! - b.a_m!);
  const unplaced = kids.filter((b) => !placed.includes(b));
  const inner = placed.length ? placed[0]!.a_m! / 1.6 : 1;
  const outer = placed.length ? placed[placed.length - 1]!.a_m! * 1.25 : 10;
  const r = (a: number) => 1.5 + (8.5 * Math.log(a / inner)) / Math.log(outer / inner);
  const ring = new Map<string, number>();
  const nodes: FrameNode[] = [{ id: centre.id, parent: null, local: [0, 0, 0] }];
  for (const b of placed) {
    const d = r(b.a_m!);
    const t = (b.phase_deg * Math.PI) / 180;
    ring.set(b.id, d);
    nodes.push({ id: b.id, parent: centre.id, local: [d * Math.cos(t), d * Math.sin(t), 0] });
  }
  return { centre, placed, unplaced, nodes, ring, inner_m: inner, outer_m: outer, dist: r };
}

/**
 * Real scale (Alex, 23 Nov 2026): the same frame with distances in proportion, the outermost orbit at 10 display
 * units, and `dist` linear, so a radius through `dist` is the body's true size on the same scale.
 */
export function layoutReal(bodies: readonly Body3D[], centreId: string): FrameLayout | null {
  const centre = bodies.find((b) => b.id === centreId);
  if (!centre) return null;
  const kids = bodies.filter((b) => b.parent === centreId);
  const placed = kids.filter((b) => b.a_m !== null && b.a_m > 0).sort((a, b) => a.a_m! - b.a_m!);
  const unplaced = kids.filter((b) => !placed.includes(b));
  const outer = placed.length ? placed[placed.length - 1]!.a_m! : 1;
  const r = (a: number) => (10 * a) / outer;
  const ring = new Map<string, number>();
  const nodes: FrameNode[] = [{ id: centre.id, parent: null, local: [0, 0, 0] }];
  for (const b of placed) {
    const d = r(b.a_m!);
    const t = (b.phase_deg * Math.PI) / 180;
    ring.set(b.id, d);
    nodes.push({ id: b.id, parent: centre.id, local: [d * Math.cos(t), d * Math.sin(t), 0] });
  }
  return { centre, placed, unplaced, nodes, ring, inner_m: placed.length ? placed[0]!.a_m! : 0, outer_m: outer, dist: r };
}

export const hasChildren = (bodies: readonly Body3D[], id: string) => bodies.some((b) => b.parent === id && b.a_m !== null);

/**
 * Every body where it really is, for shadows in True (renderer eclipse.ts): its parent's place plus its own, the
 * kernel's three-dimensional place by date where the scene has it, else its distance and phase in its parent's plane.
 * A body with no radius, no distance or a phase spread for display (a players' scene) is left out: a shadow drawn from
 * a made-up place would tell a lie. The star sits at the origin.
 */
export function placedBodies(bodies: readonly Body3D[]): Map<string, { id: string; at_m: [number, number, number]; radius_m: number }> {
  const byId = new Map(bodies.map((b) => [b.id, b]));
  const at = new Map<string, [number, number, number] | null>();
  const place = (b: Body3D, depth = 0): [number, number, number] | null => {
    if (at.has(b.id)) return at.get(b.id)!;
    let p: [number, number, number] | null = null;
    if (b.parent === null) p = [0, 0, 0];
    else if (depth < 16 && b.phase_known) {
      const up = byId.get(b.parent);
      const base = up ? place(up, depth + 1) : null;
      const t = (b.phase_deg * Math.PI) / 180;
      const own: [number, number, number] | null = b.r_m ?? (b.a_m !== null ? [b.a_m * Math.cos(t), b.a_m * Math.sin(t), 0] : null);
      p = base && own ? [base[0] + own[0], base[1] + own[1], base[2] + own[2]] : null;
    }
    at.set(b.id, p);
    return p;
  };
  const out = new Map<string, { id: string; at_m: [number, number, number]; radius_m: number }>();
  for (const b of bodies) {
    const p = place(b);
    if (p && b.radius_m && b.radius_m > 0) out.set(b.id, { id: b.id, at_m: p, radius_m: b.radius_m });
  }
  return out;
}
