// Playing a screen delivery (atlas design 11A.4a, AT15a): how a picture arrives and leaves, in the look's own way. The
// plan comes from ui-contract delivery.ts (`resolveDelivery`); this file only draws it, with the Web Animations API, so
// it runs on the compositor and never touches the data underneath.
//
// Three scales: the whole screen; each part in turn (`[data-part]` children, else the direct children); or inside each
// part (`[data-deliver-item]` elements, else the blocks of text, rows and figures), so a table fills row by row and text
// types on line by line. Everything under a delivery is live from the first frame, and a tap or a key finishes it at
// once (rule 1). Reduced motion always gets a plain fade (rule 3). While one plays, the screen carries
// `data-delivering="<kind>"`.
import type { DeliveryPlan } from "../../ui-contract/src/index.ts";

export interface Playing {
  /** Jump to the end now. */
  finish(): void;
  /** Settles when it has ended, by itself or by finish(). */
  done: Promise<void>;
}

const ITEM_SELECTOR = "[data-deliver-item]";
const TEXT_SELECTOR = "[data-deliver-text]";
// Blocks a screen is made of, when a part has not marked its own items.
const BLOCKS = "h1, h2, h3, h4, h5, h6, p, li, tr, dt, dd, figure, svg, canvas, img, pre, label, [role='row']";
const MAX_ITEMS = 80;

const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

/** The elements a plan acts on, in the order they are drawn. */
export function targetsOf(node: HTMLElement, plan: Pick<DeliveryPlan, "scale" | "kind">): HTMLElement[] {
  if (plan.scale === "screen" && plan.kind !== "type_on") return [node];
  if (plan.scale === "parts") {
    const marked = [...node.querySelectorAll<HTMLElement>(":scope [data-part]")].filter((e) => !e.parentElement?.closest("[data-part]") || e.parentElement === node);
    const parts = marked.length ? marked : ([...node.children] as HTMLElement[]);
    return parts.length ? parts : [node];
  }
  // Inside: the parts' own items, else their blocks, outermost first, in document order.
  const sel = plan.kind === "type_on" ? `${TEXT_SELECTOR}, ${ITEM_SELECTOR}` : ITEM_SELECTOR;
  let items = [...node.querySelectorAll<HTMLElement>(sel)];
  if (!items.length) items = [...node.querySelectorAll<HTMLElement>(BLOCKS)].filter((e) => !e.parentElement?.closest(BLOCKS));
  return items.length ? items.slice(0, MAX_ITEMS) : [node];
}

const inset = (d: DeliveryPlan["direction"]) =>
  d === "down" ? "inset(0 0 100% 0)" : d === "up" ? "inset(100% 0 0 0)" : d === "right" ? "inset(0 100% 0 0)" : "inset(0 0 0 100%)";
const offset = (d: DeliveryPlan["direction"], px: number) =>
  d === "up" ? `translateY(${px}px)` : d === "down" ? `translateY(${-px}px)` : d === "right" ? `translateX(${-px}px)` : `translateX(${px}px)`;

/**
 * The keyframes one target plays, arriving (they are reversed for an exit, except where the leaving has its own shape:
 * Collapse to a dot). Pure, so the shape of each delivery is tested without a browser.
 */
export function keyframesFor(plan: Pick<DeliveryPlan, "kind" | "direction" | "exit">): Keyframe[] {
  const k = ((): Keyframe[] => {
    switch (plan.kind) {
      case "none":
        return [];
      case "fade":
        return [{ opacity: 0 }, { opacity: 1 }];
      case "wipe":
      case "stepped_wipe":
        return [{ clipPath: inset(plan.direction) }, { clipPath: "inset(0 0 0 0)" }];
      case "build_up":
        // A terminal drawing a line: it appears top to bottom in a few hard steps, a little bright, then settles.
        return [
          { clipPath: "inset(0 0 100% 0)", opacity: 0.6, filter: "brightness(1.6)" },
          { clipPath: "inset(0 0 0 0)", opacity: 1, filter: "brightness(1.25)", offset: 0.7 },
          { clipPath: "inset(0 0 0 0)", opacity: 1, filter: "brightness(1)" },
        ];
      case "type_on":
        return [{ clipPath: "inset(0 100% 0 0)" }, { clipPath: "inset(0 0 0 0)" }];
      case "scan_in":
        return [{ clipPath: "inset(0 0 100% 0)" }, { clipPath: "inset(0 0 0 0)" }];
      case "warm_up":
        // Black; a faint glow swells, overshoots a little, and steadies (the slow warm up of an old tube).
        return [
          { opacity: 0, filter: "brightness(0) blur(4px)" },
          { opacity: 0.25, filter: "brightness(1.6) blur(3px)", offset: 0.35 },
          { opacity: 0.75, filter: "brightness(2) blur(1.5px)", offset: 0.7 },
          { opacity: 1, filter: "brightness(1) blur(0px)" },
        ];
      case "collapse_dot":
        // Leaving only: the picture squeezes to a bright line, the line to a dot, the dot fades.
        return [
          { transform: "scale(1, 1)", filter: "brightness(1)", opacity: 1, easing: "cubic-bezier(0.5, 0, 0.9, 0.5)" },
          { transform: "scale(1, 0.006)", filter: "brightness(3)", opacity: 1, offset: 0.45, easing: "cubic-bezier(0.5, 0, 0.9, 0.5)" },
          { transform: "scale(0.006, 0.006)", filter: "brightness(4)", opacity: 1, offset: 0.8, easing: "linear" },
          { transform: "scale(0, 0)", filter: "brightness(4)", opacity: 0 },
        ];
      case "static_burst":
      case "dissolve_blocks":
        // The picture comes up under an overlay (static, or blocks) that does the visible work.
        return [{ opacity: 0 }, { opacity: 1, offset: 0.35 }, { opacity: 1 }];
      case "slide":
        return [{ opacity: 0, transform: offset(plan.direction, 24) }, { opacity: 1, transform: "none" }];
      case "page_turn":
        return [
          { transform: "perspective(1800px) rotateY(-80deg)", opacity: 0.35 },
          { transform: "perspective(1800px) rotateY(-20deg)", opacity: 0.9, offset: 0.6 },
          { transform: "perspective(1800px) rotateY(0deg)", opacity: 1 },
        ];
      case "iris":
        return [{ clipPath: "circle(0% at 50% 50%)" }, { clipPath: "circle(75% at 50% 50%)" }];
    }
  })();
  if (!plan.exit || plan.kind === "collapse_dot") return k;
  return k
    .slice()
    .reverse()
    .map((f) => (f.offset === undefined ? f : { ...f, offset: 1 - (f.offset as number) }));
}

/** The easing a kind moves with. */
export function easingFor(plan: Pick<DeliveryPlan, "kind" | "steps">, chars = 0): string {
  if (plan.kind === "stepped_wipe") return `steps(${plan.steps}, end)`;
  if (plan.kind === "type_on") return `steps(${Math.max(1, Math.min(200, chars))}, end)`;
  if (plan.kind === "build_up") return "steps(4, end)";
  if (plan.kind === "collapse_dot") return "linear";
  if (plan.kind === "warm_up") return "ease-in";
  return "cubic-bezier(0.2, 0.8, 0.2, 1)";
}

/** When each of `n` targets starts and how long it lasts, within the plan's time: one after another, overlapping a little. */
export function staggerOf(n: number, ms: number, weights?: readonly number[]): { delay: number; ms: number }[] {
  if (n <= 1) return [{ delay: 0, ms }];
  if (weights) {
    // Type on: each line takes time for its characters, one after the other, scaled to fit.
    const total = weights.reduce((a, b) => a + Math.max(1, b), 0);
    let at = 0;
    return weights.map((w) => {
      const d = (ms * Math.max(1, w)) / total;
      const out = { delay: at, ms: d };
      at += d;
      return out;
    });
  }
  const each = Math.max(ms * 0.45, ms / n);
  const step = (ms - each) / (n - 1);
  return Array.from({ length: n }, (_, i) => ({ delay: i * step, ms: each }));
}

/** An overlay over the node for the deliveries that draw something of their own (the scan line, static, blocks, shadow). */
function overlayFor(node: HTMLElement, plan: DeliveryPlan): { el: HTMLElement; frames: Keyframe[] } | null {
  const make = (css: string) => {
    const el = document.createElement("div");
    el.setAttribute("aria-hidden", "true");
    el.dataset.deliveryOverlay = plan.kind;
    el.style.cssText = `position:absolute;inset:0;pointer-events:none;z-index:50;${css}`;
    return el;
  };
  if (plan.kind === "scan_in") {
    const el = make("inset:auto 0 auto 0;top:0;height:2px;background:var(--sse-ink,#fff);box-shadow:0 0 12px 3px var(--sse-ink,#fff);");
    return { el, frames: [{ top: "0%", opacity: 1 }, { top: "100%", opacity: 0.8 }] };
  }
  if (plan.kind === "static_burst") {
    const el = make(
      "background-image:url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='1.4' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 0.6 0 0 0 0 0.6 0 0 0 0 0.6 0 0 0 1.2 -0.1'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>\");background-size:200px 200px;mix-blend-mode:screen;",
    );
    // The static jumps about as it fades; never more than a dim flicker (no whole-screen flash).
    return { el, frames: [{ opacity: 0.85, backgroundPosition: "0 0" }, { opacity: 0.6, backgroundPosition: "37px 91px" }, { opacity: 0.45, backgroundPosition: "113px 17px" }, { opacity: 0, backgroundPosition: "61px 143px" }] };
  }
  if (plan.kind === "dissolve_blocks") {
    const el = make("display:grid;grid-template-columns:repeat(8,1fr);grid-template-rows:repeat(6,1fr);");
    const n = 48;
    for (let i = 0; i < n; i++) {
      const c = document.createElement("i");
      c.style.cssText = "background:var(--sse-ground,#000);display:block;";
      el.appendChild(c);
    }
    return { el, frames: [] };
  }
  if (plan.kind === "page_turn") {
    // The moving shadow of the turning sheet, darkest at the spine, gone as it lies flat.
    const el = make(`background:linear-gradient(${plan.direction === "left" ? "270deg" : "90deg"}, rgba(0,0,0,0.35), rgba(0,0,0,0) 60%);`);
    return { el, frames: [{ opacity: 1 }, { opacity: 0 }] };
  }
  return null;
}

/** Play a delivery on a node. A tap or key anywhere finishes it at once; a reduced-motion viewer gets a plain fade. */
export function playDelivery(node: HTMLElement, given: DeliveryPlan): Playing {
  let plan = given;
  if (reduced() && plan.kind !== "none") plan = { ...plan, kind: "fade", ms: Math.min(plan.ms, 150), scale: "screen", then: undefined };
  if (plan.kind === "none" || plan.ms <= 0 || typeof node.animate !== "function") return { finish() {}, done: Promise.resolve() };

  const anims: Animation[] = [];
  const overlays: HTMLElement[] = [];
  let stopped = false;
  let next: Playing | null = null;
  const restore: (() => void)[] = [];

  const targets = targetsOf(node, plan);
  const frames = keyframesFor(plan);
  const chars = targets.map((t) => (t.textContent ?? "").trim().length);
  const times = plan.kind === "type_on" ? staggerOf(targets.length, plan.ms, chars) : staggerOf(targets.length, plan.ms);
  if (plan.kind === "page_turn") {
    const o = node.style.transformOrigin;
    node.style.transformOrigin = plan.direction === "left" ? "right center" : "left center";
    restore.push(() => (node.style.transformOrigin = o));
  }
  if (plan.kind === "collapse_dot") {
    const o = node.style.transformOrigin;
    node.style.transformOrigin = "center center";
    restore.push(() => (node.style.transformOrigin = o));
  }
  targets.forEach((t, i) => {
    const { delay, ms } = times[i]!;
    anims.push(t.animate(frames, { duration: ms, delay, easing: easingFor(plan, chars[i]), fill: plan.exit ? "forwards" : "backwards" }));
  });

  const ov = overlayFor(node, plan);
  if (ov) {
    if (getComputedStyle(node).position === "static") {
      const o = node.style.position;
      node.style.position = "relative";
      restore.push(() => (node.style.position = o));
    }
    node.appendChild(ov.el);
    overlays.push(ov.el);
    if (ov.frames.length) anims.push(ov.el.animate(ov.frames, { duration: plan.ms, easing: plan.kind === "static_burst" ? "steps(6, end)" : "linear", fill: "forwards" }));
    if (plan.kind === "dissolve_blocks") {
      // Each block goes in a fixed scattered order (a cheap hash, so every screen dissolves alike).
      const cells = [...ov.el.children] as HTMLElement[];
      cells.forEach((c, i) => {
        const at = ((i * 29) % cells.length) / cells.length;
        anims.push(c.animate([{ opacity: 1 }, { opacity: 0 }], { duration: plan.ms * 0.18, delay: at * plan.ms * 0.82, fill: "both", easing: "steps(1, end)" }));
      });
    }
  }
  node.dataset.delivering = plan.kind;

  let settle!: () => void;
  const done = new Promise<void>((r) => (settle = r));
  const end = () => {
    if (stopped) return;
    stopped = true;
    removeEventListener("pointerdown", finish, true);
    removeEventListener("keydown", finish, true);
    for (const o of overlays) o.remove();
    for (const r of restore) r();
    if (!plan.exit) for (const a of anims) a.cancel();
    if (node.dataset.delivering === plan.kind) delete node.dataset.delivering;
  };
  function finish() {
    if (next) return next.finish();
    for (const a of anims) {
      try {
        a.finish();
      } catch {
        a.cancel();
      }
    }
    end();
    settle();
  }
  addEventListener("pointerdown", finish, true);
  addEventListener("keydown", finish, true);
  Promise.all(anims.map((a) => a.finished.catch(() => undefined))).then(() => {
    if (stopped) return;
    end();
    if (plan.then) {
      next = playDelivery(node, plan.then);
      void next.done.then(settle);
    } else settle();
  });
  return {
    finish,
    done,
  };
}

const lastRect = new WeakMap<HTMLElement, DOMRect>();

/**
 * Svelte action: plays the plan when the element appears and again whenever a new plan is given (a new send, a turned
 * page). `null` plays nothing.
 */
export function deliver(node: HTMLElement, plan: DeliveryPlan | null) {
  let playing: Playing | null = plan ? playDelivery(node, plan) : null;
  // Where the picture stands, kept up to date, so a leaving delivery can hold it there while its replacement lays out.
  const keep = () => lastRect.set(node, node.getBoundingClientRect());
  keep();
  const ro = typeof ResizeObserver === "function" ? new ResizeObserver(keep) : null;
  ro?.observe(node);
  addEventListener("scroll", keep, true);
  return {
    update(p: DeliveryPlan | null) {
      if (p === plan) return;
      plan = p;
      playing?.finish();
      playing = p ? playDelivery(node, p) : null;
    },
    destroy() {
      playing?.finish();
      ro?.disconnect();
      removeEventListener("scroll", keep, true);
    },
  };
}

/**
 * Svelte transition for a picture going away (Leave, Power off): `out:deliverOut={plan}`. Svelte keeps the element
 * until the delivery ends; a tap ends it at once.
 */
export function deliverOut(node: HTMLElement, plan: DeliveryPlan | null | Record<string, never>) {
  // Svelte hands an empty object when the expression is null.
  if (!plan || !("kind" in plan) || plan.kind === "none") return { duration: 0 };
  const given = plan as DeliveryPlan;
  // Lift the leaving picture out of the flow, where it stood, so what replaces it lays out underneath at once.
  const r = lastRect.get(node) ?? node.getBoundingClientRect();
  Object.assign(node.style, { position: "fixed", top: `${r.top}px`, left: `${r.left}px`, width: `${r.width}px`, height: `${r.height}px`, margin: "0", zIndex: "60", background: "var(--sse-ground)" });
  playDelivery(node, { ...given, exit: true, scale: "screen", then: undefined });
  return { duration: reduced() ? Math.min(given.ms, 150) : given.ms };
}
