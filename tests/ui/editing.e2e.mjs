// The handles row and the strip (ED1, editing design sections 1 to 3), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/editing.e2e.mjs [--screens <dir>]
// Open the Solar System, pick Mars. On Explore its card shows the six handles with the kernel's figures, and the
// Orbit strip opens read-only with the reference worlds. On Build, typing "1 AU" in the strip is one change: the
// handle says 1 AU, the card lists what followed (Mars warms), the state hash moves, and Undo puts it back. Dragging
// the mark and letting go is one change too; Esc while dragging writes nothing. On a phone nothing runs off the side.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk } from "./nav.mjs";
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
