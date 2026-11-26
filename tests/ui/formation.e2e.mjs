// The Formation screen (F10, the reading half) in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/formation.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Grow "first light" and add it to the campaign, then open "How it formed" for it: the chart draws a track for every
// body, the condensation lines, the gas disc and the upheaval; the Inherited panel has its rows. A stage button moves
// the time slice and the time ruler with it; the slice's handle takes the arrow keys. Pressing a world's track selects
// it. Kestrel Reach (written fact by fact) says it has no formation history and how to get one. Reading the history
// never changes the state hash.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { more } from "./nav.mjs";
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
  await page.waitForTimeout(300);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");

    // Kestrel Reach was written fact by fact: no history, and the screen says how to get one.
    await gm.evaluate(() => (location.hash = location.hash.replace(/~.*$/, "") + "~formation"));
    const fs = gm.locator(".formation");
    await gm.waitForSelector(".formation[data-formation='none']", { timeout: 15000 });
    assert.match(await fs.innerText(), /No formation history/);
    assert.match(await fs.innerText(), /Grow a new system/);

    // Grow first light and add it.
    await gm.evaluate(() => (location.hash = "#chart"));
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    const worlds = Number(await dlg.getByLabel("Formation result").getAttribute("data-worlds", { timeout: 20000 }));
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing")?.includes("sys:@first-light"), null, { timeout: 15000 });
    const h0 = await hash();

    // How it formed, for the system on the chart.
    await gm.evaluate(() => (location.hash = location.hash.replace(/~.*$/, "") + "~formation"));
    await gm.waitForSelector(".formation[data-formation='run']", { timeout: 20000 });
    const tracks = gm.locator("[data-track]");
    assert.equal(await gm.locator("[data-track-end='lasts']").count(), worlds, "a lasting track for every world");
    assert.ok((await tracks.count()) > worlds, "and tracks for the bodies lost on the way");
    assert.equal(await gm.locator("[data-line]").count(), 5, "five condensation lines");
    assert.equal(await gm.locator("[data-gas]").count(), 1, "the gas disc");
    assert.equal(await gm.locator("[data-upheaval]").count(), 1, "the upheaval span");
    assert.ok((await gm.locator("[data-event]").count()) > 0, "events as diamonds");
    for (const key of ["star", "metals", "c_o", "disc", "gas", "cluster", "al26", "seed"]) assert.equal(await gm.locator(`[data-inherited='${key}']`).count(), 1, `inherited ${key}`);
    assert.match(await gm.locator("[data-inherited='seed']").innerText(), /first light/);
    assert.match(await gm.locator("[data-slice-words]").innerText(), /stage 9/, "the slice starts at now, in stage 9");
    await shot(gm, `formation-now-${name}`);

    // A stage moves the slice and the ruler with it.
    const sliceAt = async () => Number(await gm.locator("[data-slice]").getAttribute("data-slice"));
    const before = await sliceAt();
    await gm.locator("[data-stage='6']").click();
    await gm.waitForFunction(() => document.querySelector("[data-stage='6']")?.getAttribute("aria-current") === "step");
    const at6 = await sliceAt();
    assert.ok(at6 < before / 10, `stage 6 moves the slice up (${before} to ${at6})`);
    assert.match(await gm.locator("[data-stage-text]").innerText(), /gas gone/i);
    // Moving the slice (and so the ruler) reads nothing again: the history is stored.
    await gm.waitForTimeout(800);
    assert.equal(await gm.locator(".formation").getAttribute("data-formation"), "run", "the chart stays up while the date moves");
    await shot(gm, `formation-violent-${name}`);
    // The handle takes the keys: down moves later.
    await gm.locator("[data-slice-handle]").focus();
    await gm.keyboard.press("ArrowDown");
    await gm.waitForFunction((t) => Number(document.querySelector("[data-slice]")?.getAttribute("data-slice")) > t * 1.2, at6);
    // A lasting world's track is a button: Enter on it selects the world.
    await gm.locator("[data-track-end='lasts'] .hit").first().focus();
    await gm.keyboard.press("Enter");
    await gm.waitForSelector("[data-track].selected", { timeout: 5000 });

    // Phone width: the chart first, then the panel; nothing spills sideways.
    await gm.setViewportSize({ width: 390, height: 844 });
    await gm.waitForTimeout(300);
    const spill = await gm.evaluate(() => document.querySelector(".formation").scrollWidth - document.querySelector(".formation").clientWidth);
    assert.ok(spill <= 1, `no sideways scroll on a phone (${spill})`);
    await shot(gm, `formation-phone-${name}`);

    assert.equal(await hash(), h0, "reading the history writes nothing");
    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`formation e2e ${name}: ok (${worlds} worlds, ${await tracks.count().catch(() => "?")} tracks)`);
  }
} finally {
  server.kill();
}
