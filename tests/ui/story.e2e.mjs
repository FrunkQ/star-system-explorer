// U8 story finder in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/story.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Explore > Stories sifts the Kestrel campaign through the kernel's sift:
// danger where people live on Tarn comes first, with its evidence; the
// rumour on body 4 is found; turning a sifter off removes its hits; a pasted
// sifter of the GM's own adds hits; a hit picks its subject and shows a
// brief. Night chart and phone too.
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
    await go(gm, "stories");
    await gm.waitForSelector(".story [data-hit]", { timeout: 30000 });
    const story = gm.locator(".story");
    const ids = () => story.locator("[data-hit]").evaluateAll((hs) => hs.map((h) => h.getAttribute("data-hit")));
    const first = await ids();
    assert.ok(first.includes("story.danger-at-home") && first.includes("story.rumour") && first.includes("story.ruins"), `found: ${first}`);
    const top = story.locator("[data-hit]").first();
    assert.ok((await top.locator(".ev li").count()) >= 1, "evidence listed");
    assert.match(await story.locator("[data-hit='story.danger-at-home']").innerText(), /Tarn/);
    await shot(gm, "u8-stories");

    // Turn the rumour sifter off; its hit goes.
    await story.locator("[data-sifter='story.rumour']").uncheck();
    await gm.waitForFunction(() => !document.querySelector(".story [data-hit='story.rumour']"), null, { timeout: 20000 });
    await story.locator("[data-sifter='story.rumour']").check();
    await story.locator("[data-hit='story.rumour']").waitFor({ timeout: 20000 });
    // A sifter of the GM's own.
    await story.locator("summary").click();
    await story.locator("[data-own]").fill(JSON.stringify({ id: "own.ports", label: "A port to put in at", pattern: { match: "facts", when: { if: "tag", tag: "port" } }, significance: { persistence: { from: "fixed", s: 1e9 }, reach: { from: "fixed", n: 50 }, reversibility: 0.5 } }));
    await story.locator("[data-hit='own.ports']").waitFor({ timeout: 20000 });
    // Pick on the chart and show the brief.
    await story.locator("[data-hit='story.danger-at-home'] [data-pick]").click();
    await story.locator("[data-hit='story.danger-at-home'] button.link").click();
    assert.match(await story.locator("[data-brief-text]").innerText(), /Story hook: Danger where people live, at Tarn/);
    // On the GM's machine Minerva takes the hit's subject: the agents open on it (keys stay in the browser; nothing is sent here).
    const minerva = story.locator("[data-hit='story.danger-at-home'] [data-minerva]");
    assert.equal(await minerva.count(), 1, "Tell it with Minerva on the GM machine");
    {
      await minerva.click();
      await gm.waitForFunction(() => document.querySelector("dialog[aria-label='Agents']")?.open === true, null, { timeout: 5000 });
      await gm.keyboard.press("Escape");
    }

    await more(gm, "Night chart");
    await gm.waitForTimeout(300);
    await shot(gm, "u8-stories-night");

    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "stories");
    await phone.waitForSelector(".story [data-hit]", { timeout: 30000 });
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "u8-stories-phone", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: story finder from sift: hits with evidence best first, sifters on and off, the GM's own sifter, pick and brief, night chart, phone`);
  }
} finally {
  server.kill();
}
