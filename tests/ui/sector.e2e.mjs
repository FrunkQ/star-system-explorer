// The stage 5 walk's first steps in a real browser, against the built shell
// (scripts/build-shell.sh first): add the Traveller pack, open the Sector map
// from the galaxy scale's rail, import the first four worlds (preview, then write), pick
// one; then the shared view controls' HUD (V10a): on by default, off leaves
// the grid and each world's mark and strips numbers, names, glyphs and reach.
// Usage: node tests/ui/sector.e2e.mjs
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { go } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 8900 + Math.floor(Math.random() * 300);
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shots = process.env.SSE_SHOTS; // a folder for screenshots, when set
const five = readFileSync("tests/fixtures/sector/lanternfall.tab", "utf8").split("\n").slice(0, 5).join("\n") + "\n";

try {
  const browser = await chromium.launch();
  const errors = [];
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: shots ? 2 : 1 });
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(`http://localhost:${port}/#chart`);
  await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await go(page, "packs");
  const packs = page.locator("[role=region][aria-label=Packs]");
  await packs.getByRole("button", { name: "Add the Traveller pack" }).click();
  await packs.locator("[aria-label=Proposal]").getByRole("button", { name: "Accept" }).click();
  await page.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sse.traveller"));
  await go(page, "galaxy");
  await page.locator(".rail button", { hasText: "Sector map" }).first().click();
  const desk = page.locator("[role=region][aria-label=Sector]");
  await desk.waitFor();
  await desk.locator("[data-import]").click();
  await desk.locator("[data-import-text]").fill(five);
  await desk.locator("[data-preview]").click();
  await desk.locator("[data-apply]").click();
  const map = desk.locator("svg.hex-map");
  await map.locator("text.name").first().waitFor({ timeout: 30000 });
  await desk.locator("[aria-label='hex 0912']").click();
  await desk.locator("aside.side").waitFor();

  const on = { hexes: await map.locator("polygon").count(), names: await map.locator("text.name").count(), nums: await map.locator("text.num").count(), marks: await map.locator(".mark, .ring").count(), lit: await map.locator(".range").count() };
  assert.equal(await map.getAttribute("data-hud"), "on", "HUD starts on: the sector map is a diagram");
  assert.ok(on.names >= 4 && on.nums === on.hexes && on.lit > 1, JSON.stringify(on));
  if (shots) await page.screenshot({ path: `${shots}/sector-hud-on.png` });
  await page.getByRole("group", { name: "HUD" }).getByRole("button", { name: "Off" }).click();
  await page.waitForFunction(() => document.querySelector("svg.hex-map")?.getAttribute("data-hud") === "off");
  const off = { hexes: await map.locator("polygon").count(), names: await map.locator("text.name").count(), nums: await map.locator("text.num").count(), marks: await map.locator(".mark, .ring").count(), lit: await map.locator(".range").count() };
  assert.equal(off.hexes, on.hexes, "the grid stays");
  assert.deepEqual([off.names, off.nums, off.lit], [0, 0, 0], JSON.stringify(off));
  assert.ok(off.marks >= 4 && off.marks < on.marks, `each world keeps its own mark only: ${JSON.stringify({ on, off })}`);
  if (shots) await page.screenshot({ path: `${shots}/sector-hud-off.png` });
  await page.getByRole("group", { name: "HUD" }).getByRole("button", { name: "On" }).click();
  await page.waitForFunction(() => document.querySelector("svg.hex-map")?.getAttribute("data-hud") === "on");

  assert.deepEqual(errors, []);
  console.log(`PASS chromium: Traveller pack added, four worlds imported, HUD on ${JSON.stringify(on)}, off ${JSON.stringify(off)}`);
  await browser.close();
} finally {
  server.kill();
}
