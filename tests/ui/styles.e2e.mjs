// V11 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/styles.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Every shipped view (GM and player) exposes its schema on the Styles
// screen; promoted controls render from the schema; a style that is over
// budget for a basic device or out of range is rejected with reasons; a
// new style extends a shipped one and is kept; a screen item names a style
// in the composer; the state hash never moves.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9200 + Math.floor(Math.random() * 300);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(base);
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    const hash = await page.getAttribute("[data-state-hash]", "data-state-hash");
    const screenBtn = (n) => page.getByRole("group", { name: "Screen" }).getByRole("button", { name: n, exact: true });
    await screenBtn("Styles").click();
    const sty = page.getByRole("region", { name: "Styles" });

    // Every view exposes its schema.
    const radios = sty.locator(".vlist [role=radio]");
    const n = await radios.count();
    assert.ok(n >= 15, `${n} views listed`);
    const counts = [];
    for (let i = 0; i < n; i++) {
      const r = radios.nth(i);
      await r.click();
      const [settings, promoted] = (await r.locator(".muted").innerText()).match(/\d+/g).map(Number);
      const id = (await r.locator(".vname").innerText()).trim();
      await sty.locator("h2#sty-schema").getByText(id, { exact: false }).first().waitFor();
      const rows = sty.locator("[data-schema]");
      assert.equal(await rows.count(), settings, `${id}: schema rows`);
      const meanings = await rows.evaluateAll((els) => els.map((e) => e.children[3].textContent.trim()));
      assert.ok(meanings.every((m) => m.length > 8), `${id}: every setting has a meaning`);
      assert.ok(promoted <= 5, `${id}: at most five promoted`);
      counts.push(`${id} ${settings}/${promoted}`);
    }

    // The player chart: styles, promoted controls from the schema, the budget check.
    await sty.getByRole("radiogroup", { name: "Players views" }).getByRole("radio", { name: /^chart/ }).click();
    await sty.getByRole("radio", { name: /^Orrery/ }).click();
    const promotedRows = await sty.locator(".doc [data-setting]").evaluateAll((els) => els.map((e) => e.dataset.setting));
    assert.deepEqual(promotedRows, ["label_size", "mark_scale", "animate"], "the orrery promotes its own three");
    await sty.getByLabel("Device tier to check for").selectOption("basic");
    await sty.getByText(/over budget: animate cost medium, and a basic device runs up to light/).waitFor();
    await sty.getByLabel("Device tier to check for").selectOption("standard");
    await sty.getByText(/Every setting fits the schema/).waitFor();
    await sty.locator(".pv [data-animate='true']").waitFor();
    await sty.locator(".doc [data-setting='animate'] input").uncheck();
    await sty.locator(".pv [data-animate='false']").waitFor();
    assert.match(await sty.getByRole("table", { name: /Change from Orrery/ }).innerText(), /animate\s+true\s+false/);
    if (screens && name === "chromium") (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/styles-desktop.png` }));
    // Out of range, typed into the document.
    await sty.getByText("The document").click();
    const raw = sty.getByLabel("Style document");
    const doc = JSON.parse(await raw.inputValue());
    await raw.fill(JSON.stringify({ ...doc, settings: { ...doc.settings, mark_scale: 9 } }));
    await sty.getByText(/mark_scale: must be within 0.5 to 2.5/).waitFor();
    assert.equal(await sty.getByRole("button", { name: "Keep on this device" }).isDisabled(), true);
    await sty.getByRole("button", { name: "Put back" }).click();

    // A new style extends a shipped one and is kept.
    await sty.getByRole("radio", { name: /^Across the table/ }).click();
    await sty.getByRole("button", { name: "New style from this" }).click();
    await sty.locator(".doc [data-setting='labels'] input").uncheck();
    await sty.getByRole("button", { name: "Keep on this device" }).click();
    await sty.getByText(/Kept "Across the table \(mine\)"/).waitFor();
    await sty.getByRole("radio", { name: /^Across the table \(mine\)/ }).waitFor();

    // The composer names a style for an item; its promoted settings render from the schema.
    await screenBtn("Screens").click();
    const comp = page.locator("[role=region][aria-label='Screens composer']");
    await comp.locator("[data-view='sse.view.card'] article").waitFor();
    const chartItem = comp.locator(".items li").first();
    await chartItem.getByLabel("Style").selectOption({ label: "Across the table" });
    await comp.locator(".preview .pchart").waitFor();
    const px = await comp.locator(".preview .pchart").evaluate((el) => getComputedStyle(el).getPropertyValue("--label-px").trim());
    assert.equal(px, "21px", "the style's label size reaches the view");
    await chartItem.getByText("Settings").click();
    assert.ok((await chartItem.locator("[data-setting]").count()) >= 3);
    await chartItem.locator("[data-setting='animate'] input").check();
    const verdict = (await comp.locator(".problems").count()) ? await comp.locator(".problems").innerText() : "ok";
    console.log(`V11 ${name}: composer with animate on a ${await page.evaluate(() => document.querySelector("[title^='Measured capability']")?.textContent)} device: ${verdict.replace(/\s+/g, " ").slice(0, 120)}`);

    assert.equal(await page.getAttribute("[data-state-hash]", "data-state-hash"), hash, "styles never change the state hash");
    // Phone width: no sideways scroll on the Styles screen.
    const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
    phone.on("pageerror", (e) => errors.push(e.message));
    await phone.goto(base);
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    await phone.evaluate(() => [...document.querySelectorAll("[aria-label='Screen'] button")].find((b) => b.textContent.trim() === "Styles")?.click());
    const pw = await phone.evaluate(() => document.scrollingElement.scrollWidth);
    assert.ok(pw <= 390, `phone scrolls sideways (${pw}px)`);
    assert.deepEqual(errors, []);
    console.log(`V11 ${name}: ${n} views with schemas: ${counts.join(", ")}`);
    await browser.close();
  }
  console.log("PASS");
} finally {
  server.kill();
}
