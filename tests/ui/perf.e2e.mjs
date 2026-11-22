// The performance readout (PF1, design/SSE4-performance-and-minimum-spec.md), in a real browser against the built
// shell (scripts/build-shell.sh first). Usage: node tests/ui/perf.e2e.mjs [--screens <dir>]
// Off by default: no card, and the page takes no timings. More › Performance readout switches it on: the card opens,
// fills with chunks as screens open (Working out a system, Drawing), a chunk row opens to its ops, the report copies
// as an sse.perf-report/1 with the device and no universe in it, and the choice is remembered across a reload (which
// then also times the start-up calls). × switches it off. On a phone it is a sheet along the foot, with no sideways
// scroll; in the Night chart it draws with the night tokens. The players' page has the same switch (PF1b).
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const card = (p) => p.locator("[data-perf]");

try {
  const browser = await chromium.launch();
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const context = await browser.newContext({ viewport, permissions: ["clipboard-read", "clipboard-write"] });
    const gm = await context.newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.locator("[data-engine-ready], .stage").first().waitFor();
    await gm.waitForFunction(() => !!document.querySelector(".stage[data-screen]"), null, { timeout: 30000 });
    assert.equal(await card(gm).count(), 0, "off by default");

    await more(gm, /Performance readout/);
    await card(gm).waitFor();
    assert.match(await card(gm).innerText(), /Performance/);
    // Open a screen or two so there is something to time.
    await go(gm, "body");
    await go(gm, "system");
    await gm.waitForSelector('[data-perf] [data-chunk="drawing"]', { timeout: 20000 });
    await gm.waitForSelector('[data-perf] [data-chunk="resolve"]', { timeout: 20000 });
    const row = card(gm).locator('[data-chunk="resolve"] .row');
    await row.click();
    const ops = await card(gm).locator('[data-chunk="resolve"] td.op').allInnerTexts();
    assert.ok(ops.length > 0, "a chunk opens to its ops");
    assert.ok(["within", "near", "over"].includes(await card(gm).locator('[data-chunk="resolve"]').getAttribute("data-verdict")));
    await shot(gm, `perf-${vp}`);

    // The report: copied, never sent.
    await card(gm).getByLabel("This device").fill(`test ${vp}`);
    await card(gm).locator("[data-perf-copy]").click();
    await gm.waitForFunction(() => /Copied|would not copy/.test(document.querySelector("[data-perf]")?.textContent ?? ""));
    const text = await gm.evaluate(() => navigator.clipboard.readText());
    const rep = JSON.parse(text);
    assert.equal(rep.schema, "sse.perf-report/1");
    assert.equal(rep.label, `test ${vp}`);
    assert.ok(rep.device.user_agent && rep.device.tier, "the device and its measured tier");
    assert.ok(rep.chunks.some((c) => c.id === "resolve"));
    assert.ok(!/kestrel|"seed"/i.test(text), "no universe in a report");

    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
      const box = await card(gm).boundingBox();
      assert.ok(box && box.x <= 1 && box.width >= viewport.width - 2, "a sheet along the foot on a phone");
    }

    // Remembered: a reload opens with it on, and the start-up calls are timed too.
    await gm.reload();
    await card(gm).waitFor();
    await gm.waitForSelector('[data-perf] [data-chunk="startup"]', { timeout: 30000 });
    assert.match(await card(gm).innerText(), /Start-up: .*first chart/);

    if (vp === "desk") {
      await more(gm, /Night chart/);
      await gm.waitForFunction(() => document.documentElement.dataset.theme === "night-chart");
      const bg = await card(gm).evaluate((el) => getComputedStyle(el).backgroundColor);
      const raised = await gm.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--sse-ground-raised").trim());
      assert.ok(raised, "night tokens present");
      await shot(gm, "perf-night-desk");
      assert.notEqual(bg, "rgba(0, 0, 0, 0)", "the card has a ground");
    }

    // × switches it off, and it stays off.
    await card(gm).getByTitle("Switch the readout off").click();
    assert.equal(await card(gm).count(), 0);
    await gm.reload();
    await gm.waitForFunction(() => !!document.querySelector(".stage[data-screen]"), null, { timeout: 30000 });
    assert.equal(await card(gm).count(), 0, "off is remembered");
    assert.deepEqual(errors, []);
    await context.close();
  }
  // PF1b: players get the same switch, beside the theme, for their own devices.
  {
    const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
    const p = await context.newPage();
    const errors = [];
    p.on("pageerror", (e) => errors.push(e.message));
    await p.goto(base + "players.html");
    await p.locator("[data-view='sse.view.card'] article").waitFor({ timeout: 30000 });
    assert.equal(await card(p).count(), 0, "players: off by default");
    await p.locator("[data-perf-switch]").check();
    await card(p).waitFor();
    await p.waitForSelector('[data-perf] [data-chunk="drawing"]', { timeout: 20000 });
    await shot(p, "perf-players-phone");
    const over = await p.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 0, `players phone overflows by ${over} px`);
    await card(p).getByTitle("Switch the readout off").click();
    assert.equal(await p.locator("[data-perf-switch]").isChecked(), false, "× and the switch agree");
    assert.deepEqual(errors, []);
    await context.close();
  }
  await browser.close();
  console.log("perf readout: off by default, chunks, ops, report, remembered, phone sheet, night chart, players: ok");
} finally {
  server.kill();
}
