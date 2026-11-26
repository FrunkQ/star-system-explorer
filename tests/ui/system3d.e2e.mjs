// V6 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/system3d.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The GM's chart switches to 3D and draws on the GPU path; choosing Vault and
// opening its moons hands the camera to Vault's frame; zooming out hands back.
// A players' 3D screen draws too. With WebGL turned off, the same views fall
// back to Canvas 2D. (The pixel-stability proof is packages/renderer/test.)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { view as pick } from "./nav.mjs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9300 + Math.floor(Math.random() * 300);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const VAULT = "u:kestrel/sys:1/body:4";

try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    for (const mode of ["gpu", "canvas"]) {
      // Canvas mode: the browser has no WebGL or WebGPU at all.
      const browser = await engine.launch(mode === "canvas" && name === "chromium" ? { args: ["--disable-webgl", "--disable-webgl2", "--disable-3d-apis", "--disable-gpu"] } : {});
      if (mode === "canvas" && name !== "chromium") {
        await browser.close();
        continue;
      }
      const errors = [];
      const gm = await browser.newPage({ viewport: { width: 1440, height: 900 } });
      gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
      await gm.goto(base + "#chart");
      await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
      await gm.getByRole("button", { name: "3D", exact: true }).click();
      const view = gm.locator("[aria-roledescription='3D chart']");
      await view.waitFor();
      await gm.waitForFunction(() => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-tier"), null, { timeout: 20000 });
      const tier = await view.getAttribute("data-tier");
      if (mode === "canvas") assert.equal(tier, "canvas", "no GPU: Canvas 2D");
      const note = await view.locator(".note").innerText();
      // Every planet of the star's frame has a pin; moons do not.
      const pins = () => view.locator("[data-body]").evaluateAll((b) => b.map((x) => x.getAttribute("data-body")));
      assert.ok(!(await pins()).some((a) => a.split("/").length > 3), "no moons in the star's frame");
      assert.equal((await pins()).length, 6, "the star and five planets");
      await shot(gm, `v6-gm-3d-${mode}`);
      // A click on Vault flies in to its close-up: the camera rushes at it and hands over to its frame, with its moons.
      await view.locator(`[data-body='${VAULT}']`).click();
      await gm.waitForFunction((v) => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-frame") === v, VAULT);
      await gm.waitForTimeout(900); // the camera eases back out to show the moons
      assert.deepEqual((await pins()).sort(), [VAULT, `${VAULT}/body:0`, `${VAULT}/body:1`, `${VAULT}/body:2`]);
      await shot(gm, `v6-gm-3d-vault-${mode}`);
      // Zooming out past the frame's reach hands back to the star's frame.
      for (let i = 0; i < 12 && (await view.getAttribute("data-frame")) === VAULT; i++) await view.getByRole("button", { name: "Zoom out" }).click();
      assert.notEqual(await view.getAttribute("data-frame"), VAULT, "zoom out hands back");
      // Keyboard: arrows turn the camera, so a pin moves.
      const before = await view.locator(`[data-body='${VAULT}']`).getAttribute("data-x");
      await view.focus();
      await gm.keyboard.press("ArrowLeft");
      await gm.waitForFunction((b) => document.querySelector(`[aria-roledescription='3D chart'] [data-body='${"u:kestrel/sys:1/body:4"}']`)?.getAttribute("data-x") !== b, before);
      // True (V10a), one press from the chart: the same frame against space with the star in its own colour. The test
      // drive's worlds have no surface atlas, so they are drawn plain and the note says why; the canvas tier says True
      // needs a GPU.
      await gm.getByRole("button", { name: "True", exact: true }).click();
      await gm.waitForFunction(() => /no surface|needs a GPU/.test(document.querySelector("[aria-roledescription='3D chart'] [data-true-note]")?.textContent ?? ""), null, { timeout: 60000 });
      assert.equal(await view.getAttribute("data-look"), tier === "gpu" ? "true" : "plain", "True draws on the GPU tier; a device without one keeps the plain look");
      // The living star (V10c): on the GPU tier the star is the kernel's, turning on its own clock.
      if (tier === "gpu") {
        // VP4 part 2b: True is placed and governed by the system-true recipe (tier|level|rest|frames on the canvas).
        await gm.waitForFunction(() => /^(gpu-high|gpu-basic|gpu-ultra)\|\d+\|/.test(document.querySelector("[aria-roledescription='3D chart'] canvas.gpu")?.getAttribute("data-quality") ?? ""), null, { timeout: 30000 });
        const live = view.locator("[data-star-live]");
        await live.waitFor({ timeout: 30000 });
        const turn0 = await live.getAttribute("data-star-turn");
        await gm.waitForFunction((t) => document.querySelector("[data-star-live]")?.getAttribute("data-star-turn") !== t, turn0, { timeout: 10000 });
        // SG1 to SG3: the star's glow is the eye's glare, said beside the view, and its flares and ejections are named.
        assert.match(await live.innerText(), /glare/, "the glow says what it is");
        assert.match(await live.innerText(), /flare/, "the flares are named");
        await shot(gm, `sg-true-star-${mode}`);
        // Bands through True (rule 12, one set of controls): X-ray draws the corona, Infrared each body's own heat.
        for (const [b, id] of [["X-ray", "xray"], ["Infrared", "infrared"]]) {
          await pick(gm, "Band", b);
          await gm.waitForFunction((x) => document.querySelector("[aria-roledescription='3D chart'] [data-true-band]")?.getAttribute("data-true-band") === x, id, { timeout: 20000 });
        }
        await shot(gm, `v10d-gm-true-infrared-${mode}`);
        await pick(gm, "Band", "Visible");
        await gm.waitForFunction(() => !document.querySelector("[aria-roledescription='3D chart'] [data-true-band]"));
      }
      await shot(gm, `v10a-gm-true-${mode}`);
      // Scale and HUD (Alex, 23 Nov): Real puts distances and sizes in proportion; HUD off in True by default, on
      // brings the orbit lines and names back.
      assert.equal(await view.getAttribute("data-hud"), "off");
      await gm.getByRole("group", { name: "Scale" }).getByRole("button", { name: "Real" }).click();
      await gm.waitForFunction(() => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-scale") === "real");
      assert.match(await view.locator(".note").innerText(), /Real scale/);
      await pick(gm, "HUD", "On");
      await gm.waitForFunction(() => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-hud") === "on");
      await shot(gm, `v10b-gm-real-hud-${mode}`);
      // Every body takes a click (Alex, 24 Nov: only two did): the pointer picks the nearest body, so pins that overlap
      // at Real scale no longer hide the worlds beneath them. Checked on each pin clear of its neighbours by 6 px.
      // A click also flies in to the body's close-up (Alex, 24 Nov); "‹ Sun" comes back out before the next one.
      // The camera's distance holds still for a few frames: the fly-in or fly-out has finished (a loaded runner takes
      // longer than the tween's 620 ms, so the test waits on the camera, not on a clock).
      const settled = () => gm.waitForFunction(() => {
        const v = document.querySelector("[aria-roledescription='3D chart']");
        const d = v?.getAttribute("data-dist");
        const w = /** @type {any} */ (window);
        if (w.__distWas !== d) { w.__distWas = d; w.__distSame = 0; return false; }
        return ++w.__distSame >= 6;
      }, null, { timeout: 20000, polling: 100 });
      await settled();
      const pinsNow = () => view.locator(".pin").evaluateAll((ps) => ps.map((e) => { const r = e.getBoundingClientRect(); return { id: e.dataset.body, x: r.x + r.width / 2, y: r.y + r.height / 2 }; }));
      const ids = (await pinsNow()).map((q) => q.id);
      let clicked = 0;
      // A pin is clicked only once it holds still: read while the chart is still easing, it has moved on by the time
      // the click lands and the click misses (CI, 26 Nov, on a loaded runner).
      const pinsStill = async () => {
        let was = "";
        let at = await pinsNow();
        for (let same = 0, i = 0; same < 3 && i < 100; i++) {
          await gm.waitForTimeout(100);
          at = await pinsNow();
          const now = at.map((q) => `${q.id}:${Math.round(q.x)},${Math.round(q.y)}`).join(" ");
          same = now === was ? same + 1 : 0;
          was = now;
        }
        return at;
      };
      for (const id of ids) {
        const pinsAt = await pinsStill();
        const q = pinsAt.find((x) => x.id === id);
        if (!q || pinsAt.some((o) => o.id !== q.id && Math.hypot(o.x - q.x, o.y - q.y) < 6)) continue;
        await gm.mouse.click(q.x, q.y);
        await gm.waitForFunction((id) => document.querySelector("[aria-roledescription='3D chart'] .pin.sel")?.getAttribute("data-body") === id, q.id, { timeout: 15000 });
        await settled(); // the fly-in to its close-up
        clicked++;
        const back = view.locator(".crumb button, button").filter({ hasText: /^‹ / }).first();
        if (await back.isVisible().catch(() => false)) {
          if (clicked === 1) await shot(gm, `close-up-${mode}`);
          await back.click();
          await settled();
        }
      }
      assert.ok(clicked >= 3, `bodies clicked: ${clicked}`);
      await gm.getByRole("group", { name: "Scale" }).getByRole("button", { name: "Readable" }).click();
      // Close-up (Alex, 25 Nov): in True the camera comes down to a grazing orbit over a world (Readable draws it 0.55
      // across), and the HUD's switch leaves the camera where it is.
      if (tier === "gpu") {
        await view.locator(`[data-body='${VAULT}']`).click();
        await gm.waitForFunction((v) => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-frame") === v, VAULT);
        await gm.waitForTimeout(1100);
        const d0 = await view.getAttribute("data-dist");
        await pick(gm, "HUD", "Off");
        await gm.waitForFunction(() => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-hud") === "off");
        assert.equal(await view.getAttribute("data-dist"), d0, "HUD off keeps the camera");
        for (let i = 0; i < 45; i++) await view.getByRole("button", { name: "Zoom in" }).click();
        const low = Number(await view.getAttribute("data-dist"));
        assert.ok(low > 0.55 && low < 0.552, `down to a grazing orbit: ${low}`);
        await shot(gm, `close-up-graze-${mode}`);
        // PC2: the close-up's camera goes on a bookmark, and the bookmark brings the same camera back.
        const keptAt = Number(await view.getAttribute("data-dist"));
        await gm.getByRole("button", { name: "Bookmarks" }).click();
        const marks = gm.locator("[data-bookmarks-list]");
        await marks.locator("[data-bookmark-add]").click();
        await marks.locator("[data-bookmark-name]").press("Enter");
        await gm.keyboard.press("Escape");
        await pick(gm, "HUD", "On");
        await view.locator(".crumb button, button").filter({ hasText: /^‹ / }).first().click();
        await settled();
        assert.ok(Number(await view.getAttribute("data-dist")) > 1, "out of the close-up");
        await gm.getByRole("button", { name: "Bookmarks" }).click();
        await marks.locator("[data-bookmark-go]").last().click();
        await gm.waitForFunction((d) => Math.abs(Number(document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-dist")) - d) < 1e-3, keptAt, { timeout: 15000 });
        await shot(gm, `pc2-bookmark-camera-${mode}`);
        await gm.keyboard.press("Escape");
        await pick(gm, "HUD", "On"); // the bookmark brought back its view too, made with the HUD off
        await view.locator(".crumb button, button").filter({ hasText: /^‹ / }).first().click();
      }
      await gm.getByRole("button", { name: "3D", exact: true }).click();
      await gm.getByRole("group", { name: "View style" }).getByRole("button", { name: "Chart", exact: true }).click();
      // HUD on the flat chart too (one set of controls, Alex 23 Nov): off leaves the bodies and belts without the
      // scale rings, orbit lines and names; on brings them back.
      const flat = gm.locator("svg[aria-label^='System chart of']");
      await flat.locator("[data-scale-rings]").waitFor();
      const named = await flat.locator("text.name").count();
      assert.ok(named > 0);
      await pick(gm, "HUD", "Off");
      await flat.locator("[data-scale-rings]").waitFor({ state: "detached" });
      assert.equal(await flat.locator("text.name").count(), 0, "HUD off: no names");
      assert.ok((await flat.locator("[data-address]").count()) > 0, "the bodies stay");
      if (mode === "gpu") await shot(gm, `hud-off-chart-${name}`);
      await pick(gm, "HUD", "On");
      await flat.locator("[data-scale-rings]").waitFor();
      assert.equal(await flat.locator("text.name").count(), named);

      // A players' 3D screen.
      const pl = await browser.newPage({ viewport: { width: 1280, height: 900 } });
      pl.on("pageerror", (e) => errors.push(`players: ${e.message}`));
      await pl.goto(`${base}players.html?screen=players.bridge-3d.screen.json`);
      const pv = pl.locator("[aria-roledescription='3D chart']");
      await pv.waitFor({ timeout: 20000 });
      await pl.waitForFunction(() => document.querySelector("[aria-roledescription='3D chart']")?.getAttribute("data-tier"), null, { timeout: 20000 });
      const ptier = await pv.getAttribute("data-tier");
      const pnote = await pv.locator(".note").innerText();
      assert.match(pnote, /spread for display/);
      await shot(pl, `v6-players-3d-${mode}`);
      // PC2 part 2: a postcard of the system view draws its front live, in True from its camera (off screen, the
      // same view), and says how it was made on its back.
      if (mode === "gpu" && name === "chromium") {
        const card = { schema: "sse.postcard/1", id: "pc-vault", name: "Vault up close",
          bookmark: { schema: "sse.bookmark/1", id: "b-vault", name: "Vault up close", scope: "u:kestrel/sys:1", at_yr: 4.6e9, instrument: "system", view: { style: "true", seen: "free" }, selected: VAULT,
            pose: { about: VAULT, eye_dir: [0.6, -0.5, 0.62], eye_m: 2.4e8, up: [0, 0, 1], fov_deg: 45, exposure: { mode: "long", ev: 0 }, keep: [0, 0, 1, 1] } },
          world: { how: "address", root: "u:kestrel", packs: [], made_with: "0.1.0" }, credit: ["Test"] };
        const b64 = Buffer.from(JSON.stringify(card)).toString("base64").replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
        const pc = await browser.newPage({ viewport: { width: 1440, height: 900 } });
        pc.on("pageerror", (e) => errors.push(`postcard: ${e.message}`));
        await pc.goto(`${base}#postcard=${b64}`);
        await pc.locator("[data-postcard-drawn]").waitFor({ timeout: 90000 });
        const lit = await pc.locator("[data-postcard-drawn]").evaluate((c) => {
          const g = c.getContext("2d");
          return [...g.getImageData(c.width / 2, c.height / 2, 1, 1).data];
        });
        assert.ok(lit[0] + lit[1] + lit[2] > 60, `Vault fills the middle of the front: ${lit}`);
        await shot(pc, "pc2-postcard-front");
        await pc.locator("[data-postcard-turn]").first().click();
        assert.match(await pc.locator("[data-postcard-back]").innerText(), /The system view in True from the postcard's camera/);
        await pc.close();
      }
      assert.deepEqual(errors, []);
      console.log(`V6 ${name} ${mode}: GM tier ${tier} (${note.replace(/\s+/g, " ").match(/Drawn with [^.]*/)?.[0]}); players tier ${ptier}; Vault frame handover and back`);
      await browser.close();
    }
  }
  console.log("PASS");
} finally {
  server.kill();
}
