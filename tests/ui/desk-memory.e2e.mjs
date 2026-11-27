// Remembered per desk (TM6, three desks design question 9), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/desk-memory.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Each desk keeps its own depth, and at each level its own view choice, on this device. A desk visited for the first
// time starts from the latest choice made anywhere; coming back to a desk brings back what it had; a reload keeps it.
// The ruler opens on each desk's own span (top-level 12.1) and keeps the span chosen on that desk.
// The start sheet lands a new visitor on Explore, Carry on keeps the desk, and a verb lands where what it opens is.
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

    // The ruler's span by desk (top-level 12.1): Explore at a system opens on human time with "Whole life ›", Build on
    // the whole life; each keeps the span chosen on it.
    const ruler = page.locator(".area-ruler .ruler");
    const span = async () => Number(await ruler.getAttribute("data-span"));
    const YEAR = 1;
    assert.ok((await span()) < 30 / 365, `Explore opens on human time, not ${await span()} years`);
    assert.equal(await ruler.locator("[data-whole-life]").count(), 1);
    await page.getByLabel("Show", { exact: true }).selectOption("year");
    await page.waitForFunction(() => Math.abs(Number(document.querySelector(".area-ruler .ruler")?.getAttribute("data-span")) - 1) < 0.01);
    await desk(page, "Build");
    await page.waitForFunction(() => Number(document.querySelector(".area-ruler .ruler")?.getAttribute("data-span")) > 1e6);
    assert.equal(await ruler.locator("[data-whole-life]").count(), 0, "Build opens on the whole life");
    await shot("3-build-ruler");
    await desk(page, "Explore");
    await page.waitForFunction(() => Math.abs(Number(document.querySelector(".area-ruler .ruler")?.getAttribute("data-span")) - 1) < 0.01);
    assert.ok(Math.abs((await span()) - YEAR) < 0.01, "Explore keeps the year it was shown");
    await shot("4-explore-ruler");

    // The start sheet (TM9c, design section 5): a new visitor lands on Explore; "Carry on" keeps the desk last used; a
    // verb lands on a desk that has what it opens, Explore first.
    const sheetPage = async (kept) => {
      const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
      if (kept) await ctx.addInitScript((d) => localStorage.setItem("sse4.desk", d), kept);
      const p = await ctx.newPage();
      p.on("pageerror", (e) => errors.push(e.message));
      await p.goto(base);
      await p.waitForSelector('[data-start] [data-door="grow"]:not([disabled])', { timeout: 20000 });
      return p;
    };
    const deskOf = (p) => p.locator(".app").getAttribute("data-desk");
    const fresh = await sheetPage(null);
    assert.equal(await deskOf(fresh), "explore", "a new visitor lands on Explore");
    await fresh.context().close();
    const carry = await sheetPage("run");
    await carry.locator('[data-start] [data-door="carry"]').click();
    await carry.waitForSelector("[data-start]", { state: "detached" });
    assert.equal(await deskOf(carry), "run", "Carry on keeps the desk last used");
    await carry.context().close();
    const life = await sheetPage("run");
    await life.locator('[data-start] [data-atlas="world:life"]').click();
    await life.waitForFunction(() => document.querySelector(".app")?.getAttribute("data-desk") === "explore");
    await life.waitForSelector('.stage[data-screen="life"]');
    await life.context().close();
    const light = await sheetPage("run");
    await light.locator('[data-start] [data-atlas="control:light"]').click();
    await light.waitForFunction(() => document.querySelector(".app")?.getAttribute("data-desk") === "explore");
    await light.waitForSelector('.light .toggle[aria-expanded="true"]', { timeout: 5000 });
    await light.context().close();
    const run = await sheetPage("build");
    await run.locator('[data-start] [data-atlas="desk:run"]').click();
    await run.waitForFunction(() => document.querySelector(".app")?.getAttribute("data-desk") === "run");
    await run.context().close();

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`desk memory ${name}: ok`);
  }
} finally {
  server.kill();
}
