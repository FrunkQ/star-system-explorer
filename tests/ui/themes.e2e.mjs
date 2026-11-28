// V2 acceptance in a real browser, against the built shell: one player
// screen renders correctly in every shipped theme. For each theme: the
// screen draws with the theme's frame parts (checked on computed styles),
// keeps every fact, has no sideways scroll on desktop or phone, and raises
// no page errors. A theme's vocabulary and filter chain are checked too,
// including reduced motion and the effects switch.
// Usage: node tests/ui/themes.e2e.mjs [chromium firefox webkit] [--screens <dir>]
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { PLAYER_THEMES, ALL_THEMES } from "../../packages/skins/player/src/index.ts";
import { resolveTheme } from "../../packages/skins/chart-room/src/index.ts";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8600 + Math.floor(Math.random() * 300);
const url = `http://localhost:${port}/players.html`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const hex = (rgb) => "#" + rgb.match(/\d+/g).slice(0, 3).map((n) => Number(n).toString(16).padStart(2, "0")).join("").toUpperCase();

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const rows = [];
    let baseline = null;
    for (const theme of PLAYER_THEMES) {
      const tokens = resolveTheme(theme.id, ALL_THEMES);
      for (const [form, viewport] of [["desktop", { width: 1440, height: 1000 }], ["phone", { width: 390, height: 844 }]]) {
        const page = await browser.newPage({ viewport });
        page.on("pageerror", (e) => errors.push(`${theme.id}: ${e.message}`));
        await page.goto(url);
        await page.locator("[data-view='sse.view.card'] article").waitFor();
        await page.getByLabel("Theme").selectOption(theme.id);
        await page.waitForFunction((id) => document.documentElement.dataset.theme === id, theme.id);
        const card = page.locator("[data-view='sse.view.card'] article");
        await card.waitFor();
        const look = await card.evaluate((el) => {
          const s = getComputedStyle(el);
          const h = getComputedStyle(el.querySelector("header"));
          return { bg: s.backgroundColor, radius: s.borderTopLeftRadius, cap: h.backgroundColor, title: getComputedStyle(el.querySelector("h2")).textTransform };
        });
        assert.equal(hex(look.bg), String(tokens["panel-ground"]).toUpperCase(), `${theme.id}: card is a panel`);
        assert.equal(look.radius, String(tokens["frame-radius"]), `${theme.id}: frame radius`);
        assert.equal(hex(look.cap), String(tokens["cap-ground"]).toUpperCase(), `${theme.id}: card cap`);
        assert.equal(look.title, String(tokens["title-case"]), `${theme.id}: title case`);
        const massLabel = (await card.locator("[data-field='mass_kg'] dt").innerText()).trim();
        const facts = await card.locator("[data-field]").count();
        baseline ??= facts;
        assert.equal(facts, baseline, `${theme.id}: every fact is still there`);
        const w = await page.evaluate(() => document.scrollingElement.scrollWidth);
        assert.ok(w <= viewport.width, `${theme.id} ${form}: scrolls sideways (${w}px)`);
        const fx = await page.locator("[data-filter]").evaluateAll((els) => els.map((e) => e.dataset.filter));
        if (theme.id === "phosphor" && form === "desktop") {
          assert.deepEqual(fx, ["scanlines", "vignette", "flicker"]);
          await page.getByLabel("Screen effects").uncheck();
          assert.equal(await page.locator("[data-filter]").count(), 0, "effects switch turns the chain off");
          await page.getByLabel("Screen effects").check();
        }
        if (theme.id === "guide") assert.equal(massLabel.toLowerCase(), "heft", "the theme's vocabulary is used");
        if (screens && name === "chromium") {
          mkdirSync(screens, { recursive: true });
          await page.screenshot({ path: `${screens}/theme-${theme.id}-${form}.png`, fullPage: form === "desktop" });
        }
        if (form === "desktop") rows.push(`${theme.id}: panel ${hex(look.bg)}, radius ${look.radius}, cap ${hex(look.cap)}, mass label "${massLabel}", filters [${fx}]`);
        await page.close();
      }
    }
    // Reduced motion: the flicker step is left out.
    const ctx = await browser.newContext({ reducedMotion: "reduce" });
    const quiet = await ctx.newPage();
    await quiet.goto(url);
    await quiet.locator("[data-view='sse.view.card'] article").waitFor();
    await quiet.getByLabel("Theme").selectOption("phosphor");
    await quiet.waitForFunction(() => document.querySelectorAll("[data-filter]").length > 0);
    assert.deepEqual(await quiet.locator("[data-filter]").evaluateAll((els) => els.map((e) => e.dataset.filter)), ["scanlines", "vignette"]);
    await ctx.close();
    await browser.close();
    assert.deepEqual(errors, [], "no page errors");
    console.log(rows.map((r) => `V2 ${name} ${r}`).join("\n"));
    console.log(`PASS ${name}`);
  }
} finally {
  server.kill();
}
