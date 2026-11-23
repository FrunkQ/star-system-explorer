// U6 part 1 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/galaxy.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Explore > Galaxy grows the Milky-Way-like galaxy from the universe's seed
// through the kernel's galaxy_chart: 24 rings, arms, the green band, history
// curves with epochs, and the ring at the Sun's distance picked with its
// ledger (what died here, conditions, the green verdict). Another ring, a
// layer, an earlier time and the dwarf galaxy all redraw. Night chart and
// phone (no sideways scroll) too.
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
    await go(gm, "galaxy");
    await gm.waitForSelector(".gal [data-ring]", { timeout: 30000 });
    {
      const p = await gm.evaluate(() => window.__places.at(-1));
      assert.deepEqual(p, { scale: "galaxy", id: "milky-way-like", name: "A Milky-Way-like galaxy" }, "the galaxy chart says where it is");
    }
    const gal = gm.locator(".gal");
    assert.match(await gal.getAttribute("data-galaxy"), /halo:milky-way-like$/);
    assert.equal(await gal.locator("circle.ring").count(), 24, "24 rings");
    assert.ok((await gal.locator("path.arm").count()) >= 2, "arms");
    assert.equal(await gal.locator("circle.green").count(), 1, "a green band now");
    assert.match(await gal.locator(".lede").innerText(), /barred spiral/);
    assert.ok((await gal.locator("path.curve").count()) === 3, "three history curves");
    assert.match(await gal.locator(".ledger h3").innerText(), /at the Sun's distance/);
    assert.match(await gal.locator(".ledger").innerText(), /Neutron-star mergers/);
    await gal.locator("[data-green]").waitFor({ timeout: 20000 });
    // GH9: the scars its history left, over the face-on disc and edge-on below it.
    assert.equal(await gal.locator("[data-scars] path.scar-edge").count(), 1, "the scarred outline");
    assert.match(await gal.locator("[data-edge-on] figcaption").innerText(), /Edge-on, heights 5 times\. Scars now: .*(warped edge|ripples|thick disc)/);
    await shot(gm, "u6-galaxy");

    // Another ring: its ledger.
    const before = await gal.locator("[data-ring]").getAttribute("data-ring");
    await gal.locator("[data-ring-pick]").selectOption("3");
    await gm.waitForFunction((b) => document.querySelector(".gal [data-ring]")?.getAttribute("data-ring") !== b, before);
    // A layer and an earlier time.
    await gal.locator("[data-layer=supernovae]").click();
    await gal.locator("[data-time]").fill("6");
    await gal.locator("[data-time]").dispatchEvent("change");
    await gm.waitForFunction(() => /At 6\.0 Gyr after the Big Bang/.test(document.querySelector(".gal .lede")?.textContent ?? ""), null, { timeout: 30000 });
    await shot(gm, "gh9-galaxy-6gyr");
    // The dwarf.
    await gal.locator("[data-halo]").selectOption("dwarf");
    await gm.waitForFunction(() => window.__places.at(-1)?.id === "dwarf");
    await gm.waitForFunction(() => /halo:dwarf$/.test(document.querySelector(".gal")?.getAttribute("data-galaxy") ?? ""), null, { timeout: 30000 });
    await gal.locator("[data-halo]").selectOption("milky-way-like");
    await gm.waitForFunction(() => /halo:milky-way-like$/.test(document.querySelector(".gal")?.getAttribute("data-galaxy") ?? ""), null, { timeout: 30000 });
    await gal.locator("[data-green]").waitFor({ timeout: 20000 });

    // Ink history (G6): a major merger at 8 Gyr redraws the galaxy from then on, with a ghost of the past.
    assert.equal(await gal.locator("[data-inked]").count(), 0);
    await gal.locator("[data-ink-kind]").selectOption("merger");
    await gal.locator("[data-ink-t]").fill("8");
    await gal.locator("[data-ink-ratio]").selectOption({ label: "equal (1 : 1)" });
    await gal.locator("[data-ink-add]").click();
    await gal.locator("[data-inked]").waitFor({ timeout: 30000 });
    await gm.waitForFunction(() => /from 8\.\d Gyr on/.test(document.querySelector(".gal [data-compare]")?.textContent ?? ""), null, { timeout: 30000 });
    assert.ok((await gal.locator("path.ghost").count()) >= 2, "ghost curves");
    assert.equal(await gal.locator("path.mark.inked").count(), 1, "the inked merger marked");
    const own = await gal.locator("[data-merger]").count();
    if (own) {
      await gal.locator("[data-merger='0'] button").click();
      await gm.waitForSelector(".gal [data-merger='0'].out", { timeout: 30000 });
    }
    await shot(gm, "u6-galaxy-inked");
    await gal.locator("[data-ink-clear]").click();
    await gm.waitForFunction(() => !document.querySelector(".gal [data-inked]") && !document.querySelector(".gal path.ghost"), null, { timeout: 30000 });

    await more(gm, "Night chart");
    await gm.waitForTimeout(300);
    await shot(gm, "u6-galaxy-night");

    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "galaxy");
    await phone.waitForSelector(".gal [data-ring]", { timeout: 30000 });
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "u6-galaxy-phone", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: galaxy chart from galaxy_chart: rings, arms, green band, curves, ledger, layers, time, dwarf, inked history, night chart, phone`);
  }
} finally {
  server.kill();
}
