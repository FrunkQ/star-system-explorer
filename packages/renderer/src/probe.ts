// The speed probe and tier placement (VP4, SSE4-visual-polish-design.md 8.1 to 8.3; SSE4-surfaces-and-rendering.md
// 6.2). A device is placed by what it measures, not by which graphics layer it has (Alex, 24 Nov: a fast WebGL2
// phone can earn High). Three checks feed one placement:
//
// 1. Capabilities, read on every page load: which passes *can* run (a pass whose needs are unmet does not exist).
// 2. The probe, about half a second off screen on a first visit and again when the adapter or browser changes:
//    three tiny scenes at one megapixel (fill, march, geometry), each timed, giving three device factors against
//    the reference device. They scale every pass's declared cost by its own load mix (recipe.ts `PassCost.load`).
//    The GPU side is probe-gpu.ts; this file is pure and clock-free so it can be tested anywhere.
// 3. Placement: the richest tier whose standard recipe is predicted within 60% of its frame budget (the
//    governor's own "well under" margin). The governor then walks the degrade order on real frame times, and the
//    heat watch lowers the ceiling for the session when a hot device keeps stepping down.
//
// Presentation only: nothing here reaches the kernel or the state hash, and nothing is sent anywhere.
import type { Tier } from "../../ui-contract/src/index.ts";
import { DEFAULT_BUDGET_MS, estimateMs, plan, TIER_ORDER, type DeviceFactors, type PassSpec, type ProbeScene, type RecipeDoc } from "./recipe.ts";

export type { DeviceFactors, ProbeScene } from "./recipe.ts";

/** What the device's graphics can do, read without drawing anything. */
export interface Capabilities {
  backend: "webgpu" | "webgl2" | null;
  /** Compute shaders (WebGPU only). */
  compute: boolean;
  /** Half-float colour targets (high-range pictures for the finish). */
  half_float: boolean;
  textures_3d: boolean;
  /** Largest 2D texture side, pixels. */
  max_texture: number;
  /** GPU timestamp queries (the probe then times the GPU itself, not frames). */
  timestamps: boolean;
  /** The browser's memory hint, GB (null: not given). */
  memory_gb: number | null;
  /** Screen pixels at the device's pixel ratio. */
  screen_px: number;
  pixel_ratio: number;
  reduced_motion: boolean;
  save_data: boolean;
  /** The adapter in words, as the browser gives it ("" when it gives nothing). */
  adapter: string;
}

/** The three scenes, as drawn on the reference device. */
export const PROBE_SCENES: Readonly<Record<ProbeScene, { name: string; meaning: string; reference_ms: number }>> = {
  fill: { name: "Fill", meaning: "One megapixel, sixteen texture reads a pixel: how fast the device paints and samples.", reference_ms: 0.4 },
  march: { name: "March", meaning: "One megapixel, a 24-step ray march through a noise density: how fast it loops per pixel (air, clouds, nebulae).", reference_ms: 1.2 },
  geometry: { name: "Geometry", meaning: "300,000 lit triangles: how fast it builds shapes (globes, rings, ships).", reference_ms: 0.5 },
};

/** The probe's own frame size: one megapixel. */
export const PROBE_SIDE = 1024;

/** "Well under": a tier is chosen only when its standard frame is predicted within this share of the budget. */
export const PLACE_MARGIN = 0.6;

/**
 * Until a recipe declares an Ultra form, Ultra is earned by headroom: the High standard frame predicted within this
 * share of the budget, so the device has twice the room a High placement needs. A tuned rule, in docs/fudges.md;
 * an Ultra recipe's own prediction replaces it as soon as one ships.
 */
export const ULTRA_HEADROOM = 0.3;

/** The heat watch: this many steps down inside this long lowers the ceiling for the session. */
export const HEAT_STEPS = 3;
export const HEAT_WINDOW_MS = 60_000;

/** What a probe measured, cached on the device (sse.render-probe/1). */
export interface ProbeResult {
  schema: "sse.render-probe/1";
  /** Adapter and browser: a new key means measure again. */
  key: string;
  backend: "webgpu" | "webgl2";
  /** "timestamp": the GPU's own clock; "readback": frames closed by a one-pixel read, an empty frame taken off. */
  method: "timestamp" | "readback";
  /** Median ms for each scene at one megapixel. */
  scenes_ms: Record<ProbeScene, number>;
  factors: DeviceFactors;
  /** False when a scene's runs spread widely (a pause on the device): placement still uses it, the readout says so. */
  steady: boolean;
}

/** Device factors from scene times: measured over reference, floored so a timer's grain cannot read as infinitely fast. */
export function factorsFrom(scenes_ms: Record<ProbeScene, number>): DeviceFactors {
  const f = (k: ProbeScene) => Math.max(0.05, (Number.isFinite(scenes_ms[k]) ? scenes_ms[k] : Infinity) / PROBE_SCENES[k].reference_ms);
  return { fill: f("fill"), march: f("march"), geometry: f("geometry") };
}

/** The key a cached probe is kept under: the adapter and the browser. */
export function probeKey(caps: Pick<Capabilities, "backend" | "adapter">, userAgent: string): string {
  return [caps.backend ?? "none", caps.adapter || "unknown adapter", userAgent].join("|");
}

export const PROBE_STORE = "sse4.render.probe";

/** A cached probe, if it is for this adapter and browser. */
export function loadProbe(store: Pick<Storage, "getItem"> | null | undefined, key: string): ProbeResult | null {
  try {
    const r = JSON.parse(store?.getItem(PROBE_STORE) ?? "null") as ProbeResult | null;
    if (r?.schema !== "sse.render-probe/1" || r.key !== key) return null;
    const ok = (["fill", "march", "geometry"] as const).every((k) => r.factors[k] > 0 && Number.isFinite(r.factors[k]));
    return ok ? r : null;
  } catch {
    return null;
  }
}

export function saveProbe(store: Pick<Storage, "setItem"> | null | undefined, r: ProbeResult): void {
  try {
    store?.setItem(PROBE_STORE, JSON.stringify(r));
  } catch {
    // Private windows and full storage: the probe runs again next visit.
  }
}

export function forgetProbe(store: Pick<Storage, "removeItem"> | null | undefined): void {
  try {
    store?.removeItem(PROBE_STORE);
  } catch {
    // Nothing kept.
  }
}

/** The person's quality choice (More › Quality). Auto lets the probe place the device. */
export type QualityChoice = "auto" | "light" | "full" | "ultra";

export const QUALITY_STORE = "sse4.render.quality";

export const QUALITY_WORDS: Readonly<Record<QualityChoice, string>> = { auto: "Auto", light: "Light", full: "Full", ultra: "Ultra" };

export function loadQuality(store: Pick<Storage, "getItem"> | null | undefined): QualityChoice {
  try {
    const v = store?.getItem(QUALITY_STORE);
    return v === "light" || v === "full" || v === "ultra" ? v : "auto";
  } catch {
    return "auto";
  }
}

export function saveQuality(store: Pick<Storage, "setItem"> | null | undefined, q: QualityChoice): void {
  try {
    store?.setItem(QUALITY_STORE, q);
  } catch {
    // Kept for this page only.
  }
}

const GPU_TIERS: readonly Tier[] = ["gpu-ultra", "gpu-high", "gpu-basic"];

/** The GPU tiers this device's capabilities allow at all, richest first (before any speed is known). */
export function gpuTiersAllowed(caps: Pick<Capabilities, "backend" | "compute">): Tier[] {
  if (!caps.backend) return [];
  return GPU_TIERS.filter((t) => t !== "gpu-ultra" || (caps.backend === "webgpu" && caps.compute));
}

export interface TierPrediction {
  tier: Tier;
  /** Predicted ms for the standard frame at this tier (null: the recipe has no form there). */
  ms: number | null;
  budget_ms: number;
  fits: boolean;
}

export interface Placement {
  /** Where to start: the richest tier that fits, within the ceiling. */
  tier: Tier;
  /** The richest tier this device is allowed this session (the heat watch may lower it). */
  ceiling: Tier;
  /** The device's own class: the richest GPU tier its speed earns, ignoring the person's choice and the heat. */
  earned: Tier;
  /** How the start was decided. */
  by: "probe" | "capabilities" | "choice";
  predictions: TierPrediction[];
  /** One plain line for the badge and the quality card. */
  note: string;
}

export interface PlaceOptions {
  caps: Pick<Capabilities, "backend" | "compute">;
  /** Null: not measured yet (the device starts on the basic GPU tier and is placed again when the probe returns). */
  probe: Pick<ProbeResult, "factors"> | null;
  /** The view's output pixels (before render scale). */
  pixels: number;
  /** The standard recipe for this view (system True, the Body globe's true colour, ...). */
  recipe: RecipeDoc;
  library: readonly PassSpec[];
  /** Frame budgets per tier (absent: the recipe's, then the defaults). */
  budgets?: Partial<Record<Tier, number>>;
  choice?: QualityChoice;
  /** A lowered ceiling from the heat watch. */
  heat_ceiling?: Tier | null;
}

const rank = (t: Tier) => TIER_ORDER.indexOf(t);
const poorer = (a: Tier, b: Tier) => (rank(a) >= rank(b) ? a : b);

/** Predicted standard frame at a tier, ms (null when the recipe has no form there). */
function predict(o: PlaceOptions, tier: Tier): number | null {
  if (!o.recipe.tiers.includes(tier)) return null;
  return estimateMs(plan(o.recipe, o.library, tier), o.library, o.pixels, {}, o.probe?.factors ?? null);
}

const budgetOf = (o: PlaceOptions, t: Tier) => o.budgets?.[t] ?? o.recipe.budget_ms?.[t] ?? DEFAULT_BUDGET_MS[t];

/**
 * Place the device for one view: the richest tier whose standard frame is predicted within 60% of its budget.
 * WebGL2 may earn High by speed; Ultra needs WebGPU with compute as well as the headroom.
 */
export function place(o: PlaceOptions): Placement {
  const allowed = gpuTiersAllowed(o.caps);
  const choice = o.choice ?? "auto";
  if (!allowed.length) {
    const t: Tier = o.recipe.tiers.includes("canvas") ? "canvas" : "text";
    return { tier: t, ceiling: t, earned: t, by: "capabilities", predictions: [], note: "No WebGPU or WebGL2 here, so this is drawn on a 2D canvas." };
  }
  const predictions: TierPrediction[] = allowed.map((tier) => {
    const ms = predict(o, tier);
    const budget_ms = budgetOf(o, tier);
    return { tier, ms, budget_ms, fits: ms !== null && ms <= budget_ms * PLACE_MARGIN };
  });
  // The class the speed earns. Unmeasured: basic until the probe returns.
  let earned: Tier = "gpu-basic";
  if (o.probe) {
    const high = predictions.find((p) => p.tier === "gpu-high");
    const ultra = predictions.find((p) => p.tier === "gpu-ultra");
    if (high?.fits) earned = "gpu-high";
    if (ultra) {
      // An Ultra recipe speaks for itself; until one exists, Ultra is earned by headroom over High.
      const u = ultra.ms !== null ? ultra.fits : high?.ms !== null && high !== undefined && high.ms <= high.budget_ms * ULTRA_HEADROOM;
      if (u) earned = "gpu-ultra";
    }
  }
  let ceiling = o.heat_ceiling ? poorer(earned, o.heat_ceiling) : earned;
  let by: Placement["by"] = o.probe ? "probe" : "capabilities";
  if (choice !== "auto") {
    const want: Tier = choice === "light" ? "gpu-basic" : choice === "full" ? "gpu-high" : "gpu-ultra";
    ceiling = allowed.includes(want) ? want : allowed[0]!;
    by = "choice";
  }
  // Start at the richest tier within the ceiling that the recipe declares and (on Auto) is predicted to fit.
  const within = TIER_ORDER.filter((t) => t !== "still" && rank(t) >= rank(ceiling) && o.recipe.tiers.includes(t));
  const fitting = by === "choice" ? within : within.filter((t) => !GPU_TIERS.includes(t) || t === "gpu-basic" || predictions.find((p) => p.tier === t)?.fits);
  const tier = fitting[0] ?? within[0] ?? (o.recipe.tiers.includes("canvas") ? "canvas" : "text");
  return { tier, ceiling, earned, by, predictions, note: placementNote({ tier, ceiling, earned, by, choice, heated: !!o.heat_ceiling && rank(o.heat_ceiling) > rank(earned), backend: o.caps.backend!, measured: !!o.probe }) };
}

const TIER_WORDS: Readonly<Record<Tier, string>> = { still: "Photograph", "gpu-ultra": "Ultra", "gpu-high": "High", "gpu-basic": "Light", canvas: "2D canvas", text: "Words only" };

/** A tier in the words the quality card uses. */
export const tierWords = (t: Tier) => TIER_WORDS[t];

function placementNote(x: { tier: Tier; ceiling: Tier; earned: Tier; by: Placement["by"]; choice: QualityChoice; heated: boolean; backend: "webgpu" | "webgl2"; measured: boolean }): string {
  const layer = x.backend === "webgpu" ? "WebGPU" : "WebGL2";
  if (x.by === "choice") {
    const extra = x.tier !== x.ceiling ? ` This view has no ${tierWords(x.ceiling)} form, so it is drawn at ${tierWords(x.tier)}.` : "";
    const over = rank(x.ceiling) < rank(x.earned) ? " That is above what this device measured, so frames may be slow." : "";
    return `Quality set to ${QUALITY_WORDS[x.choice]} by you, not by this device's measured speed.${over}${extra}`;
  }
  if (!x.measured) return `Starting at Light on ${layer} while this device's speed is measured (about half a second, once).`;
  const heat = x.heated ? ` Lowered to ${tierWords(x.ceiling)} for this visit: the device kept slowing down, as a hot device does.` : "";
  const fit = x.tier !== x.ceiling ? ` This view starts at ${tierWords(x.tier)}, the richest form predicted to fit its frame budget.` : "";
  return `Placed at ${tierWords(x.earned)} by this device's measured speed on ${layer}.${heat}${fit}`;
}

/**
 * The heat watch: a device that keeps stepping down (a phone heating up) stops trying the richer tier for the rest
 * of the visit. Pure: the caller passes its own clock with each step.
 */
export class HeatWatch {
  #downs: number[] = [];
  ceiling: Tier | null = null;

  /** Note one governor step down at `now_ms`, drawn at `tier`. Returns the new ceiling when it drops, else null. */
  down(now_ms: number, tier: Tier): Tier | null {
    this.#downs = this.#downs.filter((t) => now_ms - t < HEAT_WINDOW_MS);
    this.#downs.push(now_ms);
    if (this.#downs.length < HEAT_STEPS) return null;
    this.#downs = [];
    const i = GPU_TIERS.indexOf(tier);
    if (i < 0 || i === GPU_TIERS.length - 1) return null;
    const next = GPU_TIERS[i + 1]!;
    if (this.ceiling && rank(this.ceiling) >= rank(next)) return null;
    this.ceiling = next;
    return next;
  }
}

/** The probe and placement as one row of the bench report (the "Drawing" rows of docs/minimum-spec.md). */
export interface DrawingBench {
  backend: "webgpu" | "webgl2" | "none";
  adapter: string;
  method: ProbeResult["method"] | "none";
  scenes_ms: Partial<Record<ProbeScene, number>>;
  factors: Partial<DeviceFactors>;
  steady: boolean;
  /** The device's class at 1080p (1920 × 1080) for the standard True recipe, and each tier's prediction there. */
  earned: Tier;
  predictions: TierPrediction[];
}

export const BENCH_PIXELS = 1920 * 1080;

export function drawingBench(caps: Capabilities, probe: ProbeResult | null, recipe: RecipeDoc, library: readonly PassSpec[]): DrawingBench {
  const p = place({ caps, probe, pixels: BENCH_PIXELS, recipe, library });
  return {
    backend: caps.backend ?? "none",
    adapter: caps.adapter,
    method: probe?.method ?? "none",
    scenes_ms: probe?.scenes_ms ?? {},
    factors: probe?.factors ?? {},
    steady: probe?.steady ?? true,
    earned: p.earned,
    predictions: p.predictions,
  };
}
