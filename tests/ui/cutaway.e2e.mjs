// V6 cutaway in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/cutaway.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Select Tarn on the chart, open the Body screen: the cut shows its shells
// (the south pole's ice and brine in their own sector) and the ladder lists
// the nine Strata lines as the kernel words them, served by read_world_stack,
// with the kernel's physics notes under a picked line. A body with no stack says
// so and offers the worlds that have one. Nothing here changes the state hash.
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
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await gm.waitForTimeout(600);
    const h0 = await gm.locator("[data-state-hash]").getAttribute("data-state-hash");

    // Nothing selected yet: the Body screen offers the worlds that have a stack.
    await go(gm, "body");
    await gm.waitForSelector("[data-cutaway]");
    assert.equal(await gm.locator("[data-cutaway]").getAttribute("data-cutaway"), "none");
    await gm.locator("[data-cutaway]").getByRole("button", { name: "Tarn" }).click();
    await gm.waitForSelector('[data-cutaway="u:kestrel/sys:1/body:3"]');
    const cut = gm.locator("[data-cutaway]");
    assert.equal(await cut.locator("[data-shell]").count(), 6, "four global shells and two under the south pole");
    assert.equal(await cut.locator("[data-within=south-pole]").count(), 2);
    const lines = cut.getByRole("list", { name: "Above, on and within Tarn" }).locator("li");
    assert.equal(await lines.count(), 9);
    assert.match(await lines.nth(0).innerText(), /\+400 km\s+Tarn High, station\s+orbit/);
    assert.match(await lines.nth(5).innerText(), /−3 to −11 km\s+Sub-ice brine ocean\s+south pole/);
    assert.match(await lines.nth(8).innerText(), /−2,900 km\s+Liquid core, dynamo on/);
    // The card beside it is Tarn's.
    assert.match(await gm.locator(".area-card").innerText(), /Tarn/);
    assert.equal(await cut.getAttribute("data-source"), "fixture:tarn", "served by the kernel");
    await lines.nth(5).getByRole("button").click();
    assert.equal(await lines.nth(5).getByRole("button").getAttribute("aria-pressed"), "true");
    // The kernel's physics notes for the picked line.
    assert.match(await cut.getByLabel("Physics of Sub-ice brine ocean").innerText(), /freezes at 265\.1 K.*liquid/);
    await shot(gm, `cutaway-${name}`);
    // Scale is the view controls' (one set of controls, Alex 23 Nov): Readable spreads depth on a log scale, Real draws it to scale.
    assert.equal(await cut.getAttribute("data-scale"), "log");
    const scaleBar = gm.getByRole("group", { name: "Scale" });
    await scaleBar.getByRole("button", { name: "Real", exact: true }).click();
    await gm.waitForSelector('[data-cutaway][data-scale="true"]');
    assert.match(await cut.locator("figcaption").innerText(), /to scale/);
    await scaleBar.getByRole("button", { name: "Readable", exact: true }).click();
    await gm.waitForSelector('[data-cutaway][data-scale="log"]');

    // A body with no stack says so.
    await go(gm, "system");
    await gm.locator('.area-chart .body[aria-label^="Kestrel"]').first().click();
    await go(gm, "body");
    await gm.waitForSelector('[data-cutaway="none"]');
    assert.match(await gm.locator("[data-cutaway]").innerText(), /no cutaway yet/);
    assert.equal(await gm.locator("[data-state-hash]").getAttribute("data-state-hash"), h0, "looking inside changes nothing");

    // Phone width: no sideways scroll.
    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(e.message));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await phone.waitForTimeout(500);
    await phone.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    await go(phone, "body");
    await phone.waitForSelector('[data-cutaway="u:kestrel/sys:1/body:3"]');
    assert.ok(await phone.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), "no sideways scroll at phone width");
    await shot(phone, `cutaway-phone-${name}`);
    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`cutaway e2e ${name}: ok`);
  }
} finally {
  server.kill();
}
