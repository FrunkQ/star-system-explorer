// Pencil and ink on Voyages (SV3 on the travel screens), against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/travel-pencil.e2e.mjs [--screens <dir>]. In the sample campaign Ember is unknown to the table and
// Tarn only heard of. On the Run desk, Voyages draws Ember's mark in pencil (a dotted ring) with a pencilled name, Tarn
// in a thin solid ring, and the leg that reaches Ember in pencil. On Explore every mark is in ink.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9500 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 });
  const errors = [];
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await desk(gm, "Run");
  await go(gm, "voyages");
  const ins = gm.locator(".insys");
  await ins.waitFor({ timeout: 30000 });
  const placeOf = (n) => gm.evaluate((n) => [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === n)?.getAttribute("data-place"), n);
  await gm.waitForFunction(() => [...document.querySelectorAll(".insys [data-place]")].some((e) => e.getAttribute("aria-label") === "Ember"), null, { timeout: 60000 });
  const penOf = (n) => gm.evaluate((n) => {
    const hit = [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === n);
    return hit?.closest("g.place")?.querySelector("[data-pen]")?.getAttribute("data-pen") ?? null;
  }, n);
  await gm.waitForFunction(() => {
    const hit = [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === "Ember");
    return hit?.closest("g.place")?.querySelector("[data-pen]")?.getAttribute("data-pen") === "pencil";
  }, null, { timeout: 30000 });
  assert.equal(await penOf("Tarn"), "heard", "a world the table has heard of gets a thin solid ring");
  const ring = await gm.evaluate(() => {
    const hit = [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === "Ember");
    return hit?.closest("g.place")?.querySelector("[data-pen-ring]")?.getAttribute("stroke-dasharray") ?? null;
  });
  assert.ok(ring, "Ember sits in a dotted ring");
  const emberName = await gm.evaluate(() => {
    const hit = [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === "Ember");
    return hit?.closest("g.place")?.querySelector("text[data-label-lane]")?.getAttribute("style") ?? "";
  });
  assert.match(emberName, /italic/, "Ember's name is pencilled");

  // The leg that reaches Ember is in pencil.
  const ember = await placeOf("Ember");
  await ins.locator(`[data-place='${ember}']`).dispatchEvent("click");
  await gm.waitForFunction((m) => document.querySelector(".insys")?.getAttribute("data-voyage")?.endsWith(`>${m}`), ember, { timeout: 60000 });
  await gm.waitForSelector(".insys path.arc[data-pen='pencil']", { timeout: 15000 });
  await shot(gm, "voyages-pencil-run-desk");

  // On Explore everything is in ink.
  await desk(gm, "Explore");
  await gm.waitForFunction(() => !document.querySelector(".insys [data-pen='pencil'], .insys [data-pen='heard']"), null, { timeout: 15000 });
  await shot(gm, "voyages-ink-explore");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("travel pencil e2e: ok");
} finally {
  server.kill();
}
