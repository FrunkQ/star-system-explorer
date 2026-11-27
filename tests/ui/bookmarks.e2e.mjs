// Bookmarks (PB2b), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/bookmarks.e2e.mjs [--screens <dir>]
// The GM selects Tarn, bookmarks it beside Undo and names it, goes elsewhere, and the bookmark takes them back exactly
// (the world selected, the instrument in front). The bookmark is kept with the campaign: it is still there after a
// reload. Reordered by dragging; removed. On the Run desk the on-air chip sits in the top bar.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9400 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const errors = [];
  const gm = await context.newPage();
  await gm.setViewportSize({ width: 1440, height: 900 });
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");

  // Select Tarn and open the Bookmarks list beside the tools (B does the same).
  await gm.locator("[role=button][aria-label^='Tarn'] circle.hit").first().click();
  await gm.waitForSelector("section[aria-label='Tarn']");
  const screenNow = await gm.locator("main.stage").getAttribute("data-screen");
  await gm.getByRole("button", { name: "Bookmarks" }).click();
  const list = gm.locator("[data-bookmarks-list]");
  await list.waitFor();
  assert.match(await list.innerText(), /A bookmark keeps where you are/);
  await list.locator("[data-bookmark-add]").click();
  const name = list.locator("[data-bookmark-name]");
  await name.waitFor();
  assert.match(await name.inputValue(), /^Tarn/);
  await name.fill("Tarn, the approach");
  await name.press("Enter");
  assert.equal(await list.locator("[data-bookmark-go]").first().innerText(), "Tarn, the approach");
  await shot(gm, "bookmarks-list");

  // Somewhere else, then back.
  await gm.keyboard.press("Escape");
  await list.waitFor({ state: "detached" });
  await gm.keyboard.press("Escape");
  await gm.waitForFunction(() => !document.querySelector("section[aria-label='Tarn']"));
  await go(gm, "physics");
  await gm.keyboard.press("b");
  await list.locator("[data-bookmark-go]").first().click();
  await list.waitFor({ state: "detached" });
  await gm.waitForSelector("section[aria-label='Tarn']");
  assert.equal(await gm.locator("main.stage").getAttribute("data-screen"), screenNow, "back on the same instrument");

  // Kept with the campaign.
  await gm.waitForTimeout(2500);
  await gm.reload();
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await gm.getByRole("button", { name: "Bookmarks" }).click();
  await list.waitFor();
  assert.equal(await list.locator("[data-bookmark-go]").count(), 1);
  assert.equal(await list.locator("[data-bookmark-go]").first().innerText(), "Tarn, the approach");

  // A second, then drag it to the top, then remove the first.
  await gm.locator("[role=button][aria-label^='Kestrel'] circle.hit").first().click();
  await gm.getByRole("button", { name: "Bookmarks" }).click();
  await list.waitFor();
  await list.locator("[data-bookmark-add]").click();
  await list.locator("[data-bookmark-name]").press("Enter");
  const rows = list.locator("li[data-bookmark]");
  assert.equal(await rows.count(), 2);
  await rows.nth(1).dragTo(rows.nth(0));
  assert.match(await rows.nth(0).innerText(), /Kestrel/);
  await list.locator("[data-bookmark-remove='b1']").click();
  assert.equal(await rows.count(), 1);
  await shot(gm, "bookmarks-reordered");
  await gm.keyboard.press("Escape");

  // The Run desk: the on-air chip is in the top bar once shared, and opens the Switchboard.
  await desk(gm, "Run");
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const chip = gm.locator("header.bar [data-on-air]");
  await chip.waitFor();
  assert.equal(await gm.locator("[data-on-air]").count(), 1, "one chip, in the top bar");
  await shot(gm, "bookmarks-run-bar");
  await chip.click();
  await gm.locator("[data-switchboard]").waitFor();
  await go(gm, "system");
  await desk(gm, "Explore");
  assert.equal(await gm.locator("header.bar [data-on-air]").count(), 0);
  assert.equal(await gm.locator("[data-on-air]").count(), 1, "on the status line on other desks");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("bookmarks e2e: ok");
} finally {
  server.kill();
}
