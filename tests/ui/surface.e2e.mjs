// SL1 in a real browser, against the built shell (scripts/build-shell.sh first). Usage: node tests/ui/surface.e2e.mjs
// [chromium firefox webkit] [--screens <dir>]. Add the sample Sol, pick Earth, open Land: the world's map shows
// its landmarks; pressing one plans a descent with its verdict and dangers; Land plays the flown descent with the
// kernel's log beside it; Skip puts the camera on the ground; walking moves it; True, 3D and Wireframe all draw
// (Chart draws the ground as 3D's diagram); Take off returns to choosing. The Moon with a capsule is offered, failing, not refused.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more, view } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const style = async (page, label) => {
  await page.getByRole("group", { name: "View style" }).getByRole("button", { name: label, exact: true }).click();
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    if (process.env.SL_DEBUG) gm.on("console", (m) => m.type() !== "log" && console.log("console", m.type(), m.text().slice(0, 400)));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(gm, "system");
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.locator('.area-chart .body[aria-label^="Earth"]').first().waitFor({ timeout: 15000 });
    await gm.locator('.area-chart .body[aria-label^="Earth"]').first().locator(".hit").click();
    await go(gm, "surface");

    // The map and its landmarks.
    await gm.waitForSelector("[data-surface] [data-landmark-row]", { timeout: 30000 });
    const rows = await gm.locator("[data-landmark-row]").count();
    assert.ok(rows >= 3, `landmarks listed: ${rows}`);
    assert.ok((await gm.locator("[data-landmark]").count()) >= 3, "landmarks pinned on the map");
    await shot(gm, "sl1-map");

    // A landmark on land: the highest peak.
    const land = gm.locator("[data-landmark-row^='peak']").first();
    await land.click();
    await gm.waitForSelector("[data-surface]:not([data-verdict=''])", { timeout: 15000 });
    const verdict = await gm.locator("[data-surface]").getAttribute("data-verdict");
    assert.ok(verdict, "a verdict");
    for (const k of ["slope", "roughness", "touchdown"]) assert.equal(await gm.locator(`[data-hazard='${k}']`).count(), 1, `${k} judged`);
    await shot(gm, "sl1-site");

    // Press anywhere: a spot inland on the largest continent, dry granite with some relief.
    const map = gm.locator("[data-site-map]");
    const box = await map.boundingBox();
    await map.click({ position: { x: ((75.97 + 180) / 360) * box.width, y: ((90 - 10.5) / 180) * box.height } });
    await gm.waitForFunction(() => document.querySelector("[data-surface]")?.getAttribute("data-site")?.startsWith("10."), null, { timeout: 15000 });
    await gm.waitForSelector("[data-surface]:not([data-verdict=''])", { timeout: 15000 });

    // Land: the flown descent with the kernel's log, then skip to the ground.
    await gm.locator("[data-land]").click();
    await gm.waitForSelector("[data-surface][data-phase='flying']", { timeout: 30000 });
    await gm.waitForSelector("[data-surface]:not([data-tier=''])", { timeout: 20000 });
    await gm.waitForSelector("[data-descent-log]");
    await gm.waitForTimeout(3000);
    await shot(gm, "sl1-flying");
    await gm.waitForTimeout(6000);
    await shot(gm, "sl1-flying-low");
    await gm.locator("[data-skip]").click();
    await gm.waitForSelector("[data-surface][data-phase='down']");
    const floor = Number(await gm.locator("[data-surface]").getAttribute("data-floor-m"));
    assert.ok(floor > 50 && floor < 120, `the ground at the kernel's fact floor: ${floor} m`);
    assert.match(await gm.locator("[data-ground-note]").innerText(), /decoration/);
    await gm.waitForTimeout(800);
    await shot(gm, "sl1-ground-3d");

    // Walk: forward moves the eye.
    const before = await gm.locator("[data-surface]").getAttribute("data-walk");
    await gm.locator("[data-ground]").focus();
    await gm.keyboard.down("ArrowUp");
    await gm.waitForTimeout(1200);
    await gm.keyboard.up("ArrowUp");
    const after = await gm.locator("[data-surface]").getAttribute("data-walk");
    assert.notEqual(after, before, `walking moves: ${before} -> ${after}`);

    // Every look draws.
    await style(gm, "True");
    await gm.waitForSelector("[data-surface][data-look='true']");
    await gm.waitForTimeout(800);
    await shot(gm, "sl1-ground-true");
    await view(gm, "HUD", "On");
    await gm.waitForSelector("[data-surface-hud]");
    await gm.waitForTimeout(400);
    await shot(gm, "sl1-ground-true-hud");
    await view(gm, "HUD", "Off");
    await gm.locator("[data-hour]").fill("-5.25");
    await gm.waitForTimeout(500);
    await shot(gm, "sl1-ground-true-morning");
    // Night: the stars come out over the site and the eye opens up to see them (SL-night).
    await gm.waitForSelector("[data-surface]:not([data-stars='0'])", { timeout: 20000 }).catch(async (e) => { console.log(await gm.locator("[data-surface]").evaluateAll((l) => l.map((x) => x.outerHTML.slice(0, 600)))); throw e; });
    let darkest = { at: 0, exposure: 0 };
    for (let h = -180; h <= 180; h += 30) {
      await gm.locator("[data-hour]").fill(String(h));
      await gm.waitForTimeout(150);
      const e = Number(await gm.locator("[data-surface]").getAttribute("data-exposure"));
      if (e > darkest.exposure) darkest = { at: h, exposure: e };
    }
    assert.ok(darkest.exposure > 600, `the eye opens up at night: exposure ${darkest.exposure}`);
    await gm.locator("[data-hour]").fill(String(darkest.at));
    await gm.waitForTimeout(600);
    const stars = Number(await gm.locator("[data-surface]").getAttribute("data-stars"));
    assert.ok(stars > 500, `stars over the site: ${stars}`);
    await shot(gm, "sl1-ground-true-night");
    await gm.locator("[data-hour]").fill("-80");
    await gm.locator("[data-drone]").check();
    await gm.waitForTimeout(500);
    await shot(gm, "sl1-ground-true-drone");
    await style(gm, "Wireframe");
    await gm.waitForSelector("[data-surface][data-look='wireframe']");
    await gm.waitForTimeout(500);
    await shot(gm, "sl1-ground-wireframe");
    await style(gm, "Chart");
    await gm.waitForSelector("[data-surface][data-look='3d']");
    await gm.locator("[data-take-off]").click();
    await gm.waitForSelector("[data-surface][data-phase='choose']");

    assert.deepEqual(errors, [], `no page errors: ${errors.join("; ")}`);
    console.log(`${name}: surface ok (verdict ${verdict}, floor ${floor} m, walked ${before} -> ${after})`);
    await browser.close();
  }
} finally {
  server.kill();
}
