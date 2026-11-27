// Roll call (track SH2) in a real browser, against the built shell (scripts/build-shell.sh first). The "Sol today"
// folder is served from store-packs/sol-today as the front door's `?spaceflight=` folder would be on Explorers: the
// address opens the Solar System in Roll call, which is on the rail too, opens on a day with people up, takes a typed date, steps by day and by event, says an empty day
// in words, and fits a phone. No figure from a book is pinned here: only that the page agrees with the look-up.
// Then the same pack as one file through the Open door, like any pack (CM0): added to the campaign, Roll call reads it
// from the stack with no folder anywhere, and it is still there after a reload, kept beside the save in IndexedDB
// because it is too big for local storage.
// Usage: node tests/ui/rollcall.e2e.mjs [chromium firefox] [--screens DIR]   (SSE_SHOTS=<dir> for screenshots)
import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const pw = createRequire(import.meta.url)("playwright");
const PACK = "store-packs/sol-today";
if (!existsSync(join(PACK, "crews.json"))) {
  console.log("rollcall e2e: no crews.json in the pack yet (RS9); nothing to check");
  process.exit(0);
}
const names = process.argv.slice(2).filter((a) => !a.startsWith("--") && ["chromium", "firefox", "webkit"].includes(a));
const engines = names.length ? names : ["chromium"];
const port = 9500 + Math.floor(Math.random() * 300);
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shots = process.env.SSE_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });
const shot = async (page, n) => shots && (await page.screenshot({ path: `${shots}/${n}.png` }));

/** Serve the pack folder at /sol-today/ as Explorers would. */
async function servePack(page) {
  await page.route((u) => u.pathname.startsWith("/sol-today/"), async (route) => {
    const rel = new URL(route.request().url()).pathname.slice("/sol-today/".length);
    const file = join(PACK, rel);
    if (!rel || rel.includes("..") || !existsSync(file) || statSync(file).isDirectory()) return route.fulfill({ status: 404, body: "" });
    await route.fulfill({ status: 200, contentType: "application/json", body: readFileSync(file) });
  });
}

let failed = false;
try {
  for (const engine of engines) {
    const browser = await pw[engine].launch();
    const errors = [];
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: shots ? 2 : 1 });
    page.on("pageerror", (e) => errors.push(e.message));
    await servePack(page);
    await page.goto(`http://localhost:${port}/?spaceflight=/sol-today/`);
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");

    // The front door (SH8 part 1): the folder's address opens the Solar System in Roll call, no start sheet, no click.
    // Roll call is on the rail under Travel as well.
    const panel = page.locator("[data-rollcall]");
    await panel.waitFor({ timeout: 60000 });
    await page.locator('.rail [data-go="rollcall"]').first().waitFor();
    await page.getByText("The Solar System").first().waitFor({ timeout: 60000 });

    // It opens on the pack's latest day, which has people up.
    const sentence = panel.locator("[data-rollcall-sentence]");
    await page.waitForFunction(() => /in space, aboard/.test(document.querySelector("[data-rollcall-sentence]")?.textContent ?? ""));
    assert.ok((await panel.locator("[data-rollcall-person]").count()) > 0, "people listed");

    // The count (SH3): a stepped line over the pack's span; pressing it goes to that day; losses are hollow marks.
    const strip = panel.locator("[data-rollcall-strip] svg");
    await strip.waitFor();
    const before = await panel.locator("[data-rollcall-date]").textContent();
    const box = await strip.boundingBox();
    await page.mouse.click(box.x + box.width * 0.2, box.y + box.height / 2);
    await page.waitForFunction((b) => document.querySelector("[data-rollcall-date]")?.textContent !== b, before);
    assert.match(await panel.locator("[data-rollcall-strip-read]").textContent(), /people in space|person in space/i);

    // Type a date: the title takes it, and the roster is that day's.
    const typeDate = async (text) => {
      await panel.locator("[data-rollcall-date]").click();
      await panel.locator("[data-rollcall-type]").fill(text);
      await panel.locator("[data-rollcall-type]").press("Enter");
      await page.waitForFunction((t) => document.querySelector("[data-rollcall-date]")?.textContent?.trim() === t, text, { timeout: 30000 }).catch(async (e) => {
        throw new Error(`typed "${text}", the title says "${await panel.locator("[data-rollcall-date], [data-rollcall-type]").first().evaluate((el) => el.textContent || el.value)}"; alert: ${await panel.getByRole("alert").textContent().catch(() => "none")}`, { cause: e });
      });
    };
    await typeDate("20 July 1969");
    await page.waitForFunction(() => document.querySelectorAll("[data-rollcall-person]").length > 0 && !!document.querySelector("[data-rollcall-place]"));
    const places = await panel.locator("[data-rollcall-place]").allTextContents();
    assert.ok(places.length > 0, "grouped by place");
    await shot(page, `desk-roll-call-1969-${engine}`);

    // Day › moves one day; ‹ Day comes back.
    await panel.locator('[data-rollcall-step="day-on"]').click();
    await page.waitForFunction(() => document.querySelector("[data-rollcall-date]")?.textContent?.trim() === "21 July 1969");
    await panel.locator('[data-rollcall-step="day-back"]').click();
    await page.waitForFunction(() => document.querySelector("[data-rollcall-date]")?.textContent?.trim() === "20 July 1969");

    // Event › goes forward to a dated event: the date never goes back.
    await panel.locator('[data-rollcall-step="event-on"]').click();
    await page.waitForFunction(() => document.querySelector("[data-rollcall-date]")?.textContent?.trim() !== "");

    // Records and On this day (SH5): each record says how it was worked out and goes to its day; a year in On this
    // day goes to that year's same day. No figure is pinned.
    if (await panel.locator("[data-rollcall-records]").count()) {
      await panel.locator("[data-rollcall-records] summary").click();
      const most = panel.locator('[data-rollcall-record="most"]');
      assert.match(await most.textContent(), /Most people in space at once.*people.*highest step/s);
      await most.locator("button").click();
      await page.waitForFunction(() => /in space, aboard/.test(document.querySelector("[data-rollcall-sentence]")?.textContent ?? ""));
      await typeDate("20 July 1969");
      await panel.locator("[data-rollcall-this-day] summary").click();
      assert.match(await panel.locator("[data-rollcall-this-day] summary").textContent(), /On this day, 20 July/);
      await shot(page, `desk-records-${engine}`);
      await panel.locator("[data-rollcall-this-day] li button").first().click();
      await page.waitForFunction(() => / July /.test(document.querySelector("[data-rollcall-date]")?.textContent ?? "") && document.querySelector("[data-rollcall-date]")?.textContent?.trim() !== "20 July 1969");
    }

    // An empty day says so in words, with the last person down and the next up as links.
    await typeDate("1 January 1958");
    await panel.locator("[data-rollcall-empty]").waitFor();
    assert.match(await panel.locator("[data-rollcall-empty]").textContent(), /Nobody had been to space yet\./);
    assert.equal(await sentence.textContent(), "");

    // A wrong date is said, not guessed.
    await panel.locator("[data-rollcall-date]").click();
    await panel.locator("[data-rollcall-type]").fill("the day after");
    await panel.locator("[data-rollcall-type]").press("Enter");
    await panel.getByRole("alert").waitFor();
    await panel.locator("[data-rollcall-type]").press("Escape");

    // In memory (SH6), when the pack gives it: the first line, the parts, the sources; a date goes to that day in Roll
    // call, which names the loss under its roster; back again. No name or figure is pinned here.
    if (await panel.locator("[data-rollcall-in-memory]").count()) {
      await panel.locator("[data-rollcall-in-memory]").click();
      const mem = panel.locator("[data-in-memory]");
      await mem.waitFor();
      assert.match(await mem.locator("[data-in-memory-line]").textContent(), /died flying to space, preparing to, or working to send others\. These are their names\.$/);
      assert.ok((await mem.locator("[data-in-memory-part]").count()) >= 1, "parts");
      assert.ok((await mem.locator("[data-in-memory-sources] li").count()) >= 2 * (await mem.locator("[data-in-memory-loss]").count()), "two sources an entry");
      assert.equal(await mem.locator("[data-in-memory-correct]").getAttribute("target"), "_blank");
      // Names carried (SH7), when the pack gives memorials: each row says who it is for and when it was named, or why
      // that date is not given.
      if (await mem.locator("[data-in-memory-carried]").count()) {
        const rows = await mem.locator("[data-in-memory-carried-row]").allTextContents();
        const once = /naming dates are not given/.test(await mem.locator("[data-in-memory-carried-lead]").textContent());
        assert.ok(rows.length > 0 && rows.every((r) => /, for .+\./.test(r) && (once || /Named |naming date/.test(r))), "carried rows");
      }
      await shot(page, `desk-in-memory-${engine}`);
      const first = mem.locator("[data-in-memory-loss]").first();
      const id = await first.getAttribute("data-in-memory-loss");
      await first.locator("button.day").click();
      await panel.locator(`[data-rollcall-loss="${id}"]`).waitFor();
      await shot(page, `desk-day-of-a-loss-${engine}`);
      await panel.locator(`[data-rollcall-loss="${id}"] button`).click();
      await mem.waitFor();
      await mem.locator("[data-in-memory-back]").click();
      await panel.locator("[data-rollcall-date]").waitFor();
    }

    // Phone width: one column, nothing wider than the screen.
    await page.setViewportSize({ width: 390, height: 844 });
    await typeDate("20 July 1969").catch(() => {});
    const wide = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(wide <= 1, `page scrolls sideways by ${wide}px on a phone`);
    await shot(page, `phone-roll-call-${engine}`);

    assert.deepEqual(errors, [], `page errors in ${engine}`);
    await browser.close();
    console.log(`rollcall e2e: ${engine} ok`);
  }

  // CM0: Sol today as one file, through the one Open door, into a campaign.
  const bundle = join(tmpdir(), `sol-today-${port}.json`);
  execFileSync(process.execPath, ["scripts/bundle-pack.mjs", PACK, bundle], { stdio: "pipe" });
  try {
    for (const engine of engines) {
      const browser = await pw[engine].launch();
      const errors = [];
      const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: shots ? 2 : 1 });
      page.on("pageerror", (e) => errors.push(e.message));
      const folderAsked = [];
      page.on("request", (r) => new URL(r.url()).pathname.startsWith("/sol-today/") && folderAsked.push(r.url()));
      const ready = () => page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 60000 });
      await page.goto(`http://localhost:${port}/#chart`);
      await ready();

      await more(page, "Campaign…");
      const od = page.getByRole("dialog", { name: "Campaign" });
      await od.getByLabel("File", { exact: true }).setInputFiles(bundle);
      await od.locator("[data-opened='pack']").waitFor({ timeout: 60000 });
      await od.locator("[data-pack-check]").waitFor({ timeout: 120000 });
      await shot(page, `desk-open-sol-today-${engine}`);
      await od.getByRole("button", { name: "Add it to this campaign" }).click();
      await od.getByText(/^Added\./).waitFor({ timeout: 120000 });
      await od.getByRole("button", { name: "Close", exact: true }).click();

      // Roll call reads the pack from the stack: the Gregorian calendar, the people, and their craft's logs.
      const rollFromStack = async () => {
        await go(page, "rollcall");
        const panel = page.locator("[data-rollcall]");
        await panel.waitFor({ timeout: 60000 });
        await panel.locator("[data-rollcall-date]").click();
        await panel.locator("[data-rollcall-type]").fill("20 July 1969");
        await panel.locator("[data-rollcall-type]").press("Enter");
        await page.waitForFunction(() => document.querySelector("[data-rollcall-date]")?.textContent?.trim() === "20 July 1969", null, { timeout: 30000 });
        await page.waitForFunction(() => /in space, aboard/.test(document.querySelector("[data-rollcall-sentence]")?.textContent ?? "") && !!document.querySelector("[data-rollcall-place]"), null, { timeout: 30000 });
        assert.ok((await panel.locator("[data-rollcall-person]").count()) >= 3, "Apollo 11's crew at least");
        return panel;
      };
      await rollFromStack();
      await shot(page, `desk-roll-call-installed-${engine}`);

      // Kept: the save names the pack by its hash, and is small enough for local storage.
      await page.waitForFunction(() => (localStorage.getItem("sse4.campaign") ?? "").includes('"sse.pack.kept/1"'), null, { timeout: 30000 });
      const kept = await page.evaluate(() => localStorage.getItem("sse4.campaign").length);
      assert.ok(kept < 1e6, `the kept save is ${kept} characters`);

      // Reopened: the pack is put back from IndexedDB and Roll call reads it again.
      await page.reload();
      await ready();
      await page.waitForFunction(() => /Reopened the campaign/.test(document.body.textContent ?? ""), null, { timeout: 60000 });
      await rollFromStack();

      assert.deepEqual(folderAsked, [], "no folder is fetched: everything comes from the pack in the stack");
      assert.deepEqual(errors, [], `page errors in ${engine}`);
      await browser.close();
      console.log(`rollcall e2e: ${engine} Sol today opened as a file, added, read and reopened`);
    }
  } finally {
    rmSync(bundle, { force: true });
  }
} catch (e) {
  failed = true;
  console.error(e);
} finally {
  server.kill();
}
process.exit(failed ? 1 : 0);
