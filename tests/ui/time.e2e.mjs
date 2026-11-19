// U11 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/time.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Human-scale time on the ruler: it opens at the campaign's now; Show "A
// week" zooms to days; stepping forward while at now moves the campaign
// (time passes); the track itself only looks, leaving the campaign where it
// was; Back to now returns; Make this now moves it; Play advances; the
// campaign's now survives a reload. On a phone the ruler fits its width.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const DAY = 86_400 / 31_557_600;

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    for (const [vpName, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
      const page = await (await browser.newContext({ viewport })).newPage();
      page.on("pageerror", (e) => errors.push(e.message));
      const shot = async (n) => {
        if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${vpName}-${n}.png` }));
      };
      const ruler = page.locator(".area-ruler .ruler");
      const readout = () => ruler.locator(".now").innerText();
      const linked = () => ruler.getAttribute("data-linked");
      const viewing = async () => Number(await ruler.locator(".now").getAttribute("data-viewing"));
      await page.goto(base);
      await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });

      // Opens at the campaign's now, in deep time.
      assert.equal(await linked(), "yes");
      assert.match(await ruler.innerText(), /Now\s*in the campaign/i);
      assert.match(await readout(), /Gyr/);
      const start = await viewing();
      await shot("1-open");

      // A week on screen: dates and hours.
      await page.getByLabel("Show", { exact: true }).selectOption("week");
      const span = Number(await ruler.getAttribute("data-span"));
      assert.ok(Math.abs(span - 7 * DAY) < 1e-6, `span ${span}`);
      assert.equal(await readout(), "Year 1, day 1, 00:00");

      // Stepping at now: time passes in the campaign.
      await page.getByRole("button", { name: "Forward one day" }).click();
      await page.getByRole("button", { name: "Forward one day" }).click();
      assert.equal(await readout(), "Year 1, day 3, 00:00");
      assert.equal(await linked(), "yes");
      await shot("2-week");

      // The track only looks.
      await page.getByLabel("System time").focus();
      await page.keyboard.press("PageUp");
      assert.equal(await linked(), "no");
      assert.equal(await readout(), "Year 1, day 3, 16:48");
      assert.match(await ruler.locator("[data-campaign-now]").innerText(), /Campaign now: Year 1, day 3, 00:00/);
      await shot("3-looking");

      await page.getByRole("button", { name: "Back to now" }).click();
      assert.equal(await linked(), "yes");
      assert.equal(await readout(), "Year 1, day 3, 00:00");

      // Make this now.
      await page.getByLabel("System time").focus();
      await page.keyboard.press("PageDown");
      assert.equal(await readout(), "Year 1, day 2, 07:12");
      await page.getByRole("button", { name: "Make this now" }).click();
      assert.equal(await linked(), "yes");
      assert.equal(await readout(), "Year 1, day 2, 07:12");

      // Play: a day a second.
      await page.getByRole("button", { name: "Play", exact: true }).click();
      await page.waitForTimeout(1500);
      await page.getByRole("button", { name: "Pause", exact: true }).click();
      const played = (await viewing()) - start;
      assert.ok(played > 1.5 * DAY && played < 6 * DAY, `played ${played / DAY} days`);
      assert.equal(await linked(), "yes");
      const before = await readout();

      // The campaign's now is kept across a reload.
      await page.waitForTimeout(900);
      await page.reload();
      await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
      await page.getByLabel("Show", { exact: true }).selectOption("week");
      assert.equal(await readout(), before);
      assert.equal(await linked(), "yes");

      assert.match(await ruler.locator("[data-note]").innerText(), /orbits by the day/);
      // Fits the screen.
      const over = await ruler.evaluate((el) => el.scrollWidth - el.clientWidth);
      assert.ok(over <= 1, `ruler overflows by ${over}px`);
      await shot("4-kept");
      await page.context().close();
    }
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`time ${name}: ok`);
  }
} finally {
  server.kill();
}
