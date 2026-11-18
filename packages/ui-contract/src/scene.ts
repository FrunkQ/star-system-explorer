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
