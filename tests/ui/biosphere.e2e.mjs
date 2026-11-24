// U7 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/biosphere.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The Life screen opens on the example Earth: ten named epochs on the
// timeline, a food web, a confident scan. Picking an epoch tells its story in
// the chosen lens. Europa's scan is blind to the ice and its hook says hidden
// life; the red-dwarf desert is a false alarm. Selecting Tarn asks the kernel
// for its biosphere, and the view says there is none yet and keeps the
// example. On a phone the screen menu reaches it and nothing overflows.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(300);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: false }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(gm, "life");
    await gm.waitForSelector("[data-biosphere='fixture:earth']", { timeout: 15000 });
    const bio = gm.locator(".bio");
    assert.match(await bio.locator("h2").innerText(), /The living world of Earth today/);
    assert.equal(await bio.locator("[data-example]").count(), 1, "labelled as an example world");
    assert.equal(await bio.locator("rect.band").count(), 10, "ten epochs");
    assert.ok((await bio.locator("circle.node").count()) > 8, "a food web");
    assert.match(await bio.locator("[data-tier=scanned] + dd").innerText(), /life candidate, confidence 99%/);
    assert.match(await bio.locator(".why").first().innerText(), /Now: The age of shells and bones/);
    await shot(gm, "u7-earth");

    // An epoch's story, in two lenses.
    await bio.getByLabel("Epoch", { exact: true }).selectOption({ label: await bio.getByLabel("Epoch", { exact: true }).locator("option", { hasText: "The age of green mats" }).first().innerText() });
    await gm.waitForSelector("[data-epoch='The age of green mats']");
    const eco = await bio.locator("[data-epoch]").innerText();
    assert.match(eco, /a major epoch/);
    await bio.getByRole("button", { name: "History" }).click();
    assert.notEqual(await bio.locator("[data-epoch]").innerText(), eco, "the lens changes the telling");
    // A guild's detail from the table.
    await bio.locator("table.guilds tbody tr").first().getByRole("button").click();
    await bio.locator("[data-guild-why]").waitFor();

    // Europa: blind, hidden life.
    await bio.getByLabel("Example world").selectOption("europa");
    await gm.waitForSelector("[data-biosphere='fixture:europa']");
    assert.match(await bio.locator("[data-tier=scanned] + dd").innerText(), /blind to: .*ice/);
    assert.equal(await bio.locator("[data-hook=hidden_life]").count(), 1);
    await shot(gm, "u7-europa");

    // The red-dwarf desert: a false alarm.
    await bio.getByLabel("Example world").selectOption("red-dwarf-desert");
    await gm.waitForSelector("[data-biosphere='fixture:red-dwarf-desert']");
    assert.match(await bio.locator("h2").innerText(), /^Life on Red-dwarf desert/);
    assert.equal(await bio.locator("[data-hook=false_positive]").count(), 1);
    assert.match(await bio.locator("[data-tier=surveyed] + dd").innerText(), /no biosphere found/);

    // The LifeFlow board (U7 part 2): the pipeline that worked it out, stage by stage.
    await bio.locator("[data-show=board]").click();
    await bio.locator("[data-lifeflow]").waitFor();
    assert.equal(await bio.locator("[data-stage][data-live=false][data-stage=ecosystem]").count(), 1, "a false alarm has no ecosystem");
    await bio.getByLabel("Example world").selectOption("earth");
    await gm.waitForSelector("[data-biosphere='fixture:earth'] [data-lifeflow]");
    assert.equal(await bio.locator("g[data-stage]").count(), 11, "eleven stages");
    assert.ok((await bio.locator("path[data-wire]").count()) >= 12, "wired");
    await bio.locator("g[data-stage=innovations]").click();
    const stage = bio.locator("[data-stage-card=innovations]");
    await stage.waitFor();
    assert.ok(Number(await stage.locator("[data-stage-events]").getAttribute("data-stage-events")) >= 5, "its dated inventions");
    assert.equal(await stage.locator("input[data-dial=oxygenic]").isDisabled(), true, "dials are read only until the kernel takes them");
    await bio.locator("g[data-stage=origin]").press("Enter");
    await bio.locator("[data-stage-card=origin] input[data-dial=origin]").waitFor();
    await shot(gm, "u7-board");
    await bio.getByLabel("Example world").selectOption("europa");
    await gm.waitForSelector("[data-biosphere='fixture:europa'] [data-lifeflow]");
    await bio.locator("g[data-stage=hooks]").click();
    await bio.locator("[data-stage-card=hooks] [data-hook=hidden_life]").waitFor();
    await bio.locator("[data-show=story]").click();
    await bio.locator("rect.band").first().waitFor({ state: "attached" }).catch(() => {});

    // A world on the chart: the kernel has no biosphere for it yet, and says so.
    await go(gm, "system");
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    await go(gm, "life");
    await gm.locator("[data-body-note]").waitFor({ timeout: 15000 });
    assert.match(await gm.locator("[data-body-note]").innerText(), /no biosphere for Tarn yet/);
    await gm.waitForSelector("[data-biosphere^='fixture:']");

    // Night chart.
    await more(gm, "Night chart");
    await gm.waitForTimeout(200);
    await shot(gm, "u7-night");

    // Phone.
    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "life");
    await phone.waitForSelector("[data-biosphere='fixture:earth']", { timeout: 15000 });
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "u7-phone");
    await phone.locator("[data-show=board]").click();
    await phone.locator("[data-stage-row=signatures]").click();
    await phone.locator("[data-stage-card=signatures]").waitFor();
    const over2 = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over2 <= 1, `the board fits a phone (${over2}px over)`);
    await shot(phone, "u7-board-phone");

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: biosphere view of the example worlds, epochs in lenses, the LifeFlow board, a chart world's honest note, phone`);
  }
} finally {
  server.kill();
}
