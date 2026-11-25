// One controller per True view (VP4, SSE4-visual-polish-design.md 8.1 to 8.3): system True, Land, ships and
// stations ask it what to draw at, and tell it how long each frame took. It places the view by the device's measured
// speed (probe.ts `place`), walks the recipe's degrade order with the governor on real frame times, lowers the
// ceiling for the visit when the heat watch sees a hot device, and gives the quality badge its words. Sharpen at rest
// (still.ts) is scheduled here too, so a view needs one object for all of its quality.
//
// The view keeps drawing; this only says how. Presentation only: nothing here reaches the kernel or the state hash.
import type { Tier } from "../../ui-contract/src/index.ts";
import { perf } from "../../ui-contract/src/index.ts";
import { badge, Governor, type Badge } from "./governor.ts";
import { DEFAULT_BUDGET_MS, estimateMs, maxLevel, plan, type PassSpec, type Plan, type RecipeDoc } from "./recipe.ts";
import { place, type Capabilities, type Placement, type ProbeResult, type QualityChoice } from "./probe.ts";
import { REST_FRAMES, RestClock, type RestState } from "./still.ts";
import * as dev from "./device.ts";

/** What the view needs to draw a frame. */
export interface TrueSettings {
  tier: Tier;
  /** Fraction of the view's pixels drawn (the rest enlarged). */
  render_scale: number;
  /** The plan's passes' parameters by use id (e.g. `finish.glare_rungs`, `bodies.sphere_detail`). */
  params: Record<string, Record<string, unknown>>;
}

export interface TrueStatus {
  tier: Tier;
  level: number;
  max: number;
  badge: Badge;
  /** Why this tier: the placement's own words. */
  note: string;
  ewma_ms: number;
  budget_ms: number;
  estimate_ms: number;
  rest: RestState;
  rest_frames: number;
}

/** The device, as the controller reads it: device.ts by default, a stand-in in tests. */
export interface TrueDevice {
  caps(): Promise<Capabilities>;
  speed(caps: Capabilities): Promise<ProbeResult | null>;
  quality(): QualityChoice;
  heatCeiling(): Tier | null;
  heatDown(now: number, tier: Tier): Tier | null;
  onChange(fn: () => void): () => void;
}

export const DEVICE: TrueDevice = {
  caps: dev.deviceCaps,
  speed: dev.deviceSpeed,
  quality: dev.quality,
  heatCeiling: () => dev.heat.ceiling,
  heatDown: (now, tier) => dev.heat.down(now, tier),
  onChange: dev.onPlacementChange,
};

export interface TrueRunOptions {
  recipe: RecipeDoc;
  library: readonly PassSpec[];
  /** The view's output pixels (width × height at the device's pixel ratio). */
  pixels: number;
  /** The view's backend, once it has one ("webgpu" or "webgl2"; null: none). */
  backend: "webgpu" | "webgl2" | null;
  /** A forced tier (a recipe pass or a test); absent: placed by the device. */
  tier?: Tier;
  budget_ms?: number;
  /** The readout source for this view's frames (e.g. "render.system"). */
  source: string;
  /** Told when the settings change (a new placement, a governor step, the person's choice). */
  onChange?: () => void;
  device?: TrueDevice;
}

export class TrueRun {
  #o: TrueRunOptions;
  #dev: TrueDevice;
  #placed: Placement | null = null;
  #tier: Tier = "gpu-basic";
  #gov: Governor;
  #plan: Plan;
  #unsub: () => void;
  #disposed = false;
  readonly rest = new RestClock({ frames: 0 });

  constructor(o: TrueRunOptions) {
    this.#o = o;
    this.#dev = o.device ?? DEVICE;
    this.#tier = o.tier ?? (o.recipe.tiers.includes("gpu-basic") ? "gpu-basic" : o.recipe.tiers[0]!);
    this.#gov = this.#newGovernor();
    this.#plan = plan(o.recipe, o.library, this.#tier, 0);
    this.#unsub = this.#dev.onChange(() => void this.place());
  }

  #budget(t: Tier) {
    return this.#o.budget_ms ?? this.#o.recipe.budget_ms?.[t] ?? DEFAULT_BUDGET_MS[t];
  }

  #newGovernor(level = 0) {
    return new Governor(maxLevel(this.#o.recipe, this.#tier), { budget_ms: this.#budget(this.#tier) }, level);
  }

  /** Place the view by the device (again). Resolves when placed; the probe may place it once more later. */
  async place(): Promise<void> {
    if (this.#disposed) return;
    const o = this.#o;
    let tier: Tier;
    if (o.tier) tier = o.tier;
    else {
      const caps = o.backend ? await this.#dev.caps().catch(() => null) : null;
      const speed = caps && o.backend ? await this.#dev.speed({ ...caps, backend: o.backend }) : null;
      this.#placed = place({ caps: { backend: o.backend, compute: o.backend === "webgpu" && !!caps?.compute }, probe: speed, pixels: o.pixels, recipe: o.recipe, library: o.library, choice: this.#dev.quality(), heat_ceiling: this.#dev.heatCeiling() });
      tier = this.#placed.tier;
    }
    if (this.#disposed) return;
    if (!o.recipe.tiers.includes(tier)) tier = o.recipe.tiers.find((t) => t !== "still") ?? tier;
    const changed = tier !== this.#tier;
    this.#tier = tier;
    if (changed || this.#gov.max !== maxLevel(o.recipe, tier)) this.#gov = this.#newGovernor();
    this.#replan();
    this.rest.aim(REST_FRAMES[tier], 0);
    o.onChange?.();
  }

  #replan() {
    this.#plan = plan(this.#o.recipe, this.#o.library, this.#tier, this.#gov.level);
  }

  /** The view resized or changed recipe: the governor starts again from the top of the new placement. */
  update(n: Partial<Pick<TrueRunOptions, "pixels" | "backend" | "tier" | "recipe" | "budget_ms">>): void {
    const re = (n.backend !== undefined && n.backend !== this.#o.backend) || n.tier !== this.#o.tier || (n.recipe && n.recipe !== this.#o.recipe);
    this.#o = { ...this.#o, ...n };
    if (re || n.pixels !== undefined) void this.place();
  }

  /** What to draw now. At rest, the recipe's still passes at full size. */
  settings(atRest = false): TrueSettings {
    const p = atRest && this.#o.recipe.tiers.includes("still") ? plan(this.#o.recipe, this.#o.library, "still", 0) : this.#plan;
    const params: TrueSettings["params"] = {};
    for (const x of p.passes) params[x.id.replace(/-(still|basic|high|ultra)$/, "")] = x.params;
    return { tier: p.tier, render_scale: atRest ? 1 : p.render_scale, params };
  }

  get plan(): Plan {
    return this.#plan;
  }

  /**
   * One frame drawn while moving, `ms` long, at `now`. Returns true when the settings changed (redraw with them).
   * Also restarts the wait for rest: any drawn frame means the view changed.
   */
  frame(ms: number, now: number): boolean {
    perf.record(this.#o.source, ms, { detail: this.#plan.tier });
    this.rest.moved(now);
    const est = estimateMs(this.#plan, this.#o.library, this.#o.pixels, {}, dev.speedNow()?.factors ?? null);
    const ratio = est > 0 && Number.isFinite(this.#gov.ewma) ? this.#gov.ewma / est : null;
    const move = this.#gov.sample(ms, (lv) => (ratio === null ? null : estimateMs(plan(this.#o.recipe, this.#o.library, this.#tier, lv), this.#o.library, this.#o.pixels) * ratio));
    if (!move) return false;
    this.#replan();
    if (move === "down" && !this.#o.tier && this.#dev.heatDown(now, this.#tier)) void this.place();
    this.#o.onChange?.();
    return true;
  }

  status(): TrueStatus {
    const p = this.#plan;
    const budget = this.#budget(this.#tier);
    return {
      tier: p.tier,
      level: p.level,
      max: this.#gov.max,
      badge: badge({ reductions: p.reductions, ewma_ms: this.#gov.ewma, budget_ms: budget, lift: this.#o.recipe.lift, at_floor: p.level >= this.#gov.max }),
      note: this.#o.tier ? `Drawn at ${this.#o.tier} as set, not by this device's own choice.` : (this.#placed?.note ?? "Placing…"),
      ewma_ms: this.#gov.ewma,
      budget_ms: budget,
      estimate_ms: estimateMs(p, this.#o.library, this.#o.pixels, {}, dev.speedNow()?.factors ?? null),
      rest: this.rest.state,
      rest_frames: this.rest.frames,
    };
  }

  dispose(): void {
    this.#disposed = true;
    this.#unsub();
  }
}
