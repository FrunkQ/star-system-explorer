// U9 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/lenses.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The lens switcher changes what the chart draws and what the card shows,
// never the data: the state hash, the selection and the time stay as they
// were. Basic shows three headline facts; Running drops the editing tools and
// marks what the table knows; "Through Uncharted" fades every body; "Through
// Scanned" turns the card into the players' card. An authored stamp still
// shows in a lens with evidence stamps off. A lens made here survives a
// reload, leaves as a pack, and comes back from that pack identical.
import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9600 + Math.floor(Math.random() * 300);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const TARN = "Tarn";

try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    const browser = await engine.launch();
    const errors = [];
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
    const gm = await ctx.newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const lensOf = () => gm.locator("main.stage").getAttribute("data-lens");
    const fields = () => gm.locator(".area-card .field label").evaluateAll((l) => l.map((x) => x.textContent.replace(/ ·.*/, "").trim()));
    const choose = async (id) => {
      await gm.getByRole("combobox", { name: "Lens", exact: true }).selectOption(id);
      if (!id.startsWith("__")) await gm.waitForFunction((v) => document.querySelector("main.stage")?.getAttribute("data-lens") === v, id);
    };
    assert.equal(await lensOf(), "advanced", "a desktop opens in Advanced");
    const h0 = await hash();
    const kicker = await gm.locator(".kicker").innerText();

    await gm.locator(".area-chart [role=button]", { hasText: TARN }).first().click();
    const tarn = gm.locator(".area-chart [role=button][aria-pressed=true]");
    assert.match(await tarn.getAttribute("aria-label"), /^Tarn/);
    const all = await fields();
    assert.ok(all.length >= 8, `Advanced shows every field (${all.length})`);
    assert.ok(await gm.getByRole("button", { name: "Undo" }).count(), "Advanced has the undo tool");

    // Basic: three headline facts, the same selection, time and data.
    await choose("basic");
    assert.deepEqual(await fields(), ["Orbit", "Mass", "Surface"], "Basic: three headline facts");
    assert.match(await gm.locator(".area-chart [role=button][aria-pressed=true]").getAttribute("aria-label"), /^Tarn/, "selection kept");
    assert.equal(await gm.locator(".kicker").innerText(), kicker, "time kept");
    assert.equal(await hash(), h0, "switching lens leaves the state hash unchanged");
    assert.equal(await gm.locator(".area-chart text.dist").count(), 0, "Basic draws no distance rings");
    await shot(gm, "u9-basic");

    // Running: no editing clutter, player knowledge marked on the chart.
    await choose("running");
    assert.equal(await gm.getByRole("button", { name: "Undo" }).count(), 0, "Running has no undo tool");
    await gm.waitForSelector(".area-chart text.knownv");
    assert.ok((await gm.locator(".area-chart text.knownv").evaluateAll((t) => t.map((x) => x.textContent))).some((t) => /long range|scanned/.test(t)), "what the table knows is marked");
    assert.equal(await hash(), h0);
    await shot(gm, "u9-running");

    // Through Uncharted: nobody knows anything, so every body but the star is faded.
    await choose("view.uncharted");
    await gm.waitForSelector(".area-chart .body.unknown");
    const faded = await gm.locator(".area-chart [data-known=no]").count();
    const bodies = await gm.locator(".area-chart .body").count();
    assert.equal(faded, bodies - 1, "every body but the star is unknown through Uncharted");
    assert.match(await gm.locator(".area-card .as").innerText(), /not known to them/);
    assert.equal(await hash(), h0);

    // Through Scanned: the card is the players' card.
    await choose("view.scanned");
    await gm.waitForSelector(".area-card .pcard");
    // Only bodies with recorded player-facing facts exist for players at all; Tarn is seen through Scanned.
    assert.equal(await gm.locator(".area-chart .body[aria-pressed=true]").getAttribute("data-known"), "scanned");
    assert.equal(await gm.locator(".area-card .pcard [data-field]").first().isVisible(), true);
    assert.equal(await hash(), h0);
    await shot(gm, "u9-through-scanned");

    // Honesty: an authored value keeps its stamp in a lens with evidence stamps off.
    await choose("advanced");
    const mass = gm.locator(".area-card .field", { hasText: "Mass" }).locator("input");
    await mass.fill("0.95");
    await mass.press("Enter");
    await gm.waitForSelector(".area-card .stamps :text('Authored')");
    await choose("running");
    assert.ok(await gm.locator(".area-card .stamps", { hasText: "Authored" }).count(), "the authored stamp shows in Running");
    assert.equal(await gm.locator(".area-card .stamps", { hasText: "Resolved" }).count(), 0, "evidence stamps follow the lens");

    // Make your own, from Building: no orbits, a planet card of two fields.
    await choose("building");
    await choose("__make");
    const dlg = gm.getByRole("dialog", { name: "Lens editor" });
    await dlg.waitFor();
    await dlg.getByLabel("Lens name").fill("Political");
    await dlg.getByLabel("Orbits", { exact: true }).uncheck();
    await dlg.getByLabel("Card for").selectOption("planet");
    await dlg.getByLabel("Every field").uncheck();
    await dlg.getByLabel("Surface", { exact: true }).check();
    await dlg.getByLabel("Orbit", { exact: true }).check();
    await dlg.getByRole("button", { name: "Save lens" }).click();
    await gm.waitForFunction(() => document.querySelector("main.stage")?.getAttribute("data-lens") === "mine.political");
    await shot(gm, "u9-make-your-own");
    const pack = await dlg.locator("[data-lens-pack]");
    const download = gm.waitForEvent("download");
    await pack.click();
    const file = await (await download).path();
    const doc = JSON.parse(await (await import("node:fs/promises")).readFile(file, "utf8"));
    assert.equal(doc.format, "sse.pack/1");
    const before = JSON.stringify(doc.provides.other.lenses);
    await dlg.getByRole("button", { name: "Close" }).click();
    assert.deepEqual(await fields(), ["Orbit", "Surface"], "the card keeps the scene's order");

    // A reload keeps it (the GM's profile in this browser).
    await gm.reload();
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    assert.equal(await lensOf(), "mine.political");

    // Delete it, then load it back from the pack: identical.
    await choose("__make");
    await dlg.getByRole("button", { name: "Delete this lens" }).click();
    await gm.waitForFunction(() => document.querySelector("main.stage")?.getAttribute("data-lens") === "advanced");
    assert.equal(await gm.locator("option[value='mine.political']").count(), 0);
    await choose("__make");
    const tmp = `${file}.json`;
    writeFileSync(tmp, JSON.stringify(doc));
    await dlg.locator("input[type=file]").setInputFiles(tmp);
    await dlg.getByText("Loaded Political").waitFor();
    const download2 = gm.waitForEvent("download");
    await dlg.locator("[data-lens-pack]").click();
    const again = JSON.parse(await (await import("node:fs/promises")).readFile(await (await download2).path(), "utf8"));
    assert.equal(JSON.stringify(again.provides.other.lenses), before, "a lens exported in a pack and reloaded is identical");
    await dlg.getByRole("button", { name: "Close" }).click();

    // Keyboard: L moves to the next lens.
    await gm.locator("body").click({ position: { x: 5, y: 5 } }).catch(() => {});
    const was = await lensOf();
    await gm.keyboard.press("l");
    assert.notEqual(await lensOf(), was, "L switches lens");

    // Phone: the switcher is there and nothing scrolls sideways.
    const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base);
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    assert.ok(await phone.getByRole("combobox", { name: "Lens", exact: true }).isVisible(), "the lens switcher is on a phone");
    const sw = await phone.evaluate(() => document.documentElement.scrollWidth);
    assert.ok(sw <= 390, `no sideways scroll on a phone (${sw})`);
    await shot(phone, "u9-phone");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: lens switching keeps the state hash; Basic 3 fields; Running no tools; Uncharted ${faded} faded; own lens round trip through a pack identical`);
  }
} finally {
  server.kill();
}
