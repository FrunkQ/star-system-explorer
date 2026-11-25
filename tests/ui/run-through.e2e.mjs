// The landing run-through end to end (Alex, 23 Nov: "landing sequence and end to end run through ... the GM plotting
// the Players experience of the events"), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/run-through.e2e.mjs [--screens <dir>]. Track LR, status/landing.md.
//
// One journey, GM and a joined player side by side, each step checked on both screens:
//  1. the Solar System with the Traveller pack; a far trader at Earth is made the party's ship; the GM takes Run;
//  2. the table joins and sees "Aboard Beowulf, at Earth";
//  3. Voyages flies Beowulf to Mars; the player's header follows; the GM sends Mars's own screen and the table sees
//     "Beowulf in orbit of Mars", the world turning below on its own kernel (LR4); "Land here" opens Land on Mars;
//  4. the party's ship is the lander; the GM picks a landmark and sets midday; the ship flies down on its drive (LR1);
//     nothing reaches the table until it is sent;
//  5. Send this view: the player's screen is the Landing screen, judging the same site the same way;
//  6. Land: both fly; skip to the ground; the lander stands in frame, its lamps lit at night at the GM's switch (LR6),
//     the GM walks and the player follows; take off;
//  7. a whole landing (site, hour, lander) comes back from its link, as a bookmark or a Playbook preset fires it (LR5).
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9300 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(500);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const addressOf = (page, name) => page.evaluate((n) => [...document.querySelectorAll(".insys [data-place]")].find((e) => e.getAttribute("aria-label") === n)?.getAttribute("data-place"), name);

try {
  const browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const errors = [];
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));

  // 1. The Solar System, the Traveller pack, a far trader at Earth as the party's ship.
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
  await go(gm, "voyages");
  await gm.waitForFunction(() => [...document.querySelectorAll(".insys [data-place]")].some((e) => e.getAttribute("aria-label") === "Mars"), null, { timeout: 60000 });
  const earth = await addressOf(gm, "Earth");
  const mars = await addressOf(gm, "Mars");
  assert.ok(earth && mars, "Earth and Mars on the orbit line");
  await go(gm, "constructs");
  const yard = gm.locator("[role=region][aria-label='Ships and stations']");
  await yard.locator("[data-add-here]").click();
  await yard.locator("[data-under]").selectOption(earth);
  await yard.locator("[data-template='far_trader']").click();
  await yard.locator("[data-name]").fill("Beowulf");
  await yard.locator("[data-add]").click();
  const row = yard.locator("[data-construct]", { hasText: "Beowulf" });
  await row.waitFor({ timeout: 15000 });
  if ((await row.getAttribute("aria-pressed")) !== "true") await row.click();
  await yard.locator("[data-party-ship]").click();
  await yard.locator("[data-party]").first().waitFor();
  await desk(gm, "Run");

  // 2. The table joins.
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  const pl = await context.newPage();
  pl.on("pageerror", (e) => errors.push(`player: ${e.message}`));
  await pl.goto(await gm.getAttribute("[data-players-link]", "data-players-link"));
  await pl.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  const aboard = pl.locator("header [data-party-ship]");
  await aboard.waitFor({ timeout: 15000 });
  assert.match(await aboard.innerText(), /Aboard Beowulf, at Earth/);
  await shot(pl, "rt-1-player-at-earth");

  // 3. Voyages flies Beowulf to Mars.
  await go(gm, "voyages");
  const ins = gm.locator(".insys");
  await gm.waitForFunction(() => (document.querySelector(".insys [data-ship]")?.value ?? "") === "party", null, { timeout: 15000 });
  await ins.locator(`[data-place='${mars}']`).dispatchEvent("click");
  await gm.waitForFunction((m) => document.querySelector(".insys")?.getAttribute("data-voyage")?.endsWith(`>${m}`), mars, { timeout: 60000 });
  assert.ok((await ins.getAttribute("data-voyage")).startsWith(`${earth}>`), "the voyage starts where the party's ship is");
  await ins.locator("[data-next]").click();
  await ins.locator("[data-land-here]").waitFor({ timeout: 15000 });
  await pl.waitForFunction((m) => document.querySelector("header [data-party-ship]")?.getAttribute("data-party-at") === m, mars, { timeout: 15000 });
  assert.match(await aboard.innerText(), /Aboard Beowulf, at Mars/);
  await shot(gm, "rt-2-gm-arrived");
  const landHref = await ins.locator("[data-land-here]").getAttribute("href");
  assert.match(landHref, /~surface$/, `Land here opens Land: ${landHref}`);

  // 3b. In orbit (LR4): the GM opens Mars's own screen and sends it; the table sees Mars turning below on its own kernel.
  await gm.evaluate((h) => (location.hash = h), landHref.replace(/~surface$/, ""));
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "world", null, { timeout: 20000 });
  await gm.locator("[data-send-view]").click();
  await gm.waitForFunction(() => /Beowulf in orbit of Mars/.test(document.querySelector("[data-table-sees]")?.textContent ?? ""), null, { timeout: 15000 });
  await pl.waitForSelector(`[data-orbit="${mars}"][data-orbit-drawn]`, { timeout: 60000 });
  assert.match(await pl.locator("[data-orbit-caption]").innerText(), /Mars, turning below/);
  // LR4b: Mars's air, read on the GM's kernel, comes with the send and is drawn on the player's globe.
  await pl.waitForSelector(`[data-orbit="${mars}"][data-orbit-air]`, { timeout: 30000 });
  await shot(pl, "rt-2b-player-orbit");
  await gm.evaluate((h) => (location.hash = h), landHref);

  // 4. Land on Mars with the party's ship; the GM picks a landmark, still private.
  await gm.waitForSelector("[data-surface] [data-landmark-row]", { timeout: 40000 });
  await gm.locator("[data-landmark-row]").nth(5).click();
  await gm.waitForSelector("[data-surface]:not([data-verdict=''])", { timeout: 40000 });
  assert.equal(await gm.locator("[data-lander]").inputValue(), "party", "the party's ship is offered first as the lander");
  const verdict = await gm.locator("[data-surface]").getAttribute("data-verdict");
  // LR1: a far trader can hold itself up at Mars, so it flies down on its drive from orbit and lands.
  assert.equal(verdict, "landed", "Beowulf flies down from orbit on its drive");
  assert.match(await gm.locator("[data-verdict-card] .says").innerText(), /engines from orbit/);
  assert.equal(await pl.locator("[data-view='sse.view.surface']").count(), 0, "nothing of the landing reaches the table before it is sent");
  // LR2: the GM sets the landing at midday on Mars's own clock with one press.
  await gm.locator('[data-times] [data-time="midday"]').click();
  assert.match(await gm.locator("[data-times] [data-local-time]").innerText(), /^1[12]:/);
  assert.equal(await gm.locator('[data-times] [data-time="midday"]').getAttribute("aria-pressed"), "true");
  // LR5b: the address bar carries the whole landing, and the table's title names it.
  await gm.waitForFunction(() => /~surface\?site=[-\d.]+,[-\d.]+&time=midday&lander=party$/.test(location.hash), null, { timeout: 10000 });
  await shot(gm, "rt-3-gm-site");

  // 5. Sent: the players watch the same landing, judged the same way on their own kernel.
  await gm.locator("[data-send-view]").click();
  await gm.waitForFunction(() => /landing/.test(document.querySelector("[data-table-sees]")?.textContent ?? ""), null, { timeout: 15000 }).catch(async () => {
    // A send pressed before the Land screen's first landing is ready (fixed in RD1) goes again.
    if (await gm.locator("[data-send-view]").count()) await gm.locator("[data-send-view]").click();
  });
  await pl.waitForSelector("[data-surface][data-watching]:not([data-verdict=''])", { timeout: 60000 });
  assert.equal(await pl.locator("[data-surface]").getAttribute("data-verdict"), verdict, "the same verdict on both kernels");
  assert.equal(await pl.locator("[data-surface]").getAttribute("data-site"), await gm.locator("[data-surface]").getAttribute("data-site"));
  assert.match(await pl.locator("[data-lander-shown]").innerText(), /the party's ship/);
  assert.match(await gm.locator("[data-table-sees]").innerText(), /Mars, landing at .+, midday/);
  await shot(pl, "rt-4-player-site");

  // 6. Land, skip to the ground, walk; the player follows; take off.
  await gm.locator("[data-land]").click();
  await gm.waitForSelector("[data-surface][data-phase='flying']", { timeout: 30000 });
  await pl.waitForSelector("[data-surface][data-phase='flying']", { timeout: 40000 });
  await shot(pl, "rt-5-player-flying");
  await gm.locator("[data-skip]").click();
  await pl.waitForSelector("[data-surface][data-phase='down']", { timeout: 30000 });
  const before = await pl.locator("[data-surface]").getAttribute("data-walk");
  await gm.locator("[data-ground]").focus();
  await gm.keyboard.down("ArrowUp");
  await gm.waitForTimeout(1500);
  await gm.keyboard.up("ArrowUp");
  await pl.waitForFunction((b) => document.querySelector("[data-surface]")?.getAttribute("data-walk") !== b, before, { timeout: 15000 });
  assert.match(await pl.locator("[data-ground-note]").innerText(), /local time 1[12]:/, "the table lands at the GM's hour");
  await shot(pl, "rt-6-player-down");
  // LR6: the lander stands where it touched down, in frame from where the walk starts; at night its lamps are lit.
  await gm.waitForSelector("[data-surface][data-lander-drawn='dark']", { timeout: 15000 });
  await pl.waitForSelector("[data-surface][data-lander-drawn]", { timeout: 15000 });
  await gm.getByRole("group", { name: "View style" }).getByRole("button", { name: "True", exact: true }).click();
  await gm.waitForSelector("[data-surface][data-look='true']");
  await gm.locator('[data-walk-pad] [data-time="night"]').click();
  await gm.waitForTimeout(400);
  assert.equal(await gm.locator("[data-surface]").getAttribute("data-lander-drawn"), "dark", "the lamps wait for the GM's switch");
  await gm.locator("[data-walk-pad] [data-lamps]").check();
  await gm.waitForSelector("[data-surface][data-lander-drawn='lit']", { timeout: 15000 });
  await pl.waitForSelector("[data-surface][data-lander-drawn='lit']", { timeout: 15000 });
  await gm.waitForTimeout(800);
  await shot(gm, "rt-6b-gm-lander-night");
  await gm.locator('[data-walk-pad] [data-time="midday"]').click();
  await gm.waitForSelector("[data-surface][data-lander-drawn='dark']", { timeout: 15000 });
  await shot(gm, "rt-6c-gm-lander-day");
  await gm.locator("[data-take-off]").click();
  await gm.waitForFunction(() => document.querySelector("[data-surface]")?.getAttribute("data-phase") !== "down", null, { timeout: 15000 });

  // LR5: a whole landing comes back from its link (a bookmark or a Playbook preset): site, hour and lander.
  const land = (await gm.evaluate(() => location.hash)).split("?")[0];
  assert.match(land, /~surface$/, `on the Land screen: ${land}`);
  await gm.evaluate((h) => (location.hash = h), `${land}?site=-14.5,175.4&time=dusk&lander=capsule`);
  await gm.waitForFunction(() => document.querySelector("[data-surface]")?.getAttribute("data-site") === "-14.50,175.40", null, { timeout: 30000 });
  await gm.waitForSelector("[data-times] [data-local-time]", { timeout: 30000 });
  await gm.waitForFunction(() => /^1[78]:/.test(document.querySelector("[data-times] [data-local-time]")?.textContent ?? ""), null, { timeout: 15000 });
  assert.equal(await gm.locator("[data-lander]").inputValue(), "capsule");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("run-through e2e: ok");
} finally {
  server.kill();
}
