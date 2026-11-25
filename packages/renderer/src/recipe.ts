// Render recipes (R1, SSE4-surfaces-and-rendering.md section 5, handoff 8.9).
// A pass is code: small, finite, shipped and tested, and it publishes what it
// reads, which tiers it runs on, what it costs and a parameter schema. A
// recipe is data: a list of passes with parameter values, the tiers it
// declares, a few human knobs that move many deep parameters at once, and a
// degrade order the governor walks when a device misses its frame budget.
// Recipes are view configuration: they never reach the kernel and never
// change the state hash.
import { resolveParams, type Json, type ParamSpec, type Tier } from "../../ui-contract/src/index.ts";

/** Tiers from richest to plainest. A degrade step may only move down this list. */
export const TIER_ORDER: readonly Tier[] = ["still", "gpu-ultra", "gpu-high", "gpu-basic", "canvas", "text"];

/** What one pass costs per frame on the reference device, in milliseconds. */
export interface PassCost {
  fixed_ms: number;
  /** Per million output pixels, at render scale 1. */
  per_mpix_ms: number;
  /** Numeric parameters the cost grows with: cost × (value / ref) ^ power. */
  scale?: { param: string; ref: number; power: number }[];
  /**
   * What bounds the pass on the GPU, as shares of the speed probe's three scenes (probe.ts): texture reads and
   * pixels (`fill`), loops per pixel (`march`), triangles (`geometry`). Shares are normalised; absent: all fill.
   * The probe's device factors scale the cost by this mix, so a device slow at marching but quick at filling
   * is costed fairly for each pass.
   */
  load?: Partial<Record<ProbeScene, number>>;
}

/** The speed probe's three scenes (probe.ts); a device factor is measured for each. */
export type ProbeScene = "fill" | "march" | "geometry";

/** How many times the reference device's time this device takes for each probe scene (1: as fast; 2: half as fast). */
export type DeviceFactors = Record<ProbeScene, number>;

/** One pass's device factor: its load mix weighted over the probe's factors. */
export function passFactor(c: PassCost, f: DeviceFactors): number {
  const l = c.load ?? { fill: 1 };
  let w = 0;
  let x = 0;
  for (const k of ["fill", "march", "geometry"] as const) {
    const s = Math.max(0, l[k] ?? 0);
    w += s;
    x += s * f[k];
  }
  return w > 0 ? x / w : f.fill;
}

export interface PassSpec {
  id: string;
  version: string;
  name: string;
  meaning: string;
  /** "image" draws the picture, "overlay" draws over it, "text" writes it out. */
  kind: "image" | "overlay" | "text";
  /** Scene fields it reads, e.g. "surface.tiles", "body.label". */
  inputs: string[];
  tiers: Tier[];
  params: ParamSpec[];
  /** Cost per tier it runs on. Reference figures are estimates until a probe measures the device. */
  cost: Partial<Record<Tier, PassCost>>;
  references?: { text: string; doi?: string; url?: string }[];
}

export interface RecipePass {
  /** This use of the pass, unique in the recipe (a recipe may use a pass twice). */
  id: string;
  pass: string;
  params?: Record<string, Json>;
  /** Tiers this use runs on; absent means every tier of the recipe the pass supports. */
  tiers?: Tier[];
}

/** A human control, 0 to 1, that moves deep parameters together (linearly from `at0` to `at1`). */
export interface Knob {
  name: string;
  label: string;
  meaning: string;
  default: number;
  maps: { pass: string; param: string; at0: number; at1: number }[];
}

/** One reduction the governor may apply, in order. `says` is what the quality badge tells the user. */
export type DegradeStep = { says: string; tiers?: Tier[] } & ({ render_scale: number } | { set: { pass: string; param: string; value: Json } } | { drop: string } | { tier: Tier });

/** The scenes recipes draw this release. */
export const RECIPE_SCENES = ["body", "system", "land"] as const;
export type RecipeScene = (typeof RECIPE_SCENES)[number];

export interface RecipeDoc {
  schema: "sse.render-recipe/1";
  id: string;
  name: string;
  /** Recipes of one family stand in for each other when a device cannot run the one asked for. */
  family: string;
  description: string;
  /** The scene it draws ("body": one world's surface atlas; "system": a system in True, its worlds and star; "land": the ground round a site). */
  scene: RecipeScene;
  tiers: Tier[];
  passes: RecipePass[];
  knobs?: Knob[];
  degrade: DegradeStep[];
  /** Frame budget per tier in ms; the view may lower it. */
  budget_ms?: Partial<Record<Tier, number>>;
  /** What would lift a reduction, in words for the badge. */
  lift?: string;
  /** Custom shader code. Not run in this release (shared recipes are data-only, decision 7). */
  contains_code?: boolean;
  /** The recipe this one was edited from (R4), and the edits in words, newest last. */
  derived_from?: string;
  history?: string[];
}

/** Default frame budgets: 60 a second on the high and ultra GPU tiers, 30 elsewhere. */
export const DEFAULT_BUDGET_MS: Readonly<Record<Tier, number>> = { still: 1000, "gpu-ultra": 16.7, "gpu-high": 16.7, "gpu-basic": 33.3, canvas: 33.3, text: 1000 };

export interface RecipeCheck {
  ok: boolean;
  errors: string[];
}

const isTier = (t: unknown): t is Tier => typeof t === "string" && (TIER_ORDER as readonly string[]).includes(t);

/** Every problem with a recipe against a pass library. Nothing runs unless all pass. */
export function checkRecipe(doc: RecipeDoc, library: readonly PassSpec[], opts: { allowCode?: boolean } = {}): RecipeCheck {
  const errors: string[] = [];
  const e = (s: string) => errors.push(s);
  if (doc?.schema !== "sse.render-recipe/1") e(`schema must be sse.render-recipe/1, not ${String(doc?.schema)}`);
  for (const k of ["id", "name", "family", "description"] as const) if (typeof doc?.[k] !== "string" || !doc[k]) e(`${k} must be non-empty text`);
  if (!(RECIPE_SCENES as readonly string[]).includes(doc?.scene)) e(`scene ${String(doc?.scene)} is not one this release draws (${RECIPE_SCENES.join(", ")})`);
  if (doc?.contains_code && !opts.allowCode) e("contains code: shared recipes are data only, and code-bearing recipes are not run in this release");
  const tiers = Array.isArray(doc?.tiers) ? doc.tiers : [];
  if (!tiers.length) e("declares no tiers");
  for (const t of tiers) if (!isTier(t)) e(`unknown tier ${String(t)}`);
  if (new Set(tiers).size !== tiers.length) e("a tier is declared twice");
  const passes = Array.isArray(doc?.passes) ? doc.passes : [];
  const byId = new Map<string, { use: RecipePass; spec: PassSpec }>();
  for (const use of passes) {
    if (!use?.id) {
      e("a pass has no id");
      continue;
    }
    if (byId.has(use.id)) e(`pass id ${use.id} is used twice`);
    const spec = library.find((p) => p.id === use.pass);
    if (!spec) {
      e(`${use.id}: no pass ${use.pass} in the library`);
      continue;
    }
    byId.set(use.id, { use, spec });
    for (const er of resolveParams(spec.params, use.params ?? {}).errors) e(`${use.id}: ${er}`);
    for (const t of use.tiers ?? []) {
      if (!tiers.includes(t)) e(`${use.id}: runs on ${t}, which the recipe does not declare`);
      if (!spec.tiers.includes(t)) e(`${use.id}: pass ${spec.id} has no ${t} form`);
    }
  }
  // Every declared tier must have something to show.
  for (const t of tiers.filter(isTier)) {
    const here = [...byId.values()].filter((x) => runsOn(x.use, x.spec, t));
    const need = t === "text" ? "text" : "image";
    if (!here.some((x) => x.spec.kind === need)) e(`tier ${t}: no ${need} pass runs there, so the recipe would show nothing`);
  }
  // Knobs move numeric parameters within their range.
  const knobs = doc?.knobs ?? [];
  const knobNames = new Set<string>();
  for (const k of knobs) {
    if (!k?.name || !k.label || !k.meaning) e("a knob needs a name, a label and a meaning");
    if (knobNames.has(k.name)) e(`knob ${k.name} is defined twice`);
    knobNames.add(k.name);
    if (!(typeof k.default === "number" && k.default >= 0 && k.default <= 1)) e(`knob ${k.name}: default must be within 0 to 1`);
    if (!k.maps?.length) e(`knob ${k.name} moves nothing`);
    for (const m of k.maps ?? []) {
      const x = byId.get(m.pass);
      const p = x?.spec.params.find((q) => q.name === m.param);
      if (!x) e(`knob ${k.name}: no pass ${m.pass} in the recipe`);
      else if (!p) e(`knob ${k.name}: pass ${m.pass} has no parameter ${m.param}`);
      else if (p.type !== "number" && p.type !== "integer") e(`knob ${k.name}: ${m.pass}.${m.param} is not a number`);
      else if (p.range && [m.at0, m.at1].some((v) => !(v >= p.range![0] && v <= p.range![1]))) e(`knob ${k.name}: ${m.pass}.${m.param} would leave its range ${p.range[0]} to ${p.range[1]}`);
    }
  }
  // The degrade order: each step known, in range, and never leaving a tier with nothing to show.
  let scale = 1;
  const dropped = new Set<string>();
  for (const [i, st] of (doc?.degrade ?? []).entries()) {
    const at = `degrade step ${i + 1}`;
    if (!st?.says) e(`${at}: says nothing (the quality badge needs words)`);
    for (const t of st?.tiers ?? []) if (!tiers.includes(t)) e(`${at}: applies on ${t}, which the recipe does not declare`);
    if ("render_scale" in st) {
      if (!(st.render_scale >= 0.25 && st.render_scale <= 1)) e(`${at}: render scale must be within 0.25 to 1`);
      else if (st.render_scale > scale) e(`${at}: render scale goes up (${scale} to ${st.render_scale}); a degrade order only reduces`);
      else scale = st.render_scale;
    } else if ("set" in st) {
      const x = byId.get(st.set.pass);
      const p = x?.spec.params.find((q) => q.name === st.set.param);
      if (!x) e(`${at}: no pass ${st.set.pass} in the recipe`);
      else if (!p) e(`${at}: pass ${st.set.pass} has no parameter ${st.set.param}`);
      else for (const er of resolveParams([p], { [p.name]: st.set.value }).errors) e(`${at}: ${er}`);
    } else if ("drop" in st) {
      const x = byId.get(st.drop);
      if (!x) e(`${at}: no pass ${st.drop} to drop`);
      else {
        dropped.add(st.drop);
        for (const t of (st.tiers ?? tiers).filter(isTier)) {
          const left = [...byId.values()].filter((y) => !dropped.has(y.use.id) && runsOn(y.use, y.spec, t));
          if (!left.some((y) => y.spec.kind === (t === "text" ? "text" : "image"))) e(`${at}: dropping ${st.drop} leaves tier ${t} with nothing to show`);
        }
      }
    } else if ("tier" in st) {
      if (!tiers.includes(st.tier)) e(`${at}: falls to tier ${st.tier}, which the recipe does not declare`);
      for (const t of st.tiers ?? []) if (TIER_ORDER.indexOf(st.tier) <= TIER_ORDER.indexOf(t)) e(`${at}: from ${t} to ${st.tier} is not a step down`);
      if (!st.tiers?.length) e(`${at}: a tier step must say which tiers it leaves (tiers)`);
    } else e(`${at}: unknown kind of step`);
  }
  for (const [t, ms] of Object.entries(doc?.budget_ms ?? {})) {
    if (!isTier(t)) e(`budget for unknown tier ${t}`);
    if (!(typeof ms === "number" && ms > 0)) e(`budget for ${t} must be a positive number of ms`);
  }
  return { ok: !errors.length, errors };
}

function runsOn(use: RecipePass, spec: PassSpec, t: Tier): boolean {
  return spec.tiers.includes(t) && (!use.tiers || use.tiers.includes(t));
}

/** A recipe made concrete for one tier, knob setting and degrade level: what actually runs. */
export interface Plan {
  recipe: string;
  /** The tier asked for and the tier drawn (a degrade step may fall to a plainer one). */
  asked: Tier;
  tier: Tier;
  level: number;
  render_scale: number;
  passes: { id: string; pass: string; kind: PassSpec["kind"]; params: Record<string, Json> }[];
  /** What each applied degrade step says, in order. */
  reductions: string[];
}

/** The steps that apply from `tier` downwards, in order, following any tier falls. */
export function stepsFrom(doc: RecipeDoc, tier: Tier): DegradeStep[] {
  const out: DegradeStep[] = [];
  let t = tier;
  for (const st of doc.degrade) {
    if (st.tiers && !st.tiers.includes(t)) continue;
    out.push(st);
    if ("tier" in st) t = st.tier;
  }
  return out;
}

/** How many degrade levels a recipe has from a tier. */
export const maxLevel = (doc: RecipeDoc, tier: Tier) => stepsFrom(doc, tier).length;

/** The concrete plan. Knob values are 0 to 1 (absent: the knob's default). Throws if the recipe does not declare the tier. */
export function plan(doc: RecipeDoc, library: readonly PassSpec[], tier: Tier, level = 0, knobs: Record<string, number> = {}): Plan {
  if (!doc.tiers.includes(tier)) throw new Error(`recipe ${doc.id} does not declare tier ${tier}`);
  const params = new Map<string, Record<string, Json>>();
  for (const use of doc.passes) {
    const spec = library.find((p) => p.id === use.pass)!;
    params.set(use.id, resolveParams(spec.params, use.params ?? {}).params);
  }
  for (const k of doc.knobs ?? []) {
    const v = Math.min(1, Math.max(0, knobs[k.name] ?? k.default));
    for (const m of k.maps) {
      const spec = library.find((p) => p.id === doc.passes.find((u) => u.id === m.pass)!.pass)!;
      const p = spec.params.find((q) => q.name === m.param)!;
      const x = m.at0 + (m.at1 - m.at0) * v;
      params.get(m.pass)![m.param] = p.type === "integer" ? Math.round(x) : x;
    }
  }
  let t = tier;
  let scale = 1;
  const dropped = new Set<string>();
  const reductions: string[] = [];
  const steps = stepsFrom(doc, tier);
  const lv = Math.max(0, Math.min(level, steps.length));
  for (const st of steps.slice(0, lv)) {
    reductions.push(st.says);
    if ("render_scale" in st) scale = st.render_scale;
    else if ("set" in st) params.get(st.set.pass)![st.set.param] = st.set.value;
    else if ("drop" in st) dropped.add(st.drop);
    else if ("tier" in st) t = st.tier;
  }
  const passes = doc.passes
    .filter((use) => !dropped.has(use.id))
    .map((use) => ({ use, spec: library.find((p) => p.id === use.pass)! }))
    .filter(({ use, spec }) => runsOn(use, spec, t))
    .map(({ use, spec }) => ({ id: use.id, pass: spec.id, kind: spec.kind, params: params.get(use.id)! }));
  return { recipe: doc.id, asked: tier, tier: t, level: lv, render_scale: t === "text" ? 1 : scale, passes, reductions };
}

/**
 * Estimated milliseconds a frame of the plan takes at `pixels` output pixels
 * (render scale applied here), with a per-tier device factor (absent: the
 * reference device). `factors` from the speed probe (probe.ts) scale each GPU
 * pass by its own load mix instead; the canvas and text tiers run on the
 * processor and are not scaled by them.
 */
export function estimateMs(p: Plan, library: readonly PassSpec[], pixels: number, device: Partial<Record<Tier, number>> = {}, factors?: DeviceFactors | null): number {
  const mpix = (pixels * p.render_scale * p.render_scale) / 1e6;
  let ms = 0;
  for (const run of p.passes) {
    const c = library.find((s) => s.id === run.pass)?.cost[p.tier];
    if (!c) continue;
    let f = 1;
    for (const s of c.scale ?? []) {
      const v = run.params[s.param];
      if (typeof v === "number" && s.ref > 0) f *= Math.pow(Math.max(v, 1e-9) / s.ref, s.power);
    }
    const gpu = factors && p.tier !== "canvas" && p.tier !== "text" ? passFactor(c, factors) : 1;
    ms += (c.fixed_ms + c.per_mpix_ms * mpix) * f * gpu;
  }
  return ms * (device[p.tier] ?? 1);
}

/**
 * The recipe to run for a tier: the one asked for if it declares the tier,
 * else the first of its family that does (and why). Null when none can.
 */
export function pickRecipe(registry: readonly RecipeDoc[], wanted: string, tier: Tier): { doc: RecipeDoc; stood_in?: string } | null {
  const w = registry.find((r) => r.id === wanted);
  if (w && w.tiers.includes(tier)) return { doc: w };
  const fam = w?.family;
  const alt = registry.find((r) => r.family === fam && r.tiers.includes(tier));
  if (alt) return { doc: alt, stood_in: w ? `${w.name} has no ${tier} form, so ${alt.name} stands in` : `no recipe ${wanted}; ${alt.name} stands in` };
  return null;
}

/** The richest tier a recipe declares that the device can draw. */
export function bestTier(doc: RecipeDoc, available: readonly Tier[]): Tier | null {
  return TIER_ORDER.find((t) => doc.tiers.includes(t) && available.includes(t)) ?? null;
}
