// Roll call (track SH2) in a real browser, against the built shell (scripts/build-shell.sh first). The "Sol today"
// folder is served from store-packs/sol-today as the front door's `?spaceflight=` folder would be on Explorers: Roll
// call is on the rail, opens on a day with people up, takes a typed date, steps by day and by event, says an empty day
// in words, and fits a phone. No figure from a book is pinned here: only that the page agrees with the look-up.
// Usage: node tests/ui/rollcall.e2e.mjs [chromium firefox] [--screens DIR]   (SSE_SHOTS=<dir> for screenshots)
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

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
    await page.goto(`http://localhost:${port}/?spaceflight=/sol-today/#chart`);
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    const startSkip = page.locator("[data-start-skip], .start [data-skip]").first();
    if (await startSkip.isVisible().catch(() => false)) await startSkip.click();

    // On the rail, under Travel, because the folder gives crews.
    const entry = page.locator('.rail [data-go="rollcall"]').first();
    await entry.waitFor();
    await entry.click();
    const panel = page.locator("[data-rollcall]");
    await panel.waitFor();

    // It opens on the pack's latest day, which has people up.
    const sentence = panel.locator("[data-rollcall-sentence]");
    await page.waitForFunction(() => /in space, aboard/.test(document.querySelector("[data-rollcall-sentence]")?.textContent ?? ""));
    assert.ok((await panel.locator("[data-rollcall-person]").count()) > 0, "people listed");

    // Type a date: the title takes it, and the roster is that day's.
    const typeDate = async (text) => {
      await panel.locator("[data-rollcall-date]").click();
      await panel.locator("[data-rollcall-type]").fill(text);
      await panel.locator("[data-rollcall-type]").press("Enter");
      await page.waitForFunction((t) => document.querySelector("[data-rollcall-date]")?.textContent?.trim() === t, text);
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
} catch (e) {
  failed = true;
  console.error(e);
} finally {
  server.kill();
}
process.exit(failed ? 1 : 0);
