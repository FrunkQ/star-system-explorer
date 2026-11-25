// Turning a paper page (atlas design 11A.4a, answer 5 and 14A note 5; AT15a). Alex, 25 Nov: "have the paper curl as you
// turn it - less mechanical - like you are leafing through it". The bottom outer corner lifts first and the sheet peels
// back along a slanted fold that straightens as it crosses the page, with a little hand wobble. The lifted part shows
// the pale back of the paper with a soft highlight along the curl, and casts one blurred shadow on the page beneath,
// which can already be read. A turn takes about 0.7 s; going back is the same turn played in reverse (the previous page
// comes back down over this one). A drag or swipe carries the fold with the finger and lets go either way. A tap or key
// finishes a turn at once. Reduced motion: a plain change, no curl.
//
// No imports, so a browser test can run this file with its types stripped (tests/ui/page-turn.e2e.mjs).

export type Pt = [number, number];

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
/** How long a whole turn takes (14A note 5: "about 0.7 s"). The kit allows Page turn up to PAGE_TURN_MAX_MS. */
export const CURL_MS = 700;

/** The sheet's state for a drag (pure, for tests and for any renderer). */
export function turnDrag(dx: number, width: number): TurnDrag {
  if (!(width > 0) || Math.abs(dx) < SLOP_PX) return { dir: 0, progress: 0, angle: 0 };
  const dir = dx < 0 ? 1 : -1;
  const progress = Math.min(1, (Math.abs(dx) - SLOP_PX) / Math.max(1, width * 0.8));
  return { dir, progress, angle: dir === 1 ? -90 * progress : 90 * progress };
}

/**
 * Whether letting go here turns the page: past a third of the way, or flicked onwards. `velocityPxPerMs` is the
 * finger's speed across (negative leftwards); a flick back the other way lays the page down again.
 */
export function turnCommits(drag: TurnDrag, velocityPxPerMs: number): boolean {
  if (drag.dir === 0) return false;
  const onwards = -velocityPxPerMs * drag.dir;
  return onwards >= FLICK_PX_PER_MS || (drag.progress >= COMMIT_AT && onwards > -FLICK_PX_PER_MS);
}

// ------------------------------------------------------------------------------------------------------- the curl

/** The part of a convex polygon on the side of the line through (ax, ay) that the normal (nx, ny) points to. */
export function clipPoly(poly: Pt[], ax: number, ay: number, nx: number, ny: number): Pt[] {
  const out: Pt[] = [];
  for (let i = 0; i < poly.length; i++) {
    const p = poly[i]!;
    const q = poly[(i + 1) % poly.length]!;
    const dp = (p[0] - ax) * nx + (p[1] - ay) * ny;
    const dq = (q[0] - ax) * nx + (q[1] - ay) * ny;
    if (dp >= 0) out.push(p);
    if (dp >= 0 !== dq >= 0) {
      const t = dp / (dp - dq);
      out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
    }
  }
  return out;
}

/** Twice the signed area of a polygon (for tests: the curl keeps the paper's area). */
export const polyArea = (poly: Pt[]): number => {
  let a = 0;
  for (let i = 0; i < poly.length; i++) {
    const p = poly[i]!;
    const q = poly[(i + 1) % poly.length]!;
    a += p[0] * q[1] - q[0] * p[1];
  }
  return Math.abs(a) / 2;
};

/** One moment of the curl, in the page's own pixels (origin top left). */
export interface Curl {
  /** The part of the sheet still lying flat (the page's clip). Empty once it has turned. */
  keep: Pt[];
  /** The lifted part, folded over: drawn as the back of the paper. */
  flap: Pt[];
  /** The flap's blurred shadow on the page beneath. */
  shadow: Pt[];
  /** The soft shade on the page beneath, just past the fold. */
  shade: Pt[];
  /** The paper shading runs from the fold (x1, y1) to the flap's far edge (x2, y2). */
  light: { x1: number; y1: number; x2: number; y2: number };
  /** The shade under the fold runs from (x1, y1) out to (x2, y2). */
  dark: { x1: number; y1: number; x2: number; y2: number };
}

/**
 * The curl at `u` (0 flat, 1 turned over) on a page `w` by `h`. `eased` is how far the fold has travelled (the timed
 * turn eases in and out; a drag follows the finger). The fold meets the bottom edge right of the page at the start and
 * left of it at the end, leaning back at first, then straightening, with a small wobble.
 */
export function curlAt(u: number, w: number, h: number, eased = u): Curl {
  const e = Math.max(0, Math.min(1, eased));
  const fx = w * 1.04 - e * w * 1.62;
  const th = ((34 - 26 * e + 5 * Math.sin(u * Math.PI * 2)) * Math.PI) / 180;
  const dx = Math.sin(th);
  const dy = -Math.cos(th);
  const nx = -dy; // the normal points to the lifted side (right)
  const ny = dx;
  const rect: Pt[] = [
    [0, 0],
    [w, 0],
    [w, h],
    [0, h],
  ];
  const keep = clipPoly(rect, fx, h, -nx, -ny);
  const lift = clipPoly(rect, fx, h, nx, ny);
  const flap: Pt[] = lift.map((q) => {
    const k = 2 * ((q[0] - fx) * nx + (q[1] - h) * ny);
    return [q[0] - k * nx, q[1] - k * ny];
  });
  const far = Math.max(1, ...lift.map((q) => (q[0] - fx) * nx + (q[1] - h) * ny));
  const sw = Math.min(90, far * 0.5);
  return {
    keep: keep.length > 2 ? keep : [],
    flap: flap.length > 2 ? flap : [],
    shadow: flap.length > 2 ? flap.map((q) => [q[0] - 10, q[1] + 8]) : [],
    shade: lift.length > 2 ? clipPoly(lift, fx + nx * sw, h + ny * sw, -nx, -ny) : [],
    light: { x1: fx, y1: h, x2: fx - nx * far, y2: h - ny * far },
    dark: { x1: fx, y1: h, x2: fx + nx * sw, y2: h + ny * sw },
  };
}

/** Ease in and out, for the timed turn. */
export const easeTurn = (u: number): number => (u < 0.5 ? 2 * u * u : 1 - ((-2 * u + 2) * (-2 * u + 2)) / 2);

const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
const nextFrame = () => new Promise<void>((r) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => r()) : setTimeout(r, 16)));
const pts = (a: Pt[]) => a.map((q) => `${Math.round(q[0] * 10) / 10},${Math.round(q[1] * 10) / 10}`).join(" ");
let uid = 0;

/**
 * Where a page stands inside the element the copies are laid in (which the turn makes positioned first, so this and
 * the copies' containing block agree whatever the page's own offset parent is).
 */
function placeIn(host: HTMLElement, page: HTMLElement) {
  const r = page.getBoundingClientRect();
  const hr = host.getBoundingClientRect();
  return { left: r.left - hr.left - host.clientLeft + host.scrollLeft, top: r.top - hr.top - host.clientTop + host.scrollTop, width: r.width, height: r.height };
}

/** A copy of the page laid exactly over it, for the turn to peel (or to hold the old page underneath going back). */
function cover(host: HTMLElement, page: HTMLElement, z: number): HTMLElement {
  const c = page.cloneNode(true) as HTMLElement;
  c.removeAttribute("id");
  c.removeAttribute("data-deliver-item");
  c.setAttribute("aria-hidden", "true");
  c.setAttribute("inert", "");
  c.dataset.pageCurl = "copy";
  const at = placeIn(host, page);
  Object.assign(c.style, {
    position: "absolute",
    left: `${at.left}px`,
    top: `${at.top}px`,
    width: `${at.width}px`,
    height: `${at.height}px`,
    maxWidth: "none",
    margin: "0",
    zIndex: String(z),
    pointerEvents: "none",
    transform: "none",
    boxSizing: "border-box",
  });
  host.appendChild(c);
  c.scrollTop = page.scrollTop;
  return c;
}

function paperOf(node: HTMLElement): string {
  const bg = getComputedStyle(node).backgroundColor;
  return bg && bg !== "transparent" && !/rgba\([^)]*,\s*0\)$/.test(bg) ? bg : "#f1ede3";
}

/** The drawing for one turn: the folded flap over the top copy, sized and placed to that copy. */
interface CurlStage {
  draw(u: number, eased?: number): void;
  /** The top copy was replaced (the page coming back finished drawing late): follow the new one. */
  retop(top: HTMLElement): void;
  remove(): void;
}

function stage(host: HTMLElement, first: HTMLElement): CurlStage {
  let top = first;
  let w = 0;
  let h = 0;
  const id = `sse-curl-${++uid}`;
  const NS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("aria-hidden", "true");
  svg.dataset.pageCurl = "flap";
  const fit = () => {
    w = top.offsetWidth;
    h = top.offsetHeight;
    svg.setAttribute("width", String(w));
    svg.setAttribute("height", String(h));
    Object.assign(svg.style, { position: "absolute", left: top.style.left, top: top.style.top, zIndex: String(Number(top.style.zIndex) + 1), overflow: "hidden", pointerEvents: "none" });
  };
  fit();
  const paper = paperOf(top);
  svg.innerHTML =
    `<defs><linearGradient id="${id}-l" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#8f897c"/><stop offset=".07" stop-color="#d9d4c8"/><stop offset=".22" stop-color="#fffdf8"/><stop offset=".55" stop-color="${paper}"/><stop offset="1" stop-color="#d8d2c4"/></linearGradient>` +
    `<linearGradient id="${id}-d" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#000" stop-opacity=".34"/><stop offset="1" stop-color="#000" stop-opacity="0"/></linearGradient>` +
    `<filter id="${id}-b" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="7"/></filter></defs>` +
    `<polygon data-curl="shadow" fill="#000" opacity=".28" filter="url(#${id}-b)"/><polygon data-curl="shade" fill="url(#${id}-d)"/><polygon data-curl="flap" fill="url(#${id}-l)" stroke="#bdb6a6" stroke-width=".6"/>`;
  host.appendChild(svg);
  const q = (k: string) => svg.querySelector(`[data-curl="${k}"]`)!;
  const [sh, sd, fl] = [q("shadow"), q("shade"), q("flap")];
  const lg = svg.querySelector(`#${id}-l`)!;
  const dg = svg.querySelector(`#${id}-d`)!;
  const setLine = (g: Element, l: Curl["light"]) => {
    g.setAttribute("x1", String(l.x1));
    g.setAttribute("y1", String(l.y1));
    g.setAttribute("x2", String(l.x2));
    g.setAttribute("y2", String(l.y2));
  };
  let last: [number, number | undefined] = [0, undefined];
  const draw = (u: number, eased?: number) => {
    last = [u, eased];
    const c = curlAt(u, w, h, eased);
    top.style.clipPath = c.keep.length ? `polygon(${c.keep.map((p) => `${Math.round(p[0] * 10) / 10}px ${Math.round(p[1] * 10) / 10}px`).join(",")})` : "polygon(0 0, 0 0, 0 0)";
    fl.setAttribute("points", pts(c.flap));
    sh.setAttribute("points", pts(c.shadow));
    sd.setAttribute("points", pts(c.shade));
    setLine(lg, c.light);
    setLine(dg, c.dark);
    svg.dataset.u = String(Math.round(u * 1000) / 1000);
  };
  return {
    draw,
    retop(t) {
      top = t;
      fit();
      draw(...last);
    },
    remove() {
      svg.remove();
    },
  };
}

/** A turn in progress: drawn at `u` (0 this page flat, 1 turned to the next), and taken away at the end. */
interface Session {
  draw(u: number, eased?: number): void;
  /** Play from where it is to `to` (0 or 1) over the rest of the turn's time. */
  run(to: 0 | 1, ms: number): Promise<void>;
  /** Finish at once (a tap, a key). */
  finish(): void;
  end(): void;
}

/**
 * The page as it stands now: the same element, or, when the swap replaced it (a keyed block), the element that took its
 * place among the host's children.
 */
function pageNow(host: HTMLElement, was: HTMLElement): HTMLElement {
  if (was.isConnected && was.parentElement === host) return was;
  const same = [...host.children].filter((c): c is HTMLElement => c instanceof HTMLElement && !c.dataset.pageCurl && c.tagName === was.tagName);
  return same.find((c) => c.className === was.className) ?? same[0] ?? was;
}

/**
 * Begin a turn: cover the page with copies, then `swap` to the new page underneath. Forward, a copy of this page lies
 * on top and peels away from u 0 to 1, showing the new page. Back, a copy of this page lies underneath and a copy of the
 * page coming back lies on top, coming down from u 1 to 0 (re-copied if that page is still drawing as the turn plays).
 */
async function begin(node: HTMLElement, dir: 1 | -1, swap: () => unknown): Promise<Session> {
  const host = node.parentElement!;
  // The copies are laid in the page's parent; make it their containing block before measuring anything.
  const restoreHost = host.style.position;
  if (getComputedStyle(host).position === "static") host.style.position = "relative";
  const under = dir === -1 ? cover(host, node, 3) : null;
  let top: HTMLElement;
  let page = node;
  let watch: MutationObserver | null = null;
  if (dir === 1) {
    top = cover(host, node, 4);
    await swap();
    await nextFrame();
    page = pageNow(host, node);
  } else {
    await swap();
    await nextFrame();
    page = pageNow(host, node);
    top = cover(host, page, 4);
  }
  const st = stage(host, top);
  if (dir === -1 && typeof MutationObserver === "function") {
    // The page coming back may still be filling in (a lazy figure, a late fetch): copy it again as it changes.
    let queued = false;
    // Only changes to the page itself count, not the copies and flap this turn lays down.
    const ours = (n: Node | null) => !!(n instanceof Element ? n : n?.parentElement)?.closest("[data-page-curl]");
    watch = new MutationObserver((records) => {
      if (!records.some((r) => !ours(r.target) && (r.type !== "childList" || [...r.addedNodes, ...r.removedNodes].some((n) => !ours(n))))) return;
      if (queued) return;
      queued = true;
      requestAnimationFrame(() => {
        queued = false;
        if (!watch) return;
        page = pageNow(host, page);
        const fresh = cover(host, page, 4);
        top.remove();
        top = fresh;
        st.retop(fresh);
      });
    });
    watch.observe(host, { subtree: true, childList: true, characterData: true });
  }
  page.dataset.delivering = "page_turn";
  let u = dir === 1 ? 0 : 1;
  let anim: { stop: () => void; to: 0 | 1 } | null = null;
  const draw = (x: number, eased?: number) => {
    u = x;
    st.draw(x, eased);
  };
  draw(u);
  return {
    draw,
    run(to, ms) {
      return new Promise<void>((resolve) => {
        const from = u;
        const span = Math.abs(to - from);
        const dur = Math.max(1, ms * span);
        const t0 = performance.now();
        let live = true;
        // Ease the remaining stretch: whole turns ease in and out; a release from a drag eases out from the finger.
        const whole = span > 0.999;
        const step = (now: number) => {
          if (!live) return;
          const k = Math.min(1, (now - t0) / dur);
          const x = from + (to - from) * (whole ? k : 1 - (1 - k) * (1 - k));
          draw(x, whole ? easeTurn(x) : x);
          if (k < 1) requestAnimationFrame(step);
          else {
            live = false;
            anim = null;
            resolve();
          }
        };
        anim = {
          to,
          stop: () => {
            if (!live) return;
            live = false;
            anim = null;
            draw(to);
            resolve();
          },
        };
        requestAnimationFrame(step);
      });
    },
    finish() {
      anim?.stop();
    },
    end() {
      watch?.disconnect();
      watch = null;
      anim?.stop();
      st.remove();
      top.remove();
      under?.remove();
      host.style.position = restoreHost;
      for (const p of new Set([node, page])) if (p.dataset.delivering === "page_turn") delete p.dataset.delivering;
    },
  };
}

export interface CurlTurnOptions {
  /** How long the whole turn takes (default CURL_MS). */
  ms?: number;
}

/**
 * Turn a paper page with the curl: `curlTurn(page, +1, () => goTo(next))`. `swap` changes the page's content to the
 * new page (it may return a promise; the turn waits a frame after it so the new page has drawn). Under reduced motion
 * (or with no browser drawing) it just swaps. A tap or key during the turn finishes it.
 */
export function curlTurn(node: HTMLElement, dir: 1 | -1, swap: () => unknown, opts: CurlTurnOptions = {}): { finish(): void; done: Promise<void> } {
  if (reduced() || typeof document === "undefined" || !node.parentElement) {
    return { finish() {}, done: Promise.resolve(swap()).then(() => undefined) };
  }
  let session: Session | null = null;
  let finished = false;
  const finish = () => {
    finished = true;
    session?.finish();
  };
  const done = (async () => {
    addEventListener("pointerdown", finish, true);
    addEventListener("keydown", finish, true);
    try {
      session = await begin(node, dir, swap);
      if (!finished) await session.run(dir === 1 ? 1 : 0, opts.ms ?? CURL_MS);
    } finally {
      removeEventListener("pointerdown", finish, true);
      removeEventListener("keydown", finish, true);
      session?.end();
    }
  })();
  return { finish, done };
}

// --------------------------------------------------------------------------------------------------- by hand

export interface PageTurnOptions {
  /**
   * Step forward (+1) or back (-1). With `sheet`, a drag steps as soon as it becomes a turn, so the page beneath can be
   * read while the fold is carried; a drag let go short steps the other way again to put the page back.
   */
  onstep: (dir: 1 | -1) => unknown;
  /** Paper looks: the page curls with the finger. Without it a swipe just steps when let go. */
  sheet?: boolean;
}

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
  // The curl being carried: its direction, and the session once its copies are laid.
  let carried: { dir: 1 | -1; session: Promise<Session> } | null = null;
  let busy = false;
  const curling = () => !!opts.sheet && !reduced() && !!node.parentElement;

  function down(e: PointerEvent) {
    if (busy || !e.isPrimary || e.button > 0) return;
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
    const d = turnDrag(dx, node.clientWidth);
    // The turn keeps the direction it started with; dragging back past the start just lays it flat.
    drag = carried && d.dir !== carried.dir ? { dir: carried.dir, progress: 0, angle: 0 } : d;
    if (!curling() || drag.dir === 0) return;
    if (!carried) {
      const dir = drag.dir;
      carried = { dir, session: begin(node, dir, () => opts.onstep(dir)) };
    }
    const c = carried;
    const p = drag.progress;
    void c.session.then((s) => {
      if (carried === c) s.draw(c.dir === 1 ? p : 1 - p);
    });
  }
  async function up(e: PointerEvent) {
    if (!start || e.pointerId !== start.id) return;
    start = null;
    if (!horizontal) return;
    // The speed over the last stretch of the drag, for a flick.
    const v = last ? (e.clientX - last.x) / Math.max(8, e.timeStamp - last.t) : 0;
    const d = drag;
    drag = { dir: 0, progress: 0, angle: 0 };
    const commit = e.type === "pointerup" && turnCommits(d, v);
    const c = carried;
    carried = null;
    if (!c) {
      if (commit && d.dir) opts.onstep(d.dir);
      return;
    }
    busy = true;
    const s = await c.session;
    const ms = CURL_MS * 0.6;
    if (commit) {
      await s.run(c.dir === 1 ? 1 : 0, ms);
      s.end();
    } else {
      // Settle back, then put the page back the way it was before the copies go.
      await s.run(c.dir === 1 ? 0 : 1, ms);
      await opts.onstep(c.dir === 1 ? -1 : 1);
      await nextFrame();
      s.end();
    }
    busy = false;
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
      if (carried) void carried.session.then((s) => s.end());
    },
  };
}
