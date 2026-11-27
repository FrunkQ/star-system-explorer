// Energy flow, on the Body screen (track HF's screen), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/heat.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Open the Solar System from the start sheet, pick Earth, open the Body screen and "Energy flow": the flow is drawn from the
// kernel's core history, the words say how much heat it loses, sliding back to its youth shows more heat lost and
// changes nothing in the campaign, and "Back to now" returns. Above it, the outside half: sunlight in, the share
// reflected as a bar, what is given off, and that life's share is not published yet. The Moon, under a lid, carries its honesty mark. On a
// phone the open panel never scrolls sideways.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { depth, go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n, loc) => {
  if (!screens) return;
  mkdirSync(screens, { recursive: true });
  // The panel scrolled to the top of the stage, then the page as the user sees it (the ruler stays fixed below).
  if (loc) await loc.evaluate((el) => el.scrollIntoView({ block: "start" }));
  await page.waitForTimeout(250);
  await page.screenshot({ path: `${screens}/${n}.png` });
};
/** Watts from the first figure the panel prints ("12.9 PW", "34.7 TW", "279 GW"). */
const watts = (s) => {
  const m = /([\d,.]+)\s*(PW|TW|GW|MW|kW|W)\b/.exec(s);
  assert.ok(m, `a power in: ${s}`);
  return Number(m[1].replace(/,/g, "")) * { PW: 1e15, TW: 1e12, GW: 1e9, MW: 1e6, kW: 1e3, W: 1 }[m[2]];
};

async function sol(page) {
  await page.goto(base);
  await page.locator('[data-door="sol"]').click();
  await page.waitForFunction(() => document.querySelector(".place h1.title")?.textContent?.trim() === "The Solar System", null, { timeout: 30000 });
}
/** Pick a world by the picker (find it by name, press its row), open the Body screen and "Energy flow". */
async function openHeat(page, world) {
  await page.locator(".place [data-picker-chip]").click();
  await page.locator("[data-picker-pop]").waitFor();
  await page.locator("[data-picker-find]").fill(world.toLowerCase());
  await page.locator("[data-picker-pop] .tree li").filter({ has: page.locator(".name", { hasText: new RegExp(`^${world}$`) }) }).first().locator(".name").click();
  await page.waitForFunction((w) => document.querySelector(".place h1.title")?.textContent?.trim() === w, world, { timeout: 20000 });
  await go(page, "body");
  const toggle = page.locator("[data-heat-toggle]");
  await toggle.waitFor({ timeout: 15000 });
  if ((await toggle.getAttribute("aria-expanded")) !== "true") await toggle.click();
  await page.waitForSelector("[data-heat-why], [data-heat-failed], [data-heat-none]", { timeout: 30000 });
  const heat = page.locator("[data-heat]");
  assert.equal(await heat.locator("[data-heat-failed]").count(), 0, `${world}: ${await heat.innerText()}`);
  return heat;
}

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await sol(gm);
    const hashBefore = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");

    const heat = await openHeat(gm, "Earth");
    assert.equal(await heat.locator("[data-heat-when]").innerText().then((t) => t.startsWith("Now")), true);
    assert.ok((await heat.locator("[data-heat-inside] [data-flow] [data-band]").count()) >= 2, "the flow draws the kernel's bands");
    // The outside half: the climate's sunlight books, reflected share as a bar, and how it compares with the inside.
    assert.ok((await heat.locator("[data-heat-outside] [data-flow] [data-band]").count()) >= 3, "sunlight in, reflected, given off");
    assert.match(await heat.locator("[data-heat-albedo]").getAttribute("aria-label"), /^\d+% of the sunlight reflected/);
    assert.match(await heat.locator("[data-heat-sun-why]").innerText(), /reflects \d+% straight back to space[\s\S]*sunlight, not its inside, sets how warm/);
    assert.match(await heat.locator("[data-heat-life]").innerText(), /not published yet/);
    const nowWhy = await heat.locator("[data-heat-why]").innerText();
    assert.match(nowWhy, /loses .* of heat from inside through its surface/);
    const nowW = watts(nowWhy);
    assert.ok(nowW > 1e12 && nowW < 1e14, `Earth loses ${nowW} W`);
    // Today's kernel publishes only part of the books, and says so.
    assert.match(await heat.locator("[data-heat-missing]").innerText(), /not publish the rest of the books yet/);
    // A fresh browser opens at Curious: names and two sentences, no line. Science adds the figures and the line.
    assert.equal(await heat.locator("[data-heat-line]").count(), 0, "Curious keeps the line a step away");
    await shot(gm, `heat-earth-curious-${name}`, heat);
    await depth(gm, "Science");
    await gm.waitForSelector("[data-heat-line]");
    assert.equal(await heat.locator("[data-heat-line] svg").count(), 2, "temperature and heat lines");
    await shot(gm, `heat-earth-now-${name}`, heat);

    // Slide to its youth: more heat lost, and the campaign is untouched.
    const slider = heat.locator("[data-heat-slider]");
    await slider.fill("0");
    await gm.waitForSelector("[data-heat-looking]");
    const thenWhy = await heat.locator("[data-heat-why]").innerText();
    assert.match(thenWhy, /ago, Earth .*lost/);
    assert.ok(watts(thenWhy) > nowW, "young Earth lost more heat than today");
    assert.equal(await heat.locator("[data-heat-today]").count(), 1, "the sunlight half says it is today's");
    await shot(gm, `heat-earth-young-${name}`, heat);
    await heat.locator("[data-heat-now]").click();
    assert.ok((await heat.locator("[data-heat-when]").innerText()).startsWith("Now"));
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hashBefore, "reading heat never changes the state hash");

    // The Moon, under a lid: the known fault is said where it applies.
    const moon = await openHeat(gm, "Moon");
    await moon.locator("[data-heat-caveats] summary").click();
    assert.equal(await moon.locator('[data-heat-caveat="lid-hot"]').count(), 1);
    // A moon's sunlight is its planet's distance from the Sun (HV2c): about Earth's per square metre, on a smaller world.
    const moonWhy = await moon.locator("[data-heat-sun-why]").innerText();
    const moonSun = watts(moonWhy);
    assert.ok(moonSun > 5e15 && moonSun < 3e16, `the Moon gets ${moonSun} W of sunlight`);
    await shot(gm, `heat-moon-${name}`, moon);

    // Phone: the open panel fits.
    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true })).newPage();
    phone.on("pageerror", (e) => errors.push(e.message));
    await sol(phone);
    const ph = await openHeat(phone, "Earth");
    for (const f of await ph.locator("[data-flow]").all()) assert.equal(await f.getAttribute("data-vertical"), "true", "each flow runs top to bottom on a phone");
    const wide = await phone.evaluate(() => document.documentElement.scrollWidth);
    assert.ok(wide <= 390, `no sideways scroll on a phone: ${wide}`);
    await shot(phone, `heat-earth-phone-${name}`, ph);

    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`heat e2e ${name}: ok (Earth loses ${nowW / 1e12} TW now)`);
  }
} finally {
  server.kill();
}
