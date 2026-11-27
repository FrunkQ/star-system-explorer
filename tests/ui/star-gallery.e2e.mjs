// The star class gallery (SC1): Reference › Classes › "What kind of star is this?" draws every star kind live in True
// through one GPU canvas; Same size and Actual size; a star opened shows large with its cells in words; a remnant with
// no picture yet says so. Then Make a system › Grow a new system › Choose the star: the gallery as the picker.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const screens = process.env.SCREENS ?? "";
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/#chart`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: false }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 20000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await go(gm, "reference");
    await gm.locator('[data-page="classes"]').click();
    await gm.locator("[data-classes]").first().waitFor({ timeout: 20000 });
    await gm.locator('[data-scheme="star"]').click();
    const g = gm.locator("[data-star-gallery]");
    await g.waitFor({ timeout: 20000 });
    // Every tile's star read; the drawable ones drawn on the one canvas.
    await gm.waitForFunction(() => document.querySelectorAll('[data-star-state="reading"]').length === 0 && document.querySelectorAll("[data-star-class]").length > 0, null, { timeout: 60000 });
    const gpu = await g.getAttribute("data-gpu");
    const states = await g.locator("[data-star-class]").evaluateAll((els) => els.map((e) => [e.dataset.starClass, e.dataset.starState, e.dataset.starWhy ?? ""]));
    console.log(vp, "gpu", gpu, JSON.stringify(states));
    const drawn = states.filter((s) => s[1] === "drawn").length;
    assert.ok(drawn >= 10, `most kinds drawn (${drawn})`);
    // Remnants with no picture yet say so, never a stand-in disc.
    for (const fam of ["neutron-star", "black-hole"]) {
      await g.locator(`[data-family="${fam}"]`).click();
      const rem = await g.locator("[data-star-class]").evaluateAll((els) => els.map((e) => [e.dataset.starClass, e.dataset.starState]));
      assert.ok(rem.length > 0, `${fam} has tiles`);
      for (const s of rem) assert.equal(s[1], "none", `${s[0]} is not drawn yet`);
    }
    await g.getByRole("button", { name: "All", exact: true }).click();
    if (gpu === "ready") await gm.waitForFunction(() => Number(document.querySelector("[data-star-gallery]")?.getAttribute("data-drawn")) >= 10, null, { timeout: 20000 });
    await shot(gm, `stars-same-${vp}`);
    await g.locator('[data-size-pick="actual"]').click();
    assert.equal(await g.getAttribute("data-size"), "actual");
    assert.match(await g.locator("[data-scale]").innerText(), /largest on this page/);
    await shot(gm, `stars-actual-${vp}`);
    await g.locator('[data-size-pick="same"]').click();
    // The red supergiant opened: its cells in words, and they show at this size. Its kind is chosen first, since
    // the star scheme has more classes than one page holds.
    await g.locator('[data-family="supergiant"]').click();
    const rsg = g.locator('[data-star-class="star/supergiant/M"]');
    await rsg.click();
    const card = gm.locator("[data-star-card]");
    await card.waitFor();
    await card.locator("canvas").waitFor();
    const words = await card.innerText();
    assert.match(words, /cells across its face/);
    assert.match(words, /so they show/);
    await shot(gm, `stars-supergiant-${vp}`);
    await card.locator('[data-zoom="2"]').click();
    await shot(gm, `stars-supergiant-x2-${vp}`);
    // A K giant up close.
    await g.locator('[data-family="giant"]').click();
    await g.locator('[data-star-class="star/giant/K"]').click();
    await card.locator('[data-zoom="2"]').click();
    await shot(gm, `stars-red-giant-x2-${vp}`);

    // One star's life (Alex, 26 Nov): the red giant followed back to its birth, its stages in the life line's order,
    // Earth's orbit to the same scale; then a heavy star, whose remnant is not drawn yet.
    await card.locator("[data-star-follow]").click();
    const life = gm.locator("[data-star-life]");
    await life.waitFor();
    await gm.waitForFunction(() => document.querySelector("[data-star-life]")?.getAttribute("data-read") === "done", null, { timeout: 60000 });
    const phases = await life.locator("[data-life-stage]").evaluateAll((els) => els.map((e) => e.dataset.lifeStage));
    console.log(vp, "life of 1 solar mass", JSON.stringify(phases));
    assert.ok(phases.length >= 4 && phases[0] === "pre-main-sequence" && phases.includes("main-sequence") && phases.at(-1) === "white-dwarf", phases.join(","));
    await life.locator('[data-life-stage="main-sequence"]').click();
    assert.match(await life.locator("[data-life-now]").innerText(), /main sequence/);
    await shot(gm, `life-sun-main-${vp}`);
    await life.locator('[data-fit="star"]').click();
    await shot(gm, `life-sun-main-star-${vp}`);
    await life.locator('[data-fit="life"]').click();
    await life.locator('[data-life-stage="giant"]').click();
    await life.locator("input[type=range]").nth(1).evaluate((el, v) => ((el.value = String(v)), el.dispatchEvent(new Event("input", { bubbles: true }))), phases.indexOf("giant") + 0.97);
    // The giant's tip is the giant's own size, never blended towards the white dwarf after it.
    assert.match(await life.locator("[data-life-scale]").innerText(), /Its radius is 0\.[5-9]/);
    await shot(gm, `life-sun-giant-tip-${vp}`);
    await life.locator('[data-fit="earth"]').click();
    await shot(gm, `life-sun-giant-earth-${vp}`);
    await life.locator("[data-life-mass]").evaluate((el) => ((el.value = String(Math.log10(20))), el.dispatchEvent(new Event("input", { bubbles: true })), el.dispatchEvent(new Event("change", { bubbles: true }))));
    await gm.waitForFunction(() => document.querySelector("[data-star-life]")?.getAttribute("data-read") === "done" && document.querySelector('[data-life-stage="neutron-star"],[data-life-stage="black-hole"]'), null, { timeout: 60000 });
    await life.locator('[data-fit="life"]').click();
    await life.locator('[data-life-stage="giant"]').click();
    await shot(gm, `life-20-giant-${vp}`);
    await life.locator("[data-life-stage]").last().click();
    await life.locator("[data-not-drawn]").first().waitFor();
    await life.locator("[data-life-play]").click();
    await shot(gm, `life-20-playing-${vp}`);
    await gm.locator('[data-star-view="kinds"]').click();
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "looking never changes the universe");

    // The gallery as the star picker (Alex, 26 Nov): Make a system › Grow a new system › Choose the star.
    if (vp === "desk") {
      await more(gm, "Make a system");
      const dlg = gm.getByRole("dialog", { name: "Make a system" });
      await dlg.getByRole("tab", { name: "Grow a new system" }).click();
      await dlg.locator("[data-star-gallery-open]").click();
      const pickDlg = gm.getByRole("dialog", { name: "Choose the star" });
      await pickDlg.locator('[data-star-class="star/main-sequence/K"][data-star-state="drawn"]').waitFor({ timeout: 60000 });
      await pickDlg.locator('[data-star-class="star/main-sequence/K"]').click();
      await shot(gm, "picker-gallery-desk");
      await pickDlg.locator("[data-star-pick]").click();
      await dlg.locator('[data-picked="star/main-sequence/K"]').waitFor();
      await dlg.getByLabel("Seed", { exact: true }).fill("orange dawn");
      await dlg.locator("[data-grow-picked]").click();
      const result = dlg.getByLabel("Formation result");
      await result.waitFor({ timeout: 30000 });
      assert.match(await result.innerText(), /0\.79 M☉, 0\.19 Gyr old/, "grown at the picked star's own mass and age (the K main-sequence sample, star-classes.json)");
      await shot(gm, "picker-grown-desk");
      // Fill it in myself: a system with the picked star alone, added to the campaign.
      await dlg.getByLabel("Seed", { exact: true }).fill("lonely orange");
      await dlg.locator("[data-alone-picked]").click();
      // Added: the dialog gives way to the chart, which shows the new system with its one star.
      await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash0, { timeout: 20000 });
      await gm.waitForFunction(() => document.querySelectorAll(".area-chart .body").length >= 1, null, { timeout: 15000 });
      console.log("viewing", await gm.getByLabel("System on the chart").getAttribute("data-viewing"), "bodies", await gm.locator(".area-chart .body").count());
      await shot(gm, "picker-alone-desk");
      assert.notEqual(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "adding the star writes history");
    }
    assert.deepEqual(errors, [], errors.join("\n"));
    await browser.close();
  }
  console.log("star-gallery.e2e ok");
} finally {
  server.kill();
}
