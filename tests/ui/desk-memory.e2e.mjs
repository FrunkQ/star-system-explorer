// Remembered per desk (TM6, three desks design question 9), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/desk-memory.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Each desk keeps its own depth, and at each level its own view choice, on this device. A desk visited for the first
// time starts from the latest choice made anywhere; coming back to a desk brings back what it had; a reload keeps it.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, depth } from "./nav.mjs";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    page.on("pageerror", (e) => errors.push(e.message));
    const shot = async (n) => {
      if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${name}-${n}.png` }));
    };
    const ready = () => page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const vc = page.locator(".viewbar [data-view-controls]");
    const styleNow = () => vc.getAttribute("data-style");
    const depthNow = () => page.locator(".app").getAttribute("data-depth");
    const pick = (group, label) => vc.getByRole("group", { name: group }).getByRole("button", { name: label, exact: true }).click();
    await page.goto(base + "#chart");
    await ready();

    // Explore: 3D seen tilted, at Science.
    await desk(page, "Explore");
    assert.equal(await styleNow(), "chart");
    await pick("Seen from", "Tilted");
    await page.waitForFunction(() => document.querySelector(".viewbar [data-view-controls]")?.getAttribute("data-style") === "3d");
    await depth(page, "Science");
    assert.equal(await depthNow(), "science");

    // Build, first visit: it starts from the latest choice. Then Build's own: the flat chart, at Curious.
    await desk(page, "Build");
    assert.equal(await styleNow(), "3d", "a first visit starts from the latest choice");
    assert.equal(await depthNow(), "science");
    await pick("View style", "Chart");
    await pick("Seen from", "Above");
    await depth(page, "Curious");
    assert.equal(await styleNow(), "chart");
    await shot("1-build");

    // Back to Explore: its own choices come back.
    await desk(page, "Explore");
    await page.waitForFunction(() => document.querySelector(".viewbar [data-view-controls]")?.getAttribute("data-style") === "3d");
    assert.equal(await vc.getAttribute("data-seen"), "tilted");
    assert.equal(await depthNow(), "science");
    await shot("2-explore");

    // And Build's, again, and after a reload.
    await desk(page, "Build");
    await page.waitForFunction(() => document.querySelector(".viewbar [data-view-controls]")?.getAttribute("data-style") === "chart");
    assert.equal(await depthNow(), "curious");
    await page.reload();
    await ready();
    assert.equal(await styleNow(), "chart");
    assert.equal(await depthNow(), "curious");
    await desk(page, "Explore");
    await page.waitForFunction(() => document.querySelector(".viewbar [data-view-controls]")?.getAttribute("data-style") === "3d");
    assert.equal(await depthNow(), "science");

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`desk memory ${name}: ok`);
  }
} finally {
  server.kill();
}
