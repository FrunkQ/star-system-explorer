// The Playbook (PB3), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/playbook.e2e.mjs [--screens <dir>]
// The GM bookmarks Vault, goes on air, and on the Switchboard adds the bookmark to the page as a preset. Selecting it
// shows it in the monitor and sends nothing; its fire button sends it, and the players' screen turns to Vault. Pressed
// again it takes it back. A Read aloud preset shows its words large in the monitor and sends nothing. Keep on Live
// makes a preset from where the GM is. Key 1 fires the first preset. A folded section hides its presets. The Playbook
// is kept with the campaign: it is all still there after a reload.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9500 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const cardIs = (p, name) => p.waitForFunction((n) => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label") === n, name, { timeout: 15000 });
const playerCard = () => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label");

try {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const errors = [];
  const gm = await context.newPage();
  await gm.setViewportSize({ width: 1440, height: 1000 });
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await desk(gm, "Run");

  // Bookmark Vault on the chart.
  await gm.locator("[role=button][aria-label^='Vault'] circle.hit").first().click();
  await gm.keyboard.press("b");
  await gm.locator("[data-bookmark-add]").click();
  await gm.locator("[data-bookmark-name]").fill("Vault, the rings");
  await gm.locator("[data-bookmark-name]").press("Enter");
  await gm.keyboard.press("Escape");
  // And Tarn, for a reveal.
  await gm.locator("[role=button][aria-label^='Tarn'] circle.hit").first().click();
  await gm.keyboard.press("b");
  await gm.locator("[data-bookmark-add]").click();
  await gm.locator("[data-bookmark-name]").fill("Tarn");
  await gm.locator("[data-bookmark-name]").press("Enter");
  await gm.keyboard.press("Escape");
  await gm.keyboard.press("Escape");

  // On air, with one screen.
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const link = await gm.getAttribute("[data-players-link]", "data-players-link");
  const pl = await context.newPage();
  await pl.setViewportSize({ width: 1280, height: 860 });
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(`${link}&name=Wall`);
  await pl.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await pl.waitForFunction(playerCard, null, { timeout: 20000 });
  assert.equal(await pl.evaluate(playerCard), "Tarn", "the first body is in focus to begin with");

  await gm.locator("header.bar [data-on-air]").click();
  const sb = gm.locator("[data-switchboard]");
  await sb.waitFor();
  const book = gm.locator("[data-playbook]");
  await book.waitFor();

  // "+ Add", A bookmark, Vault: the new preset is in the monitor, being edited, and nothing was sent.
  await book.locator("[data-add-to]").first().click();
  await book.locator("[data-add-kind='bookmark']").click();
  await book.locator("[data-add-bookmark]").first().click();
  const monitor = gm.locator("[data-monitor]");
  await monitor.waitFor();
  assert.equal(await monitor.getAttribute("data-editing"), "");
  await monitor.locator("[data-edit-title]").fill("The rings of Vault");
  await monitor.locator("[data-edit-title]").press("Tab");
  await monitor.locator("[data-monitor-done]").click();
  assert.equal(await monitor.locator("[data-monitor-title]").innerText(), "The rings of Vault");
  const id = await monitor.getAttribute("data-monitor");
  await pl.waitForTimeout(1000);
  assert.equal(await pl.evaluate(playerCard), "Tarn", "selecting and editing send nothing");
  await shot(gm, "playbook-monitor");

  // Fire: the screen turns to Vault; the button stays pressed; pressed again it goes back.
  const fireBtn = book.locator(`[data-fire='${id}']`);
  await fireBtn.click();
  await pl.waitForFunction(() => document.querySelector("[data-view='sse.view.card'] article")?.getAttribute("aria-label") === "Vault", null, { timeout: 15000 });
  assert.equal(await fireBtn.getAttribute("aria-pressed"), "true");
  assert.equal(await book.locator(`[data-fired='${id}']`).count(), 1, "a quiet tick with the time");
  assert.equal(await gm.locator("[data-line='everyone']").count(), 1, "a line from the preset");
  await shot(gm, "playbook-fired");
  await fireBtn.click();
  assert.equal(await fireBtn.getAttribute("aria-pressed"), "false");

  // Read aloud: words in the monitor, nothing sent.
  await book.locator("[data-add-to]").first().click();
  await book.locator("[data-add-kind='say']").click();
  await monitor.locator("[data-edit-say]").fill("The beacon repeats every ninety seconds.");
  await monitor.locator("[data-monitor-done]").click();
  assert.match(await monitor.locator("[data-monitor-say]").innerText(), /ninety seconds/);
  await shot(gm, "playbook-read-aloud");
  await monitor.locator("[data-monitor-close]").click();
  await monitor.waitFor({ state: "detached" });

  // Keep on Live: a preset from where the GM is.
  await sb.locator("[data-keep-live]").click();
  await monitor.waitFor();
  assert.equal(await book.locator("[data-preset]").count(), 3);
  await monitor.locator("[data-monitor-done]").click();
  await monitor.locator("[data-monitor-close]").click();

  // Key 1 fires the first preset in view.
  await book.locator("[data-page-picker]").focus();
  await gm.keyboard.press("1");
  assert.equal(await fireBtn.getAttribute("aria-pressed"), "true");
  await gm.keyboard.press("1");
  assert.equal(await fireBtn.getAttribute("aria-pressed"), "false");

  // Folding a section hides its presets; it is this device's own.
  await book.locator(".fold").first().click();
  assert.equal(await book.locator("[data-preset]").count(), 0);
  assert.match(await book.locator(".fold").first().innerText(), /3 presets/i);
  await book.locator(".fold").first().click();

  // A new page; the first page's presets stay where they were.
  await book.locator("[data-page-picker]").click();
  await book.locator("[data-add-page]").first().click();
  await book.locator("[data-pb-name]").fill("Going down");
  await book.locator("[data-pb-name]").press("Enter");
  assert.equal(await book.locator("[data-preset]").count(), 0);
  assert.match(await book.locator("[data-page-picker]").innerText(), /Going down/);
  await book.locator("[data-page-picker]").click();
  await book.locator("[data-page-choose='p1']").click();
  assert.equal(await book.locator("[data-preset]").count(), 3);

  // The "just now" strip: a live send, kept afterwards as a preset; Again sends it again.
  await sb.locator("[data-send-live]").click();
  const just = gm.locator("[data-just-now]");
  await just.waitFor();
  assert.equal(await just.locator("[data-just]").count(), 1);
  await just.locator("[data-just-keep]").first().click();
  await monitor.waitFor();
  await monitor.locator("[data-monitor-done]").click();
  await monitor.locator("[data-monitor-close]").click();
  assert.equal(await book.locator("[data-preset]").count(), 4);
  await just.locator("[data-just-again]").first().click();
  await shot(gm, "playbook-just-now");

  // A reveal: the row wears the eye; firing it tells the table about Tarn, and the player's card fills in. Not a send.
  await book.locator("[data-add-to]").first().click();
  await book.locator("[data-add-kind='bookmark']").click();
  await book.locator("[data-add-bookmark]").filter({ hasText: /^Tarn$/ }).click();
  await monitor.locator("[data-monitor-done]").click();
  await monitor.locator("[data-monitor-fire]").click();
  await cardIs(pl, "Tarn");
  await monitor.locator("[data-monitor-close]").click();
  const plMass = () => pl.locator("[data-field='mass_kg'] dd").first().innerText();
  assert.match(await plMass(), /not yet scanned/);
  await book.locator("[data-add-to]").first().click();
  await book.locator("[data-add-kind='reveal']").click();
  await book.locator("[data-add-bookmark]").filter({ hasText: /^Tarn$/ }).click();
  await monitor.locator("[data-monitor-done]").click();
  const rid = await monitor.getAttribute("data-monitor");
  await monitor.locator("[data-monitor-close]").click();
  assert.equal(await book.locator(`[data-preset='${rid}'] [data-reveals]`).count(), 1, "the eye on the row");
  await book.locator(`[data-fire='${rid}']`).click();
  await gm.locator("[data-reveal-note]").waitFor();
  assert.match(await gm.locator("[data-reveal-note]").innerText(), /told the table/);
  await pl.waitForFunction(() => !/not yet scanned/.test(document.querySelector("[data-field='mass_kg'] dd")?.textContent ?? "not yet scanned"), null, { timeout: 15000 });
  await shot(gm, "playbook-reveal");

  // Kept with the campaign.
  await gm.waitForTimeout(2500);
  await gm.reload();
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await gm.locator("[data-playbook]").waitFor({ timeout: 15000 });
  assert.equal(await gm.locator("[data-playbook] [data-preset]").count(), 6);
  assert.match(await gm.locator("[data-playbook]").innerText(), /The rings of Vault/);

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("playbook e2e: ok");
} finally {
  server.kill();
}
