// Who steers a screen (PB3b), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/steer.e2e.mjs [--screens <dir>]
// A screen joins; it is Theirs to begin with. Set Read only on the Switchboard, it says the GM is steering and takes
// no input. Drive it: the GM's chart opens and, as the GM selects bodies, the screen follows. Let go: it stays where it
// was left. Theirs again: the player has it back.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const cardIs = (p, name) => p.waitForFunction((n) => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label") === n, name, { timeout: 15000 });

try {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const errors = [];
  const gm = await context.newPage();
  await gm.setViewportSize({ width: 1440, height: 1000 });
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await desk(gm, "Run");
  await go(gm, "switchboard"); // On Run the rail is folded into the view controls (three desks design 3.1).
  await gm.locator("[data-go-on-air]").click();
  const link = await gm.locator("[data-switchboard] [data-players-link]").getAttribute("data-players-link");
  const wall = await context.newPage();
  await wall.setViewportSize({ width: 1280, height: 860 });
  wall.on("pageerror", (e) => errors.push(`wall: ${e.message}`));
  await wall.goto(`${link}&name=Wall`);
  await wall.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await cardIs(wall, "Tarn");
  const box = gm.locator("[data-switchboard] .screen", { hasText: "Wall" });
  await box.waitFor({ timeout: 15000 });
  const id = await box.getAttribute("data-screen");
  const steer = gm.locator(`[data-steer="screen:${id}"]`);
  assert.equal(await steer.getAttribute("data-steer-mode"), "theirs");
  assert.equal(await wall.locator("[data-read-only]").count(), 0);

  // Read only.
  await steer.click();
  await wall.locator("[data-steered]").waitFor({ timeout: 10000 });
  assert.equal(await wall.locator("main[data-read-only]").count(), 1);
  assert.equal(await wall.locator("section.ask").count(), 0, "a read-only screen asks nothing");

  // Drive it: the chart opens; the screen follows the GM's selection.
  await gm.locator(`[data-drive="screen:${id}"]`).click();
  await gm.locator("[role=button][aria-label^='Vault'] circle.hit").first().waitFor();
  assert.match(await gm.locator("header.bar [data-on-air]").innerText(), /You are driving Wall/);
  await gm.locator("[role=button][aria-label^='Vault'] circle.hit").first().click();
  await cardIs(wall, "Vault");
  await gm.locator("[role=button][aria-label^='Tarn'] circle.hit").first().click();
  await cardIs(wall, "Tarn");
  await shot(gm, "steer-driving-gm");
  await shot(wall, "steer-driving-wall");

  // Let go on the Switchboard; it stays where it was left.
  await gm.locator("header.bar [data-on-air]").click();
  await gm.locator("[data-switchboard]").waitFor();
  assert.equal(await gm.locator("[data-line].driving").count(), 1, "the driven line is drawn in accent");
  await shot(gm, "steer-switchboard");
  await gm.locator("[data-switchboard] [data-driving] [data-let-go]").click();
  assert.equal(await gm.locator("[data-driving]").count(), 0);
  await gm.waitForTimeout(800);
  assert.equal(await wall.evaluate(() => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label")), "Tarn");

  // Theirs again.
  await steer.click();
  await wall.locator("[data-steered]").waitFor({ state: "detached", timeout: 10000 });

  // A role's heading sets all its screens.
  await gm.locator('[data-steer="role:table"]').click();
  await wall.locator("[data-steered]").waitFor({ timeout: 10000 });
  await gm.locator('[data-steer="role:table"]').click();
  await wall.locator("[data-steered]").waitFor({ state: "detached", timeout: 10000 });

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("steer e2e: ok");
} finally {
  server.kill();
}
