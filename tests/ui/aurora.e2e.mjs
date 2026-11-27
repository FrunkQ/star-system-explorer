// Aurorae on the Body globe (VP-AU5 part 1), in a real browser against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/aurora.e2e.mjs [--screens <dir>]
// Open the Solar System, pick Earth, open its Body screen in True: the aurora pass is in the plan, the kernel's oval
// is named in the badge's words with oxygen's two lines at their heights, the stand-in line shows under the globe,
// and the HUD line says the long exposure lifts faint light (Alex, 24 Nov: Long exposure is the default). Seen from
// above and edge-on, both drawn. Mars (no field facts in the fixture, thin CO2 air) draws none and says why. From the ground on Earth near the oval at night (desk), the Land screen names the same oval and
// lines and the long exposure. Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, view, pinClock } from "./nav.mjs";
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
    const browser = pinClock(await chromium.launch());
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
    // Earth's air has the kernel's own lines now (MF9 part 2b), so no stand-in is borrowed.
    await gm.waitForSelector("[data-badge]", { timeout: 30000 });
    await gm.locator("[data-badge]").click();
    await gm.waitForFunction(() => /^Aurora:/.test(document.querySelector("[data-aurora-note]")?.textContent ?? ""), null, { timeout: 30000 });
    const note = await gm.locator("[data-aurora-note]").innerText();
    if (process.env.AU_DEBUG) console.log(note);
    assert.equal(await gm.locator("[data-aurora-stand-in]").count(), 0, "Earth draws the kernel's lines, not the stand-in");
    assert.doesNotMatch(note, /Stand-in/, note);
    assert.match(note, /wind-driven oval at 6\d° magnetic latitude/, note);
    assert.match(note, /O( I\])? 557\.7 nm at about 1\d\d km/, note);
    assert.match(note, /N2\+? 391\.4 nm/, note);
    assert.match(note, /630\.0 nm at about 2\d\d km/, note);
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
    // From the ground (VP-AU7), desk only: land on Earth near the oval, at night, in True.
    if (vp === "desk") {
      await view(gm, "View style", "Chart");
      await go(gm, "system");
      await pick("Earth");
      await go(gm, "surface");
      const map = gm.locator("[data-site-map]");
      await map.waitFor({ timeout: 30000 });
      const box = await map.boundingBox();
      await map.click({ position: { x: ((10 + 180) / 360) * box.width, y: ((90 - 64) / 180) * box.height } });
      await gm.waitForFunction(() => document.querySelector("[data-surface]")?.getAttribute("data-site")?.startsWith("6"), null, { timeout: 15000 });
      await gm.waitForSelector("[data-surface]:not([data-verdict=''])", { timeout: 15000 });
      // Land: the ground may still be loading for the site, so press again until the descent (or the ground) starts.
      const off = () => gm.evaluate(() => ["flying", "down"].includes(document.querySelector("[data-surface]")?.getAttribute("data-phase") ?? ""));
      for (let i = 0; i < 6 && !(await off()); i++) {
        if (await gm.locator("[data-land]:not([disabled])").count()) await gm.locator("[data-land]").click();
        await gm.waitForFunction(() => ["flying", "down"].includes(document.querySelector("[data-surface]")?.getAttribute("data-phase") ?? ""), null, { timeout: 10000 }).catch(() => {});
      }
      if ((await gm.locator("[data-surface]").getAttribute("data-phase")) === "flying") await gm.locator("[data-skip]").click();
      await gm.waitForSelector("[data-surface][data-phase='down']", { timeout: 30000 });
      await view(gm, "View style", "True");
      await gm.waitForSelector("[data-surface][data-look='true']");
      let darkest = { at: 0, exposure: 0 };
      for (let h = -12; h <= 12; h += 2) {
        await gm.locator("[data-hour]").fill(String(h));
        await gm.waitForTimeout(150);
        const e = Number(await gm.locator("[data-surface]").getAttribute("data-exposure"));
        if (e > darkest.exposure) darkest = { at: h, exposure: e };
      }
      await gm.locator("[data-hour]").fill(String(darkest.at));
      await gm.waitForSelector("[data-aurora-sky]", { timeout: 20000 });
      const sky = await gm.locator("[data-aurora-sky]").innerText();
      assert.match(sky, /wind-driven oval .* 557\.7 nm .* Long exposure: faint light is lifted/, sky);
      // Look about: four headings, a quarter turn apart.
      await gm.locator("[data-ground]").focus();
      for (const k of [0, 1, 2, 3]) {
        // A page screenshot clipped to the ground: the ground view redraws every frame, so a locator screenshot
        // can wait forever for it to hold still.
        if (screens) {
          mkdirSync(screens, { recursive: true });
          await gm.waitForTimeout(800);
          const box = await gm.locator("[data-ground]").boundingBox();
          await gm.screenshot({ path: `${screens}/ground-night-${k}.png`, clip: box ?? undefined, animations: "disabled" });
        }
        await gm.keyboard.down("ArrowLeft");
        await gm.waitForTimeout(1120);
        await gm.keyboard.up("ArrowLeft");
      }
    }

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
