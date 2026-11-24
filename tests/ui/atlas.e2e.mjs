// The Atlas (track AT, AT2), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/atlas.e2e.mjs [--screens <dir>]
// "Atlas" on the rail opens the system as a book: contents down the left, the page, and step by the buttons, the
// arrow keys alone, or a link on the page. Seen by builds the book from that audience's own scene, so the table's
// book has no more pages than the GM's and its figures are its own. Reading the Atlas never changes the state hash.
// On a phone the contents fold into a button and nothing runs off the side. Both chart modes.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9500 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(300);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const ready = (page) => page.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 30000 });
const hashOf = (page) => page.locator("[data-state-hash]").first().getAttribute("data-state-hash");
const title = (page) => page.locator("[data-atlas-page] h1").innerText();

try {
  const browser = await chromium.launch();
  const errors = [];
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 })).newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(root + "#chart");
  await ready(gm);
  const hash0 = await hashOf(gm);

  await go(gm, "atlas");
  const atlas = gm.getByRole("region", { name: "Atlas" });
  await atlas.locator("[data-atlas-page]").waitFor({ timeout: 15000 });
  assert.equal(await title(gm), "Kestrel", "the system opens on its own page");
  const contents = atlas.getByRole("navigation", { name: "Contents" });
  const gmPages = await contents.locator("li").count();
  assert.ok(gmPages >= 4, `the GM's book has the system, its worlds and moons (${gmPages} pages)`);
  // Each block is a direct child of the page marked for the look transitions (AT15a seam).
  assert.ok((await gm.locator("[data-atlas-page] > [data-deliver-item]").count()) >= 2);
  await shot(gm, "atlas-system-desk");

  // A link on the page goes to that page; the arrow keys step the book.
  await atlas.locator("[data-atlas-page] table .link", { hasText: "Tarn" }).first().click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-page] h1")?.textContent === "Tarn");
  assert.equal(await contents.locator('[aria-current="page"]').innerText().then((t) => t.split("\n")[0]), "Tarn");
  assert.match(await atlas.locator("[data-atlas-page]").innerText(), /\d AU/, "an orbit is said with its unit");
  await shot(gm, "atlas-tarn-gm-desk");
  await gm.keyboard.press("ArrowRight");
  await gm.waitForFunction(() => document.querySelector("[data-atlas-page] h1")?.textContent !== "Tarn");
  await gm.keyboard.press("ArrowLeft");
  await gm.waitForFunction(() => document.querySelector("[data-atlas-page] h1")?.textContent === "Tarn");
  assert.equal(await gm.evaluate(() => document.activeElement?.closest("[data-atlas-page]") !== null), true, "focus follows the page");

  // The Edition switch (the GM's own, for browsing) re-dresses the same page in place: words and look change, figures do not.
  const figures = () => gm.locator("[data-atlas-page] .fig").allInnerTexts();
  const surveyFigs = await figures();
  const edition = atlas.getByRole("group", { name: "Edition" });
  await edition.getByRole("button", { name: "The Guide" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-edition]")?.getAttribute("data-atlas-edition") === "sse.edition.guide");
  assert.equal(await title(gm), "Tarn", "the same page");
  assert.match(await atlas.locator("[data-atlas-page]").innerText(), /Heft/i, "the Guide's own words");
  assert.deepEqual(await figures(), surveyFigs, "switching edition changes no figure");
  await shot(gm, "atlas-guide-desk");
  await edition.getByRole("button", { name: "Company" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-edition]")?.getAttribute("data-atlas-edition") === "sse.edition.company");
  assert.deepEqual(await figures(), surveyFigs);
  await shot(gm, "atlas-company-desk");

  // Seen by the table: their own book, never larger than the GM's, with gaps said in plain words.
  await atlas.getByRole("group", { name: "Seen by" }).getByRole("button", { name: "Everyone" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-audience]")?.getAttribute("data-atlas-audience") === "everyone", null, { timeout: 10000 });
  const tablePages = await contents.locator("li").count();
  assert.ok(tablePages <= gmPages, "the table's book has no page the GM's lacks");
  assert.ok((await gm.locator("[data-atlas-page] .bar").count()) > 0, "the Company edition bars what the table has not learnt");
  await shot(gm, "atlas-table-company-desk");
  await edition.getByRole("button", { name: "Survey atlas" }).click();
  await shot(gm, "atlas-table-desk");

  await more(gm, /Night chart/);
  await shot(gm, "atlas-night-desk");
  assert.equal(await hashOf(gm), hash0, "reading the Atlas never changes the universe's state");

  // Phone: the contents fold into a button; the page and the step fit the width.
  const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: screens ? 3 : 1, hasTouch: true, isMobile: true })).newPage();
  phone.on("pageerror", (e) => errors.push(e.message));
  await phone.goto(root + "#chart");
  await ready(phone);
  await go(phone, "atlas");
  const pa = phone.getByRole("region", { name: "Atlas" });
  await pa.locator("[data-atlas-page]").waitFor({ timeout: 15000 });
  assert.equal(await pa.getByRole("navigation", { name: "Contents" }).isVisible(), false, "the contents are folded");
  const fold = pa.getByRole("button", { name: "Contents" });
  await fold.click();
  assert.equal(await pa.getByRole("navigation", { name: "Contents" }).isVisible(), true);
  await shot(phone, "atlas-contents-phone");
  await pa.getByRole("navigation", { name: "Contents" }).locator("li button").nth(1).click();
  assert.equal(await pa.getByRole("navigation", { name: "Contents" }).isVisible(), false, "choosing a page folds them again");
  const wide = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(wide <= 1, `nothing runs off the side (${wide}px)`);
  await shot(phone, "atlas-page-phone");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("atlas e2e: ok");
} finally {
  server.kill();
}
