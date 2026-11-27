// The grand zoom, first part (GZ8): a journey from the cosmic web to the galaxies nearby, against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/journey.e2e.mjs [chromium] [--screens <dir>]
// A link whose way in starts at the web opens the player; it rests at the web until the web is read, then Zoom in
// travels down: the place line is the progress bar, the width line counts down, the seam draws both levels (the
// web kept behind past the catalogue's reach), and it ends on the galaxies nearby. Back up returns to the start;
// Escape leaves. More > "Zoom in from the cosmic web" opens it too.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const errors = [];
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 900 } })).newPage();
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(base + "#chart/cluster?from=web");
  const player = page.locator("[data-journey-player]");
  await player.waitFor({ timeout: 30000 });
  // Resting at the web: the place line says where, and what lies below that nothing draws yet.
  assert.equal(await page.locator("[data-journey-step=web]").getAttribute("data-state"), "current");
  assert.equal(await page.locator("[data-journey-step=cluster]").getAttribute("data-state"), "coming");
  assert.equal(await page.locator("[data-journey-step=system]").getAttribute("data-state"), "off");
  await page.waitForSelector("[data-journey-layer=web][data-ready]", { timeout: 120000 });
  assert.equal(await player.getAttribute("data-journey-t"), "0.00", "held at the start until Zoom in");
  assert.match(await page.locator("[data-journey-go]").innerText(), /Zoom in/);
  assert.ok(await page.locator("[data-journey-caption]").isVisible());
  assert.ok(await page.locator("[data-web-3d][data-journey]").isVisible(), "the web cube's own view, aimed by the journey");
  await shot(page, "1-web");

  await page.keyboard.press("Space");
  // While moving: the width line counts down in light years and powers of ten.
  await page.waitForFunction(() => Number(document.querySelector("[data-journey-player]")?.getAttribute("data-journey-t")) > 0.3);
  assert.match(await page.locator("[data-journey-width]").innerText(), /light years across · [\d.]+ × 10/);
  // The seam blends (it is not a cut): at some moment the galaxies nearby are partly faded in over the web.
  await page.waitForFunction(() => {
    const o = Number(document.querySelector("[data-journey-layer=cluster]")?.style.opacity ?? 0);
    return o > 0.05 && o < 0.95;
  }, null, { timeout: 60000, polling: "raf" });
  // The seam: past it the web is kept behind, with only the light within the catalogue's reach fading.
  await page.waitForFunction(() => document.querySelector("[data-journey-player]")?.getAttribute("data-journey-rung") === "cluster", null, { timeout: 60000 });
  await shot(page, "2-seam");
  await page.waitForSelector("[data-journey-done]", { timeout: 60000 });
  assert.equal(await page.locator("[data-journey-step=web]").getAttribute("data-state"), "passed");
  assert.equal(await page.locator("[data-journey-step=cluster]").getAttribute("data-state"), "current");
  assert.ok(await page.locator("[data-journey-layer=cluster]").isVisible());
  assert.ok(await page.locator("[data-journey-layer=web]").isVisible(), "the web kept behind the galaxies nearby");
  const inner = Number(await page.locator("[data-web-3d]").getAttribute("data-journey-inner"));
  const near = Number(await page.locator("[data-nearby]").getAttribute("data-nearby"));
  assert.ok(inner > 0 && near > 0, `both levels hold galaxies near home (web ${inner}, catalogue ${near})`);
  assert.match(await page.locator("[data-journey-go]").innerText(), /Again/);
  await shot(page, "3-nearby");

  // Back up: to the stop before (the start).
  await page.locator("[data-journey-back]").click();
  assert.equal(await player.getAttribute("data-journey-t"), "0.00");
  assert.equal(await page.locator("[data-journey-step=web]").getAttribute("data-state"), "current");
  // Escape leaves; More opens it again.
  await page.keyboard.press("Escape");
  await player.waitFor({ state: "detached" });
  await more(page, "Zoom in from the cosmic web");
  await player.waitFor();
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("journey e2e: ok");
} finally {
  server.kill();
}
