// Ship manifest and crew roster (track MC) in a real browser, against the built shell (scripts/build-shell.sh
// first): add the Traveller pack, put a stock courier in the system from Ships and stations, bring two people aboard
// (one of a kind of being with no life support figures), load water, food and oxygen, and read the days of life
// support; put one person in cold sleep and see the days grow; move cargo to another ship. Phone width fits.
// Usage: node tests/ui/manifest.e2e.mjs   (SSE_SHOTS=<dir> for screenshots)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 9200 + Math.floor(Math.random() * 300);
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shots = process.env.SSE_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });
const shot = async (page, n) => shots && (await page.screenshot({ path: `${shots}/${n}.png` }));

try {
  const browser = await chromium.launch();
  const errors = [];
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: shots ? 2 : 1 });
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => process.env.SSE_DEBUG && console.log("console", m.type(), m.text()));
  await page.goto(`http://localhost:${port}/#chart`);
  await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await go(page, "packs");
  const packs = page.locator("[role=region][aria-label=Packs]");
  await packs.getByRole("button", { name: "Add the Traveller pack" }).click();
  await packs.locator("[aria-label=Proposal]").getByRole("button", { name: "Accept" }).click();
  await page.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sse.traveller"));
  await go(page, "system");
  await page.locator(".rail button", { hasText: "Ships and stations" }).first().click();
  const desk = page.locator("[role=region][aria-label='Ships and stations']");
  await desk.waitFor();

  // Two ships: a courier and a free trader.
  for (const [tpl, name] of [["courier", "Kittiwake"], ["trader", "Brass Lark"]]) {
    await desk.locator("[data-add-here]").click();
    await desk.locator(`[data-template="${tpl}"]`).click();
    await desk.locator("[data-name]").fill(name);
    await desk.locator("[data-add]").click();
    await desk.locator("[data-note]", { hasText: `${name} added` }).waitFor();
  }
  await desk.locator("[data-construct]", { hasText: "Kittiwake" }).click();

  // Crew: two people, one with a kind of being the packs have no figures for.
  // One Aboard button; the panel opens on Crew.
  await desk.locator("[data-aboard-toggle]").click();
  const crew = desk.locator("[data-aboard=crew]");
  for (const [name, role, being, age] of [["Ada Moss", "pilot", "", "30"], ["Tovan", "engineer", "vesh", "112"], ["Rhee", "medic", "human", "44"]]) {
    await crew.locator("[data-add-person]").click();
    await crew.locator("[data-person-name]").fill(name);
    await crew.locator("[data-role]").fill(role);
    if (being) await crew.locator("[data-being]").fill(being);
    await crew.locator("[data-age]").fill(age);
    await crew.locator("[data-add-confirm]").click();
    await crew.locator("[data-person]", { hasText: name }).waitFor({ timeout: 5000 }).catch(async (e) => { console.log("NOTE", await crew.innerText()); throw e; });
  }
  assert.equal(await crew.locator("[data-person]").count(), 3);
  assert.match(await crew.locator("[data-person]", { hasText: "Ada Moss" }).innerText(), /30 yr/);
  assert.match(await crew.locator("[data-person]", { hasText: "Tovan" }).innerText(), /vesh · 112 yr/);

  // Manifest: load stores and goods.
  await desk.locator("[data-part=manifest]").click();
  const man = desk.locator("[data-aboard=manifest]");
  for (const [what, amount, unit] of [["water", "500", "kg"], ["food", "200", "kg"], ["oxygen", "60", "kg"], ["machine parts", "3", "t"]]) {
    await man.locator("[data-add-cargo]").click();
    await man.locator("[data-what]").fill(what);
    await man.locator("[data-amount]").fill(amount);
    await man.locator("[data-unit]").fill(unit);
    await man.locator("[data-load]").click();
    await man.locator("[data-cargo] li", { hasText: what }).waitFor();
  }
  assert.match(await man.locator("[data-people-count]").innerText(), /^3 of 4/);
  // Two humans awake draw on the stores; the vesh have no figures and are named apart.
  const life = await man.locator("[data-life]").innerText();
  assert.match(life, /Oxygen runs out first/, life);
  assert.match(life, /2 people awake/);
  assert.ok(await man.locator("[data-unrated]", { hasText: "vesh" }).count());
  const before = await man.locator("[data-first-days]").innerText();
  await shot(page, "manifest-desk");

  // Cold sleep for Rhee: one person draws, so the oxygen lasts twice as long.
  await desk.locator("[data-part=crew]").click();
  await crew.locator("[data-person]", { hasText: "Rhee" }).locator("[data-state]").selectOption("cold_sleep");
  await page.waitForTimeout(400);
  await shot(page, "crew-desk");
  await desk.locator("[data-part=manifest]").click();
  await man.locator("[data-life]", { hasText: "1 person awake" }).waitFor();
  const after = await man.locator("[data-first-days]").innerText();
  assert.notEqual(before, after, `days left changed with cold sleep (${before} then ${after})`);

  // Move the machine parts to the other ship.
  await man.locator("[data-cargo] li", { hasText: "machine parts" }).getByRole("button", { name: "Move…" }).click();
  const to = man.locator("[data-move-to]");
  const lark = await to.locator("option", { hasText: "Brass Lark" }).getAttribute("value");
  await to.selectOption(lark);
  await man.locator("[data-move-confirm]").click();
  await man.locator("[data-aboard-note]", { hasText: "moved to Brass Lark" }).waitFor();
  await man.locator("[data-cargo] li", { hasText: "machine parts" }).waitFor({ state: "detached" });
  // The panel stays open as the card changes ship.
  await desk.locator("[data-construct]", { hasText: "Brass Lark" }).click();
  await desk.locator("[data-aboard=manifest] [data-cargo] li", { hasText: "machine parts" }).waitFor();

  // Night chart, then phone width.
  await desk.locator("[data-construct]", { hasText: "Kittiwake" }).click();
  await man.locator("[data-life]").waitFor();
  await more(page, "Night chart");
  await page.waitForTimeout(300);
  await shot(page, "manifest-night-desk");
  await more(page, "Night chart");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.waitForTimeout(400);
  const over = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  assert.ok(over <= 1, `page overflows by ${over}px at phone width`);
  await man.scrollIntoViewIfNeeded();
  await shot(page, "manifest-phone");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log(`PASS chromium: two ships added, three aboard Kittiwake (vesh named apart), stores loaded; oxygen first ${before}, ${after} with one in cold sleep; parts moved to Brass Lark; phone fits`);
} finally {
  server.kill();
}
