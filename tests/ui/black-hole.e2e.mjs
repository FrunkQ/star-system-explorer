// Black holes in True (XO4; exotic objects design 3.1 to 3.4, 7), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/black-hole.e2e.mjs [--screens <dir>]
// From the galaxy's rail, Black holes opens on the kernel's 10 Sun hole fed by a companion, seen from 10° above its
// disc (Luminet's view). The frame is measured: straight up from the middle the shadow is dark out to the shadow's edge
// the kernel's bend table gives (√27 gravitational radii), then the dark gap, then the far side of the disc bent over
// the top at the distance the table gives, each within two pixels. In X-rays the side turning towards the viewer
// (left) is the brighter. The note says how far visible light was dimmed (the kernel's 3.2 × 10⁴ times the Sun's
// surface). The starved centre is almost dark in visible light. Every band and Composite draw; reading writes nothing;
// a phone gets the same geometry.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go } from "./nav.mjs";

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

/** The canvas as the user sees it: the luminance up the middle column (from the centre to the top), and the mean of each half. */
const frameOf = async (page) => {
  const cv = page.locator("[data-hole-canvas]");
  const b64 = (await cv.screenshot()).toString("base64");
  return page.evaluate(async (b) => {
    const img = new Image();
    img.src = `data:image/png;base64,${b}`;
    await img.decode();
    const c = document.createElement("canvas");
    c.width = img.width;
    c.height = img.height;
    const g = c.getContext("2d");
    g.drawImage(img, 0, 0);
    const d = g.getImageData(0, 0, img.width, img.height).data;
    const lum = (x, y) => {
      const o = (y * img.width + x) * 4;
      return 0.2126 * d[o] + 0.7152 * d[o + 1] + 0.0722 * d[o + 2];
    };
    const cx = img.width >> 1;
    const cy = img.height >> 1;
    const up = [];
    for (let y = cy; y >= 0; y--) up.push(lum(cx, y));
    let left = 0;
    let right = 0;
    let all = 0;
    for (let y = 0; y < img.height; y += 2)
      for (let x = 0; x < img.width; x += 2) {
        const v = lum(x, y);
        all += v;
        if (x < cx) left += v;
        else right += v;
      }
    const n = (img.width * img.height) / 4;
    return { up, left: left / (n / 2), right: right / (n / 2), mean: all / n, width: img.width, height: img.height };
  }, b64);
};

/** Bright runs straight up from the middle: [start, end) in pixels from the centre. */
const runs = (up, floor = 12) => {
  const out = [];
  let at = -1;
  up.forEach((v, i) => {
    if (v > floor && at < 0) at = i;
    if (v <= floor && at >= 0) (out.push([at, i]), (at = -1));
  });
  if (at >= 0) out.push([at, up.length]);
  return out;
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => (errors.push(e.message), console.error("page error:", e.message)));
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 30000 });
    await gm.locator('[data-door="sol"]').click();
    await gm.waitForSelector("[data-state-hash]", { timeout: 30000 });
    // The door is through once the chart shows the Solar System (it turns to the chart when it finishes).
    await gm.locator('.area-chart .body[aria-label^="Jupiter,"]').first().waitFor({ timeout: 60000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await go(gm, "galaxy");
    await go(gm, "black-hole");
    const cv = gm.locator("[data-hole-canvas]");
    await gm.waitForSelector('[data-hole-canvas][data-drawn^="fixture/stellar-hole"]', { timeout: 90000 });
    const words = await gm.locator("[data-hole-words]").innerText();
    assert.match(words, /Seen 10° above the disc's plane/);
    assert.match(words, /3\.2 × 10⁴ times as bright as the Sun's surface/, "the kernel's dimming figure");
    assert.match(await gm.locator("[data-hole-chip]").innerText(), /dimmed 3\.2 × 10⁴ times/);
    assert.match(words, /7\.1 times as bright/, "the kernel's side ratio");

    // The frame against the table: shadow dark to its edge, then the gap, then the arch.
    const vc = gm.locator(".viewbar [data-view-controls]");
    const band = vc.getByRole("group", { name: "Band", exact: true });
    const pickBand = async (name) => {
      if (!(await band.isVisible())) await vc.locator("[data-view-more]").click();
      await band.getByRole("button", { name, exact: true }).click();
    };
    await pickBand("X-ray");
    await gm.waitForSelector('[data-hole-canvas][data-drawn$="|xray|nosky"], [data-hole-canvas][data-drawn$="|xray|sky"]', { timeout: 150000 });
    const shadowPx = Number(await cv.getAttribute("data-shadow-px"));
    const archPx = Number(await cv.getAttribute("data-arch-px"));
    assert.ok(shadowPx > 20 && archPx > shadowPx, `marks: shadow ${shadowPx}, arch ${archPx}`);
    const f = await frameOf(gm);
    const r = runs(f.up);
    assert.ok(r.length >= 1, "something bright straight up from the middle");
    const first = r[0][0];
    assert.ok(f.up.slice(0, Math.floor(shadowPx) - 2).every((v) => v <= 12), "the shadow is dark to its edge");
    assert.ok(first >= shadowPx - 2, `first light at ${first} px, the shadow's edge at ${shadowPx}`);
    const arch = r.find(([a]) => a >= archPx - 2);
    assert.ok(arch && Math.abs(arch[0] - archPx) <= 2, `the arch starts at ${arch?.[0]} px, the table says ${archPx}`);
    assert.ok(f.left > f.right * 1.2, `X-rays: the side turning towards the viewer is brighter (${f.left} against ${f.right})`);
    await shot(gm, `black-hole-1-stellar-xray-${vp}`);

    // Every band draws, and Composite lays them together.
    for (const [name, id] of [["Visible", "visible"], ["Ultraviolet", "ultraviolet"], ["Infrared", "infrared"], ["Radio", "radio"]]) {
      await pickBand(name);
      await gm.waitForSelector(`[data-hole-canvas][data-drawn*="|${id}|"]`, { timeout: 150000 });
      const fb = await frameOf(gm);
      // In radio the thin disc carries a hundred-millionth of this hole's light (the rest is the hot flow round it,
      // which the kernel gives as a total only): drawn as faint as that, and said.
      if (id === "radio") assert.match(await gm.locator("[data-hole-words]").innerText(), /In radio the drawn disc carries .*% of this body's light; the rest is the hot flow/);
      else assert.ok(fb.mean > 2, `${name}: the disc shows`);
      if (id === "visible") assert.match(await gm.locator("[data-hole-chip]").innerText(), /dimmed/);
      else assert.match(await gm.locator("[data-hole-chip]").innerText(), /mapped/);
      if (vp === "desk") await shot(gm, `black-hole-2-stellar-${id}-desk`);
    }
    if (vp === "desk") {
      await pickBand("Composite");
      await gm.waitForSelector('[data-hole-canvas][data-drawn*="+"]', { timeout: 150000 });
      await shot(gm, "black-hole-3-stellar-composite-desk");
      await pickBand("Visible");
    }

    // The quasar: ultraviolet is its peak; the starved centre is nearly dark in visible light.
    await gm.locator('[data-hole-pick="fixture/quasar"]').click();
    await gm.waitForSelector('[data-hole-canvas][data-drawn^="fixture/quasar"]', { timeout: 150000 });
    assert.match(await gm.locator("[data-hole-words]").innerText(), /peaks in the ultraviolet/);
    await shot(gm, `black-hole-4-quasar-visible-${vp}`);
    await gm.locator('[data-hole-pick="fixture/starved-nucleus"]').click();
    await gm.waitForSelector('[data-hole-canvas][data-drawn^="fixture/starved-nucleus"][data-drawn*="|visible|"]', { timeout: 150000 });
    const starved = await frameOf(gm);
    assert.ok(starved.mean < 3, `the starved centre is almost dark in visible light (${starved.mean})`);
    assert.match(await gm.locator("[data-hole-words]").innerText(), /of the Sun's surface brightness, so it is drawn almost dark/);
    await shot(gm, `black-hole-5-starved-visible-${vp}`);

    // Seen from above: the disc is a plain ring round a dark middle; the shadow's edge stays where the table puts it.
    if (!(await vc.getByRole("group", { name: "Seen from" }).isVisible())) await vc.locator("[data-view-more]").click();
    await gm.locator('[data-hole-pick="fixture/stellar-hole"]').click();
    await vc.getByRole("group", { name: "Seen from" }).getByRole("button", { name: "Above" }).click();
    await gm.waitForSelector('[data-hole-canvas][data-drawn^="fixture/stellar-hole|349|"]', { timeout: 150000 });
    assert.match(await gm.locator("[data-hole-words]").innerText(), /Seen 88° above the disc's plane/);
    await shot(gm, `black-hole-6-stellar-above-${vp}`);

    // Reading changes nothing.
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0);
    assert.deepEqual(errors, [], errors.join("\n"));
    await browser.close();
  }
  console.log("black hole e2e: ok");
} finally {
  server.kill();
}
