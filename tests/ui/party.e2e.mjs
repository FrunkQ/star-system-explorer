// Landing run-through (blocker 4, gap 9) in a real browser, against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/party.e2e.mjs [--screens <dir>]. The Solar System with the Traveller pack: picking Earth makes
// Earth the voyage's start; a far trader added at Earth becomes the party's ship; Voyages flies it from Earth to
// Mars ("Fly … there" moves it) and offers "Land here", which opens Land on Mars. Voyages' names never overlap.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: true }));
};
const noOverlap = async (page) => {
  const boxes = await page.$$eval(".insys text[data-label-lane]", (ts) => ts.map((t) => { const r = t.getBoundingClientRect(); return { n: t.textContent, l: r.left, r: r.right, t: r.top, b: r.bottom }; }));
  for (let i = 0; i < boxes.length; i++)
    for (let j = i + 1; j < boxes.length; j++) {
      const [a, b] = [boxes[i], boxes[j]];
      const hit = a.l < b.r && b.l < a.r && a.t < b.b && b.t < a.b;
      assert.ok(!hit, `${a.n} and ${b.n} overlap`);
    }
  return boxes.length;
};

try {
  const browser = await chromium.launch();
  const errors = [];
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 });
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(base);
  await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
  await gm.locator('[data-door="sol"]').click();
  const earthDot = gm.locator('.area-chart .body[aria-label^="Earth,"]').first();
  await earthDot.waitFor({ timeout: 20000 });
  await go(gm, "packs");
  const packs = gm.locator("[role=region][aria-label=Packs]");
  await packs.getByRole("button", { name: "Add the Traveller pack" }).click();
  await packs.locator("[aria-label=Proposal]").getByRole("button", { name: "Accept" }).click();
  await gm.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sse.traveller"));
  await go(gm, "system");
  await earthDot.waitFor({ timeout: 20000 });
  await earthDot.locator(".hit").click();

  // Gap 9: the picked world is the start.
  await go(gm, "voyages");
  const ins = gm.locator(".insys");
  await ins.waitFor({ timeout: 30000 });
  await gm.waitForFunction(() => Number(document.querySelector(".insys")?.getAttribute("data-legs") ?? 0) >= 1, null, { timeout: 60000 });
  const earth = await ins.locator("[data-place] >> nth=0").evaluate(() => [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === "Earth")?.getAttribute("data-place"));
  assert.ok(earth, "Earth on the orbit line");
  assert.ok((await ins.getAttribute("data-voyage")).startsWith(`${earth}>`), `starts at Earth: ${await ins.getAttribute("data-voyage")}`);
  const named = await noOverlap(gm);
  assert.ok(named >= 4, `names drawn: ${named}`);
  await shot(gm, "party-voyage-from-earth");

  // Blocker 4: a far trader at Earth becomes the party's ship.
  await go(gm, "constructs");
  const desk = gm.locator("[role=region][aria-label='Ships and stations']");
  await desk.locator("[data-add-here]").click();
  await desk.locator("[data-under]").selectOption(earth);
  await desk.locator("[data-template='far_trader']").click();
  await desk.locator("[data-name]").fill("Beowulf");
  await desk.locator("[data-add]").click();
  const row = desk.locator("[data-construct]", { hasText: "Beowulf" });
  await row.waitFor({ timeout: 15000 });
  if ((await row.getAttribute("aria-pressed")) !== "true") await row.click();
  await desk.locator("[data-party-ship]").click();
  await desk.locator("[data-party]").first().waitFor();
  await shot(gm, "party-ship-picked");

  // The table always knows where the party's ship is: a joined player sees it in the header, and it moves with the voyage.
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const pl = await context.newPage();
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(await gm.getAttribute("[data-players-link]", "data-players-link"));
  await pl.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  const aboard = pl.locator("header [data-party-ship]");
  await aboard.waitFor({ timeout: 15000 });
  assert.equal(await aboard.getAttribute("data-party-at"), earth);
  assert.match(await aboard.innerText(), /Aboard Beowulf, at Earth/);

  // Voyages flies it from Earth to Mars, and offers Land there.
  await go(gm, "voyages");
  await gm.waitForFunction(() => (document.querySelector(".insys [data-ship]")?.value ?? "") === "party", null, { timeout: 15000 });
  await gm.waitForFunction(() => [...document.querySelectorAll(".insys [data-place]")].some((e) => e.getAttribute("aria-label") === "Mars"), null, { timeout: 60000 });
  const mars = await gm.evaluate(() => [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === "Mars")?.getAttribute("data-place"));
  await ins.locator(`[data-place='${mars}']`).dispatchEvent("click");
  await gm.waitForFunction((m) => document.querySelector(".insys")?.getAttribute("data-voyage")?.endsWith(`>${m}`), mars, { timeout: 60000 });
  assert.ok((await ins.getAttribute("data-voyage")).startsWith(`${earth}>`));
  assert.match(await ins.locator("[data-next]").innerText(), /Fly Beowulf there/);
  // X3h: with the Traveller pack on, each danger on the At world card carries the game's own task (GM only).
  await gm.waitForSelector(".insys [data-at-world] [data-act]", { timeout: 60000 });
  const heading = await ins.locator("[data-at-world] h4").first().textContent();
  assert.ok(!heading.includes("body:"), `the At world card names the world, not its address: ${heading}`);
  const dangers = await ins.locator("[data-at-world] [data-danger]").count();
  assert.equal(await ins.locator("[data-at-world] [data-danger] [data-game-difficulty]").count(), dangers);
  if (dangers) assert.match(await ins.locator("[data-game-difficulty]").first().innerText(), /\d+\+/);
  console.log(`dangers with the game's task: ${dangers}`);
  if (dangers) await ins.locator("[data-game-difficulty]").first().scrollIntoViewIfNeeded();
  await shot(gm, "x3h-at-world-traveller");
  await ins.locator("[data-next]").click();
  await ins.locator("[data-land-here]").waitFor({ timeout: 15000 });
  assert.match(await ins.locator("[data-arrived]").innerText(), /Beowulf is at Mars/);
  // The flight goes into Beowulf's flight log on the record (V8 flight log).
  await ins.locator("[data-logged]").waitFor({ timeout: 15000 });
  assert.match(await ins.locator("[data-logged]").innerText(), /\d+ entries written to Beowulf's flight log/);
  // What the voyage burned comes off Beowulf's tanks, on its record (fuel draw-down).
  await ins.locator("[data-drawn]").waitFor({ timeout: 15000 });
  assert.match(await ins.locator("[data-drawn]").innerText(), /^Tanks now: (reaction mass|jump fuel) [\d.]+ t/);
  await noOverlap(gm);
  await shot(gm, "party-arrived-at-mars");
  await pl.waitForFunction((m) => document.querySelector("header [data-party-ship]")?.getAttribute("data-party-at") === m, mars, { timeout: 15000 });
  assert.match(await aboard.innerText(), /Aboard Beowulf, at Mars/);
  assert.match(await aboard.getAttribute("title"), /Last voyage: Earth to Mars, \d/);
  await shot(pl, "party-player-header");

  // Arriving at a world the table does not fully know, the GM is offered one press to tell them, with Undo.
  const slip = gm.locator(`[data-arrival="${mars}"]`);
  await slip.waitFor({ timeout: 15000 });
  assert.match(await slip.innerText(), /Beowulf is at Mars\. Tell the table about it\?/);
  await shot(gm, "party-arrival-offer");
  await gm.locator("[data-arrival-tell]").click();
  await gm.locator("[data-arrival-told]").waitFor({ timeout: 15000 });
  assert.match(await gm.locator("[data-arrival-told]").innerText(), /The table knows Mars now/);
  assert.equal(await gm.locator("[data-arrival-undo]").count(), 1, "the reveal can be taken back");
  await ins.locator("[data-land-here]").click();
  await gm.waitForSelector("[data-surface]", { timeout: 30000 });
  await shot(gm, "party-land-on-mars");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("party e2e: ok");
} finally {
  server.kill();
}
