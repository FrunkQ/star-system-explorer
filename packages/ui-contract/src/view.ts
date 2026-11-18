// The view contract (H8.1) and the hanging-configuration parameter schema
// (H8.9). A view is a plug-in: it receives a scene, draws it with theme
// tokens and marks from the symbology, and may send intents. It never reads
// kernel state and never computes physics.
import type { Cost } from "../../contracts/src/index.ts";
import type { IntentSchema } from "./intent.ts";
import type { Mark } from "./mark.ts";
import type { SceneSubject, SystemScene } from "./scene.ts";
import type { ViewSession } from "./session.ts";
import type { Json } from "./json.ts";

export type Tier = "still" | "gpu-high" | "gpu-basic" | "canvas" | "text";

export interface TopicQuery {
  addresses?: string[];
  quantity_kinds?: string[];
  event_kinds?: string[];
}

/** One setting, described for Orac first: name, type, range, default, unit, meaning, cost. */
export interface ParamSpec {
  name: string;
  type: "number" | "integer" | "boolean" | "enum" | "string";
  default: Json;
  meaning: string;
  range?: [number, number];
  options?: string[];
  unit?: string;
  /** Shown as a plain control to people; everything else is for Orac or the raw document. */
  promoted?: boolean;
  cost?: Cost;
}

/** Shared, reactive state of one workspace: what is selected, what time it is, what mode. */
export interface Workspace {
  selection: string | null;
  now_yr: number;
  mode: "edit" | "view";
  /** Values applied by intents in this session, keyed "address#field", shown until scenes come from the kernel. */
  applied: Record<string, { value: Json; display: string; event_id: string }>;
  /** Bumped whenever an intent is applied, so views can refresh history. */
  revision: number;
  select(address: string | null): void;
  setTime(yr: number): void;
  noteApplied(address: string, field: string, value: Json, display: string, event_id: string): void;
}

export interface ViewContext {
  session: ViewSession;
  workspace: Workspace;
  params: Record<string, Json>;
  mark(subject: SceneSubject): Mark;
}

export interface ViewHandle {
  update(scene: SystemScene): void;
  dispose(): void;
}

export interface ViewModule {
  id: string;
  version: string;
  tiers: Tier[];
  inputs: TopicQuery[];
  params: ParamSpec[];
  intents?: IntentSchema[];
  mount(el: HTMLElement, ctx: ViewContext, scene: SystemScene): ViewHandle;
}

/** Fill defaults and reject anything outside the schema. */
export function resolveParams(specs: ParamSpec[], given: Record<string, Json> = {}): { params: Record<string, Json>; errors: string[] } {
  const errors: string[] = [];
  const params: Record<string, Json> = {};
  const known = new Set(specs.map((s) => s.name));
  for (const k of Object.keys(given)) if (!known.has(k)) errors.push(`unknown parameter ${k}`);
  for (const s of specs) {
    const v = s.name in given ? given[s.name]! : s.default;
    const bad = checkParam(s, v);
    if (bad) errors.push(`${s.name}: ${bad}`);
    params[s.name] = v;
  }
  return { params, errors };
}

function checkParam(s: ParamSpec, v: Json): string | null {
  switch (s.type) {
    case "boolean":
      return typeof v === "boolean" ? null : "must be true or false";
    case "string":
      return typeof v === "string" ? null : "must be text";
    case "enum":
      return typeof v === "string" && (s.options ?? []).includes(v) ? null : `must be one of ${(s.options ?? []).join(", ")}`;
    case "number":
    case "integer": {
      if (typeof v !== "number" || !Number.isFinite(v)) return "must be a number";
      if (s.type === "integer" && !Number.isInteger(v)) return "must be a whole number";
      if (s.range && (v < s.range[0] || v > s.range[1])) return `must be within ${s.range[0]} to ${s.range[1]}`;
      return null;
    }
  }
}
