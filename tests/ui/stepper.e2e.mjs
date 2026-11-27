// The stepper on the Run desk (TM7, three desks design 3.2), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/stepper.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// On Run the place line draws every level; a level not visited reads as its word. Out and In sit at its end, with Page Up
// and Page Down: In goes onto the body selected, Out up a level, and a world the trail remembers stays pencilled with
// its name. On Explore the place line is as it was, without Out and In.
// Fill the screen (F) on Run: the picture alone, the bar back at the top edge, Esc brings everything back.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk } from "./nav.mjs";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    page.on("pageerror", (e) => errors.push(e.message));
    const shot = async (n) => {
      if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${name}-${n}.png` }));
    };
    await page.goto(base + "#chart");
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const nav = page.locator("nav.place");
    const scale = () => nav.getAttribute("data-scale");
    const atScale = (s) => page.waitForFunction((s) => document.querySelector("nav.place")?.getAttribute("data-scale") === s, s, { timeout: 15000 });

    // Explore: the place line as it was.
    await desk(page, "Explore");
    assert.equal(await nav.getAttribute("data-stepper"), null);
    assert.equal(await page.locator("[data-step-in]").count(), 0);

    // Run: every level drawn, the world not visited yet; In waits for something to go into.
    await desk(page, "Run");
    await page.waitForSelector("nav.place[data-stepper]");
    assert.equal(await scale(), "system");
    const levels = await nav.evaluate((n) => [...n.querySelectorAll("[data-place-step], h1[data-place]")].map((e) => e.getAttribute("data-place-step") ?? e.getAttribute("data-place")));
    assert.deepEqual(levels, ["web", "cluster", "galaxy", "system", "world"]);
    assert.equal(await nav.locator('[data-place-step="world"]').getAttribute("data-unvisited"), "");
    assert.ok(await page.locator("[data-step-in]").isDisabled(), "nothing selected: nothing to go into");
    await shot("1-run");

    // Select Tarn; Page Down goes into it, Page Up comes back, and the line keeps Tarn below, named.
    await page.locator('.area-chart .body[aria-label^="Tarn,"]').first().click();
    await page.waitForFunction(() => !document.querySelector("[data-step-in]")?.hasAttribute("disabled"));
    await page.locator("body").press("PageDown");
    await atScale("world");
    await shot("2-world");
    await page.locator("body").press("PageUp");
    await atScale("system");
    const below = nav.locator('[data-place-step="world"]');
    assert.equal(await below.getAttribute("data-unvisited"), null, "the trail's world is pencilled, not a level word");
    assert.match(await below.innerText(), /Tarn/);

    // Out goes up a level; In comes back down to the system.
    await page.locator("[data-step-out]").click();
    await atScale("galaxy");
    await shot("3-galaxy");
    await page.locator("[data-step-in]").click();
    await atScale("system");

    // Fill the screen (TM9a, design question 11): F on Run leaves the picture alone; the bar comes back at the top edge;
    // Esc brings everything back. On Explore F does nothing.
    const app = page.locator(".app");
    await page.locator("body").press("f");
    await page.waitForSelector(".app[data-filled]");
    assert.ok(!(await page.locator("[data-campaign-clock]").isVisible()), "no clock");
    assert.ok(!(await page.locator(".viewbar").isVisible()), "no view controls");
    await page.mouse.move(700, 400);
    await page.waitForFunction(() => (document.querySelector("header.bar")?.getBoundingClientRect().bottom ?? 1) <= 1);
    // The picture takes the whole window (the stage must not slide into the bar's empty grid row).
    // Firefox honours the full-screen request and the window becomes the screen, so compare with the window as it is.
    const pic = await page.locator(".area-chart").boundingBox();
    const winH = await page.evaluate(() => window.innerHeight);
    assert.ok(pic && pic.y <= 1 && pic.height >= winH - 10, `the picture fills the window (${winH} px): ${JSON.stringify(pic)}`);
    await shot("4-filled");
    await page.mouse.move(700, 2);
    await page.waitForSelector(".app[data-peek]");
    await page.waitForFunction(() => (document.querySelector("header.bar")?.getBoundingClientRect().top ?? -1) >= 0);
    await shot("5-peek");
    await page.mouse.move(700, 400);
    await page.waitForSelector(".app:not([data-peek])");
    await page.locator("body").press("Escape");
    await page.waitForSelector(".app:not([data-filled])");
    assert.ok(await page.locator("[data-campaign-clock]").isVisible(), "the clock is back");
    await desk(page, "Explore");
    await page.locator("body").press("f");
    await page.waitForTimeout(300);
    assert.equal(await app.getAttribute("data-filled"), null, "F does nothing on Explore");

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`stepper ${name}: ok`);
  }
} finally {
  server.kill();
}
