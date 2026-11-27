// The Hazard map (exotic objects design 3.8, XO18; Alex 24 Nov: "a hazard map that can be brought up", Jupiter's belts
// named), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/hazard.e2e.mjs [--screens <dir>]
// Open the Solar System, pick Jupiter, open Hazard map from the rail: the Profile has the belts' row and the climb
// out, the moons' lanes are marked, Park and Dive are drawn and said, Io reads deadly and Callisto clear. Over the poles
// the belts' row folds into the quiet line. The Sun at system scale: heat and the climb set the marks. Reading writes
// nothing; a phone gets the same strip.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, pinClock } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = pinClock(await chromium.launch());
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    const b = gm.locator('.area-chart .body[aria-label^="Jupiter,"]').first();
    await b.waitFor({ timeout: 20000 });
    await b.locator(".hit").click();
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await go(gm, "hazard");
    await gm.waitForSelector('[data-hazard-map="sol/jupiter"]', { timeout: 30000 });
    const hm = gm.locator("[data-hazard-map]");
    assert.equal(await hm.locator('[data-row="radiation_belts"]').count(), 1, "the belts' row");
    assert.equal(await hm.locator('[data-row="climbing_out"]').count(), 1, "the climb out");
    assert.ok((await hm.locator('[data-row="radiation_belts"] [data-tone="deadly"]').count()) >= 1);
    assert.equal(await hm.locator('[data-ship-mark="park"]').count(), 1);
    assert.equal(await hm.locator('[data-ship-mark="dive"]').count(), 1);
    assert.match(await hm.locator('[data-says="park"]').innerText(), /^Park .* radii, set by the radiation from the belts/);
    assert.match(await hm.locator('[data-says="dive"]').innerText(), /Inside it is too late/);
    // Clocks at the marks (XO20): what a clock loses a day there, the push to hover, the time lost on a pass.
    assert.match(await hm.locator('[data-says="park"]').innerText(), /loses .* a day against one far away.*of thrust/);
    assert.match(await hm.locator('[data-says="dive"]').innerText(), /falls .* behind a far one/);
    assert.ok((await hm.locator('[data-mark^="lane swept by Io"]').count()) === 1, "Io's lane is marked");
    assert.equal(await hm.locator('[data-moon="sol/io"]').getAttribute("data-tone"), "deadly");
    assert.equal(await hm.locator('[data-moon="sol/callisto"]').getAttribute("data-tone"), "clear");
    assert.match(await hm.innerText(), /stock ship/);
    await shot(gm, `hazard-1-jupiter-equator-${vp}`);

    // Over the poles: no field line closes there, so the belts' row folds away.
    await hm.locator('[data-line-pick="poles"]').click();
    await gm.waitForSelector('[data-hazard-map="sol/jupiter"][data-line="poles"]');
    await gm.waitForFunction(() => !document.querySelector('[data-hazard-map] [data-row="radiation_belts"]') && document.querySelector('[data-hazard-map] [data-says="quiet"]'));
    assert.match(await hm.locator('[data-says="quiet"]').innerText(), /Radiation: trapped belts/);
    await shot(gm, `hazard-2-jupiter-poles-${vp}`);

    // Reading changes nothing.
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0);
    assert.deepEqual(errors, [], errors.join("\n"));
    await browser.close();
  }
  console.log("hazard e2e: ok");
} finally {
  server.kill();
}
