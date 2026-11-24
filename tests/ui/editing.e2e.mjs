// The handles row and the strip (ED1, editing design sections 1 to 3), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/editing.e2e.mjs [--screens <dir>]
// Open the Solar System, pick Mars. On Explore its card shows the six handles with the kernel's figures, and the
// Orbit strip opens read-only with the reference worlds. On Build, typing "1 AU" in the strip is one change: the
// handle says 1 AU, the card lists what followed (Mars warms), the state hash moves, and Undo puts it back. Dragging
// the mark and letting go is one change too; Esc while dragging writes nothing. On a phone nothing runs off the side.
// ED12, the Orbit and Spin boxes at Science depth: the year and the stability verdict under the orbit; a year typed on
// the size strip is the distance with that year; Retrograde mirrors the lean past 90 degrees; the place now is a ring
// that catches at the far point; Locked sets the day to the year and the sun's day says so; each one change, one Undo.
// ED7: on Explore, "Try changing it" opens an example what-if where the strip moves and the main line does not.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { depth, desk } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9500 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => (errors.push(e.message), console.log("PAGEERROR", e.stack?.slice(0, 600))));
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    const mars = gm.locator('.area-chart .body[aria-label^="Mars,"]').first();
    await mars.waitFor({ timeout: 30000 });
    await mars.locator(".hit").click();

    // Explore: the row with the kernel's figures, the strip read-only.
    const row = gm.locator('[data-handles="world"]');
    await row.waitFor({ timeout: 20000 });
    assert.deepEqual(await row.locator("[data-handle]").evaluateAll((bs) => bs.map((b) => b.dataset.handle)), ["orbit", "mass", "makeup", "liquids", "air", "spin"]);
    await gm.waitForFunction(() => /AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
    assert.match(await row.locator('[data-handle="orbit"] .value').innerText(), /1\.52 AU/);
    assert.match(await row.locator('[data-handle="air"] .value').innerText(), /6\.36 mbar/);
    await row.locator('[data-handle="orbit"]').click();
    const strip = gm.locator('[data-strip="orbit"]');
    await strip.waitFor();
    assert.ok(await strip.locator("[data-strip-figure]").isDisabled(), "read-only on Explore");
    for (const n of ["Earth", "Jupiter"]) assert.equal(await strip.locator(`[data-dot="${n}"]`).count(), 1, `${n} is a reference dot`);
    await shot(gm, `explore-orbit-${vp}`);

    // Press a unit (ED1a): the card's Orbit steps to the next reading and the handle follows, for every planet; round
    // the readings it comes back to AU.
    const cardOrbit = gm.locator("section[aria-label='Mars'] .field").filter({ hasText: /^(orbit|how far)/i }).first();
    const unitButton = cardOrbit.locator("[data-unit]");
    {
      assert.equal(await unitButton.count(), 1, "the card's orbit has a unit to press");
      assert.equal(await unitButton.innerText(), "AU");
      await unitButton.click();
      await gm.waitForFunction(() => !/AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 5000 });
      const other = await cardOrbit.locator("[data-unit]").innerText();
      assert.notEqual(other, "AU");
      assert.ok((await row.locator('[data-handle="orbit"] .value').innerText()).endsWith(other), "the handle says it in the chosen unit too");
      for (let i = 0; i < 5 && (await cardOrbit.locator("[data-unit]").innerText()) !== "AU"; i++) await cardOrbit.locator("[data-unit]").click();
      assert.equal(await cardOrbit.locator("[data-unit]").innerText(), "AU", "round the readings and back");
    }

    // ED7: on Explore the strip is read-only, with "Try changing it": an example what-if where it moves; the main line
    // does not; Throw away puts the world back.
    {
      if (!(await strip.isVisible())) await row.locator('[data-handle="orbit"]').click();
      const main = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      await strip.locator("[data-try]").click();
      await gm.waitForFunction(() => /EXAMPLE/.test(document.querySelector(".whatif b")?.textContent ?? ""), null, { timeout: 20000 });
      assert.ok(!(await strip.locator("[data-strip-figure]").isDisabled()), "the example moves");
      await strip.locator("[data-strip-figure]").click();
      await strip.locator("[data-strip-entry]").fill("1 AU");
      await strip.locator("[data-strip-entry]").press("Enter");
      await gm.waitForFunction(() => /(^|about )1 AU$/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      assert.match(await gm.locator("[data-followed]").innerText(), /EXAMPLE · change anything only/);
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), main, "the main line does not move");
      await shot(gm, `explore-example-${vp}`);
      await gm.locator(".whatif button", { hasText: "Throw away" }).click();
      await gm.waitForFunction(() => /1\.52 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      assert.equal(await strip.locator("[data-try]").count(), 1, "read-only again, with Try changing it");
    }

    // Build: type "1 AU", one change, what followed, Undo.
    await desk(gm, "Build");
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    if (!(await strip.isVisible())) await row.locator('[data-handle="orbit"]').click();
    await strip.locator("[data-strip-figure]").click();
    const entry = strip.locator("[data-strip-entry]");
    await entry.fill("1 AU");
    await entry.press("Enter");
    const followed = gm.locator("[data-followed]");
    await followed.waitFor({ timeout: 20000 }).catch(async (e) => {
      throw new Error(`${e.message}\nstrip says: ${await strip.innerText()}`);
    });
    assert.match(await followed.innerText(), /(Orbit|How far out)\s+(about )?1\.52 AU → 1 AU/);
    assert.match(await followed.locator('[data-followed-row="Surface"]').innerText(), /K → (about )?\d+ K/);
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash0);
    assert.match(await row.locator('[data-handle="orbit"] .value').innerText(), /(^|about )1 AU$/);
    await shot(gm, `build-followed-${vp}`);
    await gm.keyboard.press("Control+z");
    await gm.waitForFunction(() => /1\.52 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });

    // Drag the mark and let go: one change. Esc during a drag writes nothing.
    if (!(await strip.isVisible())) await row.locator('[data-handle="orbit"]').click();
    const track = strip.locator(".track");
    await track.scrollIntoViewIfNeeded();
    const box = await track.boundingBox();
    const hash1 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await gm.mouse.move(box.x + box.width * 0.5, box.y + 15);
    await gm.mouse.down();
    await gm.mouse.move(box.x + box.width * 0.7, box.y + 15, { steps: 5 });
    assert.ok(await strip.locator(".figure.pencil").count(), "the figure is in pencil while dragging");
    await gm.mouse.up();
    await followed.waitFor({ timeout: 20000 });
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash1);
    const hash2 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await strip.locator("[data-strip-mark]").focus();
    await gm.keyboard.press("ArrowRight");
    await gm.keyboard.press("Escape");
    await gm.waitForTimeout(400);
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash2, "Esc writes nothing");
    assert.equal(await row.count(), 1, "the strip's Esc leaves the world selected");

    // ED3 part 1, the chart: dragging a world shows a ghost and the distance as the engine says it; the world stays
    // put; Esc puts it back and writes nothing.
    if (vp === "desk") {
      await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
      await gm.keyboard.press("Control+z");
      await gm.waitForFunction(() => /1\.52 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      await gm.waitForTimeout(1000); // the chart redraws Mars back at 1.52 AU after the card
      const hit = gm.locator('.area-chart .body[aria-label^="Mars,"] .hit').first();
      const sun = gm.locator('.area-chart .body[aria-label^="Sun"] .hit').first();
      const mb = await hit.boundingBox();
      const sb = await sun.boundingBox();
      const [mx, my, sx, sy] = [mb.x + mb.width / 2, mb.y + mb.height / 2, sb.x + sb.width / 2, sb.y + sb.height / 2];
      const hash3 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      await gm.mouse.move(mx, my);
      await gm.mouse.down();
      for (let k = 1; k <= 6; k++) await gm.mouse.move(mx + ((mx - sx) * 0.3 * k) / 6, my + ((my - sy) * 0.3 * k) / 6);
      await gm.locator("[data-ghost]").waitFor({ timeout: 5000 });
      assert.match(await gm.locator("[data-drag-readout]").textContent(), /AU$/);
      const still = await hit.boundingBox();
      assert.ok(Math.abs(still.x - mb.x) < 1 && Math.abs(still.y - mb.y) < 1, `the world stays put while the ghost moves: ${JSON.stringify([mb, still])}`);
      await shot(gm, `chart-drag-${vp}`);
      await gm.keyboard.press("Escape");
      await gm.mouse.up();
      await gm.waitForTimeout(400);
      assert.equal(await gm.locator("[data-ghost]").count(), 0, "Esc takes the ghost away");
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash3, "Esc on the chart writes nothing");
    }

    // ED12: the boxes, at Science depth on the Build desk.
    if (vp === "desk") {
      await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
      await depth(gm, "Science");
      const hash = () => gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      // Undo is a change of its own (the hash moves on); the figure it names comes back.
      const undoTo = async (back) => {
        const h = await hash();
        await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
        await gm.keyboard.press("Control+z");
        await gm.waitForFunction((x) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== x, h, { timeout: 20000 });
        if (back) await gm.waitForFunction(([sel, re]) => new RegExp(re).test(document.querySelector(sel)?.textContent ?? ""), back, { timeout: 20000 });
        assert.equal(await gm.getByText("Nothing to undo").count(), 0, "each Undo takes back the box's own change");
      };
      const marsAt = ['[data-handle="orbit"] .value', "1\\.52 AU"];
      if (!(await gm.locator('[data-box="orbit"]').isVisible())) await row.locator('[data-handle="orbit"]').click();
      const obox = gm.locator('[data-box="orbit"]');
      await obox.waitFor();
      for (const p of ["orbit.shape", "orbit.lean"]) assert.equal(await obox.locator(`[data-strip="${p}"]`).count(), 1, `${p} strip`);
      for (const r of ["now", "peri", "node"]) assert.equal(await obox.locator(`[data-ring="${r}"]`).count(), 1, `${r} ring`);
      await obox.locator('[data-result="year"]').waitFor({ timeout: 20000 });
      assert.match(await obox.locator('[data-result="year"]').innerText(), /68\d (d|days)|1\.88 (years|yr)/);
      assert.match(await obox.locator('[data-result="stability"]').innerText(), /Lasts|Watch/);
      assert.match(await obox.locator('[data-strip="orbit.shape"] [data-strip-figure]').innerText(), /^0\.09/);
      await shot(gm, `orbit-box-${vp}`);

      // A year typed on the size: one Earth year puts Mars at 1 AU.
      await obox.locator('[data-strip="orbit"] [data-strip-figure]').click();
      await obox.locator('[data-strip="orbit"] [data-strip-entry]').fill("365.25 d");
      await obox.locator('[data-strip="orbit"] [data-strip-entry]').press("Enter");
      await gm.waitForFunction(() => /(^|about )1(\.00)? AU$/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nbox says: ${await obox.innerText()}`);
      });
      await undoTo(marsAt);

      // Retrograde: the lean goes past 90 degrees, and the chip says so.
      await obox.locator('[data-chip="retrograde"]').click();
      await gm.waitForFunction(() => document.querySelector('[data-box="orbit"] [data-chip="retrograde"]')?.getAttribute("aria-pressed") === "true", null, { timeout: 20000 });
      assert.match(await obox.locator('[data-strip="orbit.lean"] [data-strip-figure]').innerText(), /18\d|17\d/);
      await gm.waitForFunction(() => /Lean/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo(['[data-box="orbit"] [data-chip="retrograde"]', "^Retrograde$"]);
      await gm.waitForFunction(() => document.querySelector('[data-box="orbit"] [data-chip="retrograde"]')?.getAttribute("aria-pressed") === "false");

      // Where it is now: typed 180 catches at the farthest point.
      await obox.locator('[data-ring="now"] [data-ring-figure]').click();
      await obox.locator('[data-ring="now"] [data-ring-entry]').fill("180");
      await obox.locator('[data-ring="now"] [data-ring-entry]').press("Enter");
      await gm.waitForFunction(() => /farthest/.test(document.querySelector('[data-ring="now"] [data-ring-caught]')?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nring says: ${await obox.locator('[data-ring="now"]').innerText()}`);
      });
      await gm.waitForFunction(() => /Where it is now 180° from its closest point/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo();

      // Spin: Locked sets the day to the year; the sun's day says the same face looks at the star.
      await row.locator('[data-handle="spin"]').click();
      const sbox = gm.locator('[data-box="spin"]');
      await sbox.waitFor();
      assert.equal(await sbox.locator('[data-strip="spin.tilt"]').count(), 1);
      await gm.waitForFunction(() => /h|min/.test(document.querySelector('[data-box="spin"] [data-result="solar_day"]')?.textContent ?? ""), null, { timeout: 20000 });
      await sbox.locator('[data-chip="locked"]').click();
      await gm.locator("[data-chip-said]").waitFor({ timeout: 20000 });
      await gm.waitForFunction(() => /same face/.test(document.querySelector('[data-box="spin"] [data-result="solar_day"]')?.textContent ?? ""), null, { timeout: 20000 });
      assert.match(await sbox.locator("[data-chip-said]").innerText(), /year/);
      await shot(gm, `spin-box-locked-${vp}`);
      await undoTo(['[data-handle="spin"] .value', "^24\\.6 h"]);
      // Backwards: the change says so in words.
      await sbox.locator('[data-chip="retrograde"]').click();
      await gm.locator("[data-followed]").waitFor({ timeout: 20000 });
      await gm.waitForFunction(() => /backwards/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo();
      await depth(gm, "Curious");
    }

    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
    }
    assert.deepEqual(errors, [], `page errors: ${errors.join("; ")}`);
    await browser.close();
    console.log(`editing ${vp}: ok`);
  }
} finally {
  server.kill();
}
