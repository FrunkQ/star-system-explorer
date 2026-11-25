// Known to, live on the kernel (SV2 part 3, secrets and visibility design 4 and 5), against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/known.e2e.mjs [--screens <dir>].
// The GM shares Kestrel from the Run desk and a player joins. On Tarn's card the eye shows; one press reveals Tarn to
// the table (held two seconds while players are connected), the player's screen fills in, and Undo takes it back.
// The popup opens from the caret and Escape closes it; the eye is reachable and pressable by keyboard alone.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const context = await browser.newContext({ deviceScaleFactor: screens ? 2 : 1 });
  const errors = [];
  const gm = await context.newPage();
  await gm.setViewportSize({ width: 1440, height: 900 });
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await desk(gm, "Run");
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const pl = await context.newPage();
  await pl.setViewportSize({ width: 1280, height: 860 });
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(await gm.getAttribute("[data-players-link]", "data-players-link"));
  await pl.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });

  // The chart follows the table's knowledge on the Run desk's own lens (pencil and ink, SV3): Ember, unknown to the
  // table, is in pencil (mark and name); Tarn, heard of, is not.
  const body = (n) => gm.locator(`.area-chart .body[aria-label^="${n},"]`).first();
  await body("Ember").locator("[data-pencil]").waitFor({ timeout: 20000 });
  assert.equal(await body("Ember").getAttribute("data-known"), "no");
  assert.equal(await body("Tarn").locator("[data-pencil]").count(), 0, "a body the table has heard of is not pencilled");
  const tarnBefore = await body("Tarn").getAttribute("data-known");

  // Tarn's card shows its eye, at the title.
  await body("Tarn").locator(".hit").click();
  const mark = gm.locator("span.known[data-known]:not([data-known-field])").first();
  const eye = mark.locator("[data-known-eye]");
  await eye.waitFor({ timeout: 20000 });
  assert.equal(await mark.getAttribute("data-known"), "heard_of", "the table has heard of Tarn, no more");
  const lines = gm.locator("span.known[data-known-field]");
  assert.ok((await lines.count()) >= 3, "each card line has its eye");
  const plMass = () => pl.locator("[data-field='mass_kg'] dd").first().innerText();
  assert.match(await plMass(), /not yet scanned/);
  await shot(gm, "known-eye-heard-of");

  // One press reveals: the eye opens at once, the send waits out the two-second hold, the player's card fills in.
  await eye.click();
  assert.equal(await mark.getAttribute("data-known"), "known");
  await gm.locator("[data-known-slip]").waitFor();
  assert.match(await gm.locator("[data-known-slip]").innerText(), /Undo/);
  // The chart follows the press: Tarn's mark reads again from the table's knowledge.
  await gm.waitForFunction((was) => document.querySelector('.area-chart .body[aria-label^="Tarn,"]')?.getAttribute("data-known") !== was, tarnBefore, { timeout: 10000 });
  await gm.waitForTimeout(400);
  await shot(gm, "known-revealed-slip");
  await pl.waitForFunction(() => !/not yet scanned/.test(document.querySelector("[data-field='mass_kg'] dd")?.textContent ?? "not yet scanned"), null, { timeout: 15000 });
  const massNow = await plMass();
  console.log("player's Tarn mass after the reveal:", massNow);
  await shot(pl, "known-player-after-reveal");

  // Undo puts back exactly what the press did.
  await gm.locator("[data-known-undo]").click();
  await pl.waitForFunction(() => /not yet scanned/.test(document.querySelector("[data-field='mass_kg'] dd")?.textContent ?? ""), null, { timeout: 15000 });
  await gm.waitForFunction(() => document.querySelector("span.known[data-known]:not([data-known-field])")?.getAttribute("data-known") === "heard_of", null, { timeout: 10000 });

  // The popup opens from the caret, by keyboard, and Escape closes it.
  const caret = mark.locator("[data-known-open]");
  await caret.focus();
  await gm.keyboard.press("Enter");
  const pop = gm.locator("[data-known-pop]");
  await pop.waitFor();
  assert.ok((await pop.locator("[data-known-audience]").count()) >= 1, "a row per audience");
  await gm.waitForTimeout(600);
  await shot(gm, "known-popup");
  await gm.keyboard.press("Escape");
  await pop.waitFor({ state: "detached", timeout: 5000 });

  // The ruler stepped on a few days: the eye reads and changes at the campaign's now, so it stays where it was pressed.
  for (let i = 0; i < 3; i++) await gm.getByRole("button", { name: /^Forward one/ }).click();
  // Keyboard only: focus the eye and press Space.
  await eye.focus();
  await gm.keyboard.press(" ");
  assert.equal(await mark.getAttribute("data-known"), "known");
  await pl.waitForFunction(() => !/not yet scanned/.test(document.querySelector("[data-field='mass_kg'] dd")?.textContent ?? "not yet scanned"), null, { timeout: 15000 });
  await gm.waitForTimeout(1000);
  assert.equal(await mark.getAttribute("data-known"), "known", "the eye stays open after the kernel is read again");

  // The body globe in pencil (SV3, the shared draw style): Ember, still unknown to the table, opens in a dotted ring
  // with words saying so on the Run desk; on Explore the same globe is plain ink.
  await body("Ember").click();
  await go(gm, "body");
  await gm.locator("[data-body] .globe[data-pen=pencil]").waitFor({ timeout: 20000 });
  assert.match(await gm.locator("[data-pen-note]").innerText(), /not known to the table/);
  await shot(gm, "known-body-pencil");
  await desk(gm, "Explore");
  await gm.locator("[data-body] .globe[data-pen=ink]").waitFor();
  assert.equal(await gm.locator("[data-pen-note]").count(), 0);
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("known e2e: ok");
} finally {
  server.kill();
}
