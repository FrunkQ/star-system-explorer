// The standard view controls (interface style rule 12), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/view-controls.e2e.mjs [--screens <dir>]
// Every diagram shows the same two switches at the top of the stage: View style (Chart, 3D, True, Wireframe) and Seen
// from (Above, Tilted, Edge-on, Free). What a view cannot do stays listed, disabled, with its reason. On the system
// chart, Tilted turns the flat chart into 3D; dragging switches to Free and the hint says how Free is worked; the
// choice is kept on the device. The galaxy chart follows it in 3D; Galaxies nearby, flat for now, shows Chart seen
// from above with the rest disabled.
// On a phone the controls sit over the chart with nothing off the side. Choosing writes nothing.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: false }));
};
const ready = (page) => page.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 20000 });

try {
  const browser = await chromium.launch();
  const errors = [];
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const gm = await ctx.newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(root + "#chart");
  await ready(gm);
  const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  const vc = gm.locator(".viewbar [data-view-controls]");
  const style = vc.getByRole("group", { name: "View style" });
  const seen = vc.getByRole("group", { name: "Seen from" });
  assert.deepEqual(await style.locator("button").allInnerTexts(), ["Chart", "3D", "True", "Wireframe"]);
  assert.deepEqual(await seen.locator("button").allInnerTexts(), ["Above", "Tilted", "Edge-on", "Free"]);
  assert.equal(await vc.getAttribute("data-style"), "chart");
  assert.equal(await vc.getAttribute("data-seen"), "above", "the flat chart is seen from above");
  // True is drawn on the system chart (V10a); Wireframe is still to come.
  assert.ok(!(await style.getByRole("button", { name: "True" }).isDisabled()));
  assert.ok(await style.getByRole("button", { name: "Wireframe" }).isDisabled());
  assert.match(await style.getByRole("button", { name: "Wireframe" }).getAttribute("title"), /coming/);
  // HUD and Band, changed far less, fold into one chip that says what they are; it opens both switches beside it.
  const chip = vc.locator("[data-view-more]");
  assert.equal(await vc.getByRole("group", { name: "Band" }).count(), 0, "Band is folded away until asked for");
  assert.match(await chip.innerText(), /^HUD on · Visible/);
  await chip.click();
  const band = vc.getByRole("group", { name: "Band" });
  assert.deepEqual(await band.locator("button").allInnerTexts(), ["Visible", "Infrared", "Ultraviolet", "X-ray", "Radio", "Mag", "Composite"]);
  assert.equal(await vc.getAttribute("data-band"), "visible");
  assert.equal(await vc.getByRole("group", { name: "HUD" }).count(), 1);
  await shot(gm, "rule12-show-open");
  await gm.keyboard.press("Escape");
  await band.waitFor({ state: "detached" });
  assert.equal(await chip.evaluate((e) => e === document.activeElement), true, "Escape gives the keyboard back to the chip");
  // The bar is one row on a wide screen.
  const rows = new Set(await vc.locator(":scope > .field").evaluateAll((fs) => fs.map((f) => Math.round(f.getBoundingClientRect().bottom / 8))));
  assert.equal(rows.size, 1, "one row on a desktop");
  await shot(gm, "rule12-system-chart");

  // Tilted turns the flat chart into 3D; dragging switches to Free and the hint appears.
  await seen.getByRole("button", { name: "Tilted" }).click();
  await gm.locator(".s3d").waitFor();
  assert.equal(await vc.getAttribute("data-style"), "3d");
  assert.equal(await gm.locator(".s3d").getAttribute("data-seen"), "tilted");
  assert.equal(await vc.locator("[data-free-hint]").count(), 0);
  const box = await gm.locator(".s3d").boundingBox();
  // Away from the star and the bodies' buttons in the middle.
  await gm.mouse.move(box.x + 140, box.y + box.height - 160);
  await gm.mouse.down();
  await gm.mouse.move(box.x + 200, box.y + box.height - 130, { steps: 6 });
  await gm.mouse.up();
  await gm.waitForFunction(() => document.querySelector(".viewbar [data-view-controls]")?.getAttribute("data-seen") === "free");
  assert.match(await vc.locator("[data-free-hint]").innerText(), /drag to turn and tilt.*Shift-drag.*wheel or pinch/);
  await shot(gm, "rule12-system-3d-free");
  await seen.getByRole("button", { name: "Edge-on" }).click();
  assert.equal(await gm.locator(".s3d").getAttribute("data-seen"), "edge");

  // Kept on this device: a fresh page opens in 3D, edge-on.
  const again = await ctx.newPage();
  again.on("pageerror", (e) => errors.push(`again: ${e.message}`));
  await again.goto(root + "#chart");
  await ready(again);
  await again.locator(".s3d").waitFor();
  assert.equal(await again.locator(".viewbar [data-view-controls]").getAttribute("data-seen"), "edge");
  await again.close();

  // The galaxy chart (V9t) follows the same choice: 3D, edge-on, the disc turned.
  await go(gm, "galaxy");
  await gm.locator(".gal").waitFor({ timeout: 30000 });
  assert.equal(await vc.getAttribute("data-style"), "3d");
  assert.equal(await vc.getAttribute("data-seen"), "edge");
  await gm.locator("[data-galaxy-3d][data-seen='edge']").waitFor({ timeout: 20000 });
  await more(gm, /Night chart/);
  await shot(gm, "rule12-galaxy-night");
  // Galaxies nearby is flat for now: Chart from above, everything else listed and disabled, the choice untouched.
  await go(gm, "cluster");
  await gm.locator(".viewbar [data-view-controls]").waitFor();
  await gm.waitForFunction(() => document.querySelector(".viewbar [data-view-controls]")?.getAttribute("data-style") === "chart", null, { timeout: 30000 });
  assert.equal(await vc.getAttribute("data-seen"), "above");
  assert.ok(await style.getByRole("button", { name: "3D" }).isDisabled());
  assert.ok(await seen.getByRole("button", { name: "Free" }).isDisabled());
  assert.match(await seen.getByRole("button", { name: "Free" }).getAttribute("title"), /flat, seen from above/);
  // Back at the system, the 3D choice is still there.
  await gm.locator('.place [data-place-step="system"]').click();
  await gm.locator(".s3d").waitFor();
  await style.getByRole("button", { name: "Chart" }).click();
  await gm.locator(".s3d").waitFor({ state: "detached" });

  // A screen that is not a diagram has no controls.
  await go(gm, "reference");
  assert.equal(await gm.locator(".viewbar").count(), 0);

  // Phone: over the chart, one row each, nothing off the side.
  const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
  phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
  await phone.goto(root + "#chart");
  await ready(phone);
  const pvc = phone.locator(".viewbar [data-view-controls]");
  assert.ok(await pvc.isVisible());
  const vbox = await pvc.boundingBox();
  const cbox = await phone.locator(".area-chart").boundingBox();
  assert.ok(vbox.y + vbox.height <= cbox.y + 1, "the controls sit above the chart");
  const overflow = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(overflow <= 1, `nothing runs off the side (${overflow}px)`);
  const prows = new Set(await pvc.locator(":scope > .field").evaluateAll((fs) => fs.map((f) => Math.round(f.getBoundingClientRect().bottom / 8))));
  assert.ok(prows.size <= 2, `two rows on a phone (${prows.size})`);
  await shot(phone, "rule12-phone");

  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "choosing writes nothing");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("view controls e2e: ok");
} finally {
  server.kill();
}
