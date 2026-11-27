// Start fresh (alpha aid, AF1) in a real browser, against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/fresh.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The start sheet and the top of More both open Start fresh. Reset settings forgets this browser's remembered choices
// and keeps the campaign and agent keys; Clear everything forgets the campaign too, and the start sheet then offers the
// sample campaign rather than "Carry on". A campaign that entered the Solar System keeps the fingerprint of the sheet it
// came from; one kept from another sheet says so, with Start fresh beside it.
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
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
    const gm = await context.newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    const store = () => gm.evaluate(() => Object.fromEntries(Object.keys(localStorage).filter((k) => k.startsWith("sse4.")).map((k) => [k, localStorage.getItem(k)])));
    const kept = () => gm.evaluate(() => JSON.parse(localStorage.getItem("sse4.campaign") ?? "null"));

    // The start sheet offers it on a first visit.
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 120000 });
    assert.ok(await gm.locator("[data-start-fresh]").isVisible(), "the start sheet offers Start fresh");
    await shot(gm, "af1-start-sheet");

    // Open the Solar System: the kept campaign names the sheet it entered Sol from.
    await gm.locator('[data-door="sol"]').click();
    await gm.waitForFunction(() => /^fnv1a:[0-9a-f]{8}:/.test(JSON.parse(localStorage.getItem("sse4.campaign") ?? "null")?.workspace?.sol_sheet ?? ""), null, { timeout: 120000 });
    assert.equal(await gm.locator("[data-sol-stale]").count(), 0, "a Sol entered from today's sheet is not called old");
    // A first visit opens in True (alpha setting, AF2), so the new pictures show without a press.
    const styleTrue = () => gm.locator("[data-view-controls]").first().getByRole("group", { name: "View style", exact: true }).getByRole("button", { name: "True", exact: true });
    await gm.waitForFunction(() => !!document.querySelector("[data-view-controls]"), null, { timeout: 120000 });
    assert.equal(await styleTrue().getAttribute("aria-pressed"), "true", "a first visit opens in True");

    // A campaign kept from another sheet says so after a reload.
    await gm.evaluate(() => {
      const s = JSON.parse(localStorage.getItem("sse4.campaign"));
      s.workspace.sol_sheet = "fnv1a:00000000:0b0r";
      localStorage.setItem("sse4.campaign", JSON.stringify(s));
    });
    await gm.reload();
    // The reopened campaign's own note comes first; the old-Sol note takes its place once it is closed.
    await gm.locator("[data-kept-note]").waitFor({ timeout: 120000 });
    await gm.locator("[data-kept-note]").getByRole("button", { name: "OK" }).click();
    await gm.locator("[data-sol-stale]").waitFor({ timeout: 120000 });
    assert.match(await gm.locator("[data-sol-stale]").innerText(), /older catalogue/);
    await shot(gm, "af1-sol-stale");

    // Reset settings from More: remembered choices go; the campaign and agent keys stay.
    await gm.evaluate(() => {
      localStorage.setItem("sse4.depth", "engine");
      localStorage.setItem("sse4.view", JSON.stringify({ style: "3d", seen: "edge" }));
      localStorage.setItem("sse4.agents.key.test", "not-a-real-key");
    });
    const campaignBefore = (await store())["sse4.campaign"];
    assert.ok(campaignBefore, "a campaign is kept");
    await more(gm, /Start fresh/);
    const fd = gm.getByRole("dialog", { name: "Start fresh" });
    await fd.getByRole("button", { name: /Reset settings…/ }).click();
    await fd.getByText(/agent keys stay/).waitFor();
    await shot(gm, "af1-reset-settings");
    await Promise.all([gm.waitForNavigation({ timeout: 120000 }), fd.locator("[data-fresh-settings]").click()]);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 120000 });
    const afterReset = await store();
    assert.equal(afterReset["sse4.depth"], undefined, "depth forgotten");
    assert.equal(afterReset["sse4.view"], undefined, "view forgotten");
    assert.equal(afterReset["sse4.agents.key.test"], "not-a-real-key", "agent keys kept");
    assert.ok((await kept())?.workspace, "the campaign kept");
    // Carry on names where the campaign was last looked at, and goes back there (not to the base system).
    const carry = gm.locator('[data-door="carry"]');
    assert.match(await carry.innerText(), /Carry on: The Solar System/);
    await carry.click();
    await gm.waitForFunction(() => /The Solar System/.test(document.querySelector(".place")?.textContent ?? ""), null, { timeout: 120000 });
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 120000 });

    // Clear everything from the start sheet: the campaign goes too.
    await gm.locator("[data-start-fresh]").click();
    await fd.getByRole("button", { name: /Clear everything…/ }).click();
    await fd.getByText(/cannot be undone/).waitFor();
    await shot(gm, "af1-clear-everything");
    await Promise.all([gm.waitForNavigation({ timeout: 120000 }), fd.locator("[data-fresh-everything]").click()]);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 120000 });
    assert.ok(await gm.locator('[data-door="carry"]').getByText(/Open the sample campaign/).isVisible(), "no campaign to carry on");
    assert.equal((await store())["sse4.agents.key.test"], undefined, "keys forgotten too");

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: Start fresh offered, settings reset keeping the campaign, everything cleared, an old Sol named`);
  }
} finally {
  server.kill();
}
