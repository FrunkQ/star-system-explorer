// Screen documents (H8.3). The GM workspace is one of these: a fixed,
// curated layout (H8.8) whose areas hold view modules.
import type { Json } from "./json.ts";
import { resolveParams, type ViewModule } from "./view.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyView = ViewModule<any, any>;

export interface ScreenItem {
  view: string;
  area: string;
  params?: Record<string, Json>;
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
  items: ScreenItem[];
}

/** Areas each layout offers. */
export const LAYOUT_AREAS: Record<ScreenDoc["layout"], string[]> = {
  "chart-room": ["chart", "card", "ruler"],
  grid: ["a", "b", "c", "d", "e", "f"],
  flow: ["main"],
  stack: ["main"],
  page: ["main"],
};

export interface CheckedScreen {
  doc: ScreenDoc;
  mounts: { module: AnyView; area: string; params: Record<string, Json> }[];
}

/** Validate a screen against the registry of view modules; nothing mounts unless all of it is valid. */
export function checkScreen(doc: ScreenDoc, registry: readonly AnyView[]): { ok: true; screen: CheckedScreen } | { ok: false; errors: string[] } {
  const errors: string[] = [];
  if (doc.schema !== "sse.screen/1") errors.push(`unknown screen schema ${doc.schema}`);
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
    mounts.push({ module, area: item.area, params: r.params });
  }
  return errors.length ? { ok: false, errors } : { ok: true, screen: { doc, mounts } };
}
