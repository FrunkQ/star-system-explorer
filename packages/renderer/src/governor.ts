// The frame-time governor (R1, SSE4-surfaces-and-rendering.md 6.3). Frame
// time is the only portable signal for load and thermal throttling, so the
// governor watches it and nothing else. It walks the recipe's degrade order
// one step down when the smoothed frame time stays over budget, and one step
// back up only after a long spell well under budget and only when the
// richer level is predicted to fit, so quality does not flicker. Each time a
// step up has to be taken back, it waits twice as long before trying again.
// Pure and clock-free: the runtime feeds it frame times.

export interface GovernorOptions {
  budget_ms: number;
  /** Frames over budget in a row before stepping down. */
  down_after?: number;
  /** Frames well under budget in a row before trying a step up. */
  up_after?: number;
  /** "Well under": below this share of the budget. */
  up_margin?: number;
  /** Smoothing of the frame time (exponential, 0 to 1; higher follows faster). */
  alpha?: number;
  /** Frames ignored after any change, while the new level settles. */
  settle?: number;
}

export type GovernorMove = "down" | "up" | null;

export class Governor {
  readonly max: number;
  level: number;
  budget_ms: number;
  /** Smoothed frame time, ms (NaN until the first sample). */
  ewma = Number.NaN;
  /** The longest frame seen, for the badge. */
  peak = 0;
  #over = 0;
  #under = 0;
  #settle = 0;
  #upAfter: number;
  #lastUp = -1;
  #frames = 0;
  readonly #o: Required<GovernorOptions>;

  constructor(max: number, o: GovernorOptions, start = 0) {
    this.max = max;
    this.level = Math.max(0, Math.min(max, start));
    this.#o = { down_after: 10, up_after: 90, up_margin: 0.6, alpha: 0.2, settle: 4, ...o };
    this.budget_ms = o.budget_ms;
    this.#upAfter = this.#o.up_after;
  }

  /**
   * Feed one frame time. `predictUp(level)` estimates the frame time at a
   * richer level (scaled by what this device actually does); null means no
   * estimate, and the governor then trusts the margin alone.
   */
  sample(ms: number, predictUp?: (level: number) => number | null): GovernorMove {
    this.#frames++;
    if (!(ms >= 0) || !Number.isFinite(ms)) return null;
    if (this.#settle > 0) {
      this.#settle--;
      return null;
    }
    const a = this.#o.alpha;
    this.ewma = Number.isNaN(this.ewma) ? ms : this.ewma + a * (ms - this.ewma);
    this.peak = Math.max(this.peak, ms);
    if (this.ewma > this.budget_ms) {
      this.#over++;
      this.#under = 0;
    } else {
      this.#over = 0;
      this.#under = this.ewma < this.budget_ms * this.#o.up_margin ? this.#under + 1 : 0;
    }
    if (this.#over >= this.#o.down_after && this.level < this.max) {
      // A step up taken back soon after: be slower to try again.
      if (this.#lastUp >= 0 && this.#frames - this.#lastUp < this.#upAfter * 2) this.#upAfter = Math.min(this.#upAfter * 2, 3600);
      this.level++;
      this.#reset();
      return "down";
    }
    if (this.#under >= this.#upAfter && this.level > 0) {
      const p = predictUp?.(this.level - 1) ?? null;
      if (p === null || p < this.budget_ms * 0.85) {
        this.level--;
        this.#lastUp = this.#frames;
        this.#reset();
        return "up";
      }
      this.#under = 0;
    }
    return null;
  }

  /** Start again at a level (a new recipe, tier or size): history is forgotten, the backoff kept. */
  restart(level = 0) {
    this.level = Math.max(0, Math.min(this.max, level));
    this.ewma = Number.NaN;
    this.peak = 0;
    this.#reset();
  }

  #reset() {
    this.#over = 0;
    this.#under = 0;
    this.#settle = this.#o.settle;
    // Carry the smoothed time forward: the next level is judged on its own frames from here.
    this.ewma = Number.NaN;
  }
}

/** What the quality badge shows: nothing at full quality, else the reductions and why. */
export interface Badge {
  reduced: boolean;
  /** One line: the first reduction, or "Full quality". */
  short: string;
  /** Every reduction in force, then the measured reason. */
  detail: string[];
  /** What would lift it. */
  lift: string;
}

export function badge(o: { reductions: readonly string[]; ewma_ms: number; budget_ms: number; lift?: string; stood_in?: string }): Badge {
  const lines = [...(o.stood_in ? [o.stood_in] : []), ...o.reductions];
  if (!lines.length) return { reduced: false, short: "Full quality", detail: [], lift: "" };
  const fps = (ms: number) => Math.max(1, Math.round(1000 / ms));
  const why = Number.isFinite(o.ewma_ms) && o.reductions.length ? [`This device was taking about ${Math.round(o.ewma_ms)} ms a frame against a budget of ${Math.round(o.budget_ms)} ms (${fps(o.budget_ms)} a second).`] : [];
  const short = o.reductions.length ? `Reduced: ${o.reductions[o.reductions.length - 1]!.replace(/\.$/, "")}` : lines[0]!.replace(/\.$/, "");
  return { reduced: true, short, detail: [...lines, ...why], lift: o.lift ?? "A faster device, or a smaller view, would lift it." };
}
