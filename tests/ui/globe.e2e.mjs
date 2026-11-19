// V6 part 4 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/globe.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Add the sample V3 save (Sol) to the campaign, pick Earth, open the Body
// screen: the globe is drawn from the kernel's six surface tiles and says
// Earth is mostly under water; arrow keys turn it. Tarn (a fixture world) says
// it has no surface yet but keeps its Cutaway. On a phone the screen menu
// reaches the Body screen.
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
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const screenBtn = (page, n) => page.getByRole("group", { name: "Screen" }).getByRole("button", { name: n, exact: true });
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });

    // Tarn first: a fixture world has a Cutaway but no surface tiles.
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    await screenBtn(gm, "Body").click();
    await gm.waitForSelector('[data-body="no-surface"]', { timeout: 15000 });
    assert.match(await gm.locator("[data-body]").innerText(), /No surface for Tarn yet/);
    await gm.waitForSelector('[data-cutaway="u:kestrel/sys:1/body:3"]');

    // Add the sample Sol, pick Earth, open the Body screen.
    await screenBtn(gm, "Chart").click();
    await gm.getByRole("button", { name: "Make a system" }).click();
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.locator('.area-chart .body[aria-label^="Earth"]').first().waitFor({ timeout: 15000 });
    await gm.locator('.area-chart .body[aria-label^="Earth"]').first().click();
    await screenBtn(gm, "Body").click();
    const t0 = Date.now();
    await gm.waitForSelector("[data-body] canvas", { timeout: 30000 });
    const built = Date.now() - t0;
    const covers = await gm.locator("[data-covers]").innerText();
    const water = Number(/(\d+)% under water/.exec(covers)?.[1] ?? "0");
    assert.ok(water > 40, `Earth is mostly under water: ${covers}`);
    assert.ok((await gm.locator("[data-regime]").innerText()).length > 0, "the regime is named");
    const canvas = gm.locator("[data-body] canvas");
    assert.match(await canvas.getAttribute("aria-label"), /Globe of Earth\. Earth: \d+% under water/);
    // The disk is drawn: the centre pixel is opaque and the corner is not.
    const px = await canvas.evaluate((c) => {
      const g = c.getContext("2d");
      const n = c.width;
      return [g.getImageData(n / 2, n / 2, 1, 1).data[3], g.getImageData(1, 1, 1, 1).data[3]];
    });
    assert.deepEqual(px, [255, 0]);
    const centre = () => canvas.evaluate((c) => Array.from(c.getContext("2d").getImageData(c.width / 2, c.height / 2, 1, 1).data).join(","));
    const before = await centre();
    await canvas.focus();
    await gm.keyboard.press("Shift+ArrowRight");
    await gm.keyboard.press("Shift+ArrowRight");
    assert.equal(await canvas.getAttribute("data-lon"), "90");
    await gm.waitForTimeout(200);
    const after = await centre();
    assert.notEqual(after, before, "turning the globe redraws it");
    // The Cutaway below names Earth too, from the kernel.
    await gm.waitForSelector("[data-cutaway]");
    await shot(gm, `globe-${name}`);

    // Phone: the screen menu reaches the Body screen.
    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true })).newPage();
    phone.on("pageerror", (e) => errors.push(e.message));
    await phone.goto(base);
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const menu = phone.getByLabel("Screen menu");
    assert.ok(await menu.isVisible(), "the screen menu shows on a phone");
    await menu.selectOption("body");
    await phone.waitForSelector("[data-body]");
    await menu.selectOption("symbols");
    await phone.waitForSelector('.stage[data-screen="symbols"]');
    const wide = await phone.evaluate(() => document.documentElement.scrollWidth);
    assert.ok(wide <= 390, `no sideways scroll on a phone: ${wide}`);
    await shot(phone, `globe-phone-${name}`);

    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`globe e2e ${name}: ok (Earth ${water}% under water; tiles in ${built} ms)`);
  }
} finally {
  server.kill();
}
