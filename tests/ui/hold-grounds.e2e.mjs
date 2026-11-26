// Grounds on held screens and "Make a postcard" (PC5), against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/hold-grounds.e2e.mjs [--screens <dir>].
// On Earth's globe (the sample V3 save) the GM keeps a postcard, and bookmarks the place too. The Bookmarks list marks
// the postcard with a stamp; "Make a postcard" on the plain bookmark says why it cannot be one yet, and on the postcard
// opens it on the wall. On air, a screen held with a card and the ground "A postcard" shows that postcard full screen
// with the card's words over it, and nothing of the place it was showing; a quote stands alone (no waiting line);
// ground None takes the picture away again.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(
  process.execPath,
  ["scripts/serve.mjs", "packages/shell/dist", String(port)],
  { stdio: "ignore" },
);
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens)
    (mkdirSync(screens, { recursive: true }),
      await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
  });
  const errors = [];
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", {
    timeout: 20000,
  });
  await more(gm, "Make a system");
  const dlg = gm.getByRole("dialog", { name: "Make a system" });
  await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
  await dlg.getByRole("button", { name: "Try a sample save" }).click();
  await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
  await dlg
    .getByRole("button", { name: "Add this system to the campaign" })
    .click();
  const earth = gm.locator('.area-chart .body[aria-label^="Earth"]').first();
  await earth.waitFor({ timeout: 15000 });
  await earth.locator(".hit").click();

  // A plain bookmark of the chart, then a postcard kept from the globe.
  await gm.getByRole("button", { name: "Bookmarks" }).first().click();
  await gm.locator("[data-bookmark-add]").click();
  await gm.keyboard.press("Enter");
  await gm.getByRole("button", { name: "Bookmarks" }).first().click();
  await go(gm, "body");
  await gm.waitForSelector(
    "[data-body] [data-tier]:not([data-tier='']):not([data-frames='0'])",
    { timeout: 30000 },
  );
  await gm.locator("[data-take-photo]").evaluate((b) => b.click());
  await gm.locator("[data-photo-note]").waitFor({ timeout: 60000 });
  await gm.locator("[data-keep-postcard]").click();
  await gm.locator("[data-kept-postcard]").waitFor();

  // The Bookmarks list: the postcard has its stamp; the plain bookmark cannot be one yet and says why.
  await gm.getByRole("button", { name: "Bookmarks" }).first().click();
  const rows = gm.locator("[data-bookmarks-list] li[data-bookmark]");
  assert.equal(await rows.count(), 2, "two bookmarks");
  const plainId = await rows.nth(0).getAttribute("data-bookmark");
  const cardId = await rows.nth(1).getAttribute("data-bookmark");
  assert.equal(
    await gm.locator(`[data-bookmark-stamp="${cardId}"]`).count(),
    1,
    "the postcard is marked with a stamp",
  );
  assert.equal(
    await gm.locator(`[data-bookmark-stamp="${plainId}"]`).count(),
    0,
  );
  await gm.locator(`[data-bookmark-postcard="${plainId}"]`).click();
  await gm
    .locator("[data-bookmarks-list] [role=status]")
    .filter({ hasText: /picture|camera/ })
    .waitFor();
  await shot(gm, "pc5-bookmarks");
  await gm.locator(`[data-bookmark-postcard="${cardId}"]`).click();
  await gm
    .locator("[data-postcard-wall] [data-postcard-close]")
    .waitFor({ timeout: 15000 });
  await shot(gm, "pc5-make-opens-wall");

  // On air: a screen joins and is sent the place; then held with a quote on the postcard's ground.
  await desk(gm, "Run");
  await go(gm, "switchboard"); // On Run the rail is folded into the view controls (three desks design 3.1).
  await gm.locator("[data-switchboard]").waitFor();
  await gm.locator("[data-go-on-air]").click();
  await gm.locator("[data-on-air]").first().waitFor();
  const link = await gm
    .locator("[data-switchboard] [data-players-link]")
    .getAttribute("data-players-link");
  const tv = await context.newPage();
  await tv.setViewportSize({ width: 1280, height: 720 });
  tv.on("pageerror", (e) => errors.push(`tv: ${e.message}`));
  await tv.goto(`${link}&name=${encodeURIComponent("Wall TV")}`);
  await tv.waitForSelector(
    "footer[data-link='live'][data-state-hash]:not([data-state-hash=''])",
    { timeout: 30000 },
  );
  const box = gm.locator("[data-switchboard] .screen", { hasText: "Wall TV" });
  await box.waitFor({ timeout: 15000 });
  const tvId = await box.getAttribute("data-screen");
  await gm.locator("[data-send-live]").click();
  await tv.locator("main.grid .area").first().waitFor({ timeout: 30000 });

  // A campaign postcard goes only to a role that knows its place: held now, the table is shown no picture.
  await gm.locator(`[data-hold="screen:${tvId}"]`).click();
  await tv.locator("[data-screens-off][data-held]").waitFor({ timeout: 10000 });
  await gm
    .locator(`[data-switchboard] [data-ground]`)
    .first()
    .selectOption("one");
  await gm
    .locator(
      `[data-switchboard] [data-ground-card="${cardId}"][aria-checked=true]`,
    )
    .waitFor();
  await gm.waitForTimeout(1500);
  assert.equal(
    await tv.locator("[data-held-ground]").count(),
    0,
    "the table does not know this world yet, so no ground",
  );
  await gm
    .locator(`[data-switchboard] [data-ground]`)
    .first()
    .selectOption("none");
  await gm.locator(`[data-hold="screen:${tvId}"]`).click();
  await tv
    .locator("[data-screens-off]")
    .waitFor({ state: "detached", timeout: 10000 });
  // The GM reveals Earth to the table (its card's eye on the chart).
  await go(gm, "system");
  await gm
    .locator('.area-chart .body[aria-label^="Earth"]')
    .first()
    .locator(".hit")
    .click();
  const eye = gm
    .locator("span.known[data-known]:not([data-known-field]) [data-known-eye]")
    .first();
  await eye.waitFor({ timeout: 20000 });
  if (
    (await gm
      .locator("span.known[data-known]:not([data-known-field])")
      .first()
      .getAttribute("data-known")) !== "known"
  )
    await eye.click();
  await gm.waitForTimeout(2600);
  await go(gm, "switchboard"); // On Run the rail is folded into the view controls (three desks design 3.1).
  await gm.locator("[data-switchboard]").waitFor();

  await gm.locator(`[data-hold="screen:${tvId}"]`).click();
  await tv.locator("[data-screens-off][data-held]").waitFor({ timeout: 10000 });
  const card = gm.locator(`[data-card="screen:${tvId}"]`);
  assert.equal(
    await card.locator("option[value=quote]").count(),
    1,
    "the Quotes kind is on the line card",
  );
  await card.selectOption("quote");
  await gm
    .locator(`[data-switchboard] [data-ground]`)
    .first()
    .selectOption("one");
  await gm
    .locator(
      `[data-switchboard] [data-ground-card="${cardId}"][aria-checked=true]`,
    )
    .waitFor();
  await tv
    .locator("[data-held-ground] img[data-ground-picture]")
    .waitFor({ timeout: 20000 });
  assert.match(
    await tv.locator("img[data-ground-picture]").getAttribute("src"),
    /^data:image\//,
    "the postcard's picture, as data",
  );
  assert.equal(
    await tv.locator("[data-held-kind=quote]").count(),
    1,
    "a quote stands alone: marked so nothing is added at its foot",
  );
  assert.equal(
    await tv.locator(".area").count(),
    0,
    "nothing of the place behind the card",
  );
  await shot(tv, "pc5-held-with-ground");
  await shot(gm, "pc5-line-card-ground");

  // Ground None takes the picture away; the card stays held.
  await gm
    .locator(`[data-switchboard] [data-ground]`)
    .first()
    .selectOption("none");
  await tv
    .locator("[data-held-ground]")
    .waitFor({ state: "detached", timeout: 10000 });
  assert.equal(await tv.locator("[data-screens-off][data-held]").count(), 1);

  assert.deepEqual(errors, []);
  console.log("hold grounds: ok");
  await browser.close();
} finally {
  server.kill();
}
