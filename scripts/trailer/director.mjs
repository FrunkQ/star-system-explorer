// Camera-operator helpers for trailer shots (scripts/trailer/README.md).
// Headless browsers draw no pointer, so a soft pointer is painted into the page;
// the pointer glides with an eased path, so clicks and drags read on screen.
// Every helper waits on the app, never on luck: a shot that cannot find what it
// films fails loudly, which is how a shot learns it needs a re-shoot.

const CURSOR = `
(() => {
  if (document.getElementById("trailer-pointer")) return;
  const s = document.createElement("style");
  s.textContent = \`
    #trailer-pointer { position: fixed; left: 0; top: 0; width: 22px; height: 22px; margin: -11px 0 0 -11px;
      border-radius: 50%; background: rgba(20,24,32,.18); border: 2px solid rgba(20,24,32,.55);
      box-shadow: 0 0 0 1px rgba(255,255,255,.6); pointer-events: none; z-index: 2147483647;
      transition: transform .12s ease-out, background .12s; }
    #trailer-pointer.down { transform: scale(.72); background: rgba(20,24,32,.35); }
    :root[data-theme="night"] #trailer-pointer, .night #trailer-pointer { border-color: rgba(230,236,255,.7); background: rgba(230,236,255,.18); box-shadow: 0 0 0 1px rgba(0,0,0,.5); }
    .trailer-ripple { position: fixed; width: 40px; height: 40px; margin: -20px 0 0 -20px; border-radius: 50%;
      border: 2px solid rgba(80,110,200,.6); pointer-events: none; z-index: 2147483646; animation: trailer-ripple .45s ease-out forwards; }
    @keyframes trailer-ripple { from { transform: scale(.3); opacity: 1 } to { transform: scale(1.4); opacity: 0 } }\`;
  document.head.appendChild(s);
  const p = document.createElement("div");
  p.id = "trailer-pointer";
  p.style.transform = "translate(-100px,-100px)";
  document.body.appendChild(p);
  window.__trailerPointer = (x, y, down) => {
    p.style.left = x + "px"; p.style.top = y + "px"; p.style.transform = "";
    p.classList.toggle("down", !!down);
  };
  window.__trailerRipple = (x, y) => {
    const r = document.createElement("div"); r.className = "trailer-ripple";
    r.style.left = x + "px"; r.style.top = y + "px"; document.body.appendChild(r);
    setTimeout(() => r.remove(), 500);
  };
})();`;

const ease = (t) => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2);

export function director(page) {
  let pos = { x: 960, y: 540 };
  let down = false;
  const paint = () => page.evaluate(([x, y, d]) => window.__trailerPointer?.(x, y, d), [pos.x, pos.y, down]).catch(() => {});
  const d = {
    /** Paint the pointer (call after every navigation). */
    async ready() {
      await page.evaluate(CURSOR);
      await paint();
    },
    /** Hold the frame still. */
    hold: (ms) => page.waitForTimeout(ms),
    /** Glide the pointer to a point or to the centre of a locator, in `ms`. */
    async glide(target, ms = 700, offset = { x: 0, y: 0 }) {
      let to = target;
      if (typeof target?.boundingBox === "function") {
        await target.waitFor({ state: "visible", timeout: 15000 });
        await target.scrollIntoViewIfNeeded();
        const b = await target.boundingBox();
        to = { x: b.x + b.width / 2 + offset.x, y: b.y + b.height / 2 + offset.y };
      }
      const from = { ...pos };
      const steps = Math.max(2, Math.round(ms / 16));
      for (let i = 1; i <= steps; i++) {
        const t = ease(i / steps);
        pos = { x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t };
        await page.mouse.move(pos.x, pos.y);
        await paint();
        await page.waitForTimeout(ms / steps);
      }
      return to;
    },
    /** Glide to a locator and click it, with a small press ripple. */
    async click(target, { ms = 700, settle = 350 } = {}) {
      await d.glide(target, ms);
      down = true;
      await paint();
      await page.evaluate(([x, y]) => window.__trailerRipple?.(x, y), [pos.x, pos.y]).catch(() => {});
      await page.mouse.down();
      await page.waitForTimeout(90);
      await page.mouse.up();
      down = false;
      await paint();
      await page.waitForTimeout(settle);
    },
    /** Press, glide by (dx, dy) and release: drags read slowly on screen. */
    async drag(target, dx, dy, ms = 1200) {
      const start = await d.glide(target, 600);
      down = true;
      await page.mouse.down();
      await paint();
      await d.glide({ x: start.x + dx, y: start.y + dy }, ms);
      await page.mouse.up();
      down = false;
      await paint();
    },
    /** Type into the focused field at a human pace. */
    async type(text, perKey = 70) {
      await page.keyboard.type(text, { delay: perKey });
    },
    /** Scroll the wheel over the pointer, in small notches (zooms charts). */
    async wheel(dy, notches = 8, ms = 900) {
      for (let i = 0; i < notches; i++) {
        await page.mouse.wheel(0, dy / notches);
        await page.waitForTimeout(ms / notches);
      }
    },
  };
  return d;
}
