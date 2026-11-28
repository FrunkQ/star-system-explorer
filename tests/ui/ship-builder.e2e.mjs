// The ship builder (CM14b, to the mock-up), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/ship-builder.e2e.mjs [--screens <dir>]
// Open the Solar System on the Build desk; the Workshop lists "Ship builder". A catalogue ship opens as parts in 3D
// over the blueprint room; a tank made longer grows the delta-v; a part armed from the list shows glowing points, and
// pressing one clips it on; the shopping list for a plan shows its bars; Chart draws the plans; "What can it do?" asks
// the kernel; Example ships opens a blank build that a part starts. Phone width fits. Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk as chooseDesk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport, deviceScaleFactor: screens ? 2 : 1 })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    const shot = async (n, sel = "[data-builder-screen]") => {
      if (!screens) return;
      mkdirSync(screens, { recursive: true });
      // A clipped page shot: an element shot waits for the 3D canvas to settle, which software drawing never reports.
      const box = await gm.locator(sel).first().boundingBox();
      await gm.screenshot({ path: `${screens}/${n}-${vp}.png`, ...(box ? { clip: box } : {}) });
    };
    const phone = vp === "phone";
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    // The door is through once the chart shows the Solar System (the place line names it when it finishes); going to the
    // Workshop before then is undone when it does.
    await gm.waitForFunction(() => document.querySelector(".place")?.textContent?.includes("The Solar System"), null, { timeout: 60000 });
    await gm.waitForTimeout(500);
    // The system opens in True (first visit, alpha); software drawing of True starves the page in headless Chromium,
    // so the system is put on its Chart before going to the Workshop.
    const chart = gm.getByRole("button", { name: "Chart", exact: true }).first();
    await chart.waitFor({ timeout: 20000 });
    await chart.click();
    await gm.waitForFunction(() => [...document.querySelectorAll("button")].some((x) => x.textContent?.trim() === "Chart" && x.getAttribute("aria-pressed") === "true"), null, { timeout: 20000 });
    await gm.waitForTimeout(500);
    await chooseDesk(gm, "Build");
    await go(gm, "ship-builder");
    const b = gm.locator("[data-builder-screen] [data-builder]");
    const model = b.locator("canvas[data-ship-model]");
    await b.locator("[data-totals]").waitFor({ timeout: 30000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    const parts = async () => Number(await model.getAttribute("data-parts"));
    const sheet = async (which) => phone && (await b.locator(".phonebar button").nth(which === "palette" ? 0 : 1).click(), await gm.waitForTimeout(300));
    const done = async () => phone && (await b.locator("aside.open .sheetclose").click(), await gm.waitForTimeout(300));

    // The first catalogue ship opens as parts, drawn in 3D over the blueprint room.
    await gm.waitForFunction(() => Number(document.querySelector("[data-builder] canvas[data-ship-model]")?.dataset.parts) >= 3, null, { timeout: 60000 }).catch(async (e) => {
      throw new Error(`the catalogue ship did not open as parts: ${(await gm.locator("[data-builder-screen]").innerText()).slice(0, 400)} ${errors.join(" | ")} (${e.message})`);
    });
    // Its tanks hold what the kernel's sheet says (the propellant read lands a moment after the parts).
    await gm.waitForFunction(() => Number(document.querySelector("[data-builder] [data-dv]")?.textContent?.replace(/[^0-9.]/g, "")) > 0, null, { timeout: 30000 });
    await gm.waitForTimeout(1500);
    await shot("catalogue");

    // A tank made longer (from the build tree): more tank room and more delta-v.
    const dv = async () => Number((await b.locator("[data-dv]").first().innerText()).replace(/[^0-9.]/g, ""));
    const dv0 = await dv();
    await sheet("inspector");
    await b.locator("[data-tab=tree]").click();
    await b.locator("[data-tree] button", { hasText: "tank" }).first().click();
    await b.locator("[data-length]").evaluate((el) => {
      el.value = String(Number(el.value) + 0.4);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await gm.waitForFunction((was) => Number(document.querySelector("[data-builder] [data-dv]").textContent.replace(/[^0-9.]/g, "")) > was, dv0);
    await done();

    // Arm a radiator panel: glowing points appear; press one and it clips on (found by its hover tip).
    const before = await parts();
    await sheet("palette");
    await b.locator('[data-palette-part="radiator-panel"]').click();
    await gm.waitForFunction(() => Number(document.querySelector("[data-builder] canvas[data-ship-model]")?.dataset.markers) > 0);
    const r = await model.boundingBox();
    let at = null;
    for (let y = r.y + 50; y < r.y + r.height - 30 && !at; y += 12)
      for (let x = r.x + 12; x < r.x + r.width - 12 && !at; x += 12) {
        await gm.mouse.move(x, y);
        if (await b.locator(".tip").count()) at = [x, y];
      }
    assert.ok(at, "a glowing point answers the pointer");
    await gm.waitForTimeout(400);
    await shot("placing");
    await gm.mouse.click(at[0], at[1]);
    await gm.waitForFunction((n) => Number(document.querySelector("[data-builder] canvas[data-ship-model]")?.dataset.parts) > n, before);

    // A shopping list for a plan: bars that count down.
    await b.locator("[data-plan]").selectOption({ value: "this" });
    await b.locator("[data-budget=tanks]").waitFor();
    await shot("plan");

    // Chart: the plans, every part a path.
    await b.locator("[data-view=chart]").click();
    await b.locator(".plans svg path[data-part-key]").first().waitFor();
    assert.ok((await b.locator(".plans svg").first().locator("path[data-part-key]").count()) >= 3);
    await b.locator("[data-view='3d']").click();
    await model.waitFor();

    // What can it do? The kernel's verdicts at Earth, Mars and Venus.
    await b.locator("[data-what-can-it-do]").click();
    await Promise.race([b.locator("[data-can-do] li").first().waitFor({ timeout: 30000 }), b.locator("[data-ask-failed]").waitFor({ timeout: 30000 }).then(async () => assert.fail(await b.locator("[data-ask-failed]").innerText()))]);
    await shot("can-do");

    // Example ships assembled from parts (the mock-up's): Kestrel opens as its parts at once; its generic parts have no
    // mass yet, so its delta-v is said to need them and "What can it do?" waits for them.
    await b.locator(".pop .close").click();
    await b.locator("[data-example-ships]").click();
    await b.locator('[data-example-ship="kestrel"]').click();
    await gm.waitForFunction(() => Number(document.querySelector("[data-builder] canvas[data-ship-model]")?.dataset.parts) >= 8);
    await b.locator("[data-needs-mass]").first().waitFor();
    assert.ok(await b.locator("[data-what-can-it-do]").isDisabled(), "no verdicts from masses that are not known");
    await gm.waitForTimeout(1200);
    await shot("example");

    // Example ships: a blank build, then a part starts it.
    await b.locator("[data-example-ships]").click();
    await b.locator('[data-catalogue-ship=""]').click();
    const blank = gm.locator("[data-builder-screen] [data-builder]");
    await sheet("palette");
    await blank.locator('[data-palette-part="habitat-4m"]').click();
    await gm.waitForFunction(() => Number(document.querySelector("[data-builder] canvas[data-ship-model]")?.dataset.parts) === 1);

    if (phone) {
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
