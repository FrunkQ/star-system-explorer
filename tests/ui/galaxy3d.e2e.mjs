// The galaxy in 3D and True (V9t), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/galaxy3d.e2e.mjs [--screens <dir>]
// Open the galaxy chart; the view controls offer 3D and True. 3D turns the disc as a diagram; True draws it in light
// on the GPU tier, and every band says what it maps; Seen from Edge-on shows the disc's bend; HUD off leaves the
// picture alone. The Chart style still carries the scars and the edge-on strip. Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
  const errors = [];
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
  await go(gm, "galaxy");
  await gm.waitForSelector(".gal [data-ring]", { timeout: 30000 });
  const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  const style = (name) => gm.getByRole("group", { name: "View style" }).getByRole("button", { name, exact: true });
  const g3d = gm.locator("[data-galaxy-3d]");
  // 3D: the chart's disc turned.
  await style("3D").click();
  await g3d.waitFor({ timeout: 20000 });
  await gm.waitForFunction(() => document.querySelector("[data-galaxy-3d]")?.getAttribute("data-tier"), null, { timeout: 20000 });
  assert.equal(await g3d.getAttribute("data-look"), "3d");
  await shot(gm, "v9t-galaxy-3d");
  // True: the galaxy in light; each band says what it maps.
  await style("True").click();
  await gm.waitForFunction(() => document.querySelector("[data-galaxy-3d]")?.getAttribute("data-look") === "true");
  const tier = await g3d.getAttribute("data-tier");
  await gm.locator("[data-galaxy-band='visible']").waitFor({ timeout: 20000 });
  await shot(gm, "v9t-galaxy-true-visible");
  for (const [b, id] of [["Infrared", "infrared"], ["Ultraviolet", "ultraviolet"], ["X-ray", "xray"], ["Radio", "radio"]]) {
    await gm.getByRole("group", { name: "Band" }).getByRole("button", { name: b }).click();
    await gm.locator(`[data-galaxy-band='${id}']`).waitFor({ timeout: 20000 });
    await shot(gm, `v9t-galaxy-true-${id}`);
  }
  await gm.getByRole("group", { name: "Band" }).getByRole("button", { name: "Visible" }).click();
  await gm.getByRole("group", { name: "Seen from" }).getByRole("button", { name: "Edge-on" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-galaxy-3d]")?.getAttribute("data-seen") === "edge");
  await shot(gm, "v9t-galaxy-true-edge");
  await gm.getByRole("group", { name: "Seen from" }).getByRole("button", { name: "Above" }).click();
  await gm.getByRole("group", { name: "HUD" }).getByRole("button", { name: "Off" }).click();
  await gm.waitForFunction(() => document.querySelector("[data-galaxy-3d]")?.getAttribute("data-hud") === "off");
  await shot(gm, "v9t-galaxy-true-above-hud-off");
  // Back to Chart: the diagram, its scars and edge-on strip as before.
  await style("Chart").click();
  await gm.locator(".gal [data-ring]").first().waitFor();
  assert.equal(await g3d.count(), 0);
  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "looking writes nothing");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log(`galaxy 3d e2e: ok (tier ${tier})`);
} finally {
  server.kill();
}
