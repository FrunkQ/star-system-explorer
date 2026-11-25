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
const pdfs = args.indexOf("--pdfs") >= 0 ? args.splice(args.indexOf("--pdfs"), 2)[1] : screens;
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
  const gmContext = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1, acceptDownloads: true });
  await gmContext.grantPermissions(["clipboard-read", "clipboard-write"]);
  const gm = await gmContext.newPage();
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
  // AT7: the Guide is a gadget. Opening it shows the cover every time; one press goes past it.
  const cover = gm.locator("[data-atlas-cover]");
  await cover.waitFor();
  assert.match(await cover.innerText(), /DON'T PANIC/);
  assert.match(await cover.innerText(), /The Guide to the Kestrel system/);
  assert.doesNotMatch(await cover.innerText(), /Megadodo/, "the base Guide's cover names no publisher");
  assert.equal(await gm.evaluate(() => document.activeElement?.hasAttribute("data-atlas-cover")), true, "the cover takes focus, so Enter opens it");
  // One-liners at the top and bottom of the gadget's screen (Alex, 25 Nov): on the cover, waiting lines.
  const banner = (at) => gm.locator(`[data-atlas-banner="${at}"]`).innerText();
  const coverLines = [await banner("top"), await banner("bottom")];
  assert.notEqual(coverLines[0], coverLines[1], "two different lines");
  await shot(gm, "atlas-guide-cover-desk");
  await gm.keyboard.press("Enter");
  await gm.locator("[data-atlas-page]").waitFor();
  assert.equal(await title(gm), "Tarn", "the same page");
  const pageLines = [await banner("top"), await banner("bottom")];
  assert.notDeepEqual(pageLines, coverLines, "a world's lines, not the waiting screen's");
  // Margin notes: at most two, signed, the same ones each time, and the line saying they are opinions.
  const notes = gm.locator("[data-margin-note]");
  const noteIds = await notes.evaluateAll((ns) => ns.map((n) => n.getAttribute("data-margin-note")));
  assert.ok(noteIds.length >= 1 && noteIds.length <= 2, `one or two margin notes (${noteIds})`);
  assert.match(await notes.first().innerText(), /The Guide says/i);
  assert.match(await atlas.locator("[data-atlas-page]").innerText(), /opinions and are not facts/);
  // The verdict stamp is read off facts and opens to say which.
  const verdict = gm.locator("[data-verdict]");
  assert.match(await verdict.locator("summary").innerText(), /VERDICT: MOSTLY SURVIVABLE/i);
  await verdict.locator("summary").click();
  assert.match(await verdict.innerText(), /1\.3 bar/);
  // The keys: Next steps the book with a stepped wipe; the cover key brings the cover back; Back returns.
  const keys = atlas.getByRole("group", { name: "Keys" });
  await keys.getByRole("button", { name: "Next ›" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-page] h1")?.textContent !== "Tarn");
  await keys.getByRole("button", { name: "‹ Back" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-page] h1")?.textContent === "Tarn");
  assert.deepEqual(await notes.evaluateAll((ns) => ns.map((n) => n.getAttribute("data-margin-note"))), noteIds, "the same notes on the same page");
  await keys.getByRole("button", { name: "DON'T PANIC" }).click();
  await cover.waitFor();
  await cover.click();
  await gm.locator("[data-atlas-page]").waitFor();
  assert.match(await atlas.locator("[data-atlas-page]").innerText(), /Heft/i, "the Guide's own words");
  assert.deepEqual(await figures(), surveyFigs, "switching edition changes no figure");
  await shot(gm, "atlas-guide-desk");
  assert.equal(await gm.locator("[data-letterhead]").count(), 0, "the Guide has no letterhead");
  await edition.getByRole("button", { name: "Company" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-atlas-edition]")?.getAttribute("data-atlas-edition") === "sse.edition.company");
  assert.deepEqual(await figures(), surveyFigs);
  // AT6: the Company's paper. With no brand, the placeholder logo drawn in text; a built-in logo replaces it.
  const lh = gm.locator("[data-letterhead]");
  await lh.waitFor();
  const art = await lh.locator("pre").innerText();
  assert.match(art, /IIIII\s+SSSS\s+OOO/, "the fallback name's initials, drawn with themselves");
  assert.match(await lh.innerText(), /COMPANY CONFIDENTIAL/);
  assert.match(await lh.innerText(), /CLEARANCE: GM EYES ONLY/);
  await shot(gm, "atlas-company-desk");
  const paperPick = atlas.getByRole("combobox", { name: "Letterhead" });
  await paperPick.selectOption("kelido");
  await lh.getByRole("img", { name: "Kelido" }).waitFor();
  assert.equal(await lh.locator("pre").count(), 0, "a logo replaces the drawn one");
  assert.match(await lh.innerText(), /KEL-KES-02\/TARN/);
  await shot(gm, "atlas-company-logo-desk");
  await paperPick.selectOption("");
  assert.equal(await lh.locator("pre").innerText(), art, "the same name draws the same art again");

  // Print under the view (AT4). The browser's print is stood in for: it keeps what would print (the print-only pages
  // and the sheet size) and says printing began. Each edition prints in its own words; Save ink prints a dark look light.
  await gm.evaluate(() => {
    window.__prints = [];
    window.print = () => {
      const node = document.querySelector(".atlas-print:not([data-kept-print])");
      const size = document.querySelector("style[data-atlas-page-size]:not([data-kept-print])");
      window.__prints.push({ pages: Number(node?.getAttribute("data-atlas-print") ?? 0), size: size?.textContent ?? "" });
      document.querySelectorAll("[data-kept-print]").forEach((n) => n.remove());
      for (const n of [node, size]) if (n) document.body.appendChild(Object.assign(n.cloneNode(true), {})).setAttribute("data-kept-print", "");
      window.dispatchEvent(new Event("beforeprint"));
    };
  });
  const prints = () => gm.evaluate(() => window.__prints);
  const pdf = async (n) => {
    if (!pdfs) return;
    mkdirSync(pdfs, { recursive: true });
    await gm.evaluate(() => {
      const kept = [...document.querySelectorAll("[data-kept-print]")];
      for (const k of kept) if (k.tagName === "STYLE") document.head.appendChild(k);
    });
    await gm.emulateMedia({ media: "print" });
    await gm.pdf({ path: `${pdfs}/${n}.pdf`, preferCSSPageSize: true, printBackground: true });
    await gm.emulateMedia({ media: "screen" });
  };
  for (const [name, id] of [["Survey atlas", "survey"], ["The Guide", "guide"], ["Company", "company"]]) {
    await edition.getByRole("button", { name }).click();
    await gm.waitForFunction((x) => document.querySelector("[data-atlas-edition]")?.getAttribute("data-atlas-edition") === x, `sse.edition.${id}`);
    if (await gm.locator("[data-atlas-cover]").count()) await gm.locator("[data-atlas-cover]").click();
    const n = (await prints()).length + 1;
    await atlas.getByRole("button", { name: "Print this page" }).click();
    await gm.waitForFunction((x) => window.__prints.length === x, n);
    const kept = await gm.locator(".atlas-print[data-kept-print] [data-atlas-page] h1").allInnerTexts();
    assert.deepEqual(kept, ["Tarn"], `${name}: this page prints, and only it`);
    const keptFigs = await gm.locator(".atlas-print[data-kept-print] [data-atlas-page] .fig").allInnerTexts();
    assert.deepEqual(keptFigs, surveyFigs, `${name}: the printed figures are the page's own`);
    await pdf(`atlas-tarn-${id}`);
  }
  assert.equal(await gm.locator(".atlas-print:not([data-kept-print])").count(), 0, "the print pages go once printing is over");
  // Print…: the whole atlas on Letter.
  await atlas.getByRole("button", { name: "Print…" }).click();
  const panel = atlas.getByRole("group", { name: "Print" });
  await panel.getByRole("group", { name: "What to print" }).getByRole("button", { name: "Whole atlas" }).click();
  await panel.getByRole("group", { name: "Paper" }).getByRole("button", { name: "Letter" }).click();
  await atlas.getByRole("button", { name: "Print…" }).scrollIntoViewIfNeeded();
  await shot(gm, "atlas-print-panel-desk");
  await panel.getByRole("button", { name: "Print", exact: true }).click();
  await gm.waitForFunction(() => window.__prints.length === 4);
  const whole = (await prints())[3];
  assert.equal(whole.pages, gmPages, "the whole atlas prints every page in the book");
  assert.match(whole.size, /size: letter/);
  await pdf("atlas-whole-company");
  // When printing cannot start, the Atlas says so in a sentence.
  await gm.evaluate(() => (window.print = () => {}));
  await atlas.getByRole("button", { name: "Print this page" }).click();
  await atlas.getByRole("status").filter({ hasText: "Printing did not start" }).waitFor();
  // Copy as text and Save page carry the same words and figures.
  await atlas.getByRole("button", { name: "Print…" }).click();
  await panel.getByRole("group", { name: "What to print" }).getByRole("button", { name: "This page" }).click();
  await panel.getByRole("button", { name: "Copy as Markdown" }).click();
  await atlas.getByRole("status").filter({ hasText: "Copied as Markdown" }).waitFor();
  const md = await gm.evaluate(() => navigator.clipboard.readText());
  assert.match(md, /^# Tarn$/m);
  assert.match(md, /COMPANY CONFIDENTIAL/, "the copy carries the letterhead");
  for (const f of surveyFigs) assert.ok(md.includes(f), `the copy says ${f}`);
  await atlas.getByRole("button", { name: "Print…" }).click();
  const [download] = await Promise.all([gm.waitForEvent("download"), panel.getByRole("button", { name: "Save page" }).click()]);
  assert.match(download.suggestedFilename(), /^Company - Tarn\.html$/);
  if (pdfs) await download.saveAs(`${pdfs}/atlas-tarn-company.html`);
  await gm.evaluate(() => document.querySelectorAll("[data-kept-print]").forEach((n) => n.remove()));

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
  // The Guide on a phone: the cover, then the page with its keys, nothing off the side.
  await pa.getByRole("group", { name: "Edition" }).getByRole("button", { name: "The Guide" }).click();
  await phone.locator("[data-atlas-cover]").waitFor();
  await shot(phone, "atlas-guide-cover-phone");
  await phone.locator("[data-atlas-cover]").click();
  await phone.locator("[data-atlas-page]").waitFor();
  const wideGuide = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(wideGuide <= 1, `the Guide fits the phone (${wideGuide}px)`);
  await shot(phone, "atlas-guide-phone");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("atlas e2e: ok");
} finally {
  server.kill();
}
