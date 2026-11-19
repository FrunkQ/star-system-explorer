// Interim scene description for the system chart. The kernel builds real
// scenes in K9 (H6.7: scene building after the data-view filter); until then
// the shell loads a fixture in this shape, labelled as a fixture. Views draw
// only from a scene and never compute physics.
import type { Json } from "./json.ts";
import type { DeriveRef } from "./session.ts";

export type FactSource = "authored" | "derived" | "resolved" | "fixture";

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
}

export interface SceneSubject {
  address: string;
  /** Object kind (star, planet, moon, belt, zone, station, ...): what symbology bindings apply to. */
  kind: string;
  label: string;
  /** One-line description, e.g. "III · temperate ocean world". */
  subtitle?: string;
  parent?: string;
  orbit?: { a_au: number; phase_deg: number };
  /** Radial extent of a belt or zone, in AU. */
  band_au?: [number, number];
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
export const KM_PER_AU = 149_597_870.7;

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
