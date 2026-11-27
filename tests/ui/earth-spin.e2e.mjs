// The drawn Earth turns by its real spin (RW17 part 3), in a real browser against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/earth-spin.e2e.mjs [--screens <dir>]
// Open the Solar System on a pinned device date (26 Nov 2026, 12:00 UTC), show the system in True: Earth, and only
// Earth, is turned by the axes the kernel gives it, with Greenwich where the sidereal time puts it (GMST 245.4 degrees,
// so longitude 0 points 116.5 degrees round from the equinox in the ecliptic). Nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { view, pinClock } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9800 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  const browser = pinClock(await chromium.launch());
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 })).newPage();
  const errors = [];
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(base);
  await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
  await gm.locator('[data-door="sol"]').click();
  // The door finishes when the chart says where the bodies came from (the system may open in any view style).
  await gm.waitForSelector("[data-motion]", { timeout: 60000 });
  const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  await view(gm, "View style", "True");
  await gm.waitForFunction(() => document.querySelector("[data-turned]")?.getAttribute("data-turned"), null, { timeout: 30000 });
  const turned = await gm.locator("[data-turned]").first().getAttribute("data-turned");
  const m = /^Earth:(-?\d+)$/.exec(turned ?? "");
  assert.ok(m, `only Earth carries a spin model: ${turned}`);
  assert.ok(Math.abs(Number(m[1]) + 116) <= 2, `Greenwich faces where the sidereal time puts it: ${turned}`);
  if (screens) (mkdirSync(screens, { recursive: true }), await gm.screenshot({ path: `${screens}/earth-spin-true.png` }));
  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "nothing written");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("earth spin e2e: ok");
} finally {
  server.kill();
}
