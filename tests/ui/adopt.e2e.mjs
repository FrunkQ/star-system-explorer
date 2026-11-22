// V13 part 2 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/adopt.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Grow a system and add it to the campaign: the state hash moves, the chart
// switches to it, read back from the universe, with the same worlds as the
// preview. Add the sample V3 save too, look ahead on it and keep with "move
// now": the star read back has aged (luminosity up). Back to Kestrel Reach,
// Tarn is on the chart again. Adding the same system twice is refused.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9500 + Math.floor(Math.random() * 90);
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
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const onChart = () => gm.getByLabel("System on the chart");
    const h0 = await hash();
    const open = async () => {
      await more(gm, "Make a system");
      return gm.getByRole("dialog", { name: "Make a system" });
    };

    // Grow and add.
    let dlg = await open();
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    const result = dlg.getByLabel("Formation result");
    await result.waitFor({ timeout: 20000 });
    const worlds = Number(await result.getAttribute("data-worlds"));
    assert.ok(worlds > 0);
    await shot(gm, "v13-adopt-preview");
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing")?.includes("sys:@first-light"), null, { timeout: 15000 });
    assert.notEqual(await hash(), h0, "adopting writes history");
    await gm.locator(".area-chart .body").first().waitFor();
    // Wait for the worlds to render (the star alone satisfies the first wait), then count them.
    await gm.waitForFunction((n) => document.querySelectorAll('.area-chart .body[aria-label^="World"]').length === n, worlds, { timeout: 15000 }).catch(() => {});
    assert.equal(await gm.locator('.area-chart .body[aria-label^="World"]').count(), worlds, "the same worlds, read back from the universe");
    assert.match(await gm.locator("header .kicker").innerText(), /In the campaign/i);
    assert.match(await gm.locator(".ruler-side [role=status]").innerText(), /Added Seed "first light" to the campaign/);
    await shot(gm, "v13-adopted-grown");

    // The same system again is refused.
    dlg = await open();
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    await dlg.getByLabel("Formation result").waitFor({ timeout: 20000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await dlg.getByRole("alert").waitFor({ timeout: 8000 });
    assert.match(await dlg.getByRole("alert").innerText(), /already in the campaign/);

    // A system whose giants stirred its belts has an Oort cloud, drawn as a sphere round it, not a ring in its plane.
    await dlg.getByRole("textbox", { name: "Seed" }).fill("cold harbour");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    await dlg.locator("[data-shape=sphere]").first().waitFor({ timeout: 20000 });
    assert.ok((await dlg.locator("text.band").allTextContents()).some((t) => /Oort cloud/i.test(t)), "the Oort cloud is named on the chart");

    // The sample V3 save.
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing")?.endsWith("sys:@1"), null, { timeout: 15000 });
    await gm.locator('.area-chart .body[aria-label^="Earth"]').first().waitFor({ timeout: 8000 });
    await gm.waitForFunction(() => document.querySelectorAll("[aria-label='System on the chart'] option").length === 3, null, { timeout: 15000 }).catch(() => {});
    assert.equal(await onChart().locator("option").count(), 3);
    assert.equal(await gm.locator("header h1").innerText(), "Sol (sample)", "the sample save says it is a sample, not the real Sol");

    // Look ahead on Sol with the kernel's own spec; keep and move now: the Sun read back has aged.
    const sunLum = async () => {
      await gm.locator('.area-chart .body[aria-label^="Sun"]').first().dispatchEvent("pointerdown");
      const t = await gm.locator(".area-card").innerText();
      return Number(/LUMINOSITY[^\d]*([\d.]+)/i.exec(t)?.[1] ?? "1");
    };
    const l0 = await sunLum();
    await gm.getByRole("button", { name: "Look ahead…" }).click();
    const panel = gm.getByRole("group", { name: "Look ahead" });
    assert.match(await panel.innerText(), /as the universe holds it/);
    await panel.getByLabel("Target age in Gyr").fill("10");
    await panel.getByRole("button", { name: "Look ahead" }).click();
    await gm.waitForSelector(".ahead[data-ahead=open]", { timeout: 30000 });
    const scrub = panel.getByLabel("Age shown");
    const max = Number(await scrub.getAttribute("max"));
    await scrub.fill(String(max));
    const h1 = await hash();
    await panel.getByRole("button", { name: /^Keep and move now/ }).click();
    await gm.waitForSelector(".ahead:not([data-ahead])", { timeout: 15000 });
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, h1);
    assert.match(await gm.locator(".ruler-side [role=status]").innerText(), /written back/);
    await gm.waitForTimeout(500);
    const l1 = await sunLum();
    assert.ok(l1 > l0 * 1.2, `the Sun read back has aged: ${l0} -> ${l1}`);
    const card = await gm.locator(".area-card").innerText();
    assert.match(card, /aged by a kept look-ahead/);
    assert.match(card, /LUMINOSITY\s*·\s*DERIVED/i, "a written-back value is marked derived, by the kernel's evidence");
    assert.doesNotMatch(card, /LUMINOSITY\s*·\s*SET/i);
    await shot(gm, "v13-adopted-aged");

    // Back to the campaign's own system.
    await onChart().selectOption("");
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().waitFor({ timeout: 8000 });

    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`adopt e2e ${name}: ok (${worlds} worlds adopted; Sun luminosity ${l0} -> ${l1})`);
  }
} finally {
  server.kill();
}
