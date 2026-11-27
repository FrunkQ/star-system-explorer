// N1 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/sound.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Silent until enabled: no AudioContext exists until the GM turns sound on,
// and a saved "on" waits for the first tap after a reload. The chart's score
// fingerprint equals the one Node computes for the same derived scene (two
// engines, one score). No sound setting changes the state hash. Players on
// long-range sensors have no periods, so their bed is silence.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Session } from "../../packages/ui-contract/src/session.ts";
import { gmSeat } from "../../packages/ui-contract/src/seat.ts";
import { deriveScene } from "../../packages/ui-contract/src/derivation.ts";
import { orbitScene, score, scoreHash, voicesFromSystem } from "../../packages/ui-contract/src/sound.ts";
import { nodeKernel } from "../../packages/ui-contract/test/node-kernel.ts";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const doc = (p) => JSON.parse(readFileSync(new URL(`../../${p}`, import.meta.url), "utf8"));
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9900 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

// The score Node computes for the GM's opening scene, as the shell derives it.
async function nodeScore(dial) {
  const k = nodeKernel();
  try {
    const scene = doc("packages/shell/src/documents/kestrel.scene.json");
    const s = await Session.open(k.client, gmSeat(), scene.universe, [doc("packs/base-physics/pack.json"), doc("packages/shell/src/documents/kestrel.player-data.pack.json")]);
    const derived = await deriveScene(s, scene, doc("packages/shell/src/documents/derivations.base.json"));
    return scoreHash(score(orbitScene(derived.system, voicesFromSystem(derived)), 0, dial, scene.universe.seed));
  } finally {
    await k.close();
  }
}
const expected = await nodeScore(0);

// Count every AudioContext the page makes.
const COUNT = () => {
  window.__audio = 0;
  const Real = window.AudioContext;
  if (Real)
    window.AudioContext = class extends Real {
      constructor(...a) {
        super(...a);
        window.__audio++;
      }
    };
};

try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    const browser = await engine.launch(name === "chromium" ? { args: ["--autoplay-policy=user-gesture-required"] } : {});
    const errors = [];
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    await ctx.addInitScript(COUNT);
    const gm = await ctx.newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const control = gm.locator(".ruler-side .sound");
    await gm.waitForTimeout(1500);
    assert.equal(await control.getAttribute("data-sound"), "off", "off by default");
    assert.equal(await gm.evaluate(() => window.__audio), 0, "no audio made before the user turns sound on");
    // The derived scene settles after the kernel answers; wait for the fingerprint to match Node's.
    await gm.waitForFunction((h) => document.querySelector(".ruler-side .sound")?.getAttribute("data-score-hash") === h, expected, { timeout: 10000 }).catch(() => {});
    assert.equal(await control.getAttribute("data-score-hash"), expected, "the browser's score equals Node's for the same scene");
    assert.equal(await control.getAttribute("data-sources"), "5");
    const h0 = await hash();

    // Turn it on: one AudioContext, captions, and the state hash unmoved by any setting.
    await control.getByRole("button", { name: /Sound off/ }).click();
    const panel = gm.getByRole("group", { name: "Sound" });
    await panel.getByLabel("Sound on").check();
    assert.equal(await gm.evaluate(() => window.__audio), 1);
    assert.equal(await control.getAttribute("data-sound"), "on");
    await gm.waitForSelector(".ruler-side .sound .caption", { timeout: 8000 });
    await panel.getByLabel("music level").fill("0.6");
    await panel.getByLabel("True ratios to settled").fill("1");
    assert.equal(await control.getAttribute("data-score-hash"), await nodeScore(1), "the settled dial: same score as Node");
    await panel.getByRole("button", { name: "Why does Tarn sound like this?" }).click();
    assert.match(await panel.locator(".why").innerText(), /raised \d+ octaves/);
    await panel.getByLabel("Device sound tier").selectOption("basic");
    assert.equal(await hash(), h0, "no sound setting changes the state hash");
    await shot(gm, "n1-gm-sound");

    // Reload: the saved "on" waits for the first tap before making audio.
    await gm.reload();
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await gm.waitForTimeout(800);
    assert.equal(await control.getAttribute("data-sound"), "on");
    assert.equal(await gm.evaluate(() => window.__audio), 0, "a saved on does not start audio by itself");
    await gm.locator(".area-chart").click({ position: { x: 30, y: 300 } });
    await gm.waitForFunction(() => window.__audio === 1);
    assert.equal(await control.getAttribute("data-started"), "true");

    // Players on long-range sensors: no periods, so silence, and the caption says so.
    const pl = await ctx.newPage();
    pl.on("pageerror", (e) => errors.push(`players: ${e.message}`));
    await pl.goto(`${base}players.html`);
    const ps = pl.locator(".soundbar .sound");
    await ps.waitFor({ timeout: 20000 });
    assert.equal(await ps.getAttribute("data-sources"), "0", "players hear nothing their view does not give");
    await ps.getByRole("button", { name: /Sound/ }).click();
    assert.match(await pl.getByRole("group", { name: "Sound" }).innerText(), /Silence/);
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: silent until enabled; score ${expected} equals Node's; settings leave the state hash alone; players' bed silent on long-range`);
  }
} finally {
  server.kill();
}
