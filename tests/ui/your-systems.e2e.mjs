// V3 gaps 17 and 39 (U6 part 2b part 1) in a real browser, against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/your-systems.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Grow a system and add the sample V3 save to the campaign, then open Your systems at the galaxy scale: the sample
// sits on the map where its save puts it, the grown one and the campaign's own system are listed as not placed yet.
// Resting on a system shows what it holds (stars, planets, moons...) without opening it; pressing one opens it.
// Night chart and phone (no sideways scroll) too.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { more } from "./nav.mjs";
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";

const sample = JSON.parse(readFileSync("packages/shell/src/documents/sample-sol.v3.json", "utf8"));

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9800 + Math.floor(Math.random() * 90);
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

    // Two systems into the campaign: one grown, one from the sample V3 save (a starmap with positions).
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    await dlg.getByLabel("Formation result").waitFor({ timeout: 20000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing"), null, { timeout: 15000 });
    await more(gm, "Make a system");
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    // The sample save as a starmap with a twin 5 light years off (and 1 above), so two systems have places of their own.
    const twin = structuredClone(sample.systems[0]);
    twin.name = "Twin";
    twin.position = { x: 4, y: 3, z: 1 };
    if (twin.system) twin.system.name = "Twin";
    await dlg.locator("input[type=file]").setInputFiles({ name: "starmap.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify({ ...sample, systems: [sample.systems[0], twin] })) });
    await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
    await dlg.getByRole("button", { name: /Add (this system|these systems|them|.*) to the campaign/ }).first().click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing")?.endsWith("sys:@1"), null, { timeout: 15000 });

    // Your systems, the galaxy scale's second instrument (a link to it, as the place line writes one).
    await gm.evaluate(() => (location.hash = "#chart/galaxy~systems"));
    const ys = gm.locator(".ys");
    await gm.waitForFunction(() => Number(document.querySelector(".ys")?.getAttribute("data-your-systems")) >= 2, null, { timeout: 30000 });
    await gm.waitForFunction(() => document.querySelectorAll(".ys svg a.sys").length >= 2, null, { timeout: 15000 });
    assert.match(await ys.locator("[data-lede]").innerText(), /systems in this campaign, 2 placed by their own map/);
    const sol = ys.locator("svg a.sys", { hasText: "Sol" }).first();
    assert.equal(await ys.locator("svg a.sys").count(), 2, "the sample and its twin sit on the map where their save puts them");
    assert.ok((await ys.locator("[data-grid] line").count()) >= 4, "a grid under HUD");
    assert.equal(await ys.locator("svg .stalk").count(), 1, "the twin above the plane has its stalk");
    const joins = await ys.locator("svg a.sys:has(.stalk)").evaluate((a) => Math.abs(Number(a.querySelector(".stalk").getAttribute("y2")) - Number(a.querySelector(".star").getAttribute("cy"))));
    assert.ok(joins < 0.01, `the stalk runs from the map up to the system (${joins})`);
    assert.ok((await ys.locator("[data-unplaced] li").count()) >= 1, "the grown system is listed, not put somewhere made up");
    // Every listed system says what it holds; the campaign's own one fills in once the chart's scene for it arrives.
    await gm.waitForFunction(() => [...document.querySelectorAll(".ys [data-unplaced] li")].every((li) => /\d+ (star|planet)/.test(li.textContent ?? "")), null, { timeout: 15000 });

    // Rest on the sample: what it holds, without opening it.
    await sol.locator("circle.star").hover();
    const peek = ys.locator("[data-system-peek]");
    await peek.waitFor({ timeout: 5000 });
    const words = await peek.locator("[data-peek-summary]").innerText();
    assert.match(words, /^1 star · \d+ planets? · \d+ moons?/, `the peek counts (${words})`);
    await shot(gm, "your-systems-desk");

    // Names here: the names gallery's block opens under the sample's row, reading that system's names.
    await ys.locator("[data-placed] [data-names-door]").first().click();
    await ys.locator("[data-placed] [data-names-here]").waitFor({ timeout: 15000 });
    assert.ok((await ys.locator("[data-placed] [data-names-here]").innerText()).trim().length > 0, "the names block says something");
    await ys.locator("[data-placed] [data-names-door]").first().click();

    // Stars around home: the home 10 pc cell's drawn stars, read when opened; resting on one gives its mass and metals.
    await ys.locator("[data-home-cell] summary").click();
    await gm.waitForFunction(() => Number(document.querySelector(".ys [data-home-cell]")?.getAttribute("data-home-cell")) > 3, null, { timeout: 30000 });
    await ys.locator("[data-home-cell] circle.st").last().hover({ force: true });
    assert.match(await ys.locator("[data-home-star]").innerText(), /M☉ .* \[Fe\/H\]/, "a home star says its mass and metals");
    await shot(gm, "your-systems-home-cell");

    // Pencil and ink (SV3): on the Build desk each system is drawn by what the table knows of it.
    await gm.getByRole("button", { name: "Build", exact: true }).first().click();
    await gm.waitForTimeout(800);
    assert.equal(await ys.locator("svg [data-pen]").count(), await ys.locator("svg a.sys").count(), "every placed system has a pen");
    const pensSeen = await ys.locator("[data-placed] b[data-pen], [data-unplaced] b[data-pen]").evaluateAll((bs) => bs.map((b) => b.getAttribute("data-pen")));
    assert.ok(pensSeen.every((p) => ["ink", "heard", "pencil"].includes(p)), pensSeen.join());
    await shot(gm, "your-systems-build-pens");
    await gm.getByRole("button", { name: "Explore", exact: true }).first().click();

    // Press a listed system: it opens.
    const grown = ys.locator("[data-unplaced] a", { hasText: "first light" });
    const target = await grown.getAttribute("data-system");
    await grown.click();
    await gm.waitForFunction((t) => (document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing") ?? "") === t, target, { timeout: 15000 });

    // Night chart, then a phone: nothing scrolls sideways.
    await gm.evaluate(() => (location.hash = "#chart/galaxy~systems"));
    await ys.locator("svg a.sys").first().waitFor({ timeout: 30000 });
    await more(gm, "Night chart");
    await shot(gm, "your-systems-night-desk");
    await gm.setViewportSize({ width: 390, height: 844 });
    await gm.waitForTimeout(300);
    const wide = await gm.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    assert.ok(wide <= 1, `no sideways scroll on a phone (${wide}px)`);
    await shot(gm, "your-systems-phone", true);

    assert.deepEqual(errors, [], "no page errors");
    console.log(`your-systems e2e ${name}: ok (${words}; pens ${pensSeen.join(" ")})`);
    await browser.close();
  }
} finally {
  server.kill();
}
