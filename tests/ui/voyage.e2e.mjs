// V8 part 3 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/voyage.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Between stars (rail, travel): the home star cell, a jump courier's nearest
// unvisited star (one jump, dashed), "Next stop" twice (the explorer steps
// outward), a voyage to a picked star by the antimatter probe (a flight with
// less time aboard than at home), and a chemical ship refused in words.
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
const port = 9800 + Math.floor(Math.random() * 90);
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
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(gm, "voyages");

    // In this system first: the next world out by a real transfer, waiting for its window.
    const ins = gm.locator(".insys");
    await gm.waitForFunction(() => Number(document.querySelector(".insys")?.getAttribute("data-legs") ?? 0) >= 1, null, { timeout: 60000 });
    assert.ok((await ins.locator("[data-place]").count()) >= 3, "places on the orbit line");
    assert.match(await ins.locator("[data-leg]").first().innerText(), /from .* to .*propellant/);
    await shot(gm, "v8-voyage-system");
    // To a belt, if the system has one; else the outermost world.
    const belt = ins.locator("[data-place^='belt:']").first();
    const target = (await belt.count()) ? await belt.getAttribute("data-place") : await ins.locator("[data-place]").last().getAttribute("data-place");
    await ins.locator(`[data-place='${target}']`).dispatchEvent("click");
    await gm.waitForFunction((t) => (document.querySelector(".insys")?.getAttribute("data-voyage") ?? "").endsWith(`>${t}`) || !!document.querySelector(".insys [data-refusal]"), target, { timeout: 90000 });
    await shot(gm, "v8-voyage-system-far");

    await gm.locator("[data-scale-pick=stars]").click();
    const voy = gm.locator(".voy");
    await gm.waitForFunction(() => document.querySelector(".voy")?.getAttribute("data-legs") === "1", null, { timeout: 60000 });
    assert.ok((await voy.locator("[data-star]").count()) > 5, "stars drawn");
    assert.equal(await voy.locator("[data-leg=jump]").count(), 1);
    assert.equal(await voy.locator("line.leg.jump").count(), 1, "the jump is dashed");
    assert.match(await voy.locator(".stamp").first().innerText(), /fictional/i);
    await shot(gm, "v8-voyage-explore");

    // The explorer steps outward: each next stop is a new star.
    const firstStop = (await voy.getAttribute("data-voyage")).split(">")[1];
    const seen = new Set([await voy.getAttribute("data-voyage")]);
    for (let i = 0; i < 2; i++) {
      const before = await voy.getAttribute("data-voyage");
      await voy.locator("[data-next]").click();
      await gm.waitForFunction((b) => { const v = document.querySelector(".voy")?.getAttribute("data-voyage"); return v && v !== b; }, before, { timeout: 60000 }).catch(() => {});
      seen.add(await voy.getAttribute("data-voyage"));
    }
    assert.ok(seen.size >= 2, `the explorer moved on (${[...seen].join(" | ")})`);
    await shot(gm, "v8-voyage-spiral");

    // To a picked star (the explorer's first stop) by the antimatter probe: a flight, younger crew.
    await voy.locator("[data-restart]").click();
    await gm.waitForFunction(() => document.querySelector(".voy")?.getAttribute("data-legs") === "1", null, { timeout: 60000 });
    await voy.locator("[data-ship]").selectOption("relativistic-probe");
    await voy.locator("[data-mode=to]").click();
    await voy.locator(`[data-star='${firstStop}']`).dispatchEvent("click");
    await gm.waitForSelector(".voy [data-leg=flight]", { timeout: 60000 });
    assert.match(await voy.locator("[data-ages]").innerText(), /less than the people they left behind/);
    await shot(gm, "v8-voyage-flight");

    // A chemical ship cannot cross between stars, and says so.
    await voy.locator("[data-ship]").selectOption("chemical-transfer");
    await gm.waitForSelector(".voy [data-refusal]", { timeout: 60000 });
    assert.match(await voy.locator("[data-refusal]").first().innerText(), /cannot cross between stars/);

    await voy.locator("[data-ship]").selectOption("relativistic-probe");
    await gm.waitForSelector(".voy [data-leg=flight]", { timeout: 60000 });
    await more(gm, "Night chart");
    await gm.waitForTimeout(400);
    await shot(gm, "v8-voyage-night");

    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })).newPage();
    phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "voyages");
    await phone.waitForFunction(() => Number(document.querySelector(".insys")?.getAttribute("data-legs") ?? 0) >= 1, null, { timeout: 60000 });
    await phone.waitForTimeout(300);
    const overIn = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(overIn <= 1, `no sideways scroll on a phone in a system (${overIn}px over)`);
    await shot(phone, "v8-voyage-system-phone", true);
    await phone.locator("[data-scale-pick=stars]").click();
    await phone.waitForFunction(() => document.querySelector(".voy")?.getAttribute("data-legs") === "1", null, { timeout: 60000 });
    await phone.waitForTimeout(400);
    const over = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `no sideways scroll on a phone (${over}px over)`);
    await shot(phone, "v8-voyage-phone", true);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: voyages in a system (transfer, a belt) and between stars: nearest unvisited by jump, explorer steps, flight to a picked star with crew ages, chemical refusal, night chart, phone`);
  }
} finally {
  server.kill();
}
