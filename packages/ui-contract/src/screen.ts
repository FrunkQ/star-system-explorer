// Screen documents (H8.3). The GM workspace is one of these: a fixed,
// curated layout (H8.8) whose areas hold view modules.
import type { Json } from "./json.ts";
import { resolveParams, type TopicQuery, type ViewModule } from "./view.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyView = ViewModule<any, any>;

export interface ScreenItem {
  view: string;
  area: string;
  params?: Record<string, Json>;
  /** What this item shows (H8.3 `bind`): only these subjects or kinds of the scene; absent shows the whole scene. */
  bind?: TopicQuery;
}

export interface ScreenDoc {
  schema: "sse.screen/1";
  id: string;
  label: string;
  audience: "gm" | "players" | `player:${string}`;
  layout: "chart-room" | "grid" | "flow" | "stack" | "page";
  theme: string;
  symbology: string;
  vocab: "physics" | "plain" | "story";
  /** One line for people and Orac choosing a screen. */
  description?: string;
  /** The data view the screen is seen through; absent uses the audience's own (H8.3). */
  data_view?: string;
  items: ScreenItem[];
}

/** Areas each layout offers. */
export const LAYOUT_AREAS: Record<ScreenDoc["layout"], string[]> = {
  "chart-room": ["chart", "card", "ruler"],
  grid: ["a", "b", "c", "d", "e", "f"],
  // One column in order (stack), wrapping tiles (flow), a printable report (page).
  flow: ["1", "2", "3", "4", "5", "6", "7", "8"],
  stack: ["1", "2", "3", "4", "5", "6", "7", "8"],
  page: ["1", "2", "3", "4", "5", "6", "7", "8"],
};

export interface CheckedScreen {
  doc: ScreenDoc;
  mounts: { module: AnyView; area: string; params: Record<string, Json>; bind?: TopicQuery }[];
}

/**
 * Validate a screen against the registry of view modules; nothing mounts
 * unless all of it is valid. `dataViews`, when given, are the data views the
 * campaign offers, and a screen naming another is refused.
 */
export function checkScreen(doc: ScreenDoc, registry: readonly AnyView[], dataViews?: readonly string[]): { ok: true; screen: CheckedScreen } | { ok: false; errors: string[] } {
  const errors: string[] = [];
  if (!doc || typeof doc !== "object") return { ok: false, errors: ["not a screen document"] };
  if (doc.schema !== "sse.screen/1") errors.push(`unknown screen schema ${doc.schema}`);
  if (typeof doc.id !== "string" || !doc.id) errors.push("a screen needs an id");
  if (!Array.isArray(doc.items)) return { ok: false, errors: [...errors, "a screen needs a list of items"] };
  if (doc.data_view !== undefined && dataViews && !dataViews.includes(doc.data_view)) errors.push(`no data view ${doc.data_view} in this campaign (${dataViews.join(", ")})`);
  const areas = LAYOUT_AREAS[doc.layout];
  if (!areas) errors.push(`unknown layout ${doc.layout}`);
  const used = new Set<string>();
  const mounts: CheckedScreen["mounts"] = [];
  for (const item of doc.items) {
    const module = registry.find((m) => m.id === item.view);
    if (!module) {
      errors.push(`no view module ${item.view}`);
      continue;
    }
    if (areas && !areas.includes(item.area)) errors.push(`layout ${doc.layout} has no area ${item.area}`);
    if (used.has(item.area)) errors.push(`area ${item.area} is used twice`);
    used.add(item.area);
    const r = resolveParams(module.params, item.params);
    errors.push(...r.errors.map((e) => `${item.view}: ${e}`));
    const bad = item.bind ? checkBind(item.bind) : null;
    if (bad) errors.push(`${item.view}: bind ${bad}`);
    mounts.push({ module, area: item.area, params: r.params, ...(item.bind ? { bind: item.bind } : {}) });
  }
  return errors.length ? { ok: false, errors } : { ok: true, screen: { doc, mounts } };
}

function checkBind(b: TopicQuery): string | null {
  for (const k of Object.keys(b)) if (!["addresses", "quantity_kinds", "event_kinds", "kinds"].includes(k)) return `has an unknown key ${k}`;
  for (const [k, v] of Object.entries(b)) if (!Array.isArray(v) || !v.every((x) => typeof x === "string")) return `${k} must be a list of names`;
  return null;
}

/**
 * The part of a player scene one item shows: its bound subjects (and their
 * children), or kinds; the scope itself stays so a chart keeps its star.
 */
export function boundScene<S extends { scope: string; subjects: { address: string; facts: { field: string; value?: Json }[] }[] }>(scene: S, bind: TopicQuery | undefined): S {
  if (!bind || (!bind.addresses?.length && !bind.kinds?.length)) return scene;
  const kind = (s: S["subjects"][number]) => String(s.facts.find((f) => f.field === "kind")?.value ?? "");
  const keep = (s: S["subjects"][number]) =>
    s.address === scene.scope ||
    (bind.addresses ?? []).some((a) => s.address === a || s.address.startsWith(`${a}/`)) ||
    (bind.kinds ?? []).includes(kind(s));
  return { ...scene, subjects: scene.subjects.filter(keep) };
}
