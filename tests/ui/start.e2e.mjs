// The start sheet (top-level design 3.1 and 16, U21 part 1), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/start.e2e.mjs [--screens <dir>]
// A fresh browser opens on the sheet, with the first expected line, the doors and the "What it can do" verbs (those
// naming features not built are held back). Escape and "Go to the chart" leave it; More › Start sheet brings it
// back with another line. "Grow a new system" opens Make on its grow tab; a verb opens the real screen; "Open
// straight to the chart next time" is remembered; "#chart" skips it. On a phone nothing runs off the side.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { more, pinClock } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9800 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(700);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = pinClock(await chromium.launch());
    const context = await browser.newContext({ viewport });
    const gm = await context.newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    const sheet = gm.locator("[data-start]");
    await sheet.waitFor();
    assert.equal(await gm.locator("[data-start-line]").getAttribute("data-start-line"), "l1", "the first visit shows the first expected line");
    assert.match(await sheet.locator("h1").innerText(), /campaign, on real physics/);
    for (const d of ["sol", "carry", "grow", "bring", "chart"]) assert.equal(await sheet.locator(`[data-door="${d}"]`).count(), 1, `door ${d}`);
    assert.match(await sheet.locator('[data-door="carry"]').innerText(), /sample campaign/);
    const verbs = await sheet.locator(".atlas li").count();
    assert.equal(verbs, 10, "ten verbs, Age the Sun among them now the Solar System is in");
    assert.match(await sheet.locator('[data-door="sol"]').innerText(), /Open the Solar System\s+Start here/i);
    await gm.waitForSelector('[data-door="grow"]:not([disabled])', { timeout: 20000 });
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
    }
    await shot(gm, `start-${vp}`);

    // Escape leaves for the chart; More brings the sheet back with the next expected line.
    await gm.keyboard.press("Escape");
    await sheet.waitFor({ state: "detached" });
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    await more(gm, "Start sheet");
    await sheet.waitFor();
    assert.equal(await gm.locator("[data-start-line]").getAttribute("data-start-line"), "l2", "the second visit: the second line, which opens the Solar System");

    // A verb opens the real thing: Cut a world open goes to the Cutaway of a world.
    await sheet.getByRole("button", { name: /Cut a world open/ }).click();
    await sheet.waitFor({ state: "detached" });
    await gm.waitForFunction(() => document.querySelector(".stage")?.getAttribute("data-screen") === "cutaway");

    // Open the Solar System: entered once from its catalogue, then on the chart with the Moon round Earth.
    await more(gm, "Start sheet");
    await sheet.locator('[data-door="sol"]').click();
    await sheet.waitFor({ state: "detached" });
    await gm.waitForFunction(() => [...document.querySelectorAll(".area-chart .body")].some((b) => /^Earth\b/.test(b.getAttribute("aria-label") ?? "")), null, { timeout: 20000 });
    const solHash = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    for (const n of ["Sun", "Mercury", "Jupiter", "Neptune", "Kuiper belt"]) assert.ok(await gm.locator(`.area-chart .body[aria-label^="${n}"]`).count() >= 1 || (await gm.locator(".area-chart").innerText()).toLowerCase().includes(n.toLowerCase()), `${n} on the chart`);
    assert.equal(await gm.locator("[data-catalogued]").count(), 1, "marked as measured, not as a V3 save");
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px on the Solar System`);
    }
    await shot(gm, `start-sol-${vp}`);
    // A second open goes straight there, writing nothing.
    await more(gm, "Start sheet");
    await sheet.getByRole("button", { name: /Age the Sun/ }).click();
    await sheet.waitFor({ state: "detached" });
    await gm.waitForTimeout(600);
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), solHash, "entered once");

    // Grow a new system opens Make on its grow tab.
    await more(gm, "Start sheet");
    await sheet.locator('[data-door="grow"]').click();
    const make = gm.getByRole("dialog", { name: "Make a system" });
    await make.waitFor();
    assert.equal(await make.getByRole("tab", { name: "Grow a new system" }).getAttribute("aria-selected"), "true");
    await make.getByRole("button", { name: "Close" }).last().click();

    // Skipping is remembered on this device.
    await more(gm, "Start sheet");
    await sheet.getByLabel("Open straight to the chart next time").check();
    await sheet.locator('[data-door="chart"]').click();
    await gm.reload();
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    assert.equal(await sheet.count(), 0, "skipped once asked");

    // "#chart" skips it on a fresh device too.
    const fresh = await (await browser.newContext({ viewport })).newPage();
    await fresh.goto(base + "#chart");
    await fresh.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    assert.equal(await fresh.locator("[data-start]").count(), 0);

    assert.deepEqual(errors, []);
    await browser.close();
  }
  console.log("start e2e: ok");
} finally {
  server.kill();
}
