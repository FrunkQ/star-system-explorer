// Turning a paper page by hand (atlas design 11A.4a and answer 5, AT15a): "A drag or swipe carries the page with the
// finger and lets go either way." The sheet lifts from its outer edge and follows the finger about its spine; let go
// past a third of the way (or with a flick) and it turns, and the caller steps to the next page (whose arrival is the
// look's Turn delivery); let go short and it settles back flat. Reduced motion: no following, a step on a long enough
// swipe. Only for looks whose Turn is Page turn; other looks step on a swipe without the sheet.

/** Where a drag of `dx` pixels across a page `width` wide has got to. */
export interface TurnDrag {
  /** +1 forward (finger moving left), -1 back (finger moving right), 0 not yet a turn. */
  dir: 1 | -1 | 0;
  /** How far over, 0 flat to 1 turned. */
  progress: number;
  /** The sheet's angle about its spine, in degrees (forward turns negative about the left edge, back positive about the right). */
  angle: number;
}

const SLOP_PX = 8;
const COMMIT_AT = 1 / 3;
const FLICK_PX_PER_MS = 0.6;

/** The sheet's state for a drag (pure, for tests and for any renderer). */
export function turnDrag(dx: number, width: number): TurnDrag {
  if (!(width > 0) || Math.abs(dx) < SLOP_PX) return { dir: 0, progress: 0, angle: 0 };
  const dir = dx < 0 ? 1 : -1;
  const progress = Math.min(1, (Math.abs(dx) - SLOP_PX) / Math.max(1, width * 0.8));
  return { dir, progress, angle: dir === 1 ? -90 * progress : 90 * progress };
}

/** Whether letting go here turns the page: past a third of the way, or flicked. */
export function turnCommits(drag: TurnDrag, velocityPxPerMs: number): boolean {
  if (drag.dir === 0) return false;
  return drag.progress >= COMMIT_AT || Math.abs(velocityPxPerMs) >= FLICK_PX_PER_MS;
}

export interface PageTurnOptions {
  /** Called when the page has turned: step forward (+1) or back (-1). */
  onstep: (dir: 1 | -1) => void;
  /** Draw the sheet following the finger (Page turn looks). Without it a swipe just steps. */
  sheet?: boolean;
}

const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

/**
 * Svelte action on the page element: `use:pageTurn={{ onstep, sheet }}`. A horizontal drag with a finger, pen or mouse
 * turns; links, buttons and fields inside keep their own presses; vertical drags scroll as usual.
 */
export function pageTurn(node: HTMLElement, options: PageTurnOptions) {
  let opts = options;
  let start: { x: number; y: number; t: number; id: number } | null = null;
  let last: { x: number; t: number } | null = null;
  let horizontal = false;
  let drag: TurnDrag = { dir: 0, progress: 0, angle: 0 };

  const draw = (d: TurnDrag) => {
    if (!opts.sheet || reduced()) return;
    node.style.transformOrigin = d.dir === -1 ? "right center" : "left center";
    node.style.transform = d.dir === 0 ? "" : `perspective(1800px) rotateY(${d.angle}deg)`;
    node.style.boxShadow = d.dir === 0 ? "" : `${-d.dir * 18 * d.progress}px 0 ${30 * d.progress}px rgba(0,0,0,${0.3 * d.progress})`;
  };
  const clear = () => {
    node.style.transform = "";
    node.style.transformOrigin = "";
    node.style.boxShadow = "";
  };
  function down(e: PointerEvent) {
    if (!e.isPrimary || e.button > 0) return;
    if ((e.target as Element).closest("a, button, input, select, textarea, [contenteditable], [role='button']")) return;
    start = { x: e.clientX, y: e.clientY, t: e.timeStamp, id: e.pointerId };
    last = { x: e.clientX, t: e.timeStamp };
    horizontal = false;
  }
  function move(e: PointerEvent) {
    if (!start || e.pointerId !== start.id) return;
    const dx = e.clientX - start.x;
    const dy = e.clientY - start.y;
    if (!horizontal) {
      if (Math.abs(dy) > SLOP_PX && Math.abs(dy) > Math.abs(dx)) return void (start = null);
      if (Math.abs(dx) < SLOP_PX) return;
      horizontal = true;
      node.setPointerCapture?.(e.pointerId);
    }
    if (e.timeStamp - (last?.t ?? 0) > 40) last = { x: e.clientX, t: e.timeStamp };
    drag = turnDrag(dx, node.clientWidth);
    draw(drag);
  }
  function up(e: PointerEvent) {
    if (!start || e.pointerId !== start.id) return;
    const was = start;
    start = null;
    if (!horizontal) return;
    // The speed over the last stretch of the drag, for a flick.
    const v = last ? (e.clientX - last.x) / Math.max(8, e.timeStamp - last.t) : 0;
    const d = drag;
    drag = { dir: 0, progress: 0, angle: 0 };
    const commit = e.type === "pointerup" && turnCommits(d, v);
    if (!opts.sheet || reduced() || typeof node.animate !== "function") {
      clear();
      if (commit && d.dir) opts.onstep(d.dir);
      return;
    }
    // Finish the gesture: over and step, or back flat.
    const to = commit ? (d.dir === 1 ? -90 : 90) : 0;
    const a = node.animate([{ transform: node.style.transform || "none" }, { transform: `perspective(1800px) rotateY(${to}deg)` }], { duration: 160, easing: "ease-out" });
    a.finished
      .catch(() => undefined)
      .then(() => {
        clear();
        if (commit && d.dir) opts.onstep(d.dir);
      });
  }
  node.addEventListener("pointerdown", down);
  node.addEventListener("pointermove", move);
  node.addEventListener("pointerup", up);
  node.addEventListener("pointercancel", up);
  // Swipes are ours across; up and down still scroll.
  const touch = node.style.touchAction;
  node.style.touchAction = "pan-y";
  return {
    update(o: PageTurnOptions) {
      opts = o;
    },
    destroy() {
      node.removeEventListener("pointerdown", down);
      node.removeEventListener("pointermove", move);
      node.removeEventListener("pointerup", up);
      node.removeEventListener("pointercancel", up);
      node.style.touchAction = touch;
      clear();
    },
  };
}
