// V8 Window in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/window.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Explore > Window opens from beside Kestrel's temperate world with the eyes:
// the star is a blinding disc, some planets are points of light, and the
// table lists only what is seen. The telescope looking at a giant resolves
// its lit disc; the GM can list what the eyes miss. Night chart and phone too.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more, view } from "./nav.mjs";
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
    // HUD off (one set of controls, Alex 23 Nov): the lights alone, without the ecliptic or names; on brings them back.
    const withHud = await ink();
    await view(gm, "HUD", "Off");
    await gm.waitForSelector("[data-sky][data-hud=off]");
    await gm.waitForTimeout(150);
    const bare = await ink();
    assert.ok(bare > 50 && bare < withHud, `HUD off draws less (${bare} of ${withHud})`);
    await view(gm, "HUD", "On");
    await gm.waitForSelector("[data-sky][data-hud=on]");

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

    // Y5: the stars beyond, from sky_from_here and sky_glow, in their own panel; a band switch redraws it and never the hash.
    {
      const hash = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      const sb = gm.locator("[data-sky-beyond]");
      await sb.getByRole("button", { name: "Show the night sky from here" }).click();
      await gm.waitForSelector("[data-sky-beyond=on], [data-sky-error]", { timeout: 120000 });
      assert.equal(await sb.locator("[data-sky-error]").count(), 0, (await sb.locator("[data-sky-error]").count()) ? await sb.locator("[data-sky-error]").innerText() : "");
      const words = await sb.locator("[data-sky-words]").innerText();
      assert.match(words, /^[\d,]+ stars brighter than magnitude 5 in V; brightest: magnitude/, words);
      assert.match(words, /as a glow, adds up to magnitude -?[\d.]+ in V over the whole sky/, words);
      const lum = await sb.locator("[data-sky-canvas]").evaluate((c) => {
        const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
        let mid = 0, top = 0;
        for (let x = 0; x < c.width; x++) {
          mid += d[(Math.floor(c.height / 2) * c.width + x) * 4 + 1];
          top += d[(4 * c.width + x) * 4 + 1];
        }
        return { mid: mid / c.width, top: top / c.width };
      });
      assert.ok(lum.mid > lum.top, `the Milky Way's band across the middle (${lum.mid.toFixed(1)}) is brighter than high latitudes (${lum.top.toFixed(1)})`);
      await sb.locator("[data-sky-canvas]").scrollIntoViewIfNeeded();
      await shot(gm, "y5-sky-visible");
      // A click on the brightest drawn star picks it.
      const box = await sb.locator("[data-sky-canvas]").boundingBox();
      const spot = await sb.locator("[data-sky-canvas]").evaluate((c) => {
        const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
        let best = 0, at = [0, 0];
        for (let y = 2; y < c.height - 2; y++) for (let x = 2; x < c.width - 2; x++) {
          const v = d[(y * c.width + x) * 4] + d[(y * c.width + x) * 4 + 1] + d[(y * c.width + x) * 4 + 2];
          if (v > best) ((best = v), (at = [x / c.width, y / c.height]));
        }
        return at;
      });
      await sb.locator("[data-sky-canvas]").click({ position: { x: spot[0] * box.width, y: spot[1] * box.height } });
      await gm.waitForSelector("[data-sky-picked]", { timeout: 5000 });
      assert.match(await sb.locator("[data-sky-picked]").innerText(), /^Picked: a star of magnitude/);
      await view(gm, "Band", "Infrared");
      await gm.waitForSelector("[data-sky-beyond=on][data-sky-band=Ks]", { timeout: 5000 });
      await gm.waitForFunction(() => /shown in Ks; brightest in Ks:/.test(document.querySelector("[data-sky-words]")?.textContent ?? ""), null, { timeout: 5000 });
      await shot(gm, "y5-sky-infrared");
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash, "the sky never changes the state hash");
      console.log(`  sky: ${words.slice(0, 240)}`);
    }

    // Y5 part 2: the stars behind the system's own lights, through the Window's camera, turned by the kernel's frame.
    {
      const hash = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      await win.locator("[data-target]").selectOption("star");
      await gm.waitForTimeout(200);
      const mean = () => win.locator("canvas[data-sky]").evaluate((c) => {
        const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
        let t = 0;
        for (let i = 0; i < d.length; i += 4) t += d[i] + d[i + 1] + d[i + 2];
        return t / (d.length / 4) / 3;
      });
      const chartMean = await mean();
      await win.getByLabel("Stars behind").check();
      await gm.waitForSelector("[data-stars-behind=on]", { timeout: 180000 });
      await gm.waitForTimeout(300);
      const skyMean = await mean();
      assert.ok(skyMean < chartMean - 40, `the night sky replaces the chart ground (${skyMean.toFixed(0)} against ${chartMean.toFixed(0)})`);
      const words = await win.locator("[data-stars-behind-words]").innerText();
      assert.match(words, /^[\d,]+ stars to magnitude [\d.]+( \(the kernel's list stops there, so some faint stars are missing\))? and the Milky Way's glow, in true colour\. This system's plane is drawn at random/, words);
      await win.locator("canvas[data-sky]").scrollIntoViewIfNeeded();
      await shot(gm, "y5-window-stars-behind");
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash, "the stars behind never change the state hash");
      console.log(`  stars behind: ${words.slice(0, 220)}`);
      await win.getByLabel("Stars behind").uncheck();
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
