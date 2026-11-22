// Tactility pass (dynamic minimalism, handover/interface-style.md), in a real
// browser against the built shell. Usage: node tests/ui/tactile.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Resting the mouse on a body peeks its name and kind; the card for a newly
// chosen body opens from it; a dialog opens from the button that opened it;
// the ruler glides to a new span while its readout answers at once. With
// reduced motion asked for, nothing animates and everything still works.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk, go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9800 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

// Animations with real length on an element (CSS or script), right now.
const moving = (loc) => loc.evaluate((el) => el.getAnimations({ subtree: true }).filter((a) => a.playState === "running" && Number(a.effect?.getTiming().duration) > 0).length);

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    for (const reduced of [false, true]) {
      const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, reducedMotion: reduced ? "reduce" : "no-preference" });
      const page = await context.newPage();
      page.on("pageerror", (e) => errors.push(e.message));
      const shot = async (n) => {
        if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/tactile-${n}.png` }));
      };
      await page.goto(base + "#chart");
      await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });

      // Peek on hover.
      const tarn = page.locator('.body[aria-label^="Tarn"]').first();
      await tarn.hover();
      const peek = page.locator("[data-peek]");
      await peek.waitFor();
      assert.match(await peek.innerText(), /Tarn/);
      if (!reduced) await shot("1-peek");
      await page.mouse.move(5, 300);
      await peek.waitFor({ state: "detached" });

      // The card opens from the body chosen.
      await tarn.click();
      const card = page.locator(".area-card");
      await page.waitForSelector("section[aria-label='Tarn']");
      const cardMoving = await moving(card);
      if (!reduced) await shot("2-card-opening");
      assert.equal(cardMoving > 0, !reduced, `card motion ${cardMoving} with reduced=${reduced}`);
      // No peek for the body the card already describes.
      await tarn.hover();
      await page.waitForTimeout(100);
      assert.equal(await peek.count(), 0);

      // A dialog opens from its button.
      await more(page, "Campaign…");
      const dialog = page.getByRole("dialog", { name: "Campaign" });
      await dialog.waitFor();
      const dialogMoving = await moving(dialog);
      if (!reduced) await shot("3-dialog-opening");
      assert.equal(dialogMoving > 0, !reduced, `dialog motion ${dialogMoving} with reduced=${reduced}`);
      await page.keyboard.press("Escape");

      // Entering a planet's moons grows the chart out of the planet; leaving shrinks back.
      await page.locator('.body[aria-label^="Vault"]').first().click();
      await page.getByRole("button", { name: "Moons of Vault ›" }).click();
      const chartSvg = page.locator(".area-chart svg[role=group]").first();
      const frameMoving = await moving(chartSvg);
      if (!reduced) await shot("5-moons-opening");
      assert.equal(frameMoving > 0, !reduced, `frame motion ${frameMoving} with reduced=${reduced}`);
      await page.locator('[role=button][aria-label^="Hasp"]').first().waitFor();
      await page.getByRole("button", { name: /‹/ }).first().click();
      await page.locator('.body[aria-label^="Tarn"]').first().waitFor();

      // The ruler glides, but its readout answers at once.
      const ruler = page.locator(".area-ruler .ruler");
      await page.getByLabel("Show", { exact: true }).selectOption("year");
      assert.equal(await ruler.locator(".now").innerText(), "Year 1, day 1");
      if (!reduced) {
        await page.waitForTimeout(60);
        await shot("4-ruler-gliding");
      }
      await page.waitForTimeout(400);
      assert.match(await ruler.locator(".tl").first().innerText(), /Year|day/);

      // Switches at rest (the desk, the depth): every option's first letter is uncovered, and the chosen one carries
      // its own ink. A trailer capture once showed "uild" for Build, the sliding ink measured too wide.
      const switchesClear = (when) =>
        page.evaluate((when) => {
          const bad = [];
          for (const seg of document.querySelectorAll(".seg")) {
            if (!seg.getClientRects().length) continue;
            const ink = seg.querySelector("[data-ink]");
            if (ink && getComputedStyle(ink).opacity !== "0") bad.push(`${seg.getAttribute("aria-label")}: sliding ink still showing at rest`);
            const ir = ink && getComputedStyle(ink).opacity !== "0" ? ink.getBoundingClientRect() : null;
            for (const b of seg.querySelectorAll("button")) {
              const r = b.getBoundingClientRect();
              const range = document.createRange();
              range.selectNodeContents(b);
              const t = range.getBoundingClientRect();
              if (t.left < r.left - 0.5 || t.right > r.right + 0.5) bad.push(`${b.textContent}: text runs outside its button`);
              const hit = document.elementFromPoint(t.left + 1.5, t.top + t.height / 2);
              if (hit !== b) bad.push(`${b.textContent}: its first letter is covered (${hit?.className || hit?.tagName})`);
              if (ir && b.getAttribute("aria-pressed") !== "true" && ir.right > t.left + 0.5 && ir.left < t.right - 0.5) bad.push(`${b.textContent}: the ink lies over its letters`);
              if (b.getAttribute("aria-pressed") === "true" && getComputedStyle(b).backgroundColor === "rgba(0, 0, 0, 0)") bad.push(`${b.textContent}: chosen but not inked`);
            }
          }
          // Nothing in the bar overlaps anything else in it (a long place name once ran under the kicker).
          const items = [...document.querySelectorAll("header.bar > *, header.bar .place > *")].filter((e) => e.getClientRects().length && getComputedStyle(e).visibility !== "hidden");
          for (const x of items)
            for (const y of items) {
              if (x === y || x.contains(y) || y.contains(x) || x.compareDocumentPosition(y) & Node.DOCUMENT_POSITION_PRECEDING) continue;
              const a = x.getBoundingClientRect();
              const c = y.getBoundingClientRect();
              if (a.width && c.width && a.right > c.left + 1 && c.right > a.left + 1 && a.bottom > c.top + 1 && c.bottom > a.top + 1) bad.push(`bar: ${x.className || x.tagName} overlaps ${y.className || y.tagName}`);
            }
          return bad.map((x) => `${when}: ${x}`);
        }, when);
      for (const width of [1920, 1536, 1440, 1280, 1100, 390]) {
        await page.setViewportSize({ width, height: 900 });
        await page.waitForTimeout(100);
        assert.deepEqual(await switchesClear(`${width} px`), []);
      }
      await page.setViewportSize({ width: 1440, height: 900 });
      for (const d of ["Build", "Run", "Explore"]) {
        await desk(page, d);
        await page.waitForTimeout(500);
        assert.deepEqual(await switchesClear(`after ${d}`), []);
      }
      await context.close();
    }
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`tactile ${name}: ok`);
  }
} finally {
  server.kill();
}
