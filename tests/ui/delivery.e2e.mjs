// Screen delivery (AT15a, atlas design 11A.4a), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/delivery.e2e.mjs [--screens <dir>]
// A players' screen in the Phosphor look (the Hauler starter's) warms up, then builds up, as it powers on. A key press
// finishes a delivery at once. With reduced motion it is a plain fade. Held by the GM from the Switchboard, the picture
// collapses to a dot before the holding card; released, it warms up again. A look without deliveries (Chart) just shows.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const look = (id) => `try { localStorage.setItem("sse4.player-theme", ${JSON.stringify(id)}); } catch {}`;
// Every delivery a page plays, in order, even the quick ones.
const WATCH = `addEventListener("DOMContentLoaded", () => { window.__played = []; new MutationObserver((ms) => { for (const m of ms) { const v = m.target.getAttribute?.("data-delivering"); if (v) window.__played.push(v); } }).observe(document.body, { attributes: true, attributeFilter: ["data-delivering"], subtree: true }); });`;
const played = (p) => p.evaluate(() => window.__played ?? []);
const delivering = (p) => p.locator("main.grid").getAttribute("data-delivering");

try {
  const browser = await chromium.launch();
  const errors = [];

  // Power on in Phosphor: warm up, then build up, then still.
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: screens ? 2 : 1 });
  await ctx.addInitScript(look("phosphor"));
  const p = await ctx.newPage();
  p.on("pageerror", (e) => errors.push(e.message));
  await p.goto(base + "players.html");
  await p.locator('main.grid[data-delivering="warm_up"]').waitFor({ timeout: 30000 });
  await p.waitForTimeout(700);
  await shot(p, "hauler-warm-up");
  await p.locator('main.grid[data-delivering="build_up"]').waitFor({ timeout: 4000 });
  await p.waitForTimeout(250);
  await shot(p, "hauler-build-up");
  await p.waitForFunction(() => !document.querySelector("main.grid")?.hasAttribute("data-delivering"), null, { timeout: 4000 });
  // Everything is live from the first frame, and a key finishes it at once.
  await p.reload();
  await p.locator('main.grid[data-delivering="warm_up"]').waitFor({ timeout: 30000 });
  await p.keyboard.press("Shift");
  await p.waitForFunction(() => !document.querySelector("main.grid")?.hasAttribute("data-delivering"), null, { timeout: 500 });
  assert.equal(await p.evaluate(() => getComputedStyle(document.querySelector("main.grid")).opacity), "1");
  await ctx.close();

  // Reduced motion: a plain fade, whatever the look says.
  const calm = await browser.newContext({ viewport: { width: 1280, height: 800 }, reducedMotion: "reduce" });
  await calm.addInitScript(look("phosphor"));
  await calm.addInitScript(WATCH);
  const q = await calm.newPage();
  q.on("pageerror", (e) => errors.push(e.message));
  await q.goto(base + "players.html");
  await q.locator("main.grid").waitFor({ timeout: 30000 });
  await q.waitForFunction(() => (window.__played ?? []).length > 0, null, { timeout: 5000 });
  assert.deepEqual(await played(q), ["fade"]);
  await calm.close();

  // A look without deliveries (Chart) just shows the picture.
  const plain = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await plain.addInitScript(look("chart"));
  await plain.addInitScript(WATCH);
  const r = await plain.newPage();
  await r.goto(base + "players.html");
  await r.locator("main.grid").waitFor({ timeout: 30000 });
  await r.waitForTimeout(800);
  assert.deepEqual(await played(r), []);
  assert.equal(await delivering(r), null);
  await plain.close();

  // Held from the Switchboard: the picture collapses to a dot, then the card; released, it warms up again.
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: screens ? 2 : 1 });
  await context.addInitScript(look("phosphor"));
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await desk(gm, "Run");
  await gm.getByRole("button", { name: "Switchboard" }).first().click();
  await gm.locator("[data-switchboard]").waitFor();
  await gm.locator("[data-go-on-air]").click();
  await gm.locator("[data-on-air]").first().waitFor();
  const link = await gm.locator("[data-switchboard] [data-players-link]").getAttribute("data-players-link");
  const wall = await context.newPage();
  await wall.setViewportSize({ width: 1024, height: 700 });
  wall.on("pageerror", (e) => errors.push(`wall: ${e.message}`));
  await wall.goto(`${link}&name=Wall`);
  await wall.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await wall.locator('main.grid[data-delivering="warm_up"]').waitFor({ timeout: 30000 });
  await wall.waitForFunction(() => !document.querySelector("main.grid")?.hasAttribute("data-delivering"), null, { timeout: 6000 });
  const box = gm.locator("[data-switchboard] .screen", { hasText: "Wall" });
  await box.waitFor({ timeout: 15000 });
  const wallId = await box.getAttribute("data-screen");
  await gm.locator(`[data-hold="screen:${wallId}"]`).click();
  await wall.locator('main.grid[data-delivering="collapse_dot"]').waitFor({ timeout: 10000 });
  await wall.waitForTimeout(250);
  await shot(wall, "hauler-collapse-to-dot");
  await wall.locator("[data-screens-off][data-held]").waitFor({ timeout: 10000 });
  await wall.locator("main.grid").waitFor({ state: "detached", timeout: 3000 });
  await gm.locator(`[data-hold="screen:${wallId}"]`).click();
  await wall.locator('main.grid[data-delivering="warm_up"]').waitFor({ timeout: 10000 });

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("delivery e2e: ok");
} finally {
  server.kill();
}
