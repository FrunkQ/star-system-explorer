// Ship manifest and crew roster (track MC) in a real browser, against the built shell (scripts/build-shell.sh
// first): add the Traveller pack, put a stock courier in the system from Ships and stations, bring two people aboard
// (one of a kind of being with no life support figures), load water, food and oxygen, and read the days of life
// support; put one person in cold sleep and see the days grow; move cargo to another ship. Phone width fits.
// Usage: node tests/ui/manifest.e2e.mjs   (SSE_SHOTS=<dir> for screenshots)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk as chooseDesk, go, more } from "./nav.mjs";
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
  await desk.locator("[data-picture-toggle]").click();
  // In play (Explore) what is aboard can change, but the ship's look cannot: amounts are open, the design is fixed.
  assert.equal(await man.locator("[data-cargo-amount]").count(), 4, "amounts can change in play");
  await desk.locator("[data-picture=designed] svg").first().waitFor();
  assert.equal(await desk.locator("[data-picture=designed] [data-remix]").count(), 0, "no remix outside Build");
  assert.equal(await desk.locator("[data-picture-file]").count(), 0, "no own picture outside Build");

  // Known to (MC2): on the Build desk each person aboard has an eye, not yet known to the table (as known as their
  // place, which under Kestrel's ladder is heard of); one press shows the whole crew, and Undo puts them back.
  await chooseDesk(page, "Build");
  await desk.locator("[data-part=crew]").click();
  await crew.locator("[data-person] [data-eye]").first().waitFor();
  const eyesOf = () => crew.locator("[data-person] [data-eye]").evaluateAll((els) => els.map((e) => e.getAttribute("data-eye")));
  const start = await eyesOf();
  assert.equal(start.length, 3);
  assert.ok(start.every((e) => e !== "known"), `people aboard start unknown to the table (${start})`);
  await crew.locator("[data-show-all]").click();
  await page.waitForFunction(() => [...document.querySelectorAll("[data-aboard=crew] [data-person] [data-eye]")].every((e) => e.getAttribute("data-eye") === "known"));
  await shot(page, "crew-known-desk");
  await crew.locator("[data-show-all-undo]").click();
  await page.waitForFunction((was) => JSON.stringify([...document.querySelectorAll("[data-aboard=crew] [data-person] [data-eye]")].map((e) => e.getAttribute("data-eye"))) === JSON.stringify(was), start);

  // The ship's picture (edit mode, here the Build desk): designed from its parts by default, its remix kept with the
  // ship; an own picture replaces it, scaled down, and "Use the designed ship" takes it back.
  const pic = desk.locator("section[aria-label=Picture]");
  await pic.locator("[data-picture=designed]").or(desk.locator("[data-picture=designed]")).first().waitFor();
  await desk.locator("[data-picture=designed] svg").first().waitFor();
  await desk.locator("[data-picture=designed] [data-remix]").click();
  await desk.locator("[data-picture-note]", { hasText: "Design kept" }).waitFor();
  assert.equal(await desk.locator("[data-field='construct.design']").count(), 0, "the recipe is drawn, not listed");
  await shot(page, "picture-designed-desk");
  const png = await page.evaluate(() => {
    const c = document.createElement("canvas");
    c.width = 1800;
    c.height = 1000;
    const g = c.getContext("2d");
    g.fillStyle = "#1b2a3a";
    g.fillRect(0, 0, 1800, 1000);
    g.fillStyle = "#e8c070";
    g.beginPath();
    g.moveTo(200, 500); g.lineTo(1500, 380); g.lineTo(1650, 500); g.lineTo(1500, 620); g.closePath();
    g.fill();
    return c.toDataURL("image/png").split(",")[1];
  });
  await desk.locator("[data-picture-file]").setInputFiles({ name: "kittiwake.png", mimeType: "image/png", buffer: Buffer.from(png, "base64") });
  const own = desk.locator("img[data-own-picture]");
  await own.waitFor();
  const natural = await own.evaluate((i) => [i.naturalWidth, i.naturalHeight]);
  assert.deepEqual(natural, [960, 533], `own picture scaled to fit 960 (${natural})`);
  await shot(page, "picture-own-desk");
  await desk.locator("[data-picture-clear]").click();
  await desk.locator("[data-picture=designed] svg").first().waitFor();
  await chooseDesk(page, "Explore");

  // Known to (MC2): on the Build desk each person aboard has an eye, not yet known to the table (as known as their
  // place, which under Kestrel's ladder is heard of); one press shows the whole crew, and Undo puts them back.
  await chooseDesk(page, "Build");
  await desk.locator("[data-part=crew]").click();
  await crew.locator("[data-person] [data-eye]").first().waitFor();
  const eyesOf = () => crew.locator("[data-person] [data-eye]").evaluateAll((els) => els.map((e) => e.getAttribute("data-eye")));
  const start = await eyesOf();
  assert.equal(start.length, 3);
  assert.ok(start.every((e) => e !== "known"), `people aboard start unknown to the table (${start})`);
  await crew.locator("[data-show-all]").click();
  await page.waitForFunction(() => [...document.querySelectorAll("[data-aboard=crew] [data-person] [data-eye]")].every((e) => e.getAttribute("data-eye") === "known"));
  await shot(page, "crew-known-desk");
  await crew.locator("[data-show-all-undo]").click();
  await page.waitForFunction((was) => JSON.stringify([...document.querySelectorAll("[data-aboard=crew] [data-person] [data-eye]")].map((e) => e.getAttribute("data-eye"))) === JSON.stringify(was), start);
  await chooseDesk(page, "Explore");

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
