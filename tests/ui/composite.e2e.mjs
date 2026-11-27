// Composite (exotic objects design 6A, XO17; Alex 24 Nov: "the composite view should be everywhere!"), in a real
// browser against the built shell (scripts/build-shell.sh first). Usage: node tests/ui/composite.e2e.mjs [--screens <dir>]
// Composite is the band switch's last entry. Pressing it shows the Observatory mix at once and opens a tick list
// with presets and each band's colour; the legend lists every layer. In 3D and True the system and the galaxy are
// drawn once per band and laid into one picture. Unticking a band takes its layer away; a preset sets the mix. The
// flat chart lists Composite quiet with its reason. On a phone at most two bands are laid. Choosing writes nothing.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go } from "./nav.mjs";

const { chromium, devices } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
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
  const openFold = async () => {
    if ((await vc.locator("[data-view-more]").getAttribute("aria-expanded")) !== "true") await vc.locator("[data-view-more]").click();
  };
  const band = vc.getByRole("group", { name: "Band", exact: true });

  // The flat chart: Composite listed, quiet, with why.
  await openFold();
  const comp = band.getByRole("button", { name: "Composite" });
  assert.equal(await comp.count(), 1, "Composite is the band switch's last entry");
  assert.ok(await comp.isDisabled(), "the flat chart paints one band");
  assert.match((await comp.getAttribute("title")) ?? "", /3D or True/);

  // 3D: one press shows the Observatory mix and its tick list.
  await vc.getByRole("group", { name: "View style" }).getByRole("button", { name: "3D" }).click();
  await openFold();
  await band.getByRole("button", { name: "Composite" }).click();
  await gm.waitForSelector("[data-composite-stack]");
  assert.equal(await vc.getAttribute("data-band"), "composite");
  assert.equal(await gm.locator("[data-composite-stack]").getAttribute("data-layers"), "visible xray infrared radio", "Visible at the bottom, then the mix");
  assert.equal(await gm.locator("[data-composite-stack] .layer").count(), 4);
  assert.match(await vc.locator("[data-composite-legend]").innerText(), /^Composite: X-ray violet · Visible actual · Infrared red · Radio orange/);
  assert.equal(await vc.locator("[data-composite-legend]").getAttribute("data-mapped"), "yes");
  await shot(gm, "composite-1-system-3d-observatory");

  // Untick radio: its layer goes.
  await vc.locator('[data-tick="radio"]').click();
  await gm.waitForFunction(() => document.querySelector("[data-composite-stack]")?.getAttribute("data-layers") === "visible xray infrared");
  // A preset sets the mix.
  await openFold();
  await vc.locator('[data-preset="hot-and-cold"]').click();
  await gm.waitForFunction(() => document.querySelector("[data-composite-stack]")?.getAttribute("data-layers") === "xray infrared");
  // True: the same mix.
  await vc.getByRole("group", { name: "View style" }).getByRole("button", { name: "True" }).click();
  await gm.waitForSelector("[data-composite-stack].dark");
  await shot(gm, "composite-2-system-true-hot-and-cold");

  // The choice is kept on the device and comes back.
  await gm.reload();
  await ready(gm);
  await gm.waitForSelector("[data-composite-stack]");
  assert.equal(await gm.locator("[data-composite-stack]").getAttribute("data-layers"), "xray infrared");

  // Galaxy chart: the same choice, in 3D.
  await go(gm, "galaxy");
  await gm.waitForSelector("[data-galaxy-3d]", { timeout: 20000 });
  await gm.waitForSelector("[data-composite-stack]", { timeout: 20000 });
  assert.equal(await gm.locator("[data-composite-stack] [data-galaxy-3d]").count(), 2, "the galaxy drawn once per band");
  await shot(gm, "composite-3-galaxy-true");

  // Back to one band: the stack goes.
  await gm.goto(root + "#chart");
  await ready(gm);
  await openFold();
  await band.getByRole("button", { name: "Visible", exact: true }).click();
  await gm.locator("[data-composite-stack]").waitFor({ state: "detached" });

  // A phone lays at most two bands, and says why on the third.
  const phone = await browser.newContext({ ...devices["Pixel 7"] });
  const p = await phone.newPage();
  p.on("pageerror", (e) => errors.push(e.message));
  await p.goto(root + "#chart");
  await ready(p);
  const pvc = p.locator(".viewbar [data-view-controls]");
  await pvc.getByRole("group", { name: "View style" }).getByRole("button", { name: "3D" }).click();
  await pvc.locator("[data-view-more]").click();
  await pvc.getByRole("group", { name: "Band", exact: true }).getByRole("button", { name: "Composite" }).click();
  await p.waitForSelector("[data-composite-stack]");
  assert.equal((await p.locator("[data-composite-stack]").getAttribute("data-layers")).split(" ").length, 2);
  await pvc.locator('[data-tick="ultraviolet"]').click();
  assert.match(await pvc.locator("[data-composite-refused]").innerText(), /phone draws 2 bands/);
  await shot(p, "composite-4-phone");
  await phone.close();

  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "choosing writes nothing");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("composite e2e: ok");
} finally {
  server.kill();
}
