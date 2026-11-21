// Recipe editing (R4, SSE4-surfaces-and-rendering.md 5 and the R4 row of
// surface design section 9). Orac (A2, not built yet) and the Body screen's
// editor change a recipe through the same small edit list, never by writing
// the document freely: every edit is checked against the pass library, the
// result is checked as a whole (`checkRecipe`), and the cost of each tier is
// estimated before and after against the device's frame budget, so whoever
// asked can see what an edit costs before keeping it. An edited recipe is a
// new recipe (its own id, `derived_from` and a history in words); shipped
// recipes are never changed. Recipes are view configuration: they never
// reach the kernel and never change the state hash.
import { resolveParams, type Json, type Tier } from "../../ui-contract/src/index.ts";
import { checkRecipe, DEFAULT_BUDGET_MS, estimateMs, plan, TIER_ORDER, type PassSpec, type RecipeDoc, type RecipePass } from "./recipe.ts";

export type RecipeEdit =
  | { op: "set"; use: string; param: string; value: Json }
  | { op: "unset"; use: string; param: string }
  | { op: "knob_default"; knob: string; value: number }
  | { op: "add"; use: RecipePass; before?: string }
  | { op: "remove"; use: string }
  | { op: "use_tiers"; use: string; tiers: Tier[] | null }
  | { op: "budget"; tier: Tier; ms: number }
  | { op: "name"; name: string; description?: string };

export interface TierCost {
  tier: Tier;
  before_ms: number | null;
  after_ms: number | null;
  budget_ms: number;
  /** The full-quality frame fits the budget (the governor need not reduce anything). */
  fits: boolean;
}

export interface EditResult {
  ok: boolean;
  /** The edited recipe (also returned when not ok, for showing what went wrong). */
  doc: RecipeDoc;
  errors: string[];
  /** What changed, in words, one line per edit. */
  changes: string[];
  cost: TierCost[];
  /** Tiers that fitted the budget before and do not after. */
  newly_over: Tier[];
}

export interface EditContext {
  library: readonly PassSpec[];
  /** Output pixels to cost a frame at (default 640 × 640). */
  pixels?: number;
  /** Per-tier device factors from the probe (absent: the reference device). */
  device?: Partial<Record<Tier, number>>;
  /** The view's budget for live frames, which wins over the recipe's own when lower (not for stills or words). */
  budget_ms?: number;
  /** Id for the edited recipe (default: the original's id with "-edited"). */
  id?: string;
}

const clone = <T>(x: T): T => JSON.parse(JSON.stringify(x)) as T;
const fmt = (v: Json) => (typeof v === "number" ? String(Math.round(v * 1000) / 1000) : JSON.stringify(v));

/** Each tier's full-quality cost and budget. */
export function tierCosts(doc: RecipeDoc, ctx: EditContext): { tier: Tier; ms: number | null; budget_ms: number }[] {
  const px = ctx.pixels ?? 640 * 640;
  return doc.tiers.map((t) => {
    const own = doc.budget_ms?.[t] ?? DEFAULT_BUDGET_MS[t];
    // The view's budget is for frames it draws live; a still and the words have their own.
    const budget = t === "still" || t === "text" ? own : Math.min(own, ctx.budget_ms ?? Infinity);
    let ms: number | null = null;
    try {
      ms = estimateMs(plan(doc, ctx.library, t, 0), ctx.library, px, ctx.device);
    } catch {
      ms = null;
    }
    return { tier: t, ms, budget_ms: budget };
  });
}

export function applyEdits(original: RecipeDoc, edits: readonly RecipeEdit[], ctx: EditContext): EditResult {
  const doc = clone(original);
  const errors: string[] = [];
  const changes: string[] = [];
  const specOf = (useId: string) => {
    const use = doc.passes.find((u) => u.id === useId);
    return { use, spec: use ? ctx.library.find((p) => p.id === use.pass) : undefined };
  };
  for (const [i, ed] of edits.entries()) {
    const at = `edit ${i + 1}`;
    switch (ed?.op) {
      case "set":
      case "unset": {
        const { use, spec } = specOf(ed.use);
        if (!use || !spec) {
          errors.push(`${at}: the recipe has no pass ${ed.use}`);
          break;
        }
        const p = spec.params.find((q) => q.name === ed.param);
        if (!p) {
          errors.push(`${at}: ${spec.name} has no setting ${ed.param} (it has ${spec.params.map((q) => q.name).join(", ")})`);
          break;
        }
        const was = resolveParams(spec.params, use.params ?? {}).params[p.name] as Json;
        if (ed.op === "unset") {
          if (use.params) delete use.params[p.name];
          changes.push(`${spec.name} (${use.id}): ${p.name} back to its default ${fmt(p.default as Json)} (was ${fmt(was)})`);
          break;
        }
        const bad = resolveParams([p], { [p.name]: ed.value }).errors;
        if (bad.length) {
          errors.push(`${at}: ${bad.join("; ")}`);
          break;
        }
        // A knob that moves this setting would override it; say so rather than silently losing it.
        const knob = (doc.knobs ?? []).find((k) => k.maps.some((m) => m.pass === use.id && m.param === p.name));
        (use.params ??= {})[p.name] = ed.value;
        changes.push(`${spec.name} (${use.id}): ${p.name} ${fmt(was)} to ${fmt(ed.value)}${p.unit ? ` ${p.unit}` : ""}${knob ? ` (the ${knob.label} knob also moves this and wins while it is set)` : ""}`);
        break;
      }
      case "knob_default": {
        const k = (doc.knobs ?? []).find((x) => x.name === ed.knob);
        if (!k) {
          errors.push(`${at}: the recipe has no knob ${ed.knob}`);
          break;
        }
        if (!(typeof ed.value === "number" && ed.value >= 0 && ed.value <= 1)) {
          errors.push(`${at}: knob ${k.label} takes 0 to 1, not ${fmt(ed.value)}`);
          break;
        }
        changes.push(`${k.label} knob: default ${fmt(k.default)} to ${fmt(ed.value)}`);
        k.default = ed.value;
        break;
      }
      case "add": {
        const u = ed.use;
        if (!u?.id || !u.pass) {
          errors.push(`${at}: a pass to add needs an id and a pass`);
          break;
        }
        if (doc.passes.some((x) => x.id === u.id)) {
          errors.push(`${at}: the recipe already has a pass ${u.id}`);
          break;
        }
        const spec = ctx.library.find((p) => p.id === u.pass);
        if (!spec) {
          errors.push(`${at}: no pass ${u.pass} in the library (it has ${ctx.library.map((p) => p.id).join(", ")})`);
          break;
        }
        const j = ed.before ? doc.passes.findIndex((x) => x.id === ed.before) : -1;
        if (ed.before && j < 0) {
          errors.push(`${at}: no pass ${ed.before} to add before`);
          break;
        }
        doc.passes.splice(j < 0 ? doc.passes.length : j, 0, clone(u));
        changes.push(`Added ${spec.name} (${u.id})${ed.before ? ` before ${ed.before}` : ""}`);
        break;
      }
      case "remove": {
        const { use, spec } = specOf(ed.use);
        if (!use) {
          errors.push(`${at}: the recipe has no pass ${ed.use}`);
          break;
        }
        doc.passes = doc.passes.filter((x) => x.id !== use.id);
        const steps = doc.degrade.length;
        doc.degrade = doc.degrade.filter((st) => !(("drop" in st && st.drop === use.id) || ("set" in st && st.set.pass === use.id)));
        const maps = (doc.knobs ?? []).reduce((n, k) => n + k.maps.filter((m) => m.pass === use.id).length, 0);
        for (const k of doc.knobs ?? []) k.maps = k.maps.filter((m) => m.pass !== use.id);
        doc.knobs = (doc.knobs ?? []).filter((k) => k.maps.length);
        const also = [steps - doc.degrade.length ? `${steps - doc.degrade.length} reduction step(s)` : "", maps ? `${maps} knob link(s)` : ""].filter(Boolean);
        changes.push(`Removed ${spec?.name ?? use.pass} (${use.id})${also.length ? `, and the ${also.join(" and ")} that used it` : ""}`);
        break;
      }
      case "use_tiers": {
        const { use, spec } = specOf(ed.use);
        if (!use) {
          errors.push(`${at}: the recipe has no pass ${ed.use}`);
          break;
        }
        if (ed.tiers === null) delete use.tiers;
        else use.tiers = [...ed.tiers];
        changes.push(`${spec?.name ?? use.pass} (${use.id}) now runs on ${ed.tiers === null ? "every tier it can" : ed.tiers.join(", ") || "no tier"}`);
        break;
      }
      case "budget": {
        if (!TIER_ORDER.includes(ed.tier)) {
          errors.push(`${at}: no tier ${String(ed.tier)}`);
          break;
        }
        if (!(typeof ed.ms === "number" && ed.ms > 0)) {
          errors.push(`${at}: a budget is a positive number of ms`);
          break;
        }
        (doc.budget_ms ??= {})[ed.tier] = ed.ms;
        changes.push(`Frame budget on ${ed.tier}: ${fmt(ed.ms)} ms`);
        break;
      }
      case "name": {
        if (!ed.name?.trim()) {
          errors.push(`${at}: a name needs words`);
          break;
        }
        changes.push(`Named "${ed.name.trim()}"`);
        doc.name = ed.name.trim();
        if (ed.description?.trim()) doc.description = ed.description.trim();
        break;
      }
      default:
        errors.push(`${at}: unknown edit ${JSON.stringify((ed as { op?: unknown })?.op)}`);
    }
  }
  if (!edits.length) errors.push("no edits");
  doc.id = ctx.id ?? (original.derived_from ? original.id : `${original.id}-edited`);
  doc.derived_from = original.derived_from ?? original.id;
  doc.history = [...(original.history ?? []), ...changes];
  if (!errors.length) errors.push(...checkRecipe(doc, ctx.library).errors);
  const before = tierCosts(original, ctx);
  const after = errors.length ? [] : tierCosts(doc, ctx);
  const cost: TierCost[] = TIER_ORDER.filter((t) => original.tiers.includes(t) || doc.tiers.includes(t)).map((t) => {
    const b = before.find((x) => x.tier === t);
    const a = after.find((x) => x.tier === t);
    const budget = a?.budget_ms ?? b?.budget_ms ?? DEFAULT_BUDGET_MS[t];
    const ms = a?.ms ?? null;
    return { tier: t, before_ms: b?.ms ?? null, after_ms: ms, budget_ms: budget, fits: ms !== null && ms <= budget };
  });
  const newly_over = cost.filter((c) => c.before_ms !== null && c.before_ms <= c.budget_ms && c.after_ms !== null && c.after_ms > c.budget_ms).map((c) => c.tier);
  return { ok: !errors.length, doc, errors, changes, cost, newly_over };
}

/** The settings an editor (or Orac) may change, per pass use, with current values: the menu of `set` edits. */
export function editableSettings(doc: RecipeDoc, library: readonly PassSpec[]) {
  return doc.passes.map((use) => {
    const spec = library.find((p) => p.id === use.pass);
    const values = spec ? resolveParams(spec.params, use.params ?? {}).params : {};
    return {
      use: use.id,
      pass: use.pass,
      name: spec?.name ?? use.pass,
      tiers: use.tiers ?? spec?.tiers.filter((t) => doc.tiers.includes(t)) ?? [],
      settings: (spec?.params ?? []).map((p) => ({ ...p, value: values[p.name] as Json, knob: (doc.knobs ?? []).find((k) => k.maps.some((m) => m.pass === use.id && m.param === p.name))?.label })),
    };
  });
}
