// V9 part 2 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/cluster.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Explore > Galaxies nearby draws one halo cell from the kernel's halo_cell:
// haloes as dots over the web, the biggest picked with its ledger and the
// mass function. A smaller halo is picked; a neighbouring cell and a higher
// floor redraw; "Chart this galaxy" opens that very halo in the galaxy chart.
// Night chart and phone (no sideways scroll) too.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
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
const shot = async (page, n, full = false) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: full }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    // The place line hears where the view is (sse:place).
    await gm.addInitScript(() => {
      window.__places = [];
      document.addEventListener("sse:place", (e) => window.__places.push(e.detail));
    });
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(gm, "cluster");
    await gm.waitForSelector(".clu [data-halo-dot]", { timeout: 30000 });
    const clu = gm.locator(".clu");
    assert.equal(await clu.getAttribute("data-cell"), "0.0.0");
    const n = await clu.locator("[data-halo-dot]").count();
    assert.ok(n >= 5, `haloes drawn (${n})`);
    assert.equal(await clu.locator("circle.halo.big").count(), 1, "the biggest marked");
    assert.match(await clu.locator("[data-lede]").innerText(), /megaparsecs.*haloes/);
    assert.ok((await clu.locator("[data-mass-function] rect.drawn").count()) >= 3, "mass function bars");
    const top = await clu.locator("[data-halo-ledger]").getAttribute("data-halo-ledger");
    {
      const p = await gm.evaluate(() => window.__places.at(-1));
      assert.equal(p.scale, "cluster");
      assert.equal(p.id, "cell:0.0.0");
      assert.equal(p.name, "Cell 0 · 0 · 0");
      assert.ok(p.children.length >= 5 && p.children.length <= 24, `galaxies below (${p.children.length})`);
      assert.match(p.children[0].id, /cell:0\.0\.0\/halo:\d+$/);
      assert.equal(p.children[0].name, `Halo ${top}`, "the biggest first");
    }
    assert.ok(top, "the biggest halo is picked");
    await gm.waitForFunction(() => document.querySelector(".clu canvas") !== null);
    await gm.waitForTimeout(600);
    await shot(gm, "v9-cluster");

    // Another halo: its ledger.
    const other = await clu.locator("[data-halo-dot]").first().getAttribute("data-halo-dot");
    await clu.locator(`[data-halo-dot='${other}']`).dispatchEvent("click");
    await gm.waitForFunction((k) => document.querySelector(".clu [data-halo-ledger]")?.getAttribute("data-halo-ledger") === k, other);
    // A neighbouring cell, then back; a higher floor shows fewer.
    await clu.locator("[data-move='x+']").click();
    await gm.waitForFunction(() => document.querySelector(".clu")?.getAttribute("data-cell") === "1.0.0", null, { timeout: 30000 });
    await gm.waitForFunction(() => window.__places.at(-1)?.id === "cell:1.0.0");
    await clu.locator("[data-move='x-']").click();
    await gm.waitForFunction(() => document.querySelector(".clu")?.getAttribute("data-cell") === "0.0.0", null, { timeout: 30000 });
    await clu.locator("[data-min]").selectOption("1e11");
    await gm.waitForFunction((k) => document.querySelectorAll(".clu [data-halo-dot]").length < k, n, { timeout: 30000 });

    // Find a home: first like the Milky Way (whatever the nearby cells hold), then a wider wish that must find some;
    // going to one moves the map to its cell and picks it.
    await clu.locator(".find summary").click();
    await clu.locator("[data-find-search]").click();
    await clu.locator("[data-find-said]").waitFor({ timeout: 120000 });
    const mw = await clu.locator("[data-find-said]").innerText();
    await clu.locator("[data-wish='neighbour']").selectOption("any");
    await clu.locator("[data-wish='quiet_gyr']").fill("0");
    await clu.locator("[data-find-search]").click();
    await gm.waitForFunction(() => Number(document.querySelector(".clu [data-find-home]")?.getAttribute("data-find-home")) >= 1 && document.querySelector(".clu [data-find-said]"), null, { timeout: 120000 });
    const home = await clu.locator("[data-find-go]").first().getAttribute("data-find-go");
    assert.match(await clu.locator(".fits li").first().innerText(), /a halo of .* suns/);
    await shot(gm, "v9-find-home");
    await clu.locator("[data-find-go]").first().click();
    const homeCell = /cell:(-?\d+)\.(-?\d+)\.(-?\d+)\//.exec(home).slice(1, 4).join(".");
    await gm.waitForFunction((c) => document.querySelector(".clu")?.getAttribute("data-cell") === c, homeCell, { timeout: 30000 });
    await gm.waitForFunction((k) => document.querySelector(".clu [data-halo-ledger]")?.getAttribute("data-halo-ledger") === k, home.split("halo:").pop(), { timeout: 30000 });
    if (homeCell !== "0.0.0") {
      for (const [axis, v] of homeCell.split(".").map((x, i) => [i, Number(x)]))
        for (let k = 0; k < Math.abs(v); k++) await clu.locator(`[data-move='${"xyz"[axis]}${v > 0 ? "-" : "+"}']`).click();
      await gm.waitForFunction(() => document.querySelector(".clu")?.getAttribute("data-cell") === "0.0.0", null, { timeout: 30000 });
    }

    // Chart this galaxy: the galaxy chart opens on that very halo.
    const key = await clu.locator("[data-halo-ledger]").getAttribute("data-halo-ledger");
    await clu.locator("[data-chart-galaxy]").click();
    await gm.waitForSelector(".gal [data-ring]", { timeout: 30000 });
    await gm.waitForFunction((k) => new RegExp(`cell:0\\.0\\.0/halo:${k}$`).test(document.querySelector(".gal")?.getAttribute("data-galaxy") ?? ""), key, { timeout: 30000 });
    assert.match(await gm.locator(".gal h2").innerText(), new RegExp(`Halo ${key}, cell 0\\.0\\.0`));
    await shot(gm, "v9-cluster-charted");
    // And back to the cluster map, which keeps the halo picked.
    await go(gm, "cluster");
    await gm.waitForFunction((k) => document.querySelector(".clu [data-halo-ledger]")?.getAttribute("data-halo-ledger") === k, key, { timeout: 30000 });

    // Galaxies nearby in 3D (GZ5 part 1): centred on our galaxy, from the kernel's sky; two reaches, a card, Real.
    const vgroup = (g, b) => gm.getByRole("group", { name: g }).getByRole("button", { name: b, exact: true });
    await vgroup("View style", "3D").click();
    await gm.waitForFunction(() => Number(document.querySelector(".clu [data-nearby]")?.getAttribute("data-nearby")) > 0, null, { timeout: 120000 });
    const inGroup = Number(await clu.locator("[data-nearby]").getAttribute("data-nearby"));
    assert.equal(await clu.locator("[data-home-galaxy]").count(), 1, "our galaxy is at the middle");
    await shot(gm, "gz5-nearby-group");
    // GZ6: the group's edge under HUD (the zero-velocity sphere), then our own satellites on their orbits with Play.
    const edgeMpc = Number(await clu.locator("[data-group-edge]").getAttribute("data-group-edge"));
    assert.ok(edgeMpc > 0.5 && edgeMpc < 3, `the group's edge is about a megaparsec out (${edgeMpc} Mpc)`);
    await clu.locator("[data-reach='satellites']").click();
    await clu.locator("[data-play-satellites]").waitFor({ timeout: 10000 });
    const streamsNow = await clu.locator("[data-stream]").count();
    assert.ok(streamsNow > 0, "torn-apart satellites drawn as streams under HUD");
    await shot(gm, "gz6-satellites-now");
    await clu.locator("[data-play-satellites]").click();
    await gm.waitForFunction(() => document.querySelectorAll(".clu [data-ghost]").length > 0, null, { timeout: 10000 });
    await gm.waitForTimeout(1500);
    await clu.locator("[data-play-satellites]").click();
    const paused = await clu.locator("[data-nearby-play]").getAttribute("data-nearby-play");
    assert.notEqual(paused, "now", "paused in the past");
    assert.match(await clu.locator("[data-nearby-said]").innerText(), /Gyr after the Big Bang: \d+ satellites on their orbits/);
    // Scrub to the middle of the history: some have fallen in, some are yet to.
    await clu.locator(".play input[type=range]").evaluate((el) => {
      el.value = String((Number(el.min) + Number(el.max)) / 2);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const midGhosts = await clu.locator("[data-ghost]").count();
    assert.ok(midGhosts > 0, "satellites on their orbits halfway through");
    await shot(gm, "gz6-satellites-play");
    // To the end: the campaign's date again, the sky's own satellites back.
    await clu.locator(".play input[type=range]").evaluate((el) => {
      el.value = el.max;
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await gm.waitForFunction(() => document.querySelector(".clu [data-nearby-play]")?.getAttribute("data-nearby-play") === "now", null, { timeout: 5000 });
    assert.equal(await clu.locator("[data-ghost]").count(), 0);
    await clu.locator("[data-reach='group']").click();
    await clu.locator("[data-reach='neighbourhood']").click();
    await gm.waitForFunction((n) => Number(document.querySelector(".clu [data-nearby]")?.getAttribute("data-nearby")) > n, inGroup, { timeout: 30000 });
    const inHood = Number(await clu.locator("[data-nearby]").getAttribute("data-nearby"));
    const morphs = await clu.locator("[data-galaxy]").evaluateAll((gs) => [...new Set(gs.map((g) => g.getAttribute("data-morphology")))]);
    assert.ok(morphs.length >= 2, `several kinds of galaxy: ${morphs.join()}`);
    // Bound pairs under HUD, from the cells' own neighbours (read in the background, one cell at a time).
    await gm.waitForFunction(() => document.querySelectorAll(".clu [data-bound-pair]").length > 0, null, { timeout: 120000 });
    const pairsSeen = await clu.locator("[data-bound-pair]").count();
    await shot(gm, "gz5-nearby-neighbourhood");
    await clu.locator("[data-galaxy]").last().click({ force: true });
    await gm.waitForFunction(() => !!document.querySelector(".clu [data-galaxy-card]")?.getAttribute("data-galaxy-card"), null, { timeout: 10000 });
    assert.match(await clu.locator("[data-galaxy-card]").innerText(), /light takes .* to reach us/);
    await vgroup("Seen from", "Edge-on").click();
    await vgroup("Scale", "Real").click();
    await gm.waitForFunction(() => document.querySelector(".clu [data-nearby-scale]")?.getAttribute("data-nearby-scale") === "real", null, { timeout: 10000 });
    await shot(gm, "gz5-nearby-edge-real");
    await vgroup("Scale", "Readable").click();
    await vgroup("Seen from", "Tilted").click();
    // GZ5 part 2: True, each galaxy a type picture in its own light, ours its real face; Free turns it by dragging.
    await clu.locator("[data-reach='group']").click();
    await vgroup("View style", "True").click();
    await clu.locator("[data-nearby-true]").waitFor({ timeout: 10000 });
    await gm.waitForFunction(() => document.querySelectorAll(".clu [data-type-picture]").length > 0, null, { timeout: 30000 });
    const typePictures = await clu.locator("[data-type-picture]").count();
    await clu.locator("[data-home-face]").waitFor({ timeout: 60000 });
    await shot(gm, "gz5-nearby-true");
    await vgroup("Seen from", "Free").click();
    const map = clu.locator("svg[data-nearby-turn]");
    assert.equal(await map.getAttribute("data-nearby-turn"), "0 60", "Free starts as Tilted");
    const box = await map.boundingBox();
    await gm.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await gm.mouse.down();
    await gm.mouse.move(box.x + box.width / 2 + 100, box.y + box.height / 2 + 40, { steps: 5 });
    await gm.mouse.up();
    const turned = await map.getAttribute("data-nearby-turn");
    assert.notEqual(turned, "0 60", "dragging turns it");
    await shot(gm, "gz5-nearby-true-free");
    await vgroup("Seen from", "Tilted").click();
    await vgroup("View style", "Chart").click();
    await clu.locator("[data-halo-dot]").first().waitFor({ timeout: 30000 });
    const nearbyNote = `galaxies nearby: ${inGroup} in the group, ${inHood} in the neighbourhood (${morphs.join(", ")}), ${pairsSeen} bound pairs, the group's edge at ${edgeMpc} Mpc, ${streamsNow} streams, Play with ${midGhosts} satellites halfway, True with ${typePictures} type pictures and our real face, Free turned to ${turned}`;

    await more(gm, "Night chart");
    await gm.waitForTimeout(400);
    await shot(gm, "v9-cluster-night");

    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "cluster");
    await phone.waitForSelector(".clu [data-halo-dot]", { timeout: 30000 });
    await phone.waitForTimeout(600);
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "v9-cluster-phone", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: cluster map from halo_cell: haloes over the web, biggest picked, ledger, mass function, neighbour cell, floor, find a home (${mw}), chart this galaxy, ${nearbyNote}, night chart, phone`);
  }
} finally {
  server.kill();
}
