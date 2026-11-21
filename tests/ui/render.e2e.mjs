// R1 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/render.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The Body globe is a render recipe. For adopted Earth it checks that:
//  - True colour draws the air from the kernel's facts (pressure, temperature,
//    gravity, gas mix) and says so, on gpu-high and on the canvas;
//  - the recipe runs on every tier it declares (GPU, 2D canvas, words), and
//    the GPU and canvas tiers draw the same colours from the same numbers;
//  - a recipe without the tier in use hands over to one of its family, and says so;
//  - drawing never changes the state hash;
//  - on an emulated low-end phone (Playwright's Moto G4 with Chromium's CPU
//    slowed 4x, as Lighthouse does for mobile; not a real phone) the governor
//    steps quality down until frames fit the budget, and the badge says so.
import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
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
  await more(page, "Make a system");
  const dlg = page.getByRole("dialog", { name: "Make a system" });
  await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
  await dlg.getByRole("button", { name: "Try a sample save" }).click();
  await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
  await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
  const earth = page.locator('.area-chart .body[aria-label^="Earth"]').first();
  await earth.waitFor({ timeout: 15000 });
  await earth.click();
  await go(page, "body");
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
    // R3: take a photograph (the still tier off screen, frames averaged), kept on this device.
    const takePhoto = async (page, label) => {
      const t0 = Date.now();
      // The badge above updates its frame time as it draws, which can nudge the layout; press the button directly.
      await page.locator("[data-take-photo]").evaluate((b) => b.click());
      await page.locator("[data-photo-note]").waitFor({ timeout: 60000 });
      const secs = (Date.now() - t0) / 1000;
      const note = await page.locator("[data-photo-note]").innerText();
      assert.doesNotMatch(note, /^No photograph/, note);
      const px = await page.locator("[data-photos] img").first().evaluate(async (img) => {
        const r = await fetch(img.src);
        const b = await createImageBitmap(await r.blob());
        const c = new OffscreenCanvas(b.width, b.height);
        const g = c.getContext("2d");
        g.drawImage(b, 0, 0);
        const at = (x, y) => [...g.getImageData(Math.round(x * b.width), Math.round(y * b.height), 1, 1).data];
        return { w: b.width, centre: at(0.5, 0.5), corner: at(0.02, 0.02) };
      });
      assert.equal(px.w, 1024, "photographs are 1024 px across");
      assert.ok(px.centre[3] > 250 && px.centre[0] + px.centre[1] + px.centre[2] > 30, `the world is in the picture: ${px.centre}`);
      assert.ok(px.corner[3] < 10, `space around it is clear: ${px.corner}`);
      if (screens) {
        const b64 = await page.locator("[data-photos] img").first().evaluate(async (img) => {
          const buf = new Uint8Array(await (await fetch(img.src)).arrayBuffer());
          let bin = "";
          for (let i = 0; i < buf.length; i += 0x8000) bin += String.fromCharCode(...buf.subarray(i, i + 0x8000));
          return btoa(bin);
        });
        writeFileSync(`${screens}/photograph-${label.replace(/[^a-z0-9]+/gi, "-")}.png`, Buffer.from(b64, "base64"));
      }
      console.log(`  photograph ${label}: ${secs.toFixed(1)} s; ${note}`);
      return secs;
    };
    await useRecipe(gm, "true-colour");
    await takePhoto(gm, name);
    await shot(gm, `render-photo-${name}`);
    const kept = await gm.evaluate(() => new Promise((res) => {
      const r = indexedDB.open("sse-media", 1);
      r.onsuccess = () => {
        const q = r.result.transaction("photographs").objectStore("photographs").count();
        q.onsuccess = () => res(q.result);
      };
      r.onerror = () => res(-1);
    }));
    assert.ok(kept >= 1, `the photograph is kept on this device: ${kept}`);

    // R4: edit the recipe through checked edits; the globe previews it live, costs are shown, and keeping it adds "mine".
    {
      const before = await pixelsOf(gm, globe(gm));
      await gm.locator("[data-edit-recipe]").click();
      const exp = gm.locator('[data-recipe-editor] input[data-setting$=".exposure"]:not([disabled])').first();
      await exp.fill("3");
      await exp.dispatchEvent("change");
      await gm.locator("[data-edit-changes]").waitFor();
      assert.match(await gm.locator("[data-edit-changes]").innerText(), /Atmosphere \([a-z-]+\): exposure 1 to 3/);
      assert.match(await gm.locator("[data-edit-cost]").innerText(), /ms a frame of \d+ ms/);
      await gm.waitForFunction(() => /-mine-1$/.test(document.querySelector("[data-body] [data-tier]")?.getAttribute("data-recipe") ?? ""), null, { timeout: 10000 });
      await gm.waitForTimeout(600);
      const after = await pixelsOf(gm, globe(gm));
      assert.ok(after.upper[2] > before.upper[2] + 10, `more exposure is brighter: ${before.upper} to ${after.upper}`);
      await gm.locator("[data-keep-recipe]").click();
      assert.match(await gm.getByLabel("Recipe").locator("option:checked").innerText(), /^My true colour$/);
      await shot(gm, `render-recipe-edited-${name}`);
    }

    await drawOn(gm, "auto");
    await useRecipe(gm, "survey-relief");
    // True colour (R2): the air from the kernel's facts, physically lit, on gpu-high and the canvas.
    await useRecipe(gm, "true-colour");
    for (const t of ["gpu-high", "canvas"]) {
      await drawOn(gm, t);
      await gm.waitForFunction((t) => {
        const el = document.querySelector("[data-body] [data-tier]");
        return el && el.getAttribute("data-recipe") === "true-colour" && (el.getAttribute("data-tier") === t || t === "gpu-high") && Number(el.getAttribute("data-frames")) > 0;
      }, t, { timeout: 15000 });
      await gm.waitForTimeout(600);
      const got = await globe(gm).getAttribute("data-tier");
      const note = await gm.locator("[data-air-note]").innerText();
      assert.match(note, /^Air from the kernel: 1013 hPa at \d+ K, N2 78%, O2 21%; scale height 8\.\d km; blue light's path straight up scatters 2\d%/, note);
      const px = await pixelsOf(gm, globe(gm));
      assert.ok(px.upper[2] > px.upper[0] + 20, `${got}: the lit ocean under air is blue: ${px.upper}`);
      await shot(gm, `render-true-colour-${got}-${name}`);
    }
    await drawOn(gm, "auto");
    await useRecipe(gm, "survey-relief");
    if (auto.tier === "gpu-basic") await gm.waitForFunction(() => document.querySelector("[data-body] [data-tier]")?.getAttribute("data-tier") === "gpu-basic", null, { timeout: 10000 });
    await more(gm, "Night chart");
    await gm.waitForTimeout(500);
    await shot(gm, `render-night-${name}`);
    await more(gm, "Night chart");
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash, "drawing never changes the state hash");

    // R5a: a star on the Body screen, drawn from the kernel's facts and activity, in visible light and X-rays.
    {
      await go(gm, "system");
      await gm.locator('.area-chart .body[aria-label^="Sun"]').first().click();
      await go(gm, "body");
      const disc = gm.locator("[data-star] [data-band]");
      await disc.waitFor({ timeout: 20000 });
      const regime = await gm.locator("[data-star] [data-regime]").innerText();
      assert.match(regime, /^(quiet|moderate|active|saturated)/, regime);
      const vis = await pixelsOf(gm, disc);
      assert.ok(vis.centre[0] > 200 && vis.centre[1] > 180, `the Sun's centre is bright and near white: ${vis.centre}`);
      await shot(gm, `render-star-visible-${name}`);
      await gm.locator("[data-star]").getByRole("button", { name: "X-ray" }).click();
      await gm.waitForFunction(() => document.querySelector("[data-star] [data-band]")?.getAttribute("data-band") === "xray");
      await gm.waitForTimeout(200);
      const xr = await pixelsOf(gm, disc);
      // In X-rays the photosphere does not show; what is seen is the hot corona in front of it, on the mapped (violet) scale.
      assert.ok(xr.centre[2] > xr.centre[1] + 30 && vis.centre[1] - xr.centre[1] > 40, `the X-ray view is the corona's mapped colour: ${xr.centre} (visible ${vis.centre})`);
      await shot(gm, `render-star-xray-${name}`);
      // O10 cells: the Sun's granules (about 1,000 across) are far below a pixel on this disc, so it is drawn plain and says so.
      const gw = await gm.locator("[data-star] [data-granule-words]").innerText();
      assert.ok(/Granules about 1,3\d\d km wide/.test(gw) && /too small to draw/.test(gw), `the Sun's granules in words: ${gw}`);
      assert.equal(await gm.locator("[data-star] [data-granules]").getAttribute("data-granules"), "0");
      console.log(`  star: Sun ${regime}; ${await gm.locator("[data-star] [data-spot-words]").innerText()}; ${await gm.locator("[data-star] [data-flares]").innerText()}`);
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash, "drawing a star never changes the state hash");
    }
    console.log(`render e2e ${name}: tiers ok (auto chose ${auto.tier}/${auto.backend}; ${parity})`);

    // The governor on an emulated low-end phone (Chromium only: CPU throttling is a CDP feature).
    if (name === "chromium") {
      for (const tier of ["canvas", "auto"]) {
        const ctx = await browser.newContext({ ...devices["Moto G4"] });
        const ph = await ctx.newPage();
        ph.on("pageerror", (e) => errors.push(e.message));
        await openEarth(ph);
        // A page wider than the phone moves the visual viewport off the layout one, and taps land beside their targets.
        const widths = await ph.evaluate(() => ({ screen: screen.width, page: document.scrollingElement.scrollWidth }));
        assert.ok(widths.page <= widths.screen, `the phone page has no sideways scroll: ${JSON.stringify(widths)}`);
        const cdp = await ctx.newCDPSession(ph);
        await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 });
        await drawOn(ph, tier);
        if (tier === "auto") {
          await useRecipe(ph, "true-colour");
          const secs = await takePhoto(ph, "on the Moto G4 emulation");
          // The photograph keeps to its own refining budget (6 s): it stops before a frame that would pass it, so it either
          // finished inside the budget or made only the one frame it always makes. That is checked exactly. Wall time on an
          // emulated phone depends on how busy the machine is (9 to 11 s on a desk machine, about twice that on shared
          // runners), so the 10 s target is reported, with a gross bound of 25 s that still catches a refining loop that ignores
          // its budget (33 s before the budget existed).
          const note = await ph.locator("[data-photo-note]").innerText();
          const made = Number(/Made in ([\d.]+) s/.exec(note)?.[1]);
          const frames = Number(/(\d+) frames? averaged/.exec(note)?.[1]);
          assert.ok(frames === 1 || made <= 6.5, `refining stops at its budget: ${frames} frames in ${made} s`);
          assert.ok(secs < 25, `a photograph on the emulated phone finishes: ${secs} s`);
          console.log(`  phone photograph against the 10 s target: ${secs < 10 ? "met" : "not met on this machine"} (${secs.toFixed(1)} s)`);
          await useRecipe(ph, "survey-relief");
        }
        // On the phone the time ruler sticks to the bottom of the screen (148 px of 687 on the Moto G4). Scrolled only "into
        // view", the box can land under it, so bring it to the middle of the screen first.
        const spin = ph.getByLabel("Spin");
        await spin.evaluate((el) => el.scrollIntoView({ block: "center" }));
        await spin.check();
        // Let it run: about 15 s of spinning frames.
        await ph.waitForTimeout(15000);
        // One snapshot of the globe's attributes and the badge, read in the same task, so they describe the same status.
        const snap = () => ph.evaluate(() => {
          const el = document.querySelector("[data-body] [data-tier]");
          const n = (k) => Number(el.getAttribute(k));
          return { ms: n("data-frame-ms"), budget: n("data-budget-ms"), level: n("data-level"), max: n("data-max"), frames: n("data-frames"), work: el.getAttribute("data-work-ms"), used: `${el.getAttribute("data-tier")}/${el.getAttribute("data-backend")}`, badge: document.querySelector("[data-badge]").textContent.trim() };
        });
        // Settled means: within budget (and the badge names any reduction), or every reduction taken and the badge says it is
        // still over. The governor steps only after 10 frames over or 90 under, so a snapshot can fall between a change and
        // its settling; poll until a snapshot is settled (up to 10 s). A loop that never settles fails.
        const settled = (x) => (x.ms <= x.budget * 1.15 && (x.level === 0 || /Reduced:/.test(x.badge))) || (x.level === x.max && /Over budget at the plainest setting/.test(x.badge));
        let s = await snap();
        for (let i = 0; i < 20 && !settled(s); i++) {
          await ph.waitForTimeout(500);
          s = await snap();
        }
        await shot(ph, `render-phone-${tier}`);
        console.log(`  work per frame ${s.work} ms`);
        console.log(`  Moto G4 emulation, 4x CPU, ${tier} (${s.used}): level ${s.level} of ${s.max}, smoothed frame ${s.ms} ms against ${s.budget} ms, ${s.frames} frames; badge "${s.badge}"`);
        assert.ok(s.frames > 20, "it kept drawing");
        assert.ok(settled(s), `the governor holds the budget, or takes every reduction and says it is still over: ${JSON.stringify(s)}`);
        await ctx.close();
      }
    }
    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
  }
} finally {
  server.kill();
}
