// RK13 in a real browser, against the built shell (scripts/build-shell.sh first): the Power button on a ship's card
// in Ships and stations opens the Power card. A stock Traveller courier flies on typed figures, so the card says to fit
// parts; the flow itself is checked against the real kernel in packages/ui-contract/test/flow.test.ts. Phone width fits.
// Usage: node tests/ui/power.e2e.mjs   (SSE_SHOTS=<dir> for screenshots)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 9500 + Math.floor(Math.random() * 300);
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shots = process.env.SSE_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });
const shot = async (page, n) => shots && (await page.screenshot({ path: `${shots}/${n}.png` }));

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
  await go(page, "system");
  await page.locator(".rail button", { hasText: "Ships and stations" }).first().click();
  const desk = page.locator("[role=region][aria-label='Ships and stations']");
  await desk.waitFor();
  await desk.locator("[data-add-here]").click();
  await desk.locator(`[data-template="courier"]`).click();
  await desk.locator("[data-name]").fill("Kittiwake");
  await desk.locator("[data-add]").click();
  await desk.locator("[data-note]", { hasText: "Kittiwake added" }).waitFor();
  // Adding a ship opens its card; pick it only if it is not open already.
  await page.waitForTimeout(500);
  if (!(await desk.locator("[data-aboard-toggle]").isVisible())) await desk.locator("[data-construct]", { hasText: "Kittiwake" }).click();
  await desk.locator("[data-aboard-toggle]").waitFor();
  await desk.locator("[data-power-toggle]").click();
  const card = desk.locator("[data-power-card]");
  await card.waitFor();
  await card.locator("[data-power-typed]").waitFor({ timeout: 15000 });
  assert.match(await card.innerText(), /typed figures rather than parts/);
  await shot(page, "power-typed-desk");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.waitForTimeout(300);
  const over = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  assert.ok(over <= 1, `no sideways scroll on a phone (${over}px)`);
  assert.deepEqual(errors, []);
  console.log("power e2e: ok");
  await browser.close();
} finally {
  server.kill();
}
