// U11 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/time.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Human-scale time on the ruler: it opens at the campaign's now, on Explore in human time with "Whole life ›" beside
// it (top-level 12.1); Show "A week" zooms to days; stepping forward while at now moves the campaign
// (time passes); the track itself only looks, leaving the campaign where it
// was; Back to now returns; Make this now moves it; Play advances; the
// campaign's now survives a reload. On a phone the ruler fits its width.
// On the Run desk the clock takes the ruler's place (three desks design 7.3): it shows the same now, says what is due
// (nothing until the kernel's happenings op), and "Jump to a date…" says where it lands before it moves the campaign.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk } from "./nav.mjs";

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
      await page.goto(base + "#chart");
      await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });

      // Opens at the campaign's now. On Explore at a system that is human time, with "Whole life ›" beside it (top-level
      // design 12.1); Whole life › shows the deep time.
      assert.equal(await linked(), "yes");
      assert.match(await ruler.innerText(), /Now\s*in the campaign/i);
      assert.equal(await readout(), "Year 1, day 1, 00:00");
      const opened = Number(await ruler.getAttribute("data-span"));
      assert.ok(Math.abs(opened - 7 * DAY) < 1e-6, `opens on a week, not ${opened} years`);
      const start = await viewing();
      await shot("1-open");
      await ruler.locator("[data-whole-life]").click();
      await page.waitForFunction(() => /Gyr/.test(document.querySelector(".area-ruler .ruler .now")?.textContent ?? ""));
      assert.equal(await ruler.locator("[data-whole-life]").count(), 0, "no Whole life › on the whole life");

      // A week on screen: dates and hours.
      await page.getByLabel("Show", { exact: true }).selectOption("week");
      const span = Number(await ruler.getAttribute("data-span"));
      assert.ok(Math.abs(span - 7 * DAY) < 1e-6, `span ${span}`);
      assert.equal(await readout(), "Year 1, day 1, 00:00");

      // Stepping at now: time passes in the campaign.
      // Worlds move along their orbits as the days pass (the kernel's orbits_at): Tarn, a 151-day year, moves about 5 degrees in two days.
      const tarnAt = () => page.locator('.body[aria-label^="Tarn"] .hit').first().evaluate((c) => [Number(c.getAttribute("cx")), Number(c.getAttribute("cy"))]);
      const [x0, y0] = await tarnAt();
      await page.getByRole("button", { name: "Forward one day" }).click();
      await page.getByRole("button", { name: "Forward one day" }).click();
      await page.waitForFunction(([x, y]) => {
        const c = document.querySelector('.body[aria-label^="Tarn"] .hit');
        return c && Math.hypot(Number(c.getAttribute("cx")) - x, Number(c.getAttribute("cy")) - y) > 2;
      }, [x0, y0], { timeout: 5000 });
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

      // Fits the screen.
      const over = await ruler.evaluate((el) => el.scrollWidth - el.clientWidth);
      assert.ok(over <= 1, `ruler overflows by ${over}px`);
      await shot("4-kept");

      // The Run desk: the clock in the ruler's place, at the same now.
      await desk(page, "Run");
      const clock = page.locator("[data-campaign-clock]");
      await clock.waitFor();
      assert.equal(await ruler.count(), 0, "no ruler on Run");
      const clockDate = () => clock.locator("[data-clock-date]").innerText();
      assert.equal(await clockDate(), before);
      // On a phone the clock is one row (three desks design section 8): the date, one step and "…" for the rest.
      const more = clock.locator("[data-clock-more]");
      assert.equal(await more.isVisible(), vpName === "phone", "… only on a phone");
      if (vpName === "phone") {
        assert.deepEqual(await clock.locator("[data-clock-step]").evaluateAll((bs) => bs.filter((b) => b.checkVisibility()).length), 1, "one step chip");
        assert.ok(!(await clock.locator("[data-clock-jump]").isVisible()), "Jump waits under …");
        const note = page.locator("[data-kept-note]");
        if (await note.isVisible()) await note.getByRole("button", { name: "OK" }).click();
        await more.click();
      }
      assert.equal(await clock.locator("[data-due-none]").innerText(), "Nothing due");
      await shot("5-run-clock");

      // Jump to a date…: Cancel moves nothing; a jump of 2 weeks lands 14 days on, at the same hour.
      // After the reload the "Reopened the campaign" note sits over the clock on a phone: put it away first.
      const kept = page.locator("[data-kept-note]");
      if (await kept.isVisible()) await kept.getByRole("button", { name: "OK" }).click();
      const dialog = page.locator("[data-jump-dialog]");
      await clock.locator("[data-clock-jump]").click();
      await dialog.waitFor({ state: "visible" });
      await dialog.getByRole("button", { name: "Cancel" }).click();
      await dialog.waitFor({ state: "hidden" });
      assert.equal(await clockDate(), before);
      await clock.locator("[data-clock-jump]").click();
      await dialog.locator("[data-jump-n]").fill("2");
      await dialog.locator("[data-jump-unit]").selectOption("week");
      // The day is exact; the minute may round either way while the clock is years in a double (time scales audit,
      // finding 3; TP3 moves the clock to whole seconds and this can then match the minute too).
      const [, day] = before.match(/day (\d+), /);
      const lands = (await dialog.locator("[data-jump-lands]").innerText()).match(/^Lands on (Year 1, day (\d+), \d\d:\d\d), 14 days from now\. Players' screens follow\.$/);
      assert.ok(lands, "the dialog says where the jump lands");
      assert.equal(Number(lands[2]), Number(day) + 14);
      const after = lands[1];
      assert.equal(await dialog.locator("[data-jump-passes]").innerText(), "Nothing is marked on the way.");
      await shot("6-jump");
      await dialog.locator("[data-jump-go]").click();
      await dialog.waitFor({ state: "hidden" });
      assert.equal(await clockDate(), after);
      const clockOver = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      assert.ok(clockOver <= 1, `the page overflows by ${clockOver}px on Run`);
      await page.context().close();
    }
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`time ${name}: ok`);
  }
} finally {
  server.kill();
}
