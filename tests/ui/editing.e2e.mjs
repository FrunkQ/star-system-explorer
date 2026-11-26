// The handles row and the strip (ED1, editing design sections 1 to 3), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/editing.e2e.mjs [--screens <dir>]
// Open the Solar System, pick Mars. On Explore its card shows the six handles with the kernel's figures, and the
// Orbit strip opens with the reference worlds, live as a try. On Build, typing "1 AU" in the strip is one change: the
// handle says 1 AU, the card lists what followed (Mars warms), the state hash moves, and Undo puts it back. Dragging
// the mark and letting go is one change too; Esc while dragging writes nothing. On a phone nothing runs off the side.
// ED12, the Orbit and Spin boxes at Science depth: the year and the stability verdict under the orbit; a year typed on
// the size strip is the distance with that year; Retrograde mirrors the lean past 90 degrees; the place now is a ring
// that catches at the far point; Locked sets the day to the year and the sun's day says so; each one change, one Undo.
// TM4: on Explore the strip is live as a try: the first change opens a what-if by itself and the main line does not move; on
// Build, Add a moon offers formation's own rules for the chosen planet, and Add round a star offers Kinds; Pluto's Orbit box shows its pair with Charon; on a phone the open strip docks above the ruler.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { depth, desk } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9500 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => (errors.push(e.message), console.log("PAGEERROR", e.stack?.slice(0, 600))));
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    const mars = gm.locator('.area-chart .body[aria-label^="Mars,"]').first();
    await mars.waitFor({ timeout: 30000 });
    await mars.locator(".hit").click();

    // Explore: the row with the kernel's figures, the strip live as a try.
    const row = gm.locator('[data-handles="world"]');
    await row.waitFor({ timeout: 20000 });
    assert.deepEqual(await row.locator("[data-handle]").evaluateAll((bs) => bs.map((b) => b.dataset.handle)), ["orbit", "mass", "makeup", "liquids", "air", "spin"]);
    await gm.waitForFunction(() => /AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
    assert.match(await row.locator('[data-handle="orbit"] .value').innerText(), /1\.52 AU/);
    assert.match(await row.locator('[data-handle="air"] .value').innerText(), /6\.36 mbar/);
    await row.locator('[data-handle="orbit"]').click();
    const strip = gm.locator('[data-strip="orbit"]');
    await strip.waitFor();
    assert.ok(!(await strip.locator("[data-strip-figure]").isDisabled()), "live on Explore, as a try");
    assert.equal(await strip.locator("[data-strip-trying]").count(), 1, "the strip says a change is a try");
    assert.equal(await gm.locator(".whatif b").count(), 0, "no what-if before the first change");
    for (const n of ["Earth", "Jupiter"]) assert.equal(await strip.locator(`[data-dot="${n}"]`).count(), 1, `${n} is a reference dot`);
    await shot(gm, `explore-orbit-${vp}`);
    if (vp === "phone") {
      // ED7 phone dock: the open strip docks above the time ruler, and the mark is where the thumb can reach it.
      const [dock, ruler, mark] = await gm.evaluate(() => [document.querySelector("[data-dock]"), document.querySelector('footer[aria-label="Time"]'), document.querySelector('[data-strip="orbit"] .mark')].map((e) => e?.getBoundingClientRect().toJSON()));
      assert.ok(dock && ruler && mark, "dock, ruler and mark");
      assert.ok(dock.bottom <= ruler.top + 1, `the dock sits above the ruler (${dock.bottom} against ${ruler.top})`);
      assert.ok(mark.top >= dock.top && mark.bottom <= dock.bottom, "the mark is in view in the dock");
    }

    // Press a unit (ED1a): the card's Orbit steps to the next reading and the handle follows, for every planet; round
    // the readings it comes back to AU.
    const cardOrbit = gm.locator("section[aria-label='Mars'] .field").filter({ hasText: /^(orbit|how far)/i }).first();
    const unitButton = cardOrbit.locator("[data-unit]");
    {
      assert.equal(await unitButton.count(), 1, "the card's orbit has a unit to press");
      assert.equal(await unitButton.innerText(), "AU");
      await unitButton.click();
      await gm.waitForFunction(() => !/AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 5000 });
      const other = await cardOrbit.locator("[data-unit]").innerText();
      assert.notEqual(other, "AU");
      assert.ok((await row.locator('[data-handle="orbit"] .value').innerText()).endsWith(other), "the handle says it in the chosen unit too");
      for (let i = 0; i < 5 && (await cardOrbit.locator("[data-unit]").innerText()) !== "AU"; i++) await cardOrbit.locator("[data-unit]").click();
      assert.equal(await cardOrbit.locator("[data-unit]").innerText(), "AU", "round the readings and back");
    }

    // TM4: on Explore the first change opens a what-if by itself; the main line does not move; Throw away puts the
    // world back.
    {
      if (!(await strip.isVisible())) await row.locator('[data-handle="orbit"]').click();
      const main = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      await strip.locator("[data-strip-figure]").click();
      await strip.locator("[data-strip-entry]").fill("1 AU");
      await strip.locator("[data-strip-entry]").press("Enter");
      await gm.waitForFunction(() => /Trying it/.test(document.querySelector(".whatif b")?.textContent ?? ""), null, { timeout: 20000 });
      await gm.waitForFunction(() => /(^|about )1 AU$/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      assert.match(await gm.locator("[data-followed]").innerText(), /Trying it only/);
      // ED1 part 3: the branch's own resolve (BC1) says what followed there: Mars at 1 AU warms.
      await gm.locator('[data-followed-row="Surface"]').waitFor({ timeout: 20000 });
      const [k0, k1] = (await gm.locator('[data-followed-row="Surface"]').innerText()).match(/\d+(?= K)/g).map(Number);
      assert.ok(k1 > k0 + 20, `Mars at 1 AU is warmer on the example: ${k0} K to ${k1} K`);
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), main, "the main line does not move");
      await shot(gm, `explore-example-${vp}`);
      await gm.locator(".whatif button", { hasText: "Throw away" }).click();
      await gm.waitForFunction(() => /1\.52 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      assert.equal(await gm.locator(".whatif b").count(), 0, "thrown away: no what-if open");
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), main, "the main line is as it was");
    }

    // Build: type "1 AU", one change, what followed, Undo.
    await desk(gm, "Build");
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    if (!(await strip.isVisible())) await row.locator('[data-handle="orbit"]').click();
    await strip.locator("[data-strip-figure]").click();
    const entry = strip.locator("[data-strip-entry]");
    await entry.fill("1 AU");
    await entry.press("Enter");
    const followed = gm.locator("[data-followed]");
    await followed.waitFor({ timeout: 20000 }).catch(async (e) => {
      throw new Error(`${e.message}\nstrip says: ${await strip.innerText()}`);
    });
    assert.match(await followed.innerText(), /(Orbit|How far out)\s+(about )?1\.52 AU → 1 AU/);
    assert.match(await followed.locator('[data-followed-row="Surface"]').innerText(), /K → (about )?\d+ K/);
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash0);
    await gm.waitForFunction(() => /(^|about )1 AU$/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
    await shot(gm, `build-followed-${vp}`);
    await gm.keyboard.press("Control+z");
    await gm.waitForFunction(() => /1\.52 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });

    // Drag the mark and let go: one change. Esc during a drag writes nothing.
    if (!(await strip.isVisible())) await row.locator('[data-handle="orbit"]').click();
    const track = strip.locator(".track");
    await track.scrollIntoViewIfNeeded();
    const box = await track.boundingBox();
    const hash1 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await gm.mouse.move(box.x + box.width * 0.5, box.y + 15);
    await gm.mouse.down();
    await gm.mouse.move(box.x + box.width * 0.7, box.y + 15, { steps: 5 });
    assert.ok(await strip.locator(".figure.pencil").count(), "the figure is in pencil while dragging");
    // ED1 part 3b: held still, the exact answer for the held value comes back from a scratch branch; nothing is written.
    await strip.locator("[data-strip-pause]").waitFor({ timeout: 20000 });
    assert.match(await strip.locator("[data-strip-pause]").innerText(), /Held here: .*(K|°C|nothing else)/);
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash1, "a held value writes nothing");
    await shot(gm, `held-${vp}`);
    await gm.mouse.up();
    await followed.waitFor({ timeout: 20000 });
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash1);
    const hash2 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await strip.locator("[data-strip-mark]").focus();
    await gm.keyboard.press("ArrowRight");
    await gm.keyboard.press("Escape");
    await gm.waitForTimeout(400);
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash2, "Esc writes nothing");
    assert.equal(await row.count(), 1, "the strip's Esc leaves the world selected");

    // ED3 part 1, the chart: dragging a world shows a ghost and the distance as the engine says it; the world stays
    // put; Esc puts it back and writes nothing.
    if (vp === "desk") {
      await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
      await gm.keyboard.press("Control+z");
      await gm.waitForFunction(() => /1\.52 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      await gm.waitForTimeout(1000); // the chart redraws Mars back at 1.52 AU after the card
      const hit = gm.locator('.area-chart .body[aria-label^="Mars,"] .hit').first();
      const sun = gm.locator('.area-chart .body[aria-label^="Sun"] .hit').first();
      const mb = await hit.boundingBox();
      const sb = await sun.boundingBox();
      const [mx, my, sx, sy] = [mb.x + mb.width / 2, mb.y + mb.height / 2, sb.x + sb.width / 2, sb.y + sb.height / 2];
      const hash3 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      await gm.mouse.move(mx, my);
      await gm.mouse.down();
      for (let k = 1; k <= 6; k++) await gm.mouse.move(mx + ((mx - sx) * 0.3 * k) / 6, my + ((my - sy) * 0.3 * k) / 6);
      await gm.locator("[data-ghost]").waitFor({ timeout: 5000 });
      assert.match(await gm.locator("[data-drag-readout]").textContent(), /AU( · \d+:\d+ with \S+)?$/);
      // ED3 part 2: Alt runs free of the catches; the figure is the drag's own.
      await gm.keyboard.down("Alt");
      await gm.mouse.move(mx + (mx - sx) * 0.33, my + (my - sy) * 0.33);
      await gm.waitForTimeout(100);
      assert.equal(await gm.locator("[data-drag-readout]").getAttribute("data-caught"), null, "Alt: no catch");
      await gm.keyboard.up("Alt");
      const still = await hit.boundingBox();
      assert.ok(Math.abs(still.x - mb.x) < 1 && Math.abs(still.y - mb.y) < 1, `the world stays put while the ghost moves: ${JSON.stringify([mb, still])}`);
      await shot(gm, `chart-drag-${vp}`);
      await gm.keyboard.press("Escape");
      await gm.mouse.up();
      await gm.waitForTimeout(400);
      assert.equal(await gm.locator("[data-ghost]").count(), 0, "Esc takes the ghost away");
      assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash3, "Esc on the chart writes nothing");
    }

    // ED12: the boxes, at Science depth on the Build desk.
    if (vp === "desk") {
      await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
      await depth(gm, "Science");
      const hash = () => gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      // Undo is a change of its own (the hash moves on); the figure it names comes back.
      const undoTo = async (back) => {
        const h = await hash();
        await gm.evaluate(() => (document.activeElement instanceof HTMLElement ? document.activeElement.blur() : undefined));
        await gm.keyboard.press("Control+z");
        await gm.waitForFunction((x) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== x, h, { timeout: 20000 });
        if (back) await gm.waitForFunction(([sel, re]) => new RegExp(re).test(document.querySelector(sel)?.textContent ?? ""), back, { timeout: 20000 });
        assert.equal(await gm.getByText("Nothing to undo").count(), 0, "each Undo takes back the box's own change");
      };
      const marsAt = ['[data-handle="orbit"] .value', "1\\.52 AU"];
      if (!(await gm.locator('[data-box="orbit"]').isVisible())) await row.locator('[data-handle="orbit"]').click();
      const obox = gm.locator('[data-box="orbit"]');
      await obox.waitFor();
      for (const p of ["orbit.shape", "orbit.lean"]) assert.equal(await obox.locator(`[data-strip="${p}"]`).count(), 1, `${p} strip`);
      for (const r of ["now", "peri", "node"]) assert.equal(await obox.locator(`[data-ring="${r}"]`).count(), 1, `${r} ring`);
      await obox.locator('[data-result="year"]').waitFor({ timeout: 20000 });
      assert.match(await obox.locator('[data-result="year"]').innerText(), /68\d (d|days)|1\.88 (years|yr)/);
      assert.match(await obox.locator('[data-result="stability"]').innerText(), /Lasts|Watch/);
      assert.match(await obox.locator('[data-strip="orbit.shape"] [data-strip-figure]').innerText(), /^0\.09/);
      // ED2, the sweep behind the strip (kernel K1): Mars's orbit run across the strip on a scratch branch marks where
      // its day locks and where it stops lasting; nothing is written.
      const hashSw = await hash();
      await obox.locator('[data-strip="orbit"] [data-tick="turns freely from here"]').waitFor({ state: "attached", timeout: 90000 }).catch(async (e) => {
        throw new Error(`${e.message}\nticks: ${await obox.locator("[data-tick]").evaluateAll((t) => t.map((x) => x.getAttribute("data-tick")))}; ${await gm.locator("[data-sweeping], [data-sweep-why]").allInnerTexts()}`);
      });
      assert.ok((await obox.locator('[data-strip="orbit"] [data-tick]').count()) >= 2, "more than one mark from the sweep");
      assert.equal(await hash(), hashSw, "the sweep writes nothing");
      await shot(gm, `orbit-box-${vp}`);

      // A year typed on the size: one Earth year puts Mars at 1 AU.
      await obox.locator('[data-strip="orbit"] [data-strip-figure]').click();
      await obox.locator('[data-strip="orbit"] [data-strip-entry]').fill("365.25 d");
      await obox.locator('[data-strip="orbit"] [data-strip-entry]').press("Enter");
      await gm.waitForFunction(() => /(^|about )1(\.00)? AU$/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nbox says: ${await obox.innerText()}`);
      });
      await undoTo(marsAt);

      // ED5 Reach it: ask for a 260 K surface; the box finds the orbit on a scratch what-if and sets it as one change.
      await obox.locator('[data-reach="t_surface_now_k"]').waitFor({ timeout: 20000 });
      await obox.locator('[data-reach="t_surface_now_k"] [data-reach-ask]').fill("260 K");
      await obox.locator('[data-reach="t_surface_now_k"] [data-reach-go]').click();
      await gm.waitForFunction(() => /surface (about )?2(59|60|61)/.test(document.querySelector("[data-reach-said]")?.textContent ?? ""), null, { timeout: 60000 }).catch(async (e) => {
        throw new Error(`${e.message}\nbox says: ${await obox.innerText()}`);
      });
      assert.match(await gm.locator('[data-handle="orbit"] .value').innerText(), /1\.[0-4]\d? AU/, "Mars moves in to warm");
      // ED5 part 2c: the orbit also reaches "locked to its star" (a yes-or-no result), offered beside the surface.
      assert.match(await obox.locator('[data-reach="locked"]').innerText(), /turns freely/);
      await shot(gm, `reach-it-${vp}`);
      await undoTo(marsAt);

      // ED5 part 2: the same on the Mass strip: a surface gravity of 1 g by changing the mass.
      await row.locator('[data-handle="mass"]').click();
      const mreach = gm.locator('[data-reach="gravity_m_s2"]');
      await mreach.waitFor({ timeout: 20000 });
      await mreach.locator("[data-reach-ask]").fill("9.81 m/s²");
      await mreach.locator("[data-reach-go]").click();
      await gm.waitForFunction(() => /gravity .*(9\.(79|8\d)|1(\.00)? g|0\.99\d* g)/.test(document.querySelector("[data-reach-said]")?.textContent ?? ""), null, { timeout: 60000 }).catch(async (e) => {
        throw new Error(`${e.message}\nreach says: ${await gm.locator("[data-reach-said]").allInnerTexts()}`);
      });
      await shot(gm, `reach-gravity-${vp}`);
      await undoTo();
      // A yes-or-no result: the lightest Mars that keeps a field of its own today.
      const dyn = gm.locator('[data-reach="core.dynamo_now"]');
      await dyn.waitFor({ timeout: 20000 });
      assert.match(await dyn.innerText(), /no field of its own/);
      await dyn.locator("[data-reach-go]").click();
      await gm.waitForFunction(() => /nearest that gives it a field of its own/.test(document.querySelector("[data-reach-said]")?.textContent ?? ""), null, { timeout: 90000 }).catch(async (e) => {
        throw new Error(`${e.message}\nreach says: ${await gm.locator("[data-reach-said]").allInnerTexts()}`);
      });
      await gm.waitForFunction(() => /a field of its own/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await shot(gm, `reach-field-${vp}`);
      await undoTo();
      // ED10 part 1, "Could be…": the card's line offers what Mars is not yet; the recipe is worked out on a scratch copy,
      // the smallest move first, and "Do this" is one change.
      await gm.locator('[data-could-be] [data-could="field"]').click();
      await gm.locator('[data-recipe="field"] [data-step="mass"]').waitFor({ timeout: 90000 });
      await shot(gm, `could-be-${vp}`);
      await gm.locator('[data-recipe="field"] [data-do="mass"]').click();
      await gm.waitForFunction(() => /to have a field of its own/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo();
      // ED10 part 2: "Stop suggesting this" takes one suggestion off this world's line, kept with the campaign; the rest stay.
      const hashQ = await hash();
      await gm.locator('[data-could-be] [data-could="locked"]').click();
      await gm.locator('[data-could-quiet="locked"]').click();
      await gm.waitForFunction(() => !document.querySelector('[data-could-be] [data-could="locked"]') && !!document.querySelector('[data-could-be] [data-could="field"]'), null, { timeout: 10000 });
      assert.equal(await hash(), hashQ, "stopping a suggestion writes nothing to the universe");

      // ED8 part 1, the Air box: Mars's air as one mix bar (carbon dioxide most of it), traces as chips in ppm. Setting
      // carbon dioxide to half trades it against the rest, which keep their proportions: one change, one Undo.
      await row.locator('[data-handle="air"]').click();
      const air = gm.locator("[data-air-box]");
      await air.locator('[data-mix-bar] [data-gas="co2"]').waitFor({ timeout: 20000 });
      assert.ok((await air.locator(".traces [data-gas]").count()) >= 3, "traces as chips");
      assert.match(await air.locator('.traces [data-gas="xe"]').innerText(), /ppm|ppb/);
      await air.locator('[data-mix-bar] [data-gas="co2"]').click();
      await air.locator('[data-strip="air.co2"] [data-strip-figure]').click();
      await air.locator('[data-strip="air.co2"] [data-strip-entry]').fill("50%");
      await air.locator('[data-strip="air.co2"] [data-strip-entry]').press("Enter");
      await gm.waitForFunction(() => /Air .*CO₂ 95.*→.*carbon dioxide 50/i.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nfollowed: ${await gm.locator("[data-followed]").allInnerTexts()}\nair: ${await air.innerText()}`);
      });
      await gm.waitForFunction(() => /N₂ 2[67]/.test(document.querySelector('[data-mix-bar] [data-gas="n2"]')?.textContent ?? ""), null, { timeout: 20000 });
      await shot(gm, `air-box-${vp}`);
      await undoTo(['[data-mix-bar] [data-gas="co2"]', "CO₂ 95"]);
      // ED8 part 2a: Kinds for air from the catalogued worlds (Earth's own pressure and mix on Mars), and Add a gas.
      await air.locator('[data-air-kind="earth"]').click();
      await gm.waitForFunction(() => /Like Earth/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await gm.waitForFunction(() => /N₂ 78/.test(document.querySelector('[data-mix-bar] [data-gas="n2"]')?.textContent ?? ""), null, { timeout: 20000 });
      await shot(gm, `air-kind-${vp}`);
      await undoTo(['[data-mix-bar] [data-gas="co2"]', "CO₂ 95"]);
      await air.locator("[data-add-gas]").selectOption("ch4");
      await gm.waitForFunction(() => /1,000 ppm|0\.1%/.test(document.querySelector('[data-air-box] [data-gas="ch4"]')?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo(['[data-mix-bar] [data-gas="co2"]', "CO₂ 95"]);

      // ED9 part 1, the Makeup box: Mars has no makeup set (the physics takes Earth's); "Like Mercury" from the class atlas
      // sets the five shares as one change, the radius the science expects for it is said in pencil, and a part's own strip
      // trades it against the rest. Each is one Undo.
      await row.locator('[data-handle="makeup"]').click();
      const mk = gm.locator("[data-makeup-box]");
      await mk.waitFor({ timeout: 10000 });
      await mk.locator('[data-makeup-radius]').waitFor({ timeout: 20000 });
      assert.match(await mk.locator("[data-makeup-radius]").innerText(), /expects a radius of/);
      await mk.locator('[data-makeup-kind="iron-rich:Mercury"]').click();
      await gm.waitForFunction(() => /Makeup .*Like Mercury/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nfollowed: ${await gm.locator("[data-followed]").allInnerTexts()}\nbox: ${await mk.innerText()}`);
      });
      await gm.waitForFunction(() => /metal 70/.test(document.querySelector('[data-makeup-bar] [data-part="metal"]')?.textContent ?? ""), null, { timeout: 20000 });
      assert.match(await row.locator('[data-handle="makeup"] .value').innerText(), /metal 70/);
      await mk.locator("[data-makeup-use-radius]").waitFor({ timeout: 20000 });
      await shot(gm, `makeup-box-${vp}`);
      await undoTo(['[data-makeup-box]', "No makeup is set"]);
      // ED2 part 2: with its makeup set (like Venus, a rocky world), the Mass strip's sweep labels each step with the class
      // atlas, so the strip marks where the world's class changes (past about ten Earth masses it is no longer rocky).
      await mk.locator('[data-makeup-kind="rocky:Venus"]').click();
      await gm.waitForFunction(() => /metal 32/.test(document.querySelector('[data-makeup-bar] [data-part="metal"]')?.textContent ?? ""), null, { timeout: 20000 });
      await row.locator('[data-handle="mass"]').click();
      await gm.locator('[data-strip="mass"] [data-tick*="rocky"]').first().waitFor({ state: "attached", timeout: 90000 }).catch(async (e) => {
        throw new Error(`${e.message}\nticks: ${await gm.locator("[data-tick]").evaluateAll((t) => t.map((x) => x.getAttribute("data-tick")))}; ${await gm.locator("[data-sweeping], [data-sweep-why]").allInnerTexts()}`);
      });
      await shot(gm, `mass-class-marks-${vp}`);
      await row.locator('[data-handle="makeup"]').click();
      await undoTo(['[data-makeup-box]', "No makeup is set"]);
      await mk.locator('[data-part-chip="ice"]').click();
      await mk.locator('[data-strip="makeup.ice"] [data-strip-figure]').click();
      await mk.locator('[data-strip="makeup.ice"] [data-strip-entry]').fill("40%");
      await mk.locator('[data-strip="makeup.ice"] [data-strip-entry]').press("Enter");
      await gm.waitForFunction(() => /ices 40/.test(document.querySelector('[data-makeup-bar] [data-part="ice"]')?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nbox: ${await mk.innerText()}`);
      });
      await undoTo(['[data-makeup-box]', "No makeup is set"]);

      // ED13, "Type it all in": every input as a row; what it understood before anything is written; one change, marked
      // catalogued with its source; one Undo.
      await row.locator('[data-handle="sheet"]').click();
      const sheet = gm.locator("[data-sheet]");
      await sheet.locator('[data-sheet-row="orbit.shape"]').waitFor({ timeout: 10000 });
      await sheet.locator('[data-sheet-entry="mass"]').fill("0.2 Earth masses");
      await sheet.locator('[data-sheet-entry="orbit.shape"]').fill("0.05");
      await sheet.locator('[data-sheet-entry="spin"]').fill("banana");
      assert.match(await sheet.locator('[data-sheet-understood="mass"]').innerText(), /0\.107 M⊕ → 0\.2 M⊕/);
      assert.equal(await sheet.locator('[data-sheet-problem="spin"]').count(), 1, "what it could not read");
      await sheet.locator('[data-sheet-entry="spin"]').fill("");
      await sheet.locator("[data-sheet-catalogued]").check();
      await sheet.locator("[data-sheet-source]").fill("A paper's table");
      const hashS = await hash();
      await sheet.locator("[data-sheet-set]").click();
      await gm.waitForFunction(() => /Typed in .*Mass 0\.2 M⊕/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await gm.waitForFunction(() => /0\.2 M⊕/.test(document.querySelector('[data-handle="mass"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      assert.notEqual(await hash(), hashS);
      await shot(gm, `sheet-${vp}`);
      await undoTo(['[data-handle="mass"] .value', "0\\.107"]);

      // ED13a, the system sheet: a pasted table for every body here, columns by name (a bracketed unit for bare
      // numbers), bodies by name; one change across them all, one Undo.
      if (!(await sheet.isVisible())) await row.locator('[data-handle="sheet"]').click();
      await sheet.locator("[data-sheet-whole]").click();
      // A spreadsheet saved as CSV (quoted cells, a byte-order mark) opens into the box to check first.
      await sheet.locator("[data-sheet-file]").setInputFiles({ name: "worlds.csv", mimeType: "text/csv", buffer: Buffer.from('\uFEFF"Name","Orbit (AU)"\r\n"Mars","1.7"\r\n') });
      await sheet.locator('[data-table-understood="Mars"]').waitFor({ timeout: 5000 });
      assert.match(await sheet.locator('[data-table-understood="Mars"]').innerText(), /1\.7 AU/);
      assert.match(await sheet.locator("[data-sheet-table]").inputValue(), /"Mars","1\.7"/);
      await shot(gm, `sheet-file-${vp}`);
      await sheet.locator("[data-sheet-table]").fill("Name\tOrbit (AU)\tMass\nMars\t1.6\t\nEarth\t\t1.1 M⊕\nVulcan\t0.1\t");
      await sheet.locator('[data-table-understood="Mars"]').waitFor({ timeout: 5000 });
      assert.match(await sheet.locator('[data-table-understood="Earth"]').innerText(), /1\.1 M⊕/);
      assert.ok(/Vulcan/.test(await sheet.locator("[data-table-problem]").allInnerTexts().then((t) => t.join(" "))), "a body not here is said");
      await sheet.locator("[data-sheet-catalogued]").uncheck();
      await sheet.locator("[data-sheet-set]").click();
      await gm.waitForFunction(() => /Typed in for the system .*2 figures for 2 bodies/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await gm.waitForFunction(() => /1\.6 AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? ""), null, { timeout: 20000 });
      await shot(gm, `system-sheet-${vp}`);
      await undoTo(marsAt);
      await row.locator('[data-handle="orbit"]').click();

      // Retrograde: the lean goes past 90 degrees, and the chip says so.
      await obox.locator('[data-chip="retrograde"]').click();
      await gm.waitForFunction(() => document.querySelector('[data-box="orbit"] [data-chip="retrograde"]')?.getAttribute("aria-pressed") === "true", null, { timeout: 20000 });
      assert.match(await obox.locator('[data-strip="orbit.lean"] [data-strip-figure]').innerText(), /18\d|17\d/);
      await gm.waitForFunction(() => /Lean/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo(['[data-box="orbit"] [data-chip="retrograde"]', "^Retrograde$"]);
      await gm.waitForFunction(() => document.querySelector('[data-box="orbit"] [data-chip="retrograde"]')?.getAttribute("aria-pressed") === "false");

      // Where it is now: typed 180 catches at the farthest point.
      await obox.locator('[data-ring="now"] [data-ring-figure]').click();
      await obox.locator('[data-ring="now"] [data-ring-entry]').fill("180");
      await obox.locator('[data-ring="now"] [data-ring-entry]').press("Enter");
      await gm.waitForFunction(() => /farthest/.test(document.querySelector('[data-ring="now"] [data-ring-caught]')?.textContent ?? ""), null, { timeout: 20000 }).catch(async (e) => {
        throw new Error(`${e.message}\nring says: ${await obox.locator('[data-ring="now"]').innerText()}`);
      });
      await gm.waitForFunction(() => /Where it is now 180° from its closest point/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo();

      // Spin: Locked sets the day to the year; the sun's day says the same face looks at the star.
      await row.locator('[data-handle="spin"]').click();
      const sbox = gm.locator('[data-box="spin"]');
      await sbox.waitFor();
      assert.equal(await sbox.locator('[data-strip="spin.tilt"]').count(), 1);
      await gm.waitForFunction(() => /h|min/.test(document.querySelector('[data-box="spin"] [data-result="solar_day"]')?.textContent ?? ""), null, { timeout: 20000 });
      await sbox.locator('[data-chip="locked"]').click();
      await gm.locator("[data-chip-said]").waitFor({ timeout: 20000 });
      await gm.waitForFunction(() => /same face/.test(document.querySelector('[data-box="spin"] [data-result="solar_day"]')?.textContent ?? ""), null, { timeout: 20000 });
      assert.match(await sbox.locator("[data-chip-said]").innerText(), /year/);
      await shot(gm, `spin-box-locked-${vp}`);
      await undoTo(['[data-handle="spin"] .value', "^24\\.6 h"]);
      // Backwards: the change says so in words.
      await sbox.locator('[data-chip="retrograde"]').click();
      await gm.locator("[data-followed]").waitFor({ timeout: 20000 });
      await gm.waitForFunction(() => /backwards/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await undoTo();

      // ED7 part 2b: Add a moon offers formation's own rules for Mars (a rocky world: an impact moon and a captured
      // body, no disc), each at its own orbit; adding one is one change, and one Undo takes it back.
      await gm.getByRole("button", { name: "More", exact: true }).click();
      await gm.locator("[data-more-add]").click();
      const add = gm.locator("dialog[data-add]");
      await add.locator("[data-moon-recipes]").waitFor({ timeout: 20000 });
      assert.deepEqual(await add.locator("[data-moon-recipe]").evaluateAll((ls) => ls.map((l) => l.dataset.moonRecipe)), ["impact", "captured"]);
      assert.match(await add.locator('[data-moon-recipe="impact"]').innerText(), /Canup 2004/);
      await shot(gm, `add-moon-${vp}`);
      const before = await gm.locator('.area-chart .body[aria-label*="moon of Mars" i]').count();
      await add.locator('[data-moon-recipe="impact"] button').click();
      await gm.waitForFunction(() => !document.querySelector("dialog[data-add]")?.open, null, { timeout: 20000 });
      await gm.waitForFunction((n) => document.querySelectorAll('.area-chart .body[aria-label*="moon of Mars" i]').length > n, before, { timeout: 20000 });
      await undoTo();

      // ED6 Kinds: round the Sun, Add offers a world of each kind the class atlas knows, smallest first.
      await gm.locator('.area-chart .body[aria-label^="Sun"]').first().locator(".hit").click();
      await gm.getByRole("button", { name: "More", exact: true }).click();
      await gm.locator("[data-more-add]").click();
      // ED4: the ghost comes first. It follows the pointer with the distance round the Sun; Esc puts it away and
      // writes nothing; a press opens Add here at that orbit.
      await gm.locator("[data-add-note]").waitFor({ timeout: 10000 });
      const sunAt = await gm.locator('.area-chart .body[aria-label^="Sun"] .hit').first().boundingBox();
      const earthAt = await gm.locator('.area-chart .body[aria-label^="Earth,"] .hit').first().boundingBox();
      const [sx0, sy0] = [sunAt.x + sunAt.width / 2, sunAt.y + sunAt.height / 2];
      const [ex0, ey0] = [earthAt.x + earthAt.width / 2, earthAt.y + earthAt.height / 2];
      await gm.mouse.move(sx0 + (ex0 - sx0) * 0.5, sy0 + (ey0 - sy0) * 0.5);
      await gm.mouse.move(ex0, ey0, { steps: 4 });
      await gm.locator("[data-add-ghost]").waitFor({ timeout: 5000 });
      assert.match(await gm.locator("[data-add-readout]").textContent(), /AU round Sun/);
      // Resting there, one formation run says what forms at that orbit (ED4 part 2a); nothing is written.
      const hashG = await hash();
      await gm.locator("[data-add-forms]").waitFor({ timeout: 15000 });
      assert.match(await gm.locator("[data-add-forms]").textContent(), /forms a world of [\d.]+ Earth masses here|no world forms here/);
      await shot(gm, `add-ghost-${vp}`);
      await gm.keyboard.press("Escape");
      await gm.waitForFunction(() => !document.querySelector("[data-add-note]") && !document.querySelector("[data-add-ghost]"), null, { timeout: 5000 });
      assert.equal(await hash(), hashG, "Esc puts the ghost away and writes nothing");
      await gm.getByRole("button", { name: "More", exact: true }).click();
      await gm.locator("[data-more-add]").click();
      await gm.locator("[data-add-note]").waitFor({ timeout: 10000 });
      await gm.mouse.move(ex0 - 3, ey0, { steps: 2 });
      await gm.mouse.move(ex0, ey0, { steps: 2 });
      await gm.mouse.down();
      await gm.mouse.up();
      await add.locator("[data-kinds]").waitFor({ timeout: 20000 });
      const at = Number(await add.locator("[data-add-orbit]").inputValue());
      assert.ok(at > 0.8 && at < 1.25, `Add here opens at the ghost's orbit, near Earth's: ${at} AU`);
      const kinds = await add.locator("[data-kind]").evaluateAll((ls) => ls.map((l) => l.dataset.kind));
      for (const k of ["rocky", "gas-giant", "ice-giant"]) assert.ok(kinds.includes(k), `${k} is a kind: ${kinds}`);
      assert.match(await add.locator('[data-kind="gas-giant"]').innerText(), /like Jupiter/);
      await shot(gm, `add-kinds-${vp}`);
      await add.getByRole("button", { name: "Close" }).click();

      // ED0 pairs: Pluto's Orbit box says where the hidden centre is (outside Pluto) and shows the pair's two orbits;
      // "Round each other" writes Charon's orbit, one change, one Undo.
      await gm.locator('.area-chart .body[aria-label^="Pluto,"]').first().locator(".hit").click();
      await gm.waitForFunction(() => /AU/.test(document.querySelector('[data-handle="orbit"] .value')?.textContent ?? "") && document.querySelector("section[aria-label='Pluto']"), null, { timeout: 20000 });
      if (!(await gm.locator('[data-box="orbit"]').isVisible())) await row.locator('[data-handle="orbit"]').click();
      const says = gm.locator("[data-pair-says]");
      await says.waitFor({ timeout: 20000 });
      assert.match(await says.innerText(), /Pluto and Charon go round a point between them .* outside Pluto/);
      const each = gm.locator('[data-strip="orbit.pair.each"]');
      await each.locator("[data-strip-figure]").click();
      await each.locator("[data-strip-entry]").fill("25000 km");
      await each.locator("[data-strip-entry]").press("Enter");
      await gm.waitForFunction(() => /Round each other/.test(document.querySelector("[data-followed]")?.textContent ?? ""), null, { timeout: 20000 });
      await shot(gm, `pair-pluto-${vp}`);
      await undoTo();
      await depth(gm, "Curious");
    }

    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
    }
    assert.deepEqual(errors, [], `page errors: ${errors.join("; ")}`);
    await browser.close();
    console.log(`editing ${vp}: ok`);
  }
} finally {
  server.kill();
}
