// The table watches a landing (the landing run-through, 24 Nov), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/landing-table.e2e.mjs [--screens <dir>]
// The GM shares, opens the Solar System, picks Earth and a site on Land, and presses "Send this view": the players'
// screen changes to the Landing screen and judges the same site the same way, on its own kernel. The GM lands: the
// player flies too; the GM skips to the ground and walks: the player follows the GM's pose. Screens off ends it.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9200 + Math.floor(Math.random() * 90);
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
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await desk(gm, "Run");
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const link = await gm.getAttribute("[data-players-link]", "data-players-link");
  const pl = await context.newPage();
  await pl.setViewportSize({ width: 1280, height: 860 });
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(link);
  await pl.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });

  // The GM goes to Earth's Land screen and picks a site inland: still private.
  await more(gm, "Start sheet");
  await gm.locator('[data-door="sol"]').click();
  await gm.waitForFunction(() => [...document.querySelectorAll(".area-chart .body")].some((b) => /^Earth\b/.test(b.getAttribute("aria-label") ?? "")), null, { timeout: 30000 });
  await gm.locator('.area-chart .body[aria-label^="Earth"]').first().locator(".hit").click();
  await go(gm, "surface");
  await gm.waitForSelector("[data-surface] [data-site-map]", { timeout: 30000 });
  const map = gm.locator("[data-site-map]");
  const box = await map.boundingBox();
  await map.click({ position: { x: ((75.97 + 180) / 360) * box.width, y: ((90 - 10.5) / 180) * box.height } });
  await gm.waitForSelector("[data-surface]:not([data-verdict=''])", { timeout: 20000 });
  assert.equal(await pl.locator("[data-surface]").count(), 0, "nothing of the landing reaches the table before it is sent");

  // Sent: the players' screen is the Landing screen, judging the same site the same way.
  await gm.locator("[data-send-view]").click();
  assert.match(await gm.locator("[data-table-sees]").innerText(), /landing/);
  await pl.waitForSelector("[data-surface][data-watching]:not([data-verdict=''])", { timeout: 60000 });
  assert.equal(await pl.locator("[data-surface]").getAttribute("data-verdict"), await gm.locator("[data-surface]").getAttribute("data-verdict"), "the same verdict on both kernels");
  await shot(pl, "landing-player-site");

  // The GM lands: the player flies too; the GM goes down and walks: the player follows.
  await gm.locator("[data-land]").click();
  await gm.waitForSelector("[data-surface][data-phase='flying']", { timeout: 30000 });
  await pl.waitForSelector("[data-surface][data-phase='flying']", { timeout: 40000 });
  await shot(pl, "landing-player-flying");
  await gm.locator("[data-skip]").click();
  await pl.waitForSelector("[data-surface][data-phase='down']", { timeout: 30000 });
  const before = await pl.locator("[data-surface]").getAttribute("data-walk");
  await gm.locator("[data-ground]").focus();
  await gm.keyboard.down("ArrowUp");
  await gm.waitForTimeout(1500);
  await gm.keyboard.up("ArrowUp");
  await pl.waitForFunction((b) => document.querySelector("[data-surface]")?.getAttribute("data-walk") !== b, before, { timeout: 15000 });
  await shot(pl, "landing-player-down");

  // Screens off ends it.
  await gm.locator("[data-screens-off]").click();
  await pl.locator("[data-screens-off]").waitFor({ timeout: 10000 });
  assert.equal(await pl.locator("[data-surface]").count(), 0);

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("landing table e2e: ok");
} finally {
  server.kill();
}
