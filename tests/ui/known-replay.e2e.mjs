// A players' screen joins after the GM has used a Known-to eye (regression, 24 Nov 2026), in a real browser against
// the built shell (scripts/build-shell.sh first). Usage: node tests/ui/known-replay.e2e.mjs
// The Solar System with the Traveller pack, a far trader at Earth made the party's ship, its row's eye on the Build
// desk pressed (the kernel's set_known), then the table is shared: the joining screen replays the GM's log to the same
// state and goes live, never asking to Reload.
import { spawn } from "node:child_process";
import assert from "node:assert/strict";
import { desk as toDesk, go } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 9800 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  const browser = await chromium.launch();
  const errors = [];
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(base);
  await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
  await gm.locator('[data-door="sol"]').click();
  const earthDot = gm.locator('.area-chart .body[aria-label^="Earth,"]').first();
  await earthDot.waitFor({ timeout: 20000 });
  await go(gm, "packs");
  const packs = gm.locator("[role=region][aria-label=Packs]");
  await packs.getByRole("button", { name: "Add the Traveller pack" }).click();
  await packs.locator("[aria-label=Proposal]").getByRole("button", { name: "Accept" }).click();
  await gm.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sse.traveller"));
  await go(gm, "system");
  await earthDot.waitFor({ timeout: 20000 });
  await earthDot.locator(".hit").click();
  await go(gm, "constructs");
  const desk = gm.locator("[role=region][aria-label='Ships and stations']");
  await desk.locator("[data-add-here]").click();
  const earth = await desk.locator("[data-under] option", { hasText: "Earth" }).first().getAttribute("value");
  await desk.locator("[data-under]").selectOption(earth);
  await desk.locator("[data-template='far_trader']").click();
  await desk.locator("[data-name]").fill("Beowulf");
  await desk.locator("[data-add]").click();
  const row = desk.locator("[data-construct]", { hasText: "Beowulf" });
  await row.waitFor({ timeout: 15000 });
  if ((await row.getAttribute("aria-pressed")) !== "true") await row.click();
  await desk.locator("[data-party-ship]").click();
  await desk.locator("[data-party]").first().waitFor();
  // The row's eye, on the Build desk: the table is told of Beowulf (set_known on the ship).
  await toDesk(gm, "Build");
  await go(gm, "constructs");
  const eye = desk.locator("[data-known-eye]").first();
  await eye.waitFor({ timeout: 15000 });
  const was = await desk.locator("span.known[data-known]").first().getAttribute("data-known");
  await eye.click();
  await gm.waitForFunction((w) => document.querySelector("[role=region][aria-label='Ships and stations'] span.known[data-known]")?.getAttribute("data-known") !== w, was, { timeout: 15000 });

  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const pl = await context.newPage();
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(await gm.getAttribute("[data-players-link]", "data-players-link"));
  await pl.waitForFunction(() => {
    const f = document.querySelector("footer[data-link]");
    return f && (f.getAttribute("data-link") === "live" || f.getAttribute("data-link") === "reload");
  }, null, { timeout: 30000 });
  const state = await pl.locator("footer[data-link]").getAttribute("data-link");
  const why = await pl.evaluate(() => [...document.querySelectorAll("strong")].map((e) => e.textContent).join(" / "));
  assert.equal(state, "live", `the screen goes live, not: ${why}`);
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("known-replay e2e: ok");
} finally {
  server.kill();
}
