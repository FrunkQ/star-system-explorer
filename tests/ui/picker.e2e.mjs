// The picker beside the place's name (24 Nov), on the Solar System, in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/picker.e2e.mjs [--screens <dir>]
// Browsing is the real hierarchy with the levels above first and moons folded behind a count that opens in place;
// typing searches a flat list that says where each match is; kind chips add together and keep the path to a match;
// Escape clears the search, then closes and gives the chip the keyboard; arrows and Enter pick. Typing also finds
// things elsewhere in the campaign, and picking one goes there. Moving about writes nothing.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { pinClock } from "./nav.mjs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: false }));
};
const names = (loc) => loc.locator(".name").evaluateAll((e) => e.map((x) => x.textContent.trim()));

try {
  const browser = pinClock(await chromium.launch());
  const errors = [];
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(root);
  await gm.locator('[data-door="sol"]').click();
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.textContent?.trim() === "The Solar System", null, { timeout: 30000 });
  const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  const chip = gm.locator(".place [data-picker-chip]");
  assert.match(await chip.innerText(), /^\d+ bodies/);

  // Open: the search box has the keyboard; the levels above come first, then the system, the planets in order out.
  await chip.click();
  const pop = gm.locator("[data-picker-pop]");
  await pop.waitFor();
  assert.ok(await gm.evaluate(() => document.activeElement?.hasAttribute("data-picker-find")), "typing finds at once");
  assert.deepEqual(await pop.locator(".above [data-pick]").evaluateAll((b) => b.map((x) => x.dataset.pick)), ["up:galaxy", "up:cluster", "up:web"]);
  const rows = await names(pop.locator(".tree"));
  for (const [a, b] of [["Mercury", "Venus"], ["Venus", "Earth"], ["Earth", "Mars"], ["Mars", "Jupiter"], ["Jupiter", "Saturn"]]) assert.ok(rows.indexOf(a) >= 0 && rows.indexOf(a) < rows.indexOf(b), `${a} before ${b}`);
  assert.ok(!rows.includes("Moon"), "Earth's moon is folded");
  const fold = pop.locator(".tree li").filter({ has: gm.locator(".name", { hasText: /^Earth$/ }) }).locator("[data-picker-fold]");
  assert.equal((await fold.innerText()).trim(), "1 moon ›");
  const saturn = pop.locator(".tree li").filter({ has: gm.locator(".name", { hasText: /^Saturn$/ }) }).locator("[data-picker-fold]");
  assert.match(await saturn.innerText(), /^\d+ (rings|belts and rings|inside) ›$/);
  await shot(gm, "picker-sol");
  await fold.click();
  assert.ok((await names(pop.locator(".tree"))).includes("Moon"), "the fold opens in place");
  await shot(gm, "picker-sol-earth-open");

  // Search: a flat list that says where a moon is.
  await pop.locator("[data-picker-find]").fill("moo");
  const moon = pop.locator(".tree li").filter({ has: gm.locator(".name", { hasText: /^Moon$/ }) });
  assert.match(await moon.locator(".cap").innerText(), /^in Earth$/);
  await shot(gm, "picker-sol-search");
  await gm.keyboard.press("Escape");
  assert.equal(await pop.locator("[data-picker-find]").inputValue(), "", "the first Escape clears the search");
  assert.ok(await pop.isVisible(), "and leaves the list open");

  // A kind chip keeps the path: moons, with their planets kept quietly (not pickable).
  await pop.locator('[data-picker-kind="Moons"]').click();
  const earth = pop.locator(".tree [data-pick-row]").filter({ has: gm.locator(".name", { hasText: /^Earth$/ }) });
  assert.ok(await earth.isDisabled(), "Earth is kept as the path to the Moon");
  assert.ok((await names(pop.locator(".tree"))).includes("Moon"));
  await shot(gm, "picker-sol-moons");
  await pop.locator('[data-picker-kind="Moons"]').click();

  // Escape again closes it and gives the keyboard back to the chip.
  await gm.keyboard.press("Escape");
  await pop.waitFor({ state: "detached" });
  assert.ok(await gm.evaluate(() => document.activeElement?.hasAttribute("data-picker-chip")));

  // The keyboard: type, Down, Enter picks.
  await gm.keyboard.press("Enter");
  await pop.waitFor();
  await gm.keyboard.type("mars");
  await gm.keyboard.press("ArrowDown");
  await gm.keyboard.press("Enter");
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.textContent?.trim() === "Mars", null, { timeout: 20000 });

  // Up one level from the picker, back to the system.
  await chip.click();
  await pop.locator('[data-pick="up:system"]').click();
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "system", null, { timeout: 20000 });

  // Search covers the campaign (V3 gap 37): from the galaxy, typing finds a body of a system already read, saying
  // where it is, and picking it goes straight there.
  await gm.locator('.place [data-place-step="galaxy"]').click();
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "galaxy", null, { timeout: 20000 });
  await chip.click();
  await pop.locator("[data-picker-find]").fill("earth");
  const away = pop.locator("[data-picker-elsewhere] + ul .row").filter({ has: gm.locator(".name", { hasText: /^Earth$/ }) });
  assert.match(await away.locator(".cap").innerText(), /^in The Solar System$/);
  await shot(gm, "picker-find-elsewhere");
  await away.click();
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "world", null, { timeout: 20000 });
  assert.equal((await gm.locator(".place h1.title").innerText()).trim(), "Earth");

  // A phone: a small ▾ beside the name opens the list across the screen.
  const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, hasTouch: true })).newPage();
  phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
  await phone.goto(root);
  await phone.locator('[data-door="sol"]').tap();
  await phone.waitForFunction(() => document.querySelector(".place h1.title")?.textContent?.trim() === "The Solar System", null, { timeout: 30000 });
  await phone.locator(".place [data-picker-chip]").tap();
  const pp = phone.locator("[data-picker-pop]");
  await pp.waitFor();
  await phone.waitForFunction(() => document.querySelector("[data-picker-pop]")?.getAnimations().length === 0, null, { timeout: 5000 });
  assert.ok(!(await phone.evaluate(() => document.activeElement?.hasAttribute("data-picker-find"))), "a touch screen keeps its keyboard down until asked");
  const box = await pp.boundingBox();
  assert.ok(box && box.x >= 15 && box.x + box.width <= 375, `the list fits the screen (${JSON.stringify(box)})`);
  const overflow = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(overflow <= 1, `nothing runs off the side (${overflow}px)`);
  await shot(phone, "picker-phone");

  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "moving about writes nothing");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("picker e2e: ok");
} finally {
  server.kill();
}
