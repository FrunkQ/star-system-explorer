// V8 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/route.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The Route screen plots Tarn to Vault for the fusion torch from the kernel's
// plan_route: up to three courses side by side, the chosen one drawn on the
// chart in Kestrel's frame with burns and the strip of numbers. Choosing
// another course redraws the chart; "Keep us safe" reorders by difficulty; a
// chemical ship to Sable is refused with what it would need. Night chart and
// phone (no sideways scroll) are checked too.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
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
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await gm.getByRole("button", { name: "Explore", exact: true }).click();
    await gm.locator("[data-section-tab=route]").click();
    const route = gm.locator(".route");
    await gm.waitForSelector(".route [data-course]", { timeout: 20000 });
    assert.equal(await route.locator("[data-from]").inputValue(), "u:kestrel/sys:1/body:3", "from Tarn, the world in the temperate zone");
    assert.equal(await route.locator("[data-to]").inputValue(), "u:kestrel/sys:1/body:4", "to Vault, the next world out");
    const cards = route.locator("article[data-course]");
    const n = await cards.count();
    assert.ok(n >= 2 && n <= 3, `two or three courses side by side (${n})`);
    assert.match(await route.locator("h2").nth(1).innerText(), /ways to Vault/);
    assert.ok((await route.locator("svg.big path.track").getAttribute("d")).length > 100, "the chosen course is drawn");
    assert.ok((await route.locator("svg.big circle.burn").count()) >= 1, "its burns are marked");
    await shot(gm, "v8-route");

    // Choose another course: the chart follows.
    const first = await route.locator("[data-chosen]").getAttribute("data-chosen");
    await cards.nth(1).getByRole("button").click();
    assert.notEqual(await route.locator("[data-chosen]").getAttribute("data-chosen"), first);

    // Keep us safe: the first card is the least difficult.
    await route.locator("[data-want=safest]").click();
    await gm.waitForTimeout(300);
    assert.match(await cards.first().locator(".tag").innerText(), /safest/i);

    // A chemical ship to Sable: refused, with what it would need.
    await route.locator("[data-ship]").selectOption("chemical-transfer");
    await route.locator("[data-to]").selectOption("u:kestrel/sys:1/body:2");
    await gm.waitForSelector(".route [data-none]", { timeout: 15000 });
    assert.match(await route.locator("[data-none]").innerText(), /needs .* km\/s/);
    await route.locator("[data-ship]").selectOption("fusion-torch");
    await route.locator("[data-to]").selectOption("u:kestrel/sys:1/body:4");
    await gm.waitForSelector(".route [data-course]", { timeout: 15000 });
    await route.locator("[data-want=balanced]").click();

    // Night chart.
    await gm.getByRole("button", { name: "Night chart" }).click();
    await gm.waitForTimeout(300);
    await shot(gm, "v8-route-night");
    await gm.getByRole("button", { name: "Chart room" }).click();

    // Phone, both modes.
    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base);
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await phone.getByLabel("Screen menu").selectOption("explore");
    await phone.locator("[data-section-tab=route]").click();
    await phone.waitForSelector(".route [data-course]", { timeout: 20000 });
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "v8-route-phone", true);
    await phone.getByRole("button", { name: "Night chart" }).click().catch(() => {});
    await phone.waitForTimeout(300);
    await shot(phone, "v8-route-phone-night", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: route console plots Tarn to Vault from plan_route, courses compared, chosen course charted, refusal shown, night chart, phone`);
  }
} finally {
  server.kill();
}
