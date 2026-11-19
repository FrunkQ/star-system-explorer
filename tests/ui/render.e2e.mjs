// R1 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/render.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The Body globe is a render recipe. For adopted Earth it checks that:
//  - the recipe runs on every tier it declares (GPU, 2D canvas, words), and
//    the GPU and canvas tiers draw the same colours from the same numbers;
//  - a recipe without the tier in use hands over to one of its family, and says so;
//  - drawing never changes the state hash;
//  - on an emulated low-end phone (Playwright's Moto G4 with Chromium's CPU
//    slowed 4x, as Lighthouse does for mobile; not a real phone) the governor
//    steps quality down until frames fit the budget, and the badge says so.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit, devices } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const pixelsOf = async (page, loc) => {
  const b64 = (await loc.screenshot()).toString("base64");
  return page.evaluate(async (b) => {
    const img = new Image();
    img.src = `data:image/png;base64,${b}`;
    await img.decode();
    const c = document.createElement("canvas");
    c.width = img.width;
    c.height = img.height;
    const g = c.getContext("2d");
    g.drawImage(img, 0, 0);
    // Mean colour of a small square at the centre and one off-centre, to be robust to a pixel's offset.
    const mean = (fx, fy) => {
      const s = Math.max(2, Math.round(img.width / 40));
      const d = g.getImageData(Math.round(img.width * fx) - s, Math.round(img.height * fy) - s, 2 * s, 2 * s).data;
      const m = [0, 0, 0];
      for (let i = 0; i < d.length; i += 4) for (let k = 0; k < 3; k++) m[k] += d[i + k];
      return m.map((x) => Math.round(x / (d.length / 4)));
    };
    return { centre: mean(0.5, 0.5), upper: mean(0.4, 0.35), corner: mean(0.03, 0.03) };
  }, b64);
};

async function openEarth(page) {
  await page.goto(base);
  await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
  const hash = await page.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  const menu = page.getByLabel("Screen menu");
  const phone = await menu.isVisible().catch(() => false);
  const go = async (n, v) => (phone ? menu.selectOption(v) : page.getByRole("group", { name: "Screen" }).getByRole("button", { name: n, exact: true }).click());
  await page.getByRole("button", { name: "Make a system" }).click();
  const dlg = page.getByRole("dialog", { name: "Make a system" });
  await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
  await dlg.getByRole("button", { name: "Try a sample save" }).click();
  await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
  await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
  const earth = page.locator('.area-chart .body[aria-label^="Earth"]').first();
  await earth.waitFor({ timeout: 15000 });
  await earth.click();
  await go("Body", "body");
  await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']):not([data-frames='0'])", { timeout: 30000 });
  return { hash: await page.locator("[data-state-hash]").first().getAttribute("data-state-hash"), before: hash };
}

const globe = (p) => p.locator("[data-body] [data-tier]");
async function drawOn(page, tier) {
  const sel = page.getByLabel("Draw on");
  if (!(await sel.isVisible().catch(() => false))) await page.locator("[data-badge]").click();
  await sel.selectOption(tier);
}
async function useRecipe(page, id) {
  await page.getByLabel("Recipe").selectOption(id);
}

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    const { hash } = await openEarth(gm);
    const auto = { tier: await globe(gm).getAttribute("data-tier"), backend: await globe(gm).getAttribute("data-backend") };

    // Every tier Survey relief declares.
    const colours = {};
    for (const t of ["gpu-basic", "canvas", "text"]) {
      await drawOn(gm, t);
      await gm.waitForFunction((t) => {
        const el = document.querySelector("[data-body] [data-tier]");
        return el && (el.getAttribute("data-tier") === t || (t === "gpu-basic" && el.getAttribute("data-tier") === "canvas")) && Number(el.getAttribute("data-frames")) > 0;
      }, t, { timeout: 15000 });
      const got = await globe(gm).getAttribute("data-tier");
      await gm.waitForTimeout(400);
      if (t === "text") {
        const words = await gm.locator("[data-narration]").innerText();
        assert.match(words, /^Earth, the face centred on 20° N, 0° E: \d+% under water/);
        assert.equal(await gm.locator("[data-body] [data-tier] canvas").count(), 0, "words only: no canvas");
      } else {
        assert.equal(await gm.locator(`[data-body] [data-tier] canvas[data-surface="${got === "gpu-basic" ? "gpu" : "canvas"}"]`).count(), 1);
        colours[got] = await pixelsOf(gm, globe(gm));
        assert.notDeepEqual(colours[got].centre, colours[got].corner, `${got}: the globe is drawn`);
      }
      await shot(gm, `render-${t}-${name}`);
    }
    let parity = "GPU tier not available here";
    if (colours["gpu-basic"] && colours.canvas) {
      const d = ["centre", "upper"].flatMap((k) => colours["gpu-basic"][k].map((v, i) => Math.abs(v - colours.canvas[k][i])));
      parity = `max channel difference ${Math.max(...d)}`;
      assert.ok(Math.max(...d) <= 28, `GPU and canvas draw the same colours: ${JSON.stringify(colours)}`);
    }

    // A recipe with no GPU form hands over to its family, and says so.
    await drawOn(gm, "gpu-basic");
    await useRecipe(gm, "survey-contours");
    await gm.waitForTimeout(500);
    if ((await globe(gm).getAttribute("data-tier")) === "gpu-basic") {
      assert.equal(await globe(gm).getAttribute("data-recipe"), "survey-relief");
      assert.match(await gm.locator("[data-badge-detail]").innerText(), /Contour survey has no gpu-basic form, so Survey relief stands in/);
    }
    await drawOn(gm, "canvas");
    await gm.waitForFunction(() => document.querySelector("[data-body] [data-tier]")?.getAttribute("data-recipe") === "survey-contours", null, { timeout: 10000 });
    await gm.waitForTimeout(300);
    await shot(gm, `render-contours-${name}`);
    await drawOn(gm, "auto");
    await useRecipe(gm, "survey-relief");
    if (auto.tier === "gpu-basic") await gm.waitForFunction(() => document.querySelector("[data-body] [data-tier]")?.getAttribute("data-tier") === "gpu-basic", null, { timeout: 10000 });
    await gm.getByRole("button", { name: "Night chart" }).click();
    await gm.waitForTimeout(500);
    await shot(gm, `render-night-${name}`);
    await gm.getByRole("button", { name: "Chart room" }).click();
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash, "drawing never changes the state hash");
    console.log(`render e2e ${name}: tiers ok (auto chose ${auto.tier}/${auto.backend}; ${parity})`);

    // The governor on an emulated low-end phone (Chromium only: CPU throttling is a CDP feature).
    if (name === "chromium") {
      for (const tier of ["canvas", "auto"]) {
        const ctx = await browser.newContext({ ...devices["Moto G4"] });
        const ph = await ctx.newPage();
        ph.on("pageerror", (e) => errors.push(e.message));
        await openEarth(ph);
        const cdp = await ctx.newCDPSession(ph);
        await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 });
        await drawOn(ph, tier);
        await ph.getByLabel("Spin").check();
        // Let it run: about 15 s of spinning frames.
        await ph.waitForTimeout(15000);
        const g = globe(ph);
        const ms = Number(await g.getAttribute("data-frame-ms"));
        const budget = Number(await g.getAttribute("data-budget-ms"));
        const level = Number(await g.getAttribute("data-level"));
        const frames = Number(await g.getAttribute("data-frames"));
        const used = `${await g.getAttribute("data-tier")}/${await g.getAttribute("data-backend")}`;
        const badgeText = await ph.locator("[data-badge]").innerText();
        await shot(ph, `render-phone-${tier}`);
        const work = await g.getAttribute("data-work-ms");
        console.log(`  work per frame ${work} ms`);
        console.log(`  Moto G4 emulation, 4x CPU, ${tier} (${used}): level ${level}, smoothed frame ${ms} ms against ${budget} ms, ${frames} frames; badge "${badgeText}"`);
        assert.ok(frames > 20, "it kept drawing");
        assert.ok(ms <= budget * 1.15, `the governor holds the budget: ${ms} ms against ${budget} ms at level ${level}`);
        if (level > 0) assert.match(badgeText, /Reduced:/);
        await ctx.close();
      }
    }
    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
  }
} finally {
  server.kill();
}
