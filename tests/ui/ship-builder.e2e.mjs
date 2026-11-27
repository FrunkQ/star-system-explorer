// The ship builder (CM14a), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/ship-builder.e2e.mjs [--screens <dir>]
// Open the Solar System on the Build desk; the Workshop lists "Ship builder". A catalogue ship opens as palette parts;
// a tank made longer grows the tanks and the delta-v; a part clipped from the palette adds to the build; Build to a
// plan shows budgets that count down; "What can it do?" asks the kernel. Phone width fits. Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk as chooseDesk } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 1000 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport, deviceScaleFactor: screens ? 2 : 1 })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    const shot = async (n, sel = "[data-builder-screen]") => {
      if (screens) (mkdirSync(screens, { recursive: true }), await gm.locator(sel).first().screenshot({ path: `${screens}/${n}-${vp}.png` }));
    };
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    await gm.locator(".area-chart .body").first().waitFor({ timeout: 20000 });
    await gm.waitForTimeout(1500);
    await chooseDesk(gm, "Build");
    await gm.locator('.rail [data-go="workshop"]').first().click();
    await gm.waitForTimeout(800);
    await gm.locator('.rail [data-go="ship-builder"]').first().click();
    const b = gm.locator("[data-builder-screen] [data-builder]");
    await b.locator("[data-totals]").waitFor({ timeout: 30000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");

    // The first catalogue ship opens as parts: several parts drawn in the plans, a tank among them.
    await b.locator("svg path[data-part-key]").first().waitFor();
    const parts = await b.locator("svg").first().locator("path[data-part-key]").count();
    assert.ok(parts >= 3, `catalogue ship opened as ${parts} parts`);
    await shot("catalogue");

    // A tank made longer: more tank room and more delta-v.
    const dv = async () => Number((await b.locator("[data-dv]").innerText()).replace(/[^0-9.]/g, ""));
    const dv0 = await dv();
    const tank = b.locator("svg").first().locator("path.tanks[data-part-key]").first();
    await tank.click({ force: true });
    await b.locator("[data-picked] [data-longer]").click();
    await b.locator("[data-picked] [data-longer]").click();
    await gm.waitForFunction((was) => Number(document.querySelector("[data-builder] [data-dv]").textContent.replace(/[^0-9.]/g, "")) > was, dv0);
    assert.ok((await dv()) > dv0);

    // Clip a radiator panel from the palette onto the hull.
    const before = await b.locator("svg").first().locator("path[data-part-key]").count();
    const hull = await b.locator("[data-clip-target] option").evaluateAll((os) => os.findIndex((o) => /hull/.test(o.textContent)));
    await b.locator("[data-clip-target]").selectOption({ index: Math.max(0, hull) });
    await b.locator('[data-palette-part="radiator-panel"]').click();
    await gm.waitForFunction((n) => document.querySelector("[data-builder] svg").querySelectorAll("path[data-part-key]").length > n, before);

    // Build to a plan: budgets from the catalogue ship, counting down.
    await b.locator('[data-mode="plan"]').click();
    await b.locator("[data-budgets] [data-budget=tanks]").waitFor();
    await shot("plan");

    // The 3D model over the fixed blueprint.
    await b.locator("[data-show-model]").click();
    await b.locator("[data-ship-model]").waitFor();
    await gm.waitForTimeout(1500);
    await shot("model");

    // What can it do? The kernel's verdicts at Earth, Mars and Venus.
    await b.locator("[data-show-plans]").click();
    await b.locator("[data-what-can-it-do]").click();
    await Promise.race([b.locator("[data-can-do] li").first().waitFor({ timeout: 30000 }), b.locator("[data-ask-failed]").waitFor({ timeout: 30000 }).then(async () => assert.fail(await b.locator("[data-ask-failed]").innerText()))]);
    await shot("can-do");

    // A blank build: nothing yet, then a part starts it.
    await gm.locator("[data-catalogue]").selectOption({ value: "" });
    await gm.locator("[data-builder-screen] [data-builder]").locator('[data-palette-part="habitat-4m"]').click();
    await gm.locator("[data-builder-screen] svg path[data-part-key]").first().waitFor();

    if (vp === "phone") {
      const wide = await gm.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 1);
      assert.ok(!wide, "no sideways scroll on a phone");
    }
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "nothing written");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`ship builder ok (${vp})`);
  }
} finally {
  server.kill();
}
