// The main map's feel (Alex, 24 Nov, Sol chart note 3: "the main map needs to feel better"), against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/chart-feel.e2e.mjs [--screens <dir>].
// The Solar System opens framed on its worlds and belts (the Oort cloud is a haze beyond, not the frame); no two names
// overlap; no name, world or scale figure sits under the view controls, tools, card, key or zoom; resting on a world
// lifts its orbit and its mark. The same on a phone, where the system is drawn above the key and the zoom.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { pinClock } from "./nav.mjs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

const boxes = (page, sel) => page.locator(sel).evaluateAll((es) => es.map((e) => { const b = e.getBoundingClientRect(); return { t: e.textContent?.trim() ?? "", x: b.left, y: b.top, w: b.width, h: b.height }; }).filter((b) => b.w > 0 && b.h > 0));
const hit = (a, b, m = 0) => a.x + m < b.x + b.w && b.x + m < a.x + a.w && a.y + m < b.y + b.h && b.y + m < a.y + a.h;

try {
  const browser = pinClock(await chromium.launch());
  const errors = [];
  for (const [vp, size] of [["desktop", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const page = await browser.newPage({ viewport: size, deviceScaleFactor: screens ? 2 : 1, ...(vp === "phone" ? { isMobile: true, hasTouch: true } : {}) });
    page.on("pageerror", (e) => errors.push(`${vp}: ${e.message}`));
    await page.goto(base);
    await page.locator('[data-door="sol"]').click();
    await page.locator('.area-chart .body[aria-label^="Neptune"]').first().waitFor({ timeout: 30000 });
    // The overlays are measured once the layout settles.
    await page.waitForTimeout(900);

    const names = await boxes(page, ".area-chart text.name");
    assert.ok(names.length >= 9, `${vp}: the worlds are named (${names.length})`);
    for (let i = 0; i < names.length; i++)
      for (let j = i + 1; j < names.length; j++) assert.ok(!hit(names[i], names[j], 1), `${vp}: "${names[i].t}" and "${names[j].t}" overlap`);

    const over = await boxes(page, "[data-chart-overlay]");
    const worlds = await boxes(page, ".area-chart .body .lift");
    // A press at any world's centre lands on a world, never on a name drawn near it (a drag starts there).
    const stolen = await page.locator(".area-chart .body").evaluateAll((gs) => gs.flatMap((g) => {
      const c = g.querySelector("circle.hit")?.getBoundingClientRect();
      if (!c || !c.width) return [];
      const at = document.elementFromPoint(c.left + c.width / 2, c.top + c.height / 2);
      // Two close worlds may share a press (their hit circles overlap); a name never takes one.
      return at && at.closest(".body") ? [] : [g.getAttribute("aria-label")];
    }));
    assert.deepEqual(stolen, [], `${vp}: a world's centre answers to a world, not a name`);
    // A press on a name still chooses its body.
    const named = page.locator(".area-chart [data-name-hit]").first();
    const nameFor = await named.getAttribute("data-name-hit");
    if (vp === "desktop") {
      await named.click({ position: { x: 2, y: 2 } });
      await page.waitForFunction((a) => document.querySelector(`.area-chart .body[data-address="${a}"]`)?.getAttribute("aria-pressed") === "true", nameFor, { timeout: 3000 });
    }
    for (const n of [...names, ...(await boxes(page, ".area-chart text.dist"))])
      for (const o of over) assert.ok(!hit(n, o, 1), `${vp}: "${n.t}" lies under an overlay`);
    for (const wb of worlds) for (const o of over) if (vp === "phone") assert.ok(!hit(wb, o, 1), `${vp}: a world lies under the key or the zoom`);

    // Framed on the worlds: Neptune is well out from the Sun, not huddled in the middle of the Oort cloud.
    const chart = (await boxes(page, ".area-chart svg"))[0];
    // The Sun is the first body drawn (":first-of-type" never matched it: the name-hit group is the first <g>).
    const sun = (await boxes(page, ".area-chart .body .hit"))[0] ?? null;
    assert.match((await page.locator(".area-chart .body").first().getAttribute("aria-label")) ?? "", /^Sun/, `${vp}: the first body drawn is the Sun`);
    const nep = (await boxes(page, '.area-chart .body[aria-label^="Neptune"] .hit'))[0];
    const centre = sun ? [sun.x + sun.w / 2, sun.y + sun.h / 2] : [chart.x + chart.w / 2, chart.y + chart.h / 2];
    const out = Math.hypot(nep.x + nep.w / 2 - centre[0], nep.y + nep.h / 2 - centre[1]);
    assert.ok(out > Math.min(chart.w, chart.h) * 0.25, `${vp}: Neptune sits ${Math.round(out)} px out, the chart fills with the worlds`);

    if (vp === "desktop") {
      if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/sol-desktop.png` }));
      // Resting on Saturn lifts its orbit into ink and grows its mark.
      await page.locator('.area-chart .body[aria-label^="Saturn"] .hit').hover();
      await page.waitForFunction(() => document.querySelectorAll(".area-chart circle.ink.t-ink").length >= 1, null, { timeout: 3000 });
      await page.waitForTimeout(300);
      const t = await page.locator('.area-chart .body[aria-label^="Saturn"] .lift').evaluate((e) => getComputedStyle(e).transform);
      assert.notEqual(t, "none", "the mark lifts under the pointer");
      if (screens) await page.screenshot({ path: `${screens}/sol-hover.png` });
    } else if (screens) await page.screenshot({ path: `${screens}/sol-phone.png` });
    await page.close();
  }
  assert.deepEqual(errors, []);
  console.log("chart feel: framed on the worlds, names clear of each other and of the overlays, hover lifts, phone fits");
  await browser.close();
} finally {
  server.kill();
}
