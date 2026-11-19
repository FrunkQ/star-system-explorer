// V12 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/symbols.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The GM chart draws in the diagrammatic grammar with a generated key that
// matches its marks; the Symbols screen switches to colour-coded and to a
// mix, changes one class's colour, and shows the symbols under every
// theme; the state hash never moves; the players' chart draws the same
// symbols under their theme (no colour under the one-colour theme).
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { ALL_THEMES } from "../../packages/skins/player/src/index.ts";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8900 + Math.floor(Math.random() * 300);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(base);
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    const hash = await page.getAttribute("[data-state-hash]", "data-state-hash");
    const screenBtn = (n) => page.getByRole("group", { name: "Screen" }).getByRole("button", { name: n });
    const chartMark = (body) => page.locator(`.area-chart [role=button][aria-label^='${body}'] [data-mark]`).getAttribute("data-mark");

    // Diagrammatic on the chart, with a key generated from the marks.
    const tarn = await chartMark("Tarn");
    assert.match(tarn, /^disc band$/, "Tarn is an ocean world: banded disc");
    assert.match(await chartMark("Vault"), /stipple ring$/, "Vault is a ringed gas giant");
    await page.getByText(/^Key · /).click();
    const key = page.getByRole("figure", { name: "Key to the chart's marks" });
    assert.equal(await key.locator("[data-legend-key='planet/ocean']").getAttribute("data-legend-mark"), tarn, "the key draws Tarn's mark");
    const keys = await key.locator("[data-legend-key]").evaluateAll((els) => els.map((e) => e.dataset.legendKey));
    assert.deepEqual(keys.sort(), ["planet/barren", "planet/desert", "planet/gas-giant", "planet/ice", "planet/ocean", "planet/ringed"]);
    await shot(page, "chart-key");

    // The Symbols screen: colour-coded, then a colour of the GM's own.
    await screenBtn("Symbols").click();
    const sym = page.getByRole("region", { name: "Symbols" });
    await sym.getByRole("radio", { name: /^Colour-coded/ }).click();
    await sym.getByText(/SSE 3's own colours/).first().waitFor();
    assert.equal(await sym.getByText(/Placeholder colours/).count(), 0, "no placeholder colours are left");
    const previews = await sym.locator("[data-preview-theme]").count();
    assert.equal(previews, ALL_THEMES.length, "a preview under every theme");
    const one = ALL_THEMES.find((t) => t.supports.colours !== "full");
    const oneMark = await sym.locator(`[data-preview-theme='${one.id}'] [data-legend-key='planet/ocean']`).getAttribute("data-legend-mark");
    assert.doesNotMatch(oneMark, /#/, `${one.id} draws no colour`);
    assert.match(await sym.locator(`[data-preview-theme='${one.id}'] figcaption`).innerText(), /colour becomes a pattern/);
    await sym.locator("[data-class='planet/ocean'] input[type=color]").evaluate((el) => {
      el.value = "#123456";
      el.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await sym.getByRole("radio", { name: /^Campaign symbols/ }).waitFor();
    assert.equal(await sym.getByRole("radio", { name: /^Campaign symbols/ }).getAttribute("aria-checked"), "true");
    // Mix: shading from the diagrammatic grammar on top of the colours.
    await sym.getByLabel("Take shading from").selectOption({ label: "Diagrammatic" });
    await shot(page, "symbols-desktop");
    assert.equal(await page.getAttribute("[data-state-hash]", "data-state-hash"), hash, "symbols never change the state hash");
    await screenBtn("Chart").click();
    assert.equal(await chartMark("Tarn"), "disc band #123456", "the chart draws the GM's colour over the diagrammatic shading");
    assert.match(await chartMark("Ember"), /^disc solid #cc6600$/i, "Ember is a hot rock: V3's terrestrial colour");

    // SSE 3's colours: a pasted palette override, then a save's tag categories.
    await screenBtn("Symbols").click();
    await sym.getByText("Colours from SSE 3").click();
    await sym.getByLabel("SSE 3 palette overrides").fill(JSON.stringify({ "--body-gas-giant": "#224466", "--bg": "#000000" }));
    await sym.getByRole("button", { name: "Apply palette" }).click();
    const rep = sym.getByRole("status", { name: "SSE 3 colour import" });
    assert.match(await rep.innerText(), /Applied 1 of 2 V3 colours[\s\S]*--bg left: A theme colour/);
    await sym.getByLabel("SSE 3 save file").setInputFiles({ name: "starmap.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify({ coiCategories: [{ id: "owner", label: "Owner", color: "#3366aa", textColor: "#ffffff", tags: [] }], systems: [{ id: "s", name: "S", nodes: [{ id: "b", name: "B", parentId: "s", apparentColorHex: "#808080", icon_color: "#ffcc00" }] }] })) });
    await sym.locator("[data-tag-category='owner']").waitFor();
    assert.match(await rep.innerText(), /1 tag category[\s\S]*1 authored object colour[\s\S]*1 derived true colour/);
    await shot(page, "symbols-v3-import");
    assert.equal(await page.getAttribute("[data-state-hash]", "data-state-hash"), hash, "importing colours never changes the state hash");
    await screenBtn("Chart").click();
    assert.match(await chartMark("Vault"), /#224466/i, "the chart draws the imported gas-giant colour");

    // Back to the shipped symbols.
    await screenBtn("Symbols").click();
    await sym.getByRole("button", { name: "Back to the shipped symbols" }).click();
    assert.equal(await sym.getByRole("radio", { name: /^Diagrammatic/ }).getAttribute("aria-checked"), "true");

    // Phone width: no sideways scroll.
    const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
    phone.on("pageerror", (e) => errors.push(e.message));
    await phone.goto(base);
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    await phone.evaluate(() => document.querySelector("[aria-label='Screen'] button:last-child")?.click());
    const pw = await phone.evaluate(() => document.scrollingElement.scrollWidth);
    assert.ok(pw <= 390, `phone scrolls sideways (${pw}px)`);

    // The players' chart: the same symbols under the player's theme.
    const players = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    players.on("pageerror", (e) => errors.push(e.message));
    await players.goto(`${base}players.html`);
    await players.locator("[data-view='sse.view.card'] article").waitFor();
    await players.getByRole("group", { name: "Sensor data" }).getByRole("button", { name: "Scanned" }).click();
    const pmark = () => players.locator(".pchart [role=button][aria-label^='Tarn'] [data-mark]").getAttribute("data-mark");
    await players.waitForFunction(() => /band/.test(document.querySelector(".pchart [role=button][aria-label^='Tarn'] [data-mark]")?.getAttribute("data-mark") ?? ""));
    await players.getByLabel("Symbols").selectOption({ label: "Colour-coded" });
    await players.getByLabel("Theme").selectOption("chart");
    assert.match(await pmark(), /#[0-9A-F]{6}/i, "colour under a full-colour theme");
    await players.getByLabel("Theme").selectOption(one.id);
    assert.doesNotMatch(await pmark(), /#/, `no colour under ${one.id}`);
    await shot(players, "players-phosphor-symbols");
    assert.deepEqual(errors, []);
    console.log(`V12 ${name}: chart ${tarn} keyed; ${previews} theme previews; ${one.id} ocean ${oneMark}; state hash unchanged`);
    await browser.close();
  }
  console.log("PASS");
} finally {
  server.kill();
}
