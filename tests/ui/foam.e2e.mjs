// V9 foam map in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/foam.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Explore > Cosmic web grows a slice of the web from the universe's seed
// through the kernel's web_slice: drawn on a canvas, all four environments
// present, with a ledger. An early time has less contrast than today; Play runs to today;
// the environment shading and another slice size redraw. Night chart changes
// the canvas ground. Phone (no sideways scroll) too.
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
const shot = async (page, n, full = false) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: full }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    // The place line hears where the view is (sse:place).
    await gm.addInitScript(() => {
      window.__places = [];
      document.addEventListener("sse:place", (e) => window.__places.push(e.detail));
    });
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(gm, "web");
    await gm.waitForSelector(".foam[data-web^='200:']", { timeout: 60000 });
    assert.deepEqual(await gm.evaluate(() => window.__places.at(-1)), { scale: "web", id: "web:200", name: "The cosmic web, 200 Mpc across" }, "the web map says where it is");
    const foam = gm.locator(".foam");
    assert.match(await foam.locator("[data-lede]").innerText(), /^Today/);
    const share = async (k) => Number((await foam.locator(`[data-env=${k}] dd`).innerText()).replace("%", ""));
    const all = [await share("void"), await share("sheet"), await share("filament"), await share("knot")];
    assert.ok(Math.abs(all.reduce((a, b) => a + b, 0) - 100) <= 3 && all.every((x) => x > 0), `every environment present (${all})`);
    const corner = () => gm.evaluate(() => { const c = document.querySelector("[data-foam]"); const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data; let sum = 0; for (let i = 0; i < d.length; i += 4) sum += d[i]; return { first: [d[0], d[1], d[2]], mean: sum / (d.length / 4) }; });
    const light = await corner();
    assert.ok(light.mean > 0 && light.mean < 255, "something drawn");
    const today = Number(await foam.locator("[data-contrast]").innerText());
    await shot(gm, "v9-foam");

    // Early: less contrast.
    await foam.locator("[data-time]").fill("1");
    await gm.waitForFunction(() => /after the Big Bang/.test(document.querySelector(".foam [data-lede]")?.textContent ?? ""), null, { timeout: 60000 });
    const early = Number(await foam.locator("[data-contrast]").innerText());
    assert.ok(early < today, `the foam sharpens with time (${early} then, ${today} now)`);
    // Play to today.
    await foam.locator("[data-play]").click();
    await gm.waitForFunction(() => /^Today/.test(document.querySelector(".foam [data-lede]")?.textContent?.trim() ?? "") && document.querySelector(".foam [data-play]")?.textContent === "Play", null, { timeout: 120000 });
    // Environment shading and another size.
    await foam.locator("[data-show=environment]").click();
    await foam.locator("[data-extent='100']").click();
    await gm.waitForSelector(".foam[data-web^='100:']", { timeout: 60000 });
    assert.equal(await gm.evaluate(() => window.__places.at(-1).id), "web:100");
    await shot(gm, "v9-foam-environment");

    await more(gm, "Night chart");
    await gm.waitForTimeout(400);
    const dark = await corner();
    assert.ok(dark.mean < light.mean, `night chart redraws the ground darker (${dark.mean} < ${light.mean})`);
    await foam.locator("[data-show=density]").click();
    await gm.waitForTimeout(300);
    await shot(gm, "v9-foam-night");

    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "web");
    await phone.waitForSelector(".foam[data-web^='200:']", { timeout: 60000 });
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "v9-foam-phone", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: foam map from web_slice: drawn, four environments, sharpens with time, plays, environments, sizes, night chart, phone`);
  }
} finally {
  server.kill();
}
