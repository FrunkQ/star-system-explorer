// Browser tests written for the Chart view keep it (AF2). A first visit opens in True (firstView, alpha setting of
// 27 Nov 2026); a test that drives the flat chart (clicking its bodies, tabbing to them, counting presses on it) says
// so by importing this module, which keeps Chart as the viewer's choice in every page it opens, exactly as a viewer
// who chose Chart would have it. A test of the first visit itself (fresh.e2e.mjs) does not import it.
import { createRequire } from "node:module";

const pw = createRequire(import.meta.url)("playwright");
const CHART = JSON.stringify({ style: "chart", seen: "tilted", scale: "readable" });
const keepChart = (view) => {
  try {
    if (localStorage.getItem("sse4.view") === null) localStorage.setItem("sse4.view", view);
  } catch {
    /* no storage here (an opaque origin): nothing to keep */
  }
};
const patched = new WeakSet();
function patchContext(ctx) {
  if (patched.has(ctx)) return ctx;
  patched.add(ctx);
  void ctx.addInitScript(keepChart, CHART);
  return ctx;
}
for (const type of [pw.chromium, pw.firefox, pw.webkit]) {
  const proto = Object.getPrototypeOf(type);
  if (proto.__chartFirst) continue;
  proto.__chartFirst = true;
  const launch = proto.launch;
  proto.launch = async function (...a) {
    const browser = await launch.apply(this, a);
    const bp = Object.getPrototypeOf(browser);
    if (!bp.__chartFirst) {
      bp.__chartFirst = true;
      const newContext = bp.newContext;
      bp.newContext = async function (...b) {
        return patchContext(await newContext.apply(this, b));
      };
      const newPage = bp.newPage;
      // browser.newPage makes its own context: give it the same keep before the first navigation.
      bp.newPage = async function (...b) {
        const ctx = await this.newContext(...b);
        const page = await ctx.newPage();
        page.close = ((close) => async (...c) => (await close.apply(page, c), ctx.close()))(page.close);
        return page;
      };
      void newPage;
    }
    return browser;
  };
  const launchPersistent = proto.launchPersistentContext;
  if (launchPersistent)
    proto.launchPersistentContext = async function (...a) {
      return patchContext(await launchPersistent.apply(this, a));
    };
}
