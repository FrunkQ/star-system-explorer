// The view contract (H8.1) and the hanging-configuration parameter schema
// (H8.9). A view is a plug-in: it receives a scene, draws it with theme
// tokens and marks from the symbology, and may send intents. It never reads
// kernel state and never computes physics.
import type { Cost } from "../../contracts/src/index.ts";
import type { IntentSchema } from "./intent.ts";
import type { Mark } from "./mark.ts";
import type { SceneSubject, SystemScene } from "./scene.ts";
import type { DeriveRef, ViewSession } from "./session.ts";
import type { ChangeRequest } from "./physics.ts";
import type { Json } from "./json.ts";

export type Tier = "still" | "gpu-high" | "gpu-basic" | "canvas" | "text";

export interface TopicQuery {
  addresses?: string[];
  quantity_kinds?: string[];
  event_kinds?: string[];
  /** Subject kinds, as the scene's `kind` fact gives them (player scenes). */
  kinds?: string[];
}

/** One setting, described for Orac first: name, type, range, default, unit, meaning, cost. */
export interface ParamSpec {
  name: string;
  type: "number" | "integer" | "boolean" | "enum" | "string" | "list";
  default: Json;
  meaning: string;
  range?: [number, number];
  options?: string[];
  unit?: string;
  /** Shown as a plain control to people; everything else is for Orac or the raw document. */
  promoted?: boolean;
  cost?: Cost;
}

/** One applied change, kept so it can be undone by a further, opposite intent. */
export interface AppliedEdit {
  address: string;
  field: string;
  before: { value: Json; display: string };
  after: { value: Json; display: string };
  event_id: string;
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
  /** Edits that can be undone, newest last; undone edits that can be redone. */
  undo: AppliedEdit[];
  redo: AppliedEdit[];
  /** The screen document in front of the GM ("system", "physics", ...). */
  screen: string;
  /** The flow on the bench in the flow editor, with the values it was opened for. */
  bench: { flow: string; node?: string; ref?: DeriveRef; label?: string } | null;
  /** Physics change requests: drafts, adopted, declined. */
  requests: ChangeRequest[];
  select(address: string | null): void;
  /** Put a flow on the bench and show the physics screen. */
  openFlow(flow: string, from?: { node?: string; ref?: DeriveRef; label?: string }): void;
  /** Add a request, or replace the one with the same id. */
  fileRequest(r: ChangeRequest): void;
  /** The physics in force changed: re-derive everything. */
  notePhysics(): void;
  setTime(yr: number): void;
  /** Record an applied edit. `as` says whether it is a new edit (clears redo), an undo or a redo. */
  noteApplied(edit: AppliedEdit, as?: "edit" | "undo" | "redo"): void;
}

export interface ViewContext {
  session: ViewSession;
  workspace: Workspace;
  params: Record<string, Json>;
  mark(subject: SceneSubject): Mark;
  /** The symbols in force, their legend and (on the symbols screen) the choice of grammar. */
  symbology?: import("./symbology.ts").SymbologyHost;
}

/**
 * What a player view gets (H8.1): its parameters, a selection shared with the
 * other views of its screen, and a way to send intents. No session, no
 * workspace: a player view sees only the scene built for its audience.
 */
export interface PlayerContext {
  params: Record<string, Json>;
  /** The screen's vocabulary layer: words and units for fields. */
  vocab: import("./player-scene.ts").Vocab;
  selection(): string | null;
  select(address: string | null): void;
  /** Ask for something (an intent); the host sends it through its seat. Absent on read-only screens. */
  send?(intent: import("./intent.ts").Intent): Promise<unknown>;
  /** Marks for player subjects, from the screen's symbology under its theme (absent: views draw plain discs). */
  mark?(subject: import("./player-scene.ts").PlayerSubject): Mark;
}

export interface ViewHandle<S = SystemScene> {
  update(scene: S): void;
  dispose(): void;
}

/**
 * One contract for every view (H8.1). `S` is the scene description it draws
 * (the GM's system scene, or a player scene built after the data-view
 * filter); `C` is what the host gives it.
 */
export interface ViewModule<S = SystemScene, C = ViewContext> {
  id: string;
  version: string;
  tiers: Tier[];
  inputs: TopicQuery[];
  params: ParamSpec[];
  intents?: IntentSchema[];
  /** Which scene description it draws: "system" (GM) or "player" (K9 scenes). */
  scene?: "system" | "player";
  /** "fill": the view sizes itself to a box the screen gives it a height for (a chart); "content" (default): it grows with what it shows. */
  box?: "fill" | "content";
  mount(el: HTMLElement, ctx: C, scene: S): ViewHandle<S>;
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
    case "list":
      return Array.isArray(v) && v.every((x) => typeof x === "string" || typeof x === "number") ? null : "must be a list of names or numbers";
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
