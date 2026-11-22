// O1 (UI side) in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/light.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The Light control starts on the chart's own colours. Every recipe, once
// chosen, shows a legend chip with its honesty label and a legend saying what
// its colours mean; stubbed spectra are marked as stubbed. The star's colour
// under Eye is the one Node computes for the same derived scene. Switching
// recipe never changes the state hash. Players get the same control.
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Session } from "../../packages/ui-contract/src/session.ts";
import { gmSeat } from "../../packages/ui-contract/src/seat.ts";
import { deriveScene } from "../../packages/ui-contract/src/derivation.ts";
import { describeLight, litFromSystem, paint } from "../../packages/ui-contract/src/light.ts";
import { nodeKernel } from "../../packages/ui-contract/test/node-kernel.ts";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const doc = (p) => JSON.parse(readFileSync(new URL(`../../${p}`, import.meta.url), "utf8"));
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9800 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const recipes = doc("packages/shell/src/documents/light.recipes.json").recipes;
const STAR = "u:kestrel/sys:1/body:0";

// What Node paints for the GM's opening scene under each recipe.
const k = nodeKernel();
let expected;
try {
  const scene = doc("packages/shell/src/documents/kestrel.scene.json");
  const s = await Session.open(k.client, gmSeat(), scene.universe, [doc("packs/base-physics/pack.json"), doc("packages/shell/src/documents/kestrel.player-data.pack.json")]);
  const derived = await deriveScene(s, scene, doc("packages/shell/src/documents/derivations.base.json"));
  const { lights } = describeLight(litFromSystem(derived));
  expected = Object.fromEntries(recipes.map((r) => [r.id, paint(r, lights).get(STAR) ?? null]));
} finally {
  await k.close();
}
assert.match(expected.eye ?? "", /^#/, "the derived star has light");

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const gm = await ctx.newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await gm.waitForTimeout(1200);
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const control = gm.locator(".ruler-side .light");
    const starMark = () => gm.locator(`.area-chart .body[data-address="${STAR}"] [data-mark]`).first().getAttribute("data-mark");
    assert.equal(await control.getAttribute("data-light"), "symbols", "opens on the chart's own colours");
    const before = await starMark();
    const h0 = await hash();

    await control.getByRole("button", { name: /Light:/ }).click();
    const panel = gm.getByRole("group", { name: "Light" });
    for (const r of recipes) {
      await panel.getByRole("radio", { name: new RegExp(`^${r.name.replace(/[()]/g, "\\$&")}`) }).check();
      assert.equal(await control.getAttribute("data-light"), r.id);
      const chip = control.locator("[data-legend-chip]");
      assert.match(await chip.innerText(), /colour/i, `${r.id}: the chip says what kind of colour`);
      assert.match(await chip.innerText(), /stubbed spectra/i, `${r.id}: stubbed spectra are marked`);
      assert.ok((await panel.getByLabel("What the colours mean").innerText()).includes(r.legend.slice(0, 30)), `${r.id}: legend shown`);
      const want = expected[r.id];
      await gm.waitForFunction(
        ([a, w]) => {
          const m = document.querySelector(`.area-chart .body[data-address="${a}"] [data-mark]`)?.getAttribute("data-mark") ?? "";
          return w ? m.includes(w) : !m.includes("#");
        },
        [STAR, want],
        { timeout: 5000 },
      );
      assert.equal(await hash(), h0, `${r.id}: the state hash does not move`);
      if (r.id === "eye" || r.id === "mid-ir") await shot(gm, `o1-gm-${r.id}`);
    }
    // Why per body.
    await panel.getByRole("button", { name: "Why is Tarn this colour?" }).click();
    assert.match(await panel.locator(".why").innerText(), /Stubbed/);
    await panel.getByRole("radio", { name: /Chart colours/ }).check();
    assert.equal(await starMark(), before, "back to the symbology exactly");
    assert.equal(await control.locator("[data-legend-chip]").count(), 0);

    // Kept on this device across a reload.
    await panel.getByRole("radio", { name: /^Eye/ }).check();
    await gm.reload();
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    assert.equal(await control.getAttribute("data-light"), "eye");

    // Players: the same control, light only from what their view gives.
    const pl = await ctx.newPage();
    pl.on("pageerror", (e) => errors.push(`players: ${e.message}`));
    await pl.goto(`${base}players.html`);
    const pc = pl.locator(".soundbar .light");
    await pc.waitFor({ timeout: 20000 });
    await pc.getByRole("button", { name: /Light:/ }).click();
    await pl.getByRole("group", { name: "Light" }).getByRole("radio", { name: /Thermal infrared/ }).check();
    assert.equal(await pc.getAttribute("data-honesty"), "mapped");
    await pl.waitForTimeout(300);
    await shot(pl, "o1-players-thermal");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: ${recipes.length} recipes each show a legend and honesty label; star colour equals Node's; state hash unmoved; players have the control`);
  }
} finally {
  server.kill();
}
