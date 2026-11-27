// The four starter kits on a role's own screen (track AT, AT15 part 2), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/kits.e2e.mjs [--screens <dir>]
// The GM shares, adds a role and opens its screen. Choosing each kit on the role card puts the kit's layout on that
// screen at the same place: the kit's look and palette on the page, the Atlas part in the kit's edition, and every
// control drawn in the kit's control set (Hauler's `[P]RINT`, Corporate's pills, Deck's caps, the Guide's keys). A
// palette chosen on the card recolours the screen. Desktop and phone sizes.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9400 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(2500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

const KITS = [
  { id: "sse.kit.hauler", look: "phosphor", set: "hauler", edition: "sse.edition.company", frame: "bracket", print: /\[P\]RINT/, palette: "phosphor-amber" },
  { id: "sse.kit.corporate", look: "glass", set: "corporate", edition: "sse.edition.survey", frame: "pill", print: /^Print$/ },
  { id: "sse.kit.deck", look: "deck", set: "deck", edition: "sse.edition.survey", frame: "cap", print: /^PRINT$/ },
  { id: "sse.kit.guide", look: "guide-gadget", set: "guide", edition: "sse.edition.guide", frame: "key", print: /^Print$/ },
];

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
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();

  await gm.locator("[data-on-air]").click();
  await gm.locator("[data-switchboard]").waitFor();
  await gm.locator("[data-add-role]").click();
  const card = gm.locator("[data-role-card='new']");
  await card.locator("[data-role-name]").fill("Bridge crew");
  // It sees what everyone has been shown, so its Atlas has pages.
  await card.locator("[data-role-knows]").selectOption("everyone");
  await card.locator("[data-role-add]").click();
  await card.waitFor({ state: "detached" });
  await gm.locator("[data-role-open='bridge-crew']").click();
  const rc = gm.locator("[data-role-card='bridge-crew']");
  const href = await rc.locator("[data-seat-link]").first().getAttribute("href");

  const desktop = await context.newPage();
  await desktop.setViewportSize({ width: 1440, height: 900 });
  // The phone shares the GM's browser (the table's link travels in it), so it is a phone-sized window with a mouse.
  const phone = await context.newPage();
  await phone.setViewportSize({ width: 390, height: 844 });
  for (const [n, p] of [["desktop", desktop], ["phone", phone]]) {
    p.on("pageerror", (e) => errors.push(`${n}: ${e.message}`));
    await p.goto(href);
    await p.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  }

  for (const k of KITS) {
    await rc.locator("[data-role-kit]").selectOption(k.id);
    if (k.palette) await rc.locator(`[data-role-palette="${k.palette}"]`).click();
    for (const [n, p] of [["desktop", desktop], ["phone", phone]]) {
      const atlas = p.locator(`[data-player-atlas="${k.edition}"][data-kit-set="${k.set}"]`);
      await atlas.waitFor({ timeout: 20000 }).catch(async (e) => {
        await p.screenshot({ path: "/tmp/claude-0/-home-user/83c287c4-ef13-5a38-ac6a-c6cd379f7903/scratchpad/fail.png" });
        console.log(await p.evaluate(() => ({ theme: document.documentElement.dataset.theme, views: [...document.querySelectorAll("[data-view]")].map((x) => x.getAttribute("data-view")), atlas: document.querySelector("[data-player-atlas]")?.outerHTML.slice(0, 400), text: "" })));
        throw e;
      });
      assert.equal(await p.evaluate(() => document.documentElement.dataset.theme), k.look, `${k.id} ${n}: the kit's look`);
      const print = atlas.locator('[data-kit-control="button"]').last();
      assert.equal(await print.getAttribute("data-frame"), k.frame, `${k.id} ${n}: Print in the kit's frame`);
      assert.match((await print.innerText()).trim(), k.print, `${k.id} ${n}: Print in the kit's words`);
      const box = await print.boundingBox();
      assert.ok(box && box.height >= 32 - 0.5, `${k.id} ${n}: Print is big enough to press (${box?.height})`);
      assert.ok(await atlas.locator("article[data-atlas-page] h1").count(), `${k.id} ${n}: a page of the Atlas`);
      if (k.palette) {
        const ink = await p.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--sse-ink").trim().toUpperCase());
        assert.equal(ink, "#FFB547", `${k.id} ${n}: the palette chosen on the role card (amber)`);
      }
      const grounds = await p.evaluate(() => [getComputedStyle(document.body).backgroundColor, getComputedStyle(document.querySelector("main") ?? document.body).color]);
      const want = await p.evaluate(() => { const d = document.createElement("i"); d.style.color = "var(--sse-ground)"; document.body.append(d); const c = getComputedStyle(d).color; d.remove(); return c; });
      assert.equal(grounds[0], want, `${k.id} ${n}: the page's ground is the kit's`);
      const overflow = await p.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      assert.ok(overflow <= 1, `${k.id} ${n}: no sideways scroll (${overflow}px)`);
      await shot(p, `kit-${k.set}-${n}`);
    }
  }

  // The steps move the book, and the chart beside it follows (the selection is the screen's).
  const hauler = KITS[0];
  await rc.locator("[data-role-kit]").selectOption(hauler.id);
  const atlas = desktop.locator(`[data-player-atlas="${hauler.edition}"]`);
  await atlas.waitFor({ timeout: 20000 });
  const before = await atlas.locator("article[data-atlas-page] h1").innerText();
  await atlas.locator('[data-kit-control="step"]').last().click();
  await desktop.waitForFunction((b) => document.querySelector("[data-player-atlas] article[data-atlas-page] h1")?.textContent !== b, before, { timeout: 5000 });
  // Hauler shows keys: B steps back.
  await atlas.locator("article[data-atlas-page] h1").focus();
  await desktop.keyboard.press("b");
  await desktop.waitForFunction((b) => document.querySelector("[data-player-atlas] article[data-atlas-page] h1")?.textContent === b, before, { timeout: 5000 });

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("kits e2e: ok");
} finally {
  server.kill();
}
