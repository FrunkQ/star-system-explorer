// Orac-first configuration (H8.9, build plan V11). Every view publishes a
// parameter schema (ParamSpec: name, type, range, default, unit, meaning,
// cost). A style document is a named set of settings for one view that can
// inherit another (`extends`), so a family of looks shares a base and a
// tweak is a small difference. Orac writes and edits these documents; people
// get the few promoted settings as plain controls. Each document is checked
// before use: unknown settings, wrong types, out-of-range values, and cost
// over what the device's tier runs. Presentation only: a style never reaches
// the kernel and never changes the state hash.
import type { Cost } from "../../contracts/src/index.ts";
import type { Json } from "./json.ts";
import { resolveParams, type ParamSpec, type ViewModule } from "./view.ts";

export interface StyleDoc {
  schema: "sse.style/1";
  id: string;
  name: string;
  description?: string;
  /** The view this style configures. */
  view: string;
  extends?: string;
  settings: Record<string, Json>;
  /** Settings to offer people as plain controls, if not the view's own promoted ones. */
  promote?: string[];
}

/** Cost classes in order, and the highest each device tier runs locally (the kernel's tier table). */
export const COSTS: readonly Cost[] = ["trivial", "light", "medium", "heavy"];
export const TIER_RUNS_UP_TO: Readonly<Record<string, Cost>> = { basic: "light", standard: "medium", capable: "heavy" };

/** The cost of one setting at one value, or null when it costs nothing extra. */
export function settingCost(spec: ParamSpec, value: Json): Cost | null {
  if (spec.type === "enum" && spec.cost_options) return typeof value === "string" ? (spec.cost_options[value] ?? null) : null;
  if (!spec.cost) return null;
  const engaged =
    spec.type === "boolean" ? value === true : spec.type === "number" || spec.type === "integer" ? typeof value === "number" && typeof spec.default === "number" && value > spec.default : JSON.stringify(value) !== JSON.stringify(spec.default);
  return engaged ? spec.cost : null;
}

/** The most expensive engaged setting, and which it is. */
export function settingsCost(specs: readonly ParamSpec[], params: Record<string, Json>): { cost: Cost; by: string[] } {
  let cost: Cost = "trivial";
  let by: string[] = [];
  for (const s of specs) {
    const c = settingCost(s, params[s.name] ?? s.default);
    if (!c) continue;
    if (COSTS.indexOf(c) > COSTS.indexOf(cost)) [cost, by] = [c, [s.name]];
    else if (c === cost && c !== "trivial") by.push(s.name);
  }
  return { cost, by };
}

/** Follow `extends` and merge settings, base first. Throws on a loop, a missing base or a base for another view. */
export function resolveStyle(id: string, registry: readonly StyleDoc[]): { view: string; settings: Record<string, Json>; chain: string[] } {
  const chain: StyleDoc[] = [];
  for (let cur: string | undefined = id; cur; ) {
    if (chain.some((d) => d.id === cur)) throw new Error(`style loop: ${[...chain.map((d) => d.id), cur].join(" > ")}`);
    const d = registry.find((x) => x.id === cur);
    if (!d) throw new Error(`no style ${cur}`);
    if (chain.length && d.view !== chain[0]!.view) throw new Error(`style ${chain[chain.length - 1]!.id} extends ${d.id}, which is for another view (${d.view})`);
    chain.push(d);
    cur = d.extends;
  }
  const settings: Record<string, Json> = {};
  for (const d of [...chain].reverse()) Object.assign(settings, d.settings);
  return { view: chain[0]!.view, settings, chain: chain.map((d) => d.id) };
}

export type StyleCheck =
  | { ok: true; params: Record<string, Json>; cost: Cost; by: string[]; chain: string[] }
  | { ok: false; errors: string[] };

/**
 * Check a style (with what it inherits) against its view's schema and a
 * device tier. Every problem is listed; nothing is applied unless all pass.
 * `extra` are settings given on top (a screen item's own params).
 */
export function checkStyle(id: string, registry: readonly StyleDoc[], views: readonly Pick<ViewModule<unknown, unknown>, "id" | "params">[], tier: string, extra: Record<string, Json> = {}): StyleCheck {
  let r: ReturnType<typeof resolveStyle>;
  try {
    r = resolveStyle(id, registry);
  } catch (e) {
    return { ok: false, errors: [(e as Error).message] };
  }
  const doc = registry.find((d) => d.id === id)!;
  if (doc.schema !== "sse.style/1") return { ok: false, errors: ["not an sse.style/1 document"] };
  const view = views.find((v) => v.id === r.view);
  if (!view) return { ok: false, errors: [`no view ${r.view}`] };
  const errors: string[] = [];
  const p = resolveParams(view.params, { ...r.settings, ...extra });
  errors.push(...p.errors);
  for (const n of doc.promote ?? []) if (!view.params.some((s) => s.name === n)) errors.push(`promotes ${n}, which ${r.view} does not have`);
  const c = settingsCost(view.params, p.params);
  const limit = TIER_RUNS_UP_TO[tier];
  if (!limit) errors.push(`unknown device tier ${tier}`);
  else if (COSTS.indexOf(c.cost) > COSTS.indexOf(limit)) errors.push(`over budget: ${c.by.join(", ")} cost ${c.cost}, and a ${tier} device runs up to ${limit}`);
  return errors.length ? { ok: false, errors } : { ok: true, params: p.params, cost: c.cost, by: c.by, chain: r.chain };
}

/** The settings people get as plain controls: the style's own choice, else the view's promoted ones. */
export function promotedOf(view: Pick<ViewModule<unknown, unknown>, "params">, style?: StyleDoc): ParamSpec[] {
  const names = style?.promote;
  return names ? view.params.filter((s) => names.includes(s.name)) : view.params.filter((s) => s.promoted);
}

/** The difference between two settings maps, as a reviewable list (an Orac change is a validated diff). */
export function settingsDiff(before: Record<string, Json>, after: Record<string, Json>): { name: string; before: Json | undefined; after: Json | undefined }[] {
  const names = [...new Set([...Object.keys(before), ...Object.keys(after)])].sort();
  return names.filter((n) => JSON.stringify(before[n]) !== JSON.stringify(after[n])).map((n) => ({ name: n, before: before[n], after: after[n] }));
}
