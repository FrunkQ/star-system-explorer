// The Mag layer on the Body screen (magnetic fields design 8 and 9, MF14b part 1), in a real browser against the built
// shell (scripts/build-shell.sh first). Usage: node tests/ui/mag.e2e.mjs [--screens <dir>]
// Open the Solar System from the start sheet, pick Earth, open its Body screen and press Mag: the Chart cut draws
// closed and open lines in the two field inks, the bubble and both axes; the words name the field and its stand-off;
// every legend entry opens its why; Science depth names the model and its stand-ins. Mars says it has no field facts
// yet. The Night chart inks the lines too. On a phone nothing runs off the side. Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { depth, go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const magBand = async (page) => {
  if ((await page.locator("[data-view-controls]").getAttribute("data-band")) !== "mag") await page.getByRole("group", { name: "Band" }).getByRole("button", { name: "Mag", exact: true }).click();
};
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
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
    const mag = gm.locator("[data-mag]");
    await mag.waitFor({ state: "attached", timeout: 20000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    // Mag is a Band in the one view controls (Alex, 23 Nov): it replaces the globe with the field.
    await magBand(gm);
    await gm.waitForSelector('[data-mag="lines"]', { timeout: 20000 });
    assert.equal(await gm.locator(".globe").isVisible(), false, "Mag replaces the picture");
    const cut = gm.locator("[data-mag-cut]");
    const closed = await cut.locator('path.line[data-kind="closed"]').count();
    const open = await cut.locator("path.line.open").count();
    assert.ok(closed >= 10 && open >= 1, `closed ${closed}, open ${open}`);
    const inks = await cut.locator("path.line").evaluateAll((ps) => [...new Set(ps.map((p) => getComputedStyle(p).stroke))]);
    assert.equal(inks.length, 2, `two inks, got ${inks.join(", ")}`);
    assert.equal(await cut.locator("path.bubble").count(), 1, "the wind's bubble");
    assert.match(await gm.locator("[data-mag-words]").innerText(), /µT at the equator.*holds the wind off at \d/);
    for (const name of ["Field out", "Field in", "Open lines", "The wind's bubble", "Axes and tilt"]) {
      await mag.getByRole("button", { name: new RegExp(name) }).click();
      assert.ok((await gm.locator("[data-mag-why]").innerText()).length > 40, `${name} has a why`);
    }
    // Curious keeps to the picture and words; Science names where the field comes from; Engine shows the numbers.
    assert.equal(await gm.locator("[data-mag-source]").count(), 0, "no source at Curious");
    await depth(gm, "Science");
    assert.match(await gm.locator("[data-mag-source]").innerText(), /read_field/);
    await depth(gm, "Engine");
    assert.match(await gm.locator('[data-mag] [data-depth="engine"]').innerText(), /g10 .* T/);
    await depth(gm, "Science");
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
    }
    await mag.scrollIntoViewIfNeeded();
    await shot(gm, `mag-earth-${vp}`);

    // A world without field facts says so.
    await go(gm, "system");
    await pick("Mars");
    await go(gm, "body");
    await gm.waitForSelector("[data-mag]");
    await magBand(gm);
    await gm.waitForSelector('[data-mag="none"]', { timeout: 20000 });
    assert.match(await gm.locator("[data-mag-none]").innerText(), /No field facts for this world yet/);

    // Night chart: the lines keep two inks.
    if (vp === "desk") {
      await go(gm, "system");
      await pick("Jupiter");
      await go(gm, "body");
      await gm.waitForSelector('[data-mag="lines"]', { timeout: 20000 }).catch(async () => {
        await magBand(gm);
        await gm.waitForSelector('[data-mag="lines"]', { timeout: 20000 });
      });
      await more(gm, /Night chart/);
      const night = await gm.locator("[data-mag-cut] path.line").evaluateAll((ps) => [...new Set(ps.map((p) => getComputedStyle(p).stroke))]);
      assert.equal(night.length, 2);
      await shot(gm, "mag-jupiter-night-desk");
    }
    if (vp === "desk") {
      // Mag at system scale (V10e): the same Band on the system chart in 3D draws every world's field round it, from
      // the same facts; the note says how many worlds have one. Real draws them at their true size.
      await go(gm, "system");
      await gm.getByRole("button", { name: "3D", exact: true }).click();
      await magBand(gm);
      const note = gm.locator("[aria-roledescription='3D chart'] [data-true-band='mag']");
      await note.waitFor({ timeout: 20000 });
      await gm.waitForFunction(() => Number(document.querySelector("[data-mag-worlds]")?.getAttribute("data-mag-worlds")) >= 5, null, { timeout: 20000 });
      assert.match(await note.innerText(), /worlds' magnetic fields from the kernel's facts/);
      await shot(gm, "v10e-system-mag-3d-night-desk");
      await gm.getByRole("button", { name: "True", exact: true }).click();
      await gm.getByRole("group", { name: "Scale" }).getByRole("button", { name: "Real" }).click();
      await gm.waitForFunction(() => /true size/.test(document.querySelector("[data-true-band='mag']")?.textContent ?? ""), null, { timeout: 20000 });
      await shot(gm, "v10e-system-mag-true-real-night-desk");
    }
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "reading the field writes nothing");
    assert.deepEqual(errors, []);
    await browser.close();
  }
  console.log("mag e2e: ok");
} finally {
  server.kill();
}
