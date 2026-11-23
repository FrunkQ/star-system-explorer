// What the table is shown (the landing run-through, 23 Nov), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/table.e2e.mjs [--screens <dir>]
// The GM shares from Kestrel: the players' screen shows Kestrel. The GM opens the Solar System privately: the players
// still see Kestrel, and the GM's bar says so and offers "Send this view". Sent, the players' screen follows to the
// Solar System, as far as the table knows it. "Screens off" blanks the players' screens until a view is sent again.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9300 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
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
  await desk(gm, "Run");

  // Share from Kestrel: the table is shown the place in view.
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const link = await gm.getAttribute("[data-players-link]", "data-players-link");
  const sees = gm.locator("[data-table-sees]");
  assert.match(await sees.innerText(), /Table sees: Kestrel/);
  assert.equal(await gm.locator("[data-send-view]").count(), 0, "nothing to send: the table already sees this");
  const pl = await context.newPage();
  await pl.setViewportSize({ width: 1280, height: 860 });
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(link);
  await pl.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  const title = () => pl.locator("header h1, header .title").first().innerText();
  await pl.waitForFunction(() => /Kestrel/.test(document.querySelector("header.bar")?.textContent ?? ""), null, { timeout: 20000 });

  // The GM opens the Solar System: private until sent.
  await more(gm, "Start sheet");
  await gm.locator('[data-door="sol"]').click();
  await gm.waitForFunction(() => [...document.querySelectorAll(".area-chart .body")].some((b) => /^Earth\b/.test(b.getAttribute("aria-label") ?? "")), null, { timeout: 30000 });
  await gm.locator("[data-send-view]").waitFor();
  assert.match(await sees.innerText(), /Table sees: Kestrel/);
  await pl.waitForTimeout(1500);
  assert.match(await pl.locator("header.bar").innerText(), /Kestrel/, "the players still see Kestrel");
  await shot(gm, "table-gm-private");

  // Sent: the players follow.
  await gm.locator("[data-send-view]").click();
  assert.match(await sees.innerText(), /Table sees: The Solar System/);
  assert.equal(await gm.locator("[data-send-view]").count(), 0);
  await pl.waitForFunction(() => !/Kestrel/.test(document.querySelector("header.bar")?.textContent ?? "") && /Sun/.test(document.body.textContent ?? ""), null, { timeout: 20000 });
  await shot(pl, "table-player-sol");

  // Screens off, then a view sent again.
  await gm.locator("[data-screens-off]").click();
  await pl.locator("[data-screens-off]").waitFor({ timeout: 10000 });
  assert.match(await sees.innerText(), /screens off/);
  await shot(pl, "table-player-off");
  await gm.locator("[data-send-view]").click();
  await pl.locator("[data-screens-off]").waitFor({ state: "detached", timeout: 10000 });
  await pl.waitForFunction(() => /Sun/.test(document.body.textContent ?? ""), null, { timeout: 20000 });
  void title;

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("table e2e: ok");
} finally {
  server.kill();
}
