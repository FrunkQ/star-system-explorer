// The grand zoom, first part (GZ8): a journey from the cosmic web to the galaxies nearby, against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/journey.e2e.mjs [chromium] [--screens <dir>]
// A link whose way in starts at the web opens the player; it rests at the web until the web is read, then Zoom in
// travels down: the place line is the progress bar, the width line counts down, the seam draws both levels (the
// web kept behind past the catalogue's reach), and it ends on the galaxies nearby. Back up returns to the start;
// Escape leaves. More > "Zoom in from the cosmic web" opens it too.
// Warming up (FL5): the levels read one at a time, the web first; Go says "Getting ready…" with the time left until
// every level is in; a press before that goes early and, with no coarse picture of the galaxies nearby yet, holds
// before the seam; a journey for film (film=1) never shows a coarse picture.
import "./chart-first.mjs";
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
  // The web reads first, alone: the galaxies nearby wait their turn so they do not delay the start.
  assert.equal(await player.getAttribute("data-journey-opened"), "0", "only the web is reading at first");
  assert.match(await page.locator("[data-journey-go]").innerText(), /Getting ready/);
  assert.match(await page.locator("[data-journey-wait]").innerText(), /^Getting the way down ready: about \d+ seconds, 0 of 2 levels\.$/);
  await page.waitForSelector("[data-journey-layer=web][data-ready]", { timeout: 120000 });
  await page.waitForFunction(() => document.querySelector("[data-journey-player]")?.getAttribute("data-journey-opened") === "0 1");
  assert.equal(await page.locator("[data-journey-step=web]").getAttribute("data-ready"), "", "the place line inks in the web");
  await shot(page, "0-getting-ready");
  await page.waitForSelector("[data-journey-layer=cluster][data-ready]", { timeout: 120000, state: "attached" });
  assert.equal(await player.getAttribute("data-journey-t"), "0.00", "held at the start until Zoom in");
  assert.match(await page.locator("[data-journey-go]").innerText(), /Zoom in/);
  assert.equal(await page.locator("[data-journey-wait]").count(), 0, "no wait line once the way down is ready");
  assert.ok(await page.locator("[data-journey-caption]").isVisible());
  assert.ok(await page.locator("[data-web-3d][data-journey]").isVisible(), "the web cube's own view, aimed by the journey");
  await shot(page, "1-web");

  await page.keyboard.press("Space");
  assert.equal(await player.getAttribute("data-journey-early"), null, "waited for the warm-up: not early");
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
  // Opened again in the same visit: the levels are in memory. The web is ready at once; the galaxies nearby answer
  // from memory too, once the kernel has finished what the chart room behind asked of it on closing.
  await page.waitForSelector("[data-journey-layer=web][data-ready]", { timeout: 5000, state: "attached" });
  await page.waitForSelector("[data-journey-layer=cluster][data-ready]", { timeout: 60000, state: "attached" });
  assert.match(await page.locator("[data-journey-go]").innerText(), /Zoom in/);

  // Gone early, on a fresh visit: the web is in, the galaxies nearby are not; with no coarse picture of them the
  // journey travels the web and holds before the seam (never half-faded), then goes on once they are read.
  for (const film of [false, true]) {
    const p2 = await (await browser.newContext({ viewport: { width: 1280, height: 900 } })).newPage();
    p2.on("pageerror", (e) => errors.push(e.message));
    await p2.goto(base + `#chart/cluster?from=web${film ? "&film=1" : ""}`);
    await p2.waitForSelector("[data-journey-layer=web][data-ready]", { timeout: 120000 });
    assert.equal(await p2.locator("[data-journey-layer=cluster][data-ready]").count(), 0, "the galaxies nearby still reading");
    await p2.keyboard.press("Space");
    const pl = p2.locator("[data-journey-player]");
    assert.equal(await pl.getAttribute("data-journey-early"), film ? null : "", film ? "film never goes early" : "went early");
    await p2.waitForFunction(() => document.querySelector("[data-journey-player]")?.getAttribute("data-journey-waiting") === "1", null, { timeout: 60000 });
    if (!(await p2.locator("[data-journey-layer=cluster][data-ready]").count())) assert.match(await p2.locator("[data-journey-reading]").innerText(), /^Reading galaxies nearby…/);
    assert.equal(await p2.locator("[data-journey-layer=cluster][data-picture]").count(), 0, "held before the seam: the galaxies nearby not shown");
    // Every frame from here to the end: no stand-in (none is declared yet, and film never takes one).
    await p2.waitForSelector("[data-journey-done]", { timeout: 120000 });
    assert.equal(await p2.locator("[data-picture=coarse]").count(), 0);
    assert.equal(await p2.locator("[data-journey-layer=cluster]").getAttribute("data-picture"), "proper");
    await p2.context().close();
  }
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("journey e2e: ok");
} finally {
  server.kill();
}
