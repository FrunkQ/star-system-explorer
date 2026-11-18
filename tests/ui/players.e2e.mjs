// V1 acceptance in a real browser, against the built shell's player page
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/players.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Checks: the five player views draw the kernel's live scenes for the
// everyone audience before and after the GM's Scanned tap (stand-in control); unknown facts keep their placeholders; selection is shared; the
// table sorts; keyboard reaches the chart; no sideways scroll on a phone.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8600 + Math.floor(Math.random() * 300);
const url = `http://localhost:${port}/players.html`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

const text = async (loc) => (await loc.innerText()).replace(/\s+/g, " ").trim();
try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(url);
    const card = page.locator("[data-view='sse.view.card'] article");
    await card.waitFor();
    assert.match(await text(page.locator("footer.status")), /live from the kernel for the everyone audience · scene [0-9a-f]{10}/);
    assert.equal(await card.getAttribute("aria-label"), "Tarn", "the first body is in focus");
    assert.equal(await text(card.locator("[data-field='mass_kg'] dd")), "not yet scanned");
    const unplaced = page.locator("[aria-label='Position not known']");
    assert.match(await text(unplaced), /Tarn/, "an unscanned orbit is not drawn by guess");
    assert.equal(await text(page.locator("[data-view='sse.view.readout']").first().locator(".big")), "not yet scanned");
    const beforeText = await text(page.locator("[data-view='sse.view.text']"));

    await page.getByRole("group", { name: "Sensor data" }).getByRole("button", { name: "Scanned" }).click();
    await page.waitForFunction(() => document.querySelector("[data-view='sse.view.card'] [data-field='mass_kg'] dd")?.textContent?.includes("Earth"));
    const mass = await text(card.locator("[data-field='mass_kg'] dd"));
    const temp = await text(card.locator("[data-field='surface.temp_mean_k'] dd"));
    assert.equal(temp, "270 to 290 K", "band precision from the data view");
    assert.equal(await text(card.locator("[data-field='resources'] dd")), "survey required");
    assert.ok(await page.locator("[role=meter]").isVisible(), "gauge drawn");
    assert.equal(await text(page.locator(".lamp .state")), "ON", "hazard light lit by the hazard tag");
    const afterText = await text(page.locator("[data-view='sse.view.text']"));
    assert.match(afterText, /Oceans cover 71 % of it\./);
    const chartBody = page.locator("[data-view='sse.view.chart'] [role=button][aria-label^='Tarn']");
    await chartBody.waitFor();

    // Shared selection: pick Vault in the table; the card and text follow.
    const table = page.locator("[data-view='sse.view.table']");
    await table.getByRole("button", { name: "Vault" }).click();
    await page.waitForFunction(() => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label") === "Vault");
    assert.match(await text(page.locator("[data-view='sse.view.text'] blockquote")), /third moon sings/);
    // Sorting: Mass ascending puts unknowns last.
    await table.getByRole("button", { name: /^Mass/ }).click();
    const order = await table.locator("tbody th").allInnerTexts();
    assert.equal(order[order.length - 1].trim() !== "Tarn", true);
    assert.equal(await table.locator("th[aria-sort='ascending']").count(), 1);
    // Keyboard: the chart's body is focusable and selects with Enter.
    await chartBody.focus();
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label") === "Tarn");

    if (screens && name === "chromium") {
      mkdirSync(screens, { recursive: true });
      await page.screenshot({ path: `${screens}/players-desktop.png`, fullPage: true });
    }
    await page.close();

    const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
    phone.on("pageerror", (e) => errors.push(e.message));
    await phone.goto(url);
    await phone.locator("[data-view='sse.view.card'] article").waitFor();
    await phone.getByRole("button", { name: "Scanned" }).click();
    await phone.waitForTimeout(200);
    const w = await phone.evaluate(() => document.scrollingElement.scrollWidth);
    assert.ok(w <= 390, `player screen scrolls sideways on a phone (${w}px)`);
    if (screens && name === "chromium") await phone.screenshot({ path: `${screens}/players-phone.png`, fullPage: true });
    await browser.close();
    assert.deepEqual(errors, [], "no page errors");
    console.log(`V1 ${name}: ${JSON.stringify({ mass, temp, before: beforeText.slice(0, 80), after: afterText.slice(0, 120), phoneWidth: w })}`);
    console.log(`PASS ${name}`);
  }
} finally {
  server.kill();
}
