// The star chooser (ED6 part 2), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/star-chooser.e2e.mjs [--screens <dir>]
// Open the Solar System, pick the Sun, open its Mass handle: the strip is painted in star colours with letters and the
// walls, the life bar marks the Sun's age, and the Kinds offer what can exist at 4.57 billion years (no blue giant).
// On Build, Red giant is one change: the card says what was set, the Sun's light changes, and Undo puts it back. On
// Explore, a kind is a try (TM4): the what-if opens by itself and the main line does not move.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    const sun = gm.locator('.area-chart .body[aria-label^="Sun"]').first();
    await sun.waitFor({ timeout: 30000 });
    await sun.locator(".hit").click();
    const row = gm.locator('[data-handles="star"]');
    await row.waitFor({ timeout: 20000 });
    await desk(gm, "Build");
    await row.locator('[data-handle="mass"]').click();
    const chooser = gm.locator("[data-star-chooser]");
    await chooser.waitFor({ timeout: 20000 });
    await chooser.locator('[data-star-kind="red-giant"]').waitFor({ timeout: 30000 });
    // The strip: painted, the Sun's mark near one solar mass, the walls said in the units engine's words.
    assert.match(await chooser.locator("[data-star-track]").getAttribute("style"), /linear-gradient/);
    assert.match(await chooser.locator("[data-star-mass]").innerText(), /^(about )?(0\.99|1|1\.00) (M☉|Sun mass)$/);
    assert.match(await chooser.locator("[data-star-walls]").innerText(), /Hydrogen fusion from (about )?0\.08 (M☉|Sun masses)/);
    await gm.waitForFunction(() => /main sequence/.test(document.querySelector("[data-star-now]")?.textContent ?? ""), null, { timeout: 20000 });
    assert.equal(await chooser.locator("[data-star-life] .phase.here").count(), 1, "the present age falls in one phase");
    // Kinds at 4.57 billion years: no blue giant lives that long; a red giant and a white dwarf can.
    await gm.waitForFunction(() => document.querySelectorAll("[data-star-kind]").length >= 8, null, { timeout: 30000 });
    assert.ok(await chooser.locator('[data-star-kind="blue-giant"]').isDisabled(), "no blue giant at this age");
    assert.ok(!(await chooser.locator('[data-star-kind="red-giant"]').isDisabled()));
    assert.ok(!(await chooser.locator('[data-star-kind="white-dwarf"]').isDisabled()));
    await shot(gm, `star-chooser-${vp}`);

    // Build: Red giant is one change; Undo puts the Sun back.
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await chooser.locator('[data-star-kind="red-giant"]').click();
    const followed = gm.locator("[data-followed]");
    await followed.waitFor({ timeout: 20000 });
    assert.match(await followed.innerText(), /Star\s+(about )?1 (M☉|Sun mass) → Red giant: born at (about )?[\d.]+ (M☉|Sun masses), giant/);
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash0);
    await gm.waitForFunction(() => /giant/.test(document.querySelector("[data-star-now]")?.textContent ?? ""), null, { timeout: 20000 });
    await shot(gm, `star-red-giant-${vp}`);
    await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
    await gm.keyboard.press("Control+z");
    await gm.waitForFunction(() => /main sequence/.test(document.querySelector("[data-star-now]")?.textContent ?? ""), null, { timeout: 20000 });

    // Explore: a kind is a try. The what-if opens by itself; the main line does not move; Throw away.
    await desk(gm, "Explore");
    if (!(await chooser.isVisible())) await row.locator('[data-handle="mass"]').click();
    await chooser.locator('[data-star-kind="white-dwarf"]').waitFor({ timeout: 30000 });
    await gm.waitForFunction(() => !document.querySelector('[data-star-kind="white-dwarf"]')?.hasAttribute("disabled"), null, { timeout: 30000 });
    const main = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await chooser.locator('[data-star-kind="white-dwarf"]').click();
    await gm.waitForFunction(() => /Trying it/.test(document.querySelector(".whatif b")?.textContent ?? ""), null, { timeout: 20000 });
    await gm.waitForFunction(() => /white dwarf/.test(document.querySelector("[data-star-now]")?.textContent ?? ""), null, { timeout: 20000 });
    // The change has landed on the what-if when the card says what was set (the what-if's own resolve takes a while).
    await gm.waitForFunction(() => /White dwarf: born at/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 60000 });
    assert.match(await gm.locator("[data-followed]").innerText(), /On Trying it only/);
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), main, "the main line does not move");
    await shot(gm, `star-explore-try-${vp}`);
    await gm.locator(".whatif button", { hasText: "Throw away" }).click();
    await gm.waitForFunction(() => /main sequence/.test(document.querySelector("[data-star-now]")?.textContent ?? ""), null, { timeout: 20000 });

    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      assert.ok(over <= 1, `nothing runs off the side on a phone (${over}px)`);
    }
    assert.deepEqual(errors, [], "no page errors");
    await browser.close();
  }
  console.log("star chooser e2e: ok");
} finally {
  server.kill();
}
