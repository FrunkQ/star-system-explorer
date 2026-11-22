// U4 part 2 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/lookahead.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Look ahead from the chart room: the main line's state hash holds while
// looking; the scrubber moves the chart through the snapshots; at the end
// Tarn has been engulfed and the star is a white dwarf, marked one plausible
// future; throwing away restores the chart; keeping and moving now changes
// the main line and the ruler's now.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await gm.waitForTimeout(800);
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const h0 = await hash();
    const tarn = () => gm.locator('.area-chart .body[aria-label^="Tarn"]').count();
    assert.ok(await tarn());

    await gm.getByRole("button", { name: "Look ahead…" }).click();
    const panel = gm.getByRole("group", { name: "Look ahead" });
    await panel.getByLabel("Target age in Gyr").fill("40");
    await panel.getByRole("button", { name: "Look ahead" }).click();
    await gm.waitForSelector(".ahead[data-ahead=open]", { timeout: 30000 });
    assert.equal(await hash(), h0, "looking does not move the main line");
    assert.equal(await gm.locator("main.stage").getAttribute("data-branch"), "ahead");
    const events = panel.getByLabel("What happens");
    assert.match(await events.innerText(), /Red giant/);
    assert.match(await events.innerText(), /Engulfed: Tarn/);
    // Scrub to the end: a white dwarf, Tarn gone, marked as one plausible future.
    const scrub = panel.getByLabel("Age shown");
    const max = await scrub.getAttribute("max");
    await scrub.fill(max);
    await gm.waitForFunction(() => !document.querySelector('.area-chart .body[aria-label^="Tarn"]'));
    assert.match(await panel.locator(".scrub").innerText(), /white dwarf/);
    assert.match(await panel.locator(".scrub").innerText(), /one plausible future/i);
    await gm.locator('.area-chart .body[aria-label^="Kestrel"]').dispatchEvent("pointerdown");
    await gm.locator(".area-card").getByText(/white dwarf/).first().waitFor({ timeout: 5000 });
    await shot(gm, "u4-lookahead-end");
    // Why for one event.
    await panel.getByRole("button", { name: /^Why: Red giant/ }).click();
    assert.match(await panel.locator(".why").innerText(), /red giant/i);

    // Throw away: the chart is back as it was.
    await panel.getByRole("button", { name: "Throw away" }).click();
    await gm.waitForFunction(() => !document.querySelector(".ahead[data-ahead]"));
    assert.ok(await tarn(), "Tarn is back");
    assert.equal(await hash(), h0);

    // Again to 10 Gyr, keep and move now there.
    await panel.getByLabel("Target age in Gyr").fill("10");
    await panel.getByRole("button", { name: "Look ahead" }).click();
    await gm.waitForSelector(".ahead[data-ahead=open]", { timeout: 30000 });
    await panel.getByLabel("Age shown").fill(await panel.getByLabel("Age shown").getAttribute("max"));
    const at = Number(await gm.locator(".ahead").getAttribute("data-ahead-at"));
    await panel.getByRole("button", { name: /^Keep and move now/ }).click();
    await gm.waitForFunction(() => !document.querySelector(".ahead[data-ahead]"));
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, h0);
    assert.match(await gm.locator(".whatif [role=status]").innerText(), /now is 10 Gyr/);
    assert.equal(at, 10000);
    await shot(gm, "u4-lookahead-kept");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: looked ahead to 40 Gyr without moving the main line; Tarn engulfed, white dwarf marked plausible; thrown away cleanly; kept to 10 Gyr with now moved`);
  }
} finally {
  server.kill();
}
