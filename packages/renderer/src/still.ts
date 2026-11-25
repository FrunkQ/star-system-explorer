// Sharpen at rest (VP5, SSE4-visual-polish-design.md 8.4): people look far more than they fly. When a True view has
// not moved for half a second it keeps refining: frames drawn with the recipe's `still` passes (finer meshes, more
// samples) and shifted by a fraction of a pixel along a Halton sequence are averaged, which smooths edges and
// sampling noise. On any input it drops back at once. Under "reduce motion" the sharpened picture fades in once it is
// done instead of changing frame by frame. A phone thus shows close to the full picture a second or two after it stops,
// at no cost while moving.
//
// Two parts: `RestClock`, the pure schedule (when to start, which jitter, when to stop), clock-free so it is tested
// anywhere; and `RestLayer`, a 2D canvas laid over the live picture that keeps the running average. Views with the
// shared finish (finish.ts) average in its half-float picture instead and use only the clock.
import type { Tier } from "../../ui-contract/src/index.ts";
import { halton } from "./photograph.ts";

/** Still this long, ms, before refining starts. */
export const REST_AFTER_MS = 500;

/** Most frames averaged at rest, by the tier the view is drawn at (a canvas gets one supersampled frame). */
export const REST_FRAMES: Readonly<Record<Tier, number>> = { still: 32, "gpu-ultra": 32, "gpu-high": 16, "gpu-basic": 16, canvas: 1, text: 0 };

/** Refining stops after this long even if frames remain, ms (the design's 0.5 to 3 s). */
export const REST_BUDGET_MS = 3000;

export type RestState = "moving" | "waiting" | "sharpening" | "sharp";

/** The jitter for frame k (k from 0): a sub-pixel offset in [-0.5, 0.5), the first frame unshifted. */
export function restJitter(k: number): [number, number] {
  return k === 0 ? [0, 0] : [halton(k, 2) - 0.5, halton(k, 3) - 0.5];
}

/**
 * The schedule. Call `moved(now)` on any change to the view, `next(now)` when the view could draw a refining frame:
 * it returns the frame's index and jitter, or null when nothing is due. `done(now, ms)` after each frame.
 */
export class RestClock {
  #since = 0;
  #k = 0;
  #started = -1;
  #slowest = 0;
  #frames: number;
  #budget: number;
  state: RestState = "moving";

  constructor(o: { frames: number; budget_ms?: number }) {
    this.#frames = Math.max(0, Math.round(o.frames));
    this.#budget = o.budget_ms ?? REST_BUDGET_MS;
  }

  /** Frames averaged so far. */
  get frames(): number {
    return this.#k;
  }

  /** Change how many frames to aim for (a new tier); restarts the wait. */
  aim(frames: number, now: number): void {
    this.#frames = Math.max(0, Math.round(frames));
    this.moved(now);
  }

  moved(now: number): void {
    this.#since = now;
    this.#k = 0;
    this.#started = -1;
    this.#slowest = 0;
    this.state = this.#frames > 0 ? "waiting" : "moving";
  }

  /** When the next refining frame becomes due, ms from `now` (Infinity when none will be). */
  dueIn(now: number): number {
    if (this.state === "waiting") return Math.max(0, this.#since + REST_AFTER_MS - now);
    return this.state === "sharpening" ? 0 : Infinity;
  }

  next(now: number): { k: number; jitter: [number, number] } | null {
    if (this.state === "waiting" && now - this.#since >= REST_AFTER_MS) {
      this.state = "sharpening";
      this.#started = now;
    }
    if (this.state !== "sharpening") return null;
    // Stop when the frames are made, or when the next would pass the budget (the slowest frame so far as its guess).
    if (this.#k >= this.#frames || (this.#k > 0 && now - this.#started + this.#slowest > this.#budget)) {
      this.state = "sharp";
      return null;
    }
    return { k: this.#k, jitter: restJitter(this.#k) };
  }

  done(now: number, ms: number): void {
    if (this.state !== "sharpening") return;
    // The first frame carries any shader compiling: later ones are timed on their own.
    if (this.#k > 0) this.#slowest = Math.max(this.#slowest, ms);
    this.#k++;
    if (this.#k >= this.#frames) this.state = "sharp";
  }
}

/**
 * The running average on a 2D canvas over the live one. `add(source, k)` mixes frame k in with weight 1/(k+1).
 * Shown at once (each frame sharper) or, under reduce motion, only when sharp, fading in.
 */
export class RestLayer {
  readonly canvas: HTMLCanvasElement;
  #g: CanvasRenderingContext2D;
  #reduced: boolean;

  constructor(host: HTMLElement, o: { reduced: boolean }) {
    this.#reduced = o.reduced;
    const c = document.createElement("canvas");
    c.setAttribute("aria-hidden", "true");
    c.dataset.rest = "moving";
    c.style.position = "absolute";
    c.style.pointerEvents = "none";
    c.style.opacity = "0";
    c.style.transition = this.#reduced ? "opacity 400ms ease" : "none";
    host.appendChild(c);
    this.canvas = c;
    this.#g = c.getContext("2d")!;
  }

  /** Place the layer exactly over the live canvas. */
  cover(live: HTMLCanvasElement): void {
    const c = this.canvas;
    c.style.left = `${live.offsetLeft}px`;
    c.style.top = `${live.offsetTop}px`;
    c.style.width = live.style.width || `${live.clientWidth}px`;
    c.style.height = live.style.height || `${live.clientHeight}px`;
  }

  add(source: CanvasImageSource, k: number, width: number, height: number): void {
    const c = this.canvas;
    if (k === 0 || c.width !== width || c.height !== height) {
      c.width = width;
      c.height = height;
    }
    this.#g.globalAlpha = 1 / (k + 1);
    this.#g.imageSmoothingEnabled = true;
    this.#g.imageSmoothingQuality = "high";
    this.#g.drawImage(source, 0, 0, width, height);
    this.#g.globalAlpha = 1;
    if (!this.#reduced && k >= 1) this.show("sharpening");
  }

  show(state: RestState): void {
    this.canvas.dataset.rest = state;
    this.canvas.style.opacity = state === "sharp" || (state === "sharpening" && !this.#reduced) ? "1" : "0";
  }

  hide(): void {
    this.canvas.dataset.rest = "moving";
    // Drop back at once, whatever the fade.
    this.canvas.style.transition = "none";
    this.canvas.style.opacity = "0";
    if (this.#reduced) requestAnimationFrame(() => (this.canvas.style.transition = "opacity 400ms ease"));
  }

  dispose(): void {
    this.canvas.remove();
  }
}
