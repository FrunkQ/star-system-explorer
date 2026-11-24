// A description and GM notes for each system and the whole campaign (V3 gap 42), in a real browser against the built
// shell (scripts/build-shell.sh first). Usage: node tests/ui/place-notes.e2e.mjs [--screens <dir>]
// Notes beside the place's name opens a card from it. At a system it holds that system's description (for everyone)
// and GM notes (for the GM alone); above the system, the whole campaign's. A change is kept when the box loses focus,
// and is there when the card opens again. Notes never change the universe's state hash. On a phone Notes is a row of
// the picker (the line keeps its room for the name), the card sits along the bottom and nothing runs off the side.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9400 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(300);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const ready = (page) => page.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 30000 });
const hashOf = (page) => page.locator("[data-state-hash]").first().getAttribute("data-state-hash");

try {
  const browser = await chromium.launch();
  const errors = [];
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 })).newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(root + "#chart");
  await ready(gm);
  const hash0 = await hashOf(gm);
  const open = gm.locator(".place [data-place-notes-open]");
  const card = gm.locator("[data-place-notes]");

  // The system's notes.
  await open.click();
  await card.waitFor();
  assert.equal(await card.getAttribute("data-place-notes"), "u:kestrel/sys:1");
  await gm.waitForFunction(() => document.querySelector("[data-notes-state]")?.getAttribute("data-notes-state") === "ready", null, { timeout: 10000 });
  const desc = card.locator('[data-note-kind="description"] textarea');
  const mine = card.locator('[data-note-kind="gm-notes"] textarea');
  assert.equal(await gm.evaluate(() => document.activeElement?.tagName), "TEXTAREA", "the description takes the keyboard");
  await desc.fill("A quiet orange star. Tarn's seas hide an old wreck.");
  await mine.click(); // leaving the description keeps it
  await gm.waitForFunction(() => document.querySelector("[data-notes-state]")?.getAttribute("data-notes-state") === "kept", null, { timeout: 10000 });
  await mine.fill("The wreck is the Kestrel Maru. Do not reveal before session 3.");
  await shot(gm, "notes-system-desk");
  await gm.keyboard.press("Escape");
  await card.waitFor({ state: "detached", timeout: 5000 });

  // Open again: both are there.
  await open.click();
  await gm.waitForFunction(() => document.querySelector("[data-notes-state]")?.getAttribute("data-notes-state") === "ready", null, { timeout: 10000 });
  assert.equal(await desc.inputValue(), "A quiet orange star. Tarn's seas hide an old wreck.");
  assert.match(await mine.inputValue(), /Kestrel Maru/);
  await gm.locator(".app").click({ position: { x: 700, y: 600 } }); // a press outside closes it
  await card.waitFor({ state: "detached", timeout: 5000 });

  // Above the system: the campaign's notes, separate from the system's.
  await go(gm, "galaxy");
  await open.click();
  await gm.waitForFunction(() => document.querySelector("[data-notes-state]")?.getAttribute("data-notes-state") === "ready", null, { timeout: 10000 });
  assert.equal(await card.getAttribute("data-place-notes"), "u:kestrel");
  assert.equal(await desc.inputValue(), "", "the campaign has its own notes");
  await desc.fill("A campaign of salvage and secrets on the frontier.");
  await gm.keyboard.press("Escape");
  await card.waitFor({ state: "detached", timeout: 5000 });
  await open.click();
  await gm.waitForFunction(() => document.querySelector("[data-notes-state]")?.getAttribute("data-notes-state") === "ready", null, { timeout: 10000 });
  assert.equal(await desc.inputValue(), "A campaign of salvage and secrets on the frontier.", "Escape keeps what was typed");
  await more(gm, /Night chart/);
  await shot(gm, "notes-campaign-night");
  await gm.keyboard.press("Escape");
  assert.equal(await hashOf(gm), hash0, "notes never change the universe's state");

  // Phone.
  const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, deviceScaleFactor: screens ? 2 : 1 })).newPage();
  phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
  await phone.goto(root + "#chart");
  await ready(phone);
  // The line has no room beside the name on a phone: Notes is a row of the picker.
  assert.equal(await phone.locator(".place [data-place-notes-open]").isVisible(), false);
  await phone.locator(".place [data-picker-chip]").tap();
  await phone.locator('[data-picker-pop] [data-pick="notes:"]').tap();
  const pc = phone.locator("[data-place-notes]");
  await pc.waitFor();
  await phone.waitForTimeout(400); // the card settles from where it opened
  const r = await pc.boundingBox();
  assert.ok(r.x >= 7 && r.x + r.width <= 383, `the card fits the screen (${r.x}, ${r.width})`);
  const overflow = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(overflow <= 1, `nothing runs off the side (${overflow}px)`);
  await shot(phone, "notes-phone");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("place notes e2e: ok");
} finally {
  server.kill();
}
