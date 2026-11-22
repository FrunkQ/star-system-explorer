// V8 Window in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/window.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Explore > Window opens from beside Kestrel's temperate world with the eyes:
// the star is a blinding disc, some planets are points of light, and the
// table lists only what is seen. The telescope looking at a giant resolves
// its lit disc; the GM can list what the eyes miss. Night chart and phone too.
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
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(gm, "window");
    await gm.waitForSelector(".win[data-window^='eye:']", { timeout: 30000 });
    const win = gm.locator(".win");
    assert.match(await win.locator("[data-lede]").innerText(), /The star is a disc, blinding/);
    const rows = await win.locator("[data-seen]").count();
    assert.ok(rows >= 2, `the star and at least one world seen (${rows})`);
    const ink = () => gm.evaluate(() => { const c = document.querySelector("[data-sky]"); const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data; let n = 0; for (let i = 0; i < d.length; i += 4) if (Math.abs(d[i] - d[0]) > 40) n++; return n; });
    assert.ok((await ink()) > 50, "the star is drawn");
    await shot(gm, "v8-window");

    // The telescope at the biggest world it lists: a lit disc.
    await win.locator("[data-sensor=telescope]").click();
    await gm.waitForSelector(".win[data-window^='telescope:']", { timeout: 30000 });
    const planets = await win.locator("[data-target] option").evaluateAll((os) => os.map((o) => o.value).filter((v) => v !== "star"));
    let disc = false;
    for (const p of planets) {
      await win.locator("[data-target]").selectOption(p);
      await gm.waitForTimeout(150);
      if (/is a disc/.test(await win.locator("[data-lede]").innerText())) { disc = true; break; }
    }
    assert.ok(disc, "the telescope resolves some world's disc");
    await shot(gm, "v8-window-telescope");

    // The eyes miss something; the GM can list it.
    await win.locator("[data-sensor=eye]").click();
    await gm.waitForSelector(".win[data-window^='eye:']", { timeout: 30000 });
    if (await win.locator("[data-missed-toggle]").count()) {
      await win.locator("[data-missed-toggle]").click();
      assert.ok((await win.locator("[data-missed]").count()) >= 1);
    }

    await more(gm, "Night chart");
    await gm.waitForTimeout(400);
    await win.locator("[data-target]").selectOption("star");
    await gm.waitForTimeout(200);
    await shot(gm, "v8-window-night");

    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "window");
    await phone.waitForSelector(".win[data-window^='eye:']", { timeout: 30000 });
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "v8-window-phone", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: window from window_view: star and worlds seen with the eyes, telescope resolves a disc, missed list for the GM, night chart, phone`);
  }
} finally {
  server.kill();
}
