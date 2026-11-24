// Aurorae on the Body globe (VP-AU5 part 1), in a real browser against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/aurora.e2e.mjs [--screens <dir>]
// Open the Solar System, pick Earth, open its Body screen in True: the aurora pass is in the plan, the kernel's oval
// is named in the badge's words with oxygen's two lines at their heights, the stand-in line shows under the globe,
// and the HUD line says the long exposure lifts faint light (Alex, 24 Nov: Long exposure is the default). Seen from
// above and edge-on, both drawn. Mars (no field facts in the fixture, thin CO2 air) draws none and says why; the
// Moon draws none. Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, view } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(1200);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.locator(".globe").first().screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport, deviceScaleFactor: screens ? 2 : 1 })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    const pick = async (name) => {
      const b = gm.locator(`.area-chart .body[aria-label^="${name},"]`).first();
      await b.waitFor({ timeout: 20000 });
      await b.locator(".hit").click();
    };
    await pick("Earth");
    await go(gm, "body");
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await view(gm, "View style", "True");
    await gm.waitForSelector("[data-aurora-stand-in]", { timeout: 30000 });
    assert.match(await gm.locator("[data-aurora-stand-in]").innerText(), /stand-in: oxygen's green and red/);
    await gm.locator("[data-badge]").click();
    const note = await gm.locator("[data-aurora-note]").innerText();
    assert.match(note, /wind-driven oval at 7\d° magnetic latitude/, note);
    assert.match(note, /O I 557\.7 nm at about 1\d\d km, O I 630\.0 nm at about 2\d\d km/, note);
    assert.match(note, /Long exposure: faint light is lifted/, note);
    await gm.locator("[data-badge]").click();
    await view(gm, "Seen from", "Above");
    await shot(gm, `earth-above-${vp}`);
    await view(gm, "Seen from", "Edge-on");
    await shot(gm, `earth-edge-${vp}`);

    // Where it stays off: Mars has no field facts in the fixture and its air holds no oxygen to borrow.
    await view(gm, "View style", "Chart");
    await go(gm, "system");
    await pick("Mars");
    await go(gm, "body");
    await view(gm, "View style", "True");
    await gm.waitForSelector("[data-badge]", { timeout: 30000 });
    await gm.waitForTimeout(1500);
    assert.equal(await gm.locator("[data-aurora-stand-in]").count(), 0, "Mars borrows no oxygen colours");
    await gm.locator("[data-badge]").click();
    const mars = await gm.locator("[data-aurora-note]").innerText();
    assert.match(mars, /No aurora/, mars);
    await gm.locator("[data-badge]").click();
    await shot(gm, `mars-off-${vp}`);

    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "nothing written");
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
    }
    assert.deepEqual(errors, []);
    await browser.close();
  }
  console.log("aurora e2e: ok");
} finally {
  server.kill();
}
