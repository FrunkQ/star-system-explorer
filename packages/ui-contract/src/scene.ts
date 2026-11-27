// Interim scene description for the system chart. The kernel builds real
// scenes in K9 (H6.7: scene building after the data-view filter); until then
// the shell loads a fixture in this shape, labelled as a fixture. Views draw
// only from a scene and never compute physics.
import type { Json } from "./json.ts";
import type { DeriveRef } from "./session.ts";
import { constant } from "./sheet.ts";

export type FactSource = "authored" | "derived" | "resolved" | "catalogued" | "fixture";

export interface SceneFact {
  field: string;
  label: string;
  value: Json;
  unit?: string;
  /** Text as shown, already formatted for the vocabulary layer. */
  display: string;
  editable: boolean;
  source: FactSource;
  /** Where "why?" goes for a derived value: the flow, its inputs and the quantity kind. */
  why?: DeriveRef & { kind: string };
  /** Kernel validity label for a derived value ("in-range", "out-of-range", ...). */
  validity?: string;
  fictional?: boolean;
  /** Pack, flow and version that published a derived value ("pack/flow@version"). */
  publisher?: string;
  /** Published by the campaign's own physics, not the base packs: badged "changed physics". */
  changed?: boolean;
  /** Display value = SI value / scale (derived facts), so a preview can show another stack's value the same way. */
  scale?: number;
  digits?: number;
  /**
   * A value read back from the universe (an adopted, grown or catalogued system) is edited at its kernel field, in SI:
   * the value sent is the typed value × `per_unit`. Without it, an edit goes to `field` as typed (the fixture's own).
   */
  kernel?: { field: string; per_unit: number };
}

export interface SceneSubject {
  address: string;
  /** Object kind (star, planet, moon, belt, zone, station, ...): what symbology bindings apply to. */
  kind: string;
  label: string;
  /** One-line description, e.g. "III · temperate ocean world". */
  subtitle?: string;
  parent?: string;
  /** `r_m`: where the kernel places it from its host's centre (orbits_at, metres, the system's frame), when placed by date. */
  orbit?: { a_au: number; phase_deg: number; r_m?: [number, number, number]; v_m_s?: [number, number, number] };
  /** Radial extent of a belt or zone, in AU. */
  band_au?: [number, number];
  /**
   * A belt's shape: a flat "disc" (the asteroid belt, the Kuiper belt) or a
   * "sphere" all round the system (the Oort cloud). From the kernel's tilt
   * model (F5: Rayleigh tilts are a disc, isotropic tilts a sphere); a disc
   * when absent.
   */
  shape?: "disc" | "sphere";
  /**
   * A ring round a world (a belt whose host is a planet, not the star): its edges from the planet's centre in metres,
   * its normal optical depth and its particles' albedo when the universe holds them (null when it does not).
   */
  ring?: {
    inner_m: number;
    outer_m: number;
    tau: number | null;
    albedo: number | null;
    /** The measured range of its optical depth, when printed (VP-R3: structure inside the band stays within it). */
    tau_range?: [number, number];
    /** A ring formation made: its cloud's address, so the kernel's `ring_structure` reads it from the run. */
    cloud?: string;
  };
  /** A world's axial tilt to its orbit, radians, when the universe holds it (its rings lie in its equator). */
  tilt_rad?: number;
  facts: SceneFact[];
  /** Evidence and honesty stamps (resolved, authored, impossible, fictional, ...). */
  stamps: string[];
  tags: string[];
  prose?: string;
  /**
   * A P4 world-stack fixture that stands in for this body until the kernel
   * builds its stack from history. The Cutaway asks the kernel by address
   * first (read_world_stack) and by this name only when that finds none.
   */
  world_fixture?: string;
}

/** True for a belt that is a sphere round the system (the Oort cloud), not a flat band. Data, not a mark's look. */
export function isShell(s: Pick<SceneSubject, "shape">): boolean {
  return s.shape === "sphere";
}

export interface SceneEvent {
  id: string;
  at_yr: number;
  label: string;
  /** "past" | "future" | "authored": drawn differently on the ruler. */
  tone: "past" | "future" | "authored";
}

export interface SystemScene {
  schema: "sse.scene.system/0";
  /** True while the scene comes from a fixture, not from the kernel. */
  fixture: boolean;
  universe: { id: string; seed: string };
  system: string;
  title: string;
  subtitle: string;
  now_yr: number;
  span_yr: [number, number];
  /** Beyond this radius the chart is unsurveyed (hatched). */
  surveyed_to_au: number;
  subjects: SceneSubject[];
  events: SceneEvent[];
}

/** Kilometres in one astronomical unit (IAU 2012 exact definition). */
export const KM_PER_AU = constant("au") / 1000;

/** The system's primary star: the star with no orbit. */
export function primaryOf(scene: Pick<SystemScene, "subjects">): SceneSubject | undefined {
  return scene.subjects.find((s) => s.kind === "star" && !s.orbit);
}

/**
 * The bodies that orbit `centre`, nearest first. Orbits are local to their
 * parent (handover/rendering-rules.md): a moon belongs to its planet's frame,
 * never to the star's. A body with an orbit and no parent orbits the primary.
 */
export function childrenOf(scene: Pick<SystemScene, "subjects">, centre: string): SceneSubject[] {
  const primary = primaryOf(scene)?.address;
  return scene.subjects
    .filter((s) => s.orbit && (s.parent ?? primary) === centre)
    .sort((a, b) => a.orbit!.a_au - b.orbit!.a_au);
}

/** Every body in tree order: the primary, then each planet followed by its moons. */
export function systemOrder(scene: Pick<SystemScene, "subjects">): SceneSubject[] {
  const out: SceneSubject[] = [];
  const walk = (s: SceneSubject) => {
    out.push(s);
    for (const c of childrenOf(scene, s.address)) walk(c);
  };
  const p = primaryOf(scene);
  if (p) walk(p);
  return out;
}

/**
 * The frame a body is drawn in: its parent's. Null is the primary's frame
 * (the system chart), so a planet maps to null and a moon to its planet.
 */
export function frameFor(scene: Pick<SystemScene, "subjects">, address: string | null): string | null {
  const primary = primaryOf(scene)?.address;
  const s = address ? scene.subjects.find((x) => x.address === address) : undefined;
  if (!s || s.address === primary) return null;
  const parent = s.parent ?? primary;
  return parent === primary ? null : (parent ?? null);
}
