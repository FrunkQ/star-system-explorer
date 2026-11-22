// The place line at every scale (U24, top-level design 2.1), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/place.e2e.mjs [--screens <dir>]
// The line runs Cosmic web › Galaxies nearby › Galaxy › Kestrel › a world. Going down a body list opens a world;
// going up to the galaxy keeps the way back (the system and the world stay on the line, quietly) and the rail offers
// "Back to <world>". The "Below" list at the galaxy offers the campaign's systems and recent places. Esc climbs one
// scale. Each place is the address bar's hash: Back and Forward move through places, and a link opens straight on its
// place with no start sheet. On a phone the line is "‹ name ▾". Moving about writes nothing.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: false }));
};
const at = (page) => page.locator(".place h1.title").getAttribute("data-place");
const until = (page, scale) => page.waitForFunction((s) => document.querySelector(".place h1.title")?.getAttribute("data-place") === s, scale, { timeout: 20000 });
const ready = (page) => page.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 20000 });

try {
  const browser = await chromium.launch();
  const errors = [];
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(root + "#chart");
  await ready(gm);
  const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  const line = gm.locator(".place");
  assert.equal(await at(gm), "system");
  assert.deepEqual(await line.locator("[data-place-step]").evaluateAll((b) => b.map((x) => x.title)), ["Up to Cosmic web", "Up to Galaxies nearby", "Up to Galaxy (Esc)"]);
  assert.equal(await line.locator("h1").innerText(), "Kestrel");

  // Down to a world from the list beside the name.
  const below = line.getByLabel("Go to a body");
  const tarn = await below.locator("option").nth(4).getAttribute("value");
  const tarnName = (await below.locator("option").nth(4).innerText()).trim();
  await below.selectOption(tarn);
  await until(gm, "world");
  assert.equal((await line.locator("h1").innerText()).trim(), tarnName);
  assert.equal(new URL(gm.url()).hash, `#chart/world//${encodeURIComponent(tarn)}`);

  // Up to the galaxy: the system and the world stay on the line as the way back.
  await line.locator('[data-place-step="galaxy"]').click();
  await until(gm, "galaxy");
  await gm.locator(".gal").waitFor({ timeout: 30000 });
  assert.equal(new URL(gm.url()).hash, "#chart/galaxy");
  assert.deepEqual((await line.locator("[data-place-step].below").allInnerTexts()).map((s) => s.trim()), ["Kestrel", tarnName]);
  assert.match(await gm.locator('.rail [data-go="back"]').innerText(), new RegExp(`Back to ${tarnName}`));
  const down = line.getByLabel("Go down");
  assert.deepEqual(await down.locator('optgroup[label="Systems in this campaign"] option').allInnerTexts(), ["Kestrel"]);
  assert.ok((await down.locator('optgroup[label="Recent"] option').allInnerTexts()).some((t) => t.startsWith(`${tarnName} · Kestrel`)), "the world is a recent place");
  await shot(gm, "u24-galaxy-trail");

  // One press back to where the GM was.
  await line.locator('[data-place-step="world"]').click();
  await until(gm, "world");
  assert.equal((await line.locator("h1").innerText()).trim(), tarnName);

  // The browser's Back and Forward move through places.
  await gm.goBack();
  await until(gm, "galaxy");
  await gm.goBack();
  await until(gm, "world");
  await gm.goForward();
  await until(gm, "galaxy");

  // Esc climbs a scale at a time; the list below goes back down.
  await gm.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
  await gm.keyboard.press("Escape");
  if ((await at(gm)) === "galaxy") await gm.keyboard.press("Escape"); // the first may clear a picked galaxy
  await until(gm, "cluster");
  await gm.keyboard.press("Escape");
  await until(gm, "web");
  assert.deepEqual(await line.locator("[data-place-step]:not(.below)").count(), 0);
  await line.getByLabel("Go down").selectOption("scale:cluster");
  await until(gm, "cluster");
  await line.getByLabel("Go down").selectOption("sys:");
  await until(gm, "system");
  assert.equal(await line.locator("h1").innerText(), "Kestrel");

  // A link opens straight on its place, with no start sheet.
  const linked = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
  linked.on("pageerror", (e) => errors.push(`link: ${e.message}`));
  await linked.goto(root + `#chart/world//${encodeURIComponent(tarn)}~cutaway`);
  await ready(linked);
  await until(linked, "world");
  assert.equal(await linked.locator(".stage").getAttribute("data-screen"), "cutaway");
  assert.equal(await linked.locator(".start, [data-start-sheet]").count(), 0);
  await linked.goto(root + "#chart/web");
  await ready(linked);
  await until(linked, "web");

  await more(gm, /Night chart/);
  await go(gm, "galaxy");
  await shot(gm, "u24-galaxy-night");

  // Phone: "‹ name ▾".
  const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
  phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
  await phone.goto(root + `#chart/world//${encodeURIComponent(tarn)}`);
  await ready(phone);
  await until(phone, "world");
  const pl = phone.locator(".place");
  assert.equal(await pl.locator("[data-place-step].far").first().isVisible(), false);
  assert.ok(await pl.locator(".step.up").isVisible(), "the up arrow shows");
  assert.ok(await pl.locator(".goto-mark").isVisible(), "the down mark shows");
  await shot(phone, "u24-phone-world");
  await go(phone, "galaxy");
  assert.equal(await at(phone), "galaxy");
  assert.ok(await pl.locator(".goto-mark").isVisible());
  await pl.getByLabel("Go down").selectOption("sys:");
  await until(phone, "system");
  const overflow = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(overflow <= 1, `nothing runs off the side (${overflow}px)`);
  await shot(phone, "u24-phone-system");

  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "moving about writes nothing");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("place e2e: ok");
} finally {
  server.kill();
}
