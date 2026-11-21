// U4 (what-if branches) acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/whatif.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Open a what-if, change Tarn's mass on the card: the chart and card show the
// branch, the main state hash does not move, the branch has its own hash.
// Compare lists the mass (set) and the gravity (follows). Throw it away: the
// main line is as it was. Again, and keep it: the main line takes the edit.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8700 + Math.floor(Math.random() * 200);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    const browser = await engine.launch();
    const errors = [];
    const gm = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await desk(gm, "Build");
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const field = (label) => gm.locator(".area-card .field", { has: gm.locator(`label:text-is("${label}")`) });
    const value = async (label) => {
      const f = field(label);
      return (await f.locator("input").count()) ? f.locator("input").inputValue() : f.locator("output").innerText();
    };
    await gm.locator(".area-chart [role=button]", { hasText: "Tarn" }).first().click();
    const h0 = await hash();
    const mass0 = await value("Mass");
    const grav0 = await value("Gravity");

    // Open a what-if and change the mass.
    await gm.getByRole("button", { name: "What if…" }).click();
    await gm.waitForSelector("main.stage[data-branch=open]");
    // The chart re-derives onto the branch as it opens; let that settle before typing.
    await gm.waitForSelector("[data-branch-hash]");
    await gm.waitForTimeout(500);
    const m = field("Mass").locator("input");
    await m.fill("1.6");
    await m.press("Enter");
    // Leaving the field commits too (one commit: the field ignores a second one with nothing new).
    await m.evaluate((el) => el.blur());
    await gm.waitForSelector("[data-whatif-edits='1']");
    await gm.waitForFunction((g) => {
      const f = [...document.querySelectorAll(".area-card .field")].find((x) => x.querySelector("label")?.textContent?.startsWith("Gravity"));
      return f && (f.querySelector("output")?.textContent ?? f.querySelector("input")?.value) !== g;
    }, grav0);
    assert.equal(await hash(), h0, "the main line's state hash does not move");
    const hb = await gm.locator("[data-branch-hash]").getAttribute("data-branch-hash");
    assert.ok(hb && hb !== h0, "the what-if has its own state hash");
    await shot(gm, "u4-whatif-open");

    // Compare: control against changed.
    await gm.getByRole("button", { name: "Compare" }).click();
    const dlg = gm.getByRole("dialog", { name: "Compare" });
    await dlg.waitFor();
    assert.match(await dlg.locator("tr[data-diff=mass_earth]").innerText(), /set/);
    assert.match(await dlg.locator("tr[data-diff=gravity_g]").innerText(), /follows/);
    await shot(gm, "u4-compare");
    await dlg.getByRole("button", { name: "Throw away" }).click();
    await gm.waitForFunction(() => !document.querySelector("main.stage[data-branch]"));
    await gm.waitForFunction((v) => {
      const f = [...document.querySelectorAll(".area-card .field")].find((x) => x.querySelector("label")?.textContent?.startsWith("Mass"));
      return f?.querySelector("input")?.value === v;
    }, mass0);
    assert.equal(await hash(), h0, "thrown away: the main line is as it was");

    // Again, and keep it.
    await gm.getByRole("button", { name: "What if…" }).click();
    await gm.waitForSelector("main.stage[data-branch=open]");
    await gm.waitForSelector("[data-branch-hash]");
    await gm.waitForTimeout(500);
    await field("Mass").locator("input").fill("1.6");
    await field("Mass").locator("input").press("Enter");
    await field("Mass").locator("input").evaluate((el) => el.blur());
    await gm.waitForSelector("[data-whatif-edits='1']");
    await gm.getByRole("button", { name: "Keep it" }).click();
    await gm.waitForFunction(() => !document.querySelector("main.stage[data-branch]"));
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, h0);
    assert.match(await gm.locator(".whatif [role=status]").innerText(), /Kept: 1 change/);
    assert.match(await value("Mass"), /1\.6/);
    assert.ok(await gm.getByRole("button", { name: "Undo" }).isEnabled(), "a kept edit can be undone");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: what-if kept the main hash ${h0.slice(0, 10)} until kept; compare lists set and follows`);
  }
} finally {
  server.kill();
}
