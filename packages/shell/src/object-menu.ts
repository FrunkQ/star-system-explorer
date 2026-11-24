// How the object menu is asked for (V3 gap 38): a right-click, or a press held on a touch screen, on anything that
// stands for an object. One general rule instead of a handler per view: an element that stands for an object carries
// `data-address` (the system chart's bodies, the selector's rows, a card's header). A view that draws on a canvas, where
// there is no element per object, sends a bubbling "sse:object-menu" event with the address and the point instead.
//
// Shift with a right-click leaves the browser's own menu alone, and so do text fields and the Known to eye (which
// answers a right-click itself and marks the event as handled).

export const OBJECT_MENU_EVENT = "sse:object-menu";

/** The detail of an "sse:object-menu" event. */
export interface ObjectMenuAsk {
  address: string;
  /** The point to open at, in viewport pixels. */
  x: number;
  y: number;
}

/** How long a touch is held before it is a press-and-hold, and how far it may wander. */
export const HOLD_MS = 500;
const HOLD_SLOP_PX = 10;

/** The element standing for an object under `t`, if any. */
export function objectAt(t: EventTarget | null): HTMLElement | null {
  const el = t instanceof Element ? t : null;
  if (!el || el.closest("input, textarea, select, [contenteditable], [data-object-menu]")) return null;
  return el.closest<HTMLElement>("[data-address]");
}

/**
 * Watch `root` for the gestures that ask for an object's menu, and call `open` with the object's address, the point
 * and the element (for the menu to open from). Returns a function that stops watching.
 */
export function watchObjectMenu(root: HTMLElement | Window, open: (ask: ObjectMenuAsk, from: Element | null) => void): () => void {
  let hold: { id: number; x: number; y: number; timer: ReturnType<typeof setTimeout> } | null = null;
  // After a hold opened the menu, the touch's own click and the browser's late contextmenu are swallowed.
  let heldUntil = 0;
  const cancel = () => {
    if (hold) clearTimeout(hold.timer);
    hold = null;
  };
  const oncontext = (e: Event) => {
    const m = e as MouseEvent;
    if (Date.now() < heldUntil) return void m.preventDefault();
    if (m.defaultPrevented || m.shiftKey) return;
    const el = objectAt(m.target);
    const address = el?.dataset.address;
    if (!el || !address) return;
    m.preventDefault();
    cancel();
    open({ address, x: m.clientX, y: m.clientY }, el);
  };
  const ondown = (e: Event) => {
    const p = e as PointerEvent;
    if (p.pointerType === "mouse" || !p.isPrimary) return;
    cancel();
    const el = objectAt(p.target);
    const address = el?.dataset.address;
    if (!el || !address) return;
    hold = {
      id: p.pointerId,
      x: p.clientX,
      y: p.clientY,
      timer: setTimeout(() => {
        hold = null;
        heldUntil = Date.now() + 700;
        open({ address, x: p.clientX, y: p.clientY }, el);
      }, HOLD_MS),
    };
  };
  const onmove = (e: Event) => {
    const p = e as PointerEvent;
    if (hold && p.pointerId === hold.id && Math.hypot(p.clientX - hold.x, p.clientY - hold.y) > HOLD_SLOP_PX) cancel();
  };
  const onclick = (e: Event) => {
    if (Date.now() < heldUntil) (e.preventDefault(), e.stopPropagation());
  };
  const onask = (e: Event) => {
    const d = (e as CustomEvent<ObjectMenuAsk>).detail;
    if (d && typeof d.address === "string" && Number.isFinite(d.x) && Number.isFinite(d.y)) open(d, e.target instanceof Element ? e.target : null);
  };
  root.addEventListener("contextmenu", oncontext);
  root.addEventListener("pointerdown", ondown);
  root.addEventListener("pointermove", onmove);
  root.addEventListener("pointerup", cancel);
  root.addEventListener("pointercancel", cancel);
  root.addEventListener("click", onclick, true);
  root.addEventListener(OBJECT_MENU_EVENT, onask);
  return () => {
    cancel();
    root.removeEventListener("contextmenu", oncontext);
    root.removeEventListener("pointerdown", ondown);
    root.removeEventListener("pointermove", onmove);
    root.removeEventListener("pointerup", cancel);
    root.removeEventListener("pointercancel", cancel);
    root.removeEventListener("click", onclick, true);
    root.removeEventListener(OBJECT_MENU_EVENT, onask);
  };
}

/** Where a menu of size w × h opens at (x, y) so that it stays on screen with a margin. */
export function placeMenu(x: number, y: number, w: number, h: number, vw: number, vh: number, margin = 8): { left: number; top: number } {
  const left = Math.max(margin, Math.min(x, vw - w - margin));
  // Below the point when it fits, else above it, else as high as it can go.
  const top = y + h + margin <= vh ? y : y - h >= margin ? y - h : Math.max(margin, vh - h - margin);
  return { left, top };
}
