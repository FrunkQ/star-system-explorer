// The Switchboard (track PB2), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/switchboard.e2e.mjs [--screens <dir>]
// The GM goes on air from the Switchboard; two screens join the table and appear under it by name. Send puts the live
// view on both. One screen is held with a card while the other carries on; released, it comes back. Live is sent to one
// screen alone by pressing its dot and then the screen; "Back with Table" ends that. Hold all holds both; Undo takes it
// back. A send to the table's role, then a Send to Everyone, is a layer: Lift gives the role its own send again.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9400 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(600);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const errors = [];
  const gm = await context.newPage();
  await gm.setViewportSize({ width: 1440, height: 1000 });
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
  await desk(gm, "Run");

  // The rail's Table group opens the Switchboard; it goes on air from there.
  await gm.getByRole("button", { name: "Switchboard" }).first().click();
  await gm.locator("[data-switchboard]").waitFor();
  await gm.locator("[data-go-on-air]").click();
  await gm.locator("[data-on-air]").first().waitFor();
  const link = await gm.locator("[data-switchboard] [data-players-link]").getAttribute("data-players-link");

  const join = async (name) => {
    const p = await context.newPage();
    await p.setViewportSize({ width: 1024, height: 700 });
    p.on("pageerror", (e) => errors.push(`${name}: ${e.message}`));
    await p.goto(`${link}&name=${encodeURIComponent(name)}`);
    await p.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
    return p;
  };
  const wall = await join("Wall TV");
  const mira = await join("Mira's tablet");
  const box = (name) => gm.locator("[data-switchboard] .screen", { hasText: name });
  await box("Wall TV").waitFor({ timeout: 15000 });
  await box("Mira's tablet").waitFor({ timeout: 15000 });
  const wallId = await box("Wall TV").getAttribute("data-screen");

  // The GM opens the Solar System and sends it: both screens follow.
  await more(gm, "Start sheet");
  await gm.locator('[data-door="sol"]').click();
  await gm.waitForFunction(() => [...document.querySelectorAll(".area-chart .body")].some((b) => /^Earth\b/.test(b.getAttribute("aria-label") ?? "")), null, { timeout: 30000 });
  await gm.locator("[data-on-air]").first().click();
  await gm.locator("[data-switchboard]").waitFor();
  assert.match(await gm.locator("[data-live-title]").innerText(), /Solar System/);
  await gm.locator("[data-send-live]").click();
  for (const p of [wall, mira]) await p.waitForFunction(() => /Sun/.test(document.body.textContent ?? ""), null, { timeout: 20000 });
  await shot(gm, "switchboard-sent");

  // One screen held with a card; the other carries on. Released, it comes back.
  await gm.locator(`[data-hold="screen:${wallId}"]`).click();
  await wall.locator("[data-screens-off][data-held]").waitFor({ timeout: 10000 });
  assert.ok((await wall.locator("[data-held]").getAttribute("data-held")).length > 0, "a holding card's words");
  assert.equal(await mira.locator("[data-screens-off]").count(), 0, "Mira's tablet carries on");
  assert.equal(await wall.locator("section.ask").count(), 0, "nothing of the place behind the card: no name to suggest");
  assert.equal(await wall.locator("header h1").innerText(), "Stand by");
  assert.equal(await gm.locator(`[data-line="screen:${wallId}"].held`).count(), 1, "the line is drawn broken");
  await shot(gm, "switchboard-held");
  await shot(wall, "switchboard-wall-held");
  // The line card (PB5): a plain card from the base presentation pack; then a particular joke; then words typed now.
  assert.ok(["Awaiting transmission", "Signal lost", "Stand by", "No carrier"].includes(await wall.locator("[data-held]").getAttribute("data-held")));
  const cardSel = gm.locator(`[data-card="screen:${wallId}"]`);
  assert.equal(await cardSel.locator("optgroup[label='Jokes'] option").count(), 24);
  await cardSel.selectOption({ label: "Tuning. This frequency is mostly hydrogen." });
  await wall.locator("[data-held='Tuning. This frequency is mostly hydrogen.']").waitFor({ timeout: 10000 });
  await shot(wall, "holding-joke");
  await cardSel.selectOption("own");
  await gm.locator(`[data-card-words="screen:${wallId}"]`).fill("Back after the break");
  await gm.locator(`[data-card-words="screen:${wallId}"]`).press("Enter");
  await wall.locator("[data-held='Back after the break']").waitFor({ timeout: 10000 });
  await shot(gm, "holding-own-words");
  await gm.locator(`[data-hold="screen:${wallId}"]`).click();
  await wall.locator("[data-screens-off]").waitFor({ state: "detached", timeout: 10000 });

  // Live to one screen alone: press the dot, then the screen. Back with Table ends it.
  await gm.locator('[data-source-dot="live"]').click();
  await box("Mira's tablet").locator("[data-dest-shows]").click();
  await gm.locator(`[data-line="screen:${await box("Mira's tablet").getAttribute("data-screen")}"]`).waitFor();
  await box("Mira's tablet").getByRole("button", { name: /Back with/ }).click();
  assert.equal(await box("Mira's tablet").getByRole("button", { name: /Back with/ }).count(), 0);

  // Hold all, and Undo takes it back.
  await gm.locator("[data-hold-all]").click();
  for (const p of [wall, mira]) await p.locator("[data-screens-off]").waitFor({ timeout: 10000 });
  await shot(gm, "switchboard-hold-all");
  await gm.locator("[data-board-undo]").click();
  for (const p of [wall, mira]) await p.locator("[data-screens-off]").waitFor({ state: "detached", timeout: 10000 });

  // A send to the table's role, then Send to Everyone: a layer over it, which Lift takes off.
  await gm.locator('[data-source-dot="live"]').click();
  await gm.locator('[data-dest="role:table"] b').click();
  await gm.locator('[data-dest="role:table"] [data-dest-shows]').filter({ hasText: /Solar System/ }).waitFor();
  await gm.locator("[data-send-live]").click();
  await gm.locator("[data-lift]").waitFor();
  assert.match(await gm.locator('[data-dest="everyone"]').innerText(), /over the set-up/);
  await shot(gm, "switchboard-layer");
  await gm.locator("[data-lift]").click();
  assert.equal(await gm.locator("[data-lift]").count(), 0);

  // Layouts (PB13): the table's layout on its heading; a new one made in the editor goes to its screens at once, and a
  // box with its control off is a picture there (AT13). Choosing the Bridge again puts it back.
  await gm.locator('[data-role-layout-chip="table"]').click();
  await gm.locator('[data-layout-picker="table"] [data-layout-new]').click();
  const ed = gm.locator("[data-layout-editor]");
  await ed.waitFor();
  await ed.locator("[data-layout-name]").fill("Wall chart");
  await ed.locator("[data-layout-arrangement]").selectOption("report");
  await ed.locator("[data-box-new-part]").selectOption("sse.view.chart");
  await ed.locator("[data-box-add]").click();
  await ed.locator("[data-box-new-part]").selectOption("sse.view.table");
  await ed.locator("[data-box-add]").click();
  await ed.locator('[data-layout-box="0"] [data-box-control="select"]').uncheck();
  // The arrows reorder: the table first, then back.
  await ed.locator('[data-layout-box="1"] [data-box-up]').click();
  assert.equal(await ed.locator('[data-layout-box="0"]').getAttribute("data-box-part"), "sse.view.table");
  await ed.locator('[data-layout-box="0"] [data-box-down]').click();
  await ed.locator("[data-layout-preview] .screen").waitFor({ timeout: 15000 });
  await shot(gm, "layout-editor");
  await ed.locator("[data-layout-save]").click();
  assert.match(await gm.locator('[data-role-layout-chip="table"]').innerText(), /Wall chart/);
  for (const p of [wall, mira]) await p.locator('[data-screen-id="mine.wall-chart"]').waitFor({ timeout: 15000 });
  assert.equal(await wall.locator('[data-screen-id="mine.wall-chart"] [data-view="sse.view.chart"][data-picture][inert]').count(), 1, "the chart's one control is off: a picture");
  assert.equal(await wall.locator('[data-view="sse.view.table"][data-picture]').count(), 0, "the table keeps its controls");
  await shot(wall, "layout-on-wall");
  await gm.locator('[data-role-layout-chip="table"]').click();
  await gm.locator('[data-layout-picker="table"] [data-layout-choose="bridge"]').click();
  for (const p of [wall, mira]) await p.locator('[data-screen-id="sse.screen.players.bridge"]').waitFor({ timeout: 15000 });

  // The clean output (PB8): the role offers it; opened, it is a screen marked Output, with no controls, at 16:9.
  assert.equal(await gm.locator('[data-output-link="table"]').count(), 1);
  const out = await context.newPage();
  await out.setViewportSize({ width: 1600, height: 1000 });
  out.on("pageerror", (e) => errors.push(`output: ${e.message}`));
  await out.goto(`${link}&clean=1&ground=none&name=Stream`);
  await out.waitForSelector("[data-clean] footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { state: "attached", timeout: 30000 });
  assert.equal(await out.locator("header.bar, section.ask, .soundbar").count(), 0, "no controls on the output");
  const size = await out.locator("[data-clean]").evaluate((e) => [e.clientWidth, e.clientHeight]);
  assert.ok(Math.abs(size[0] / size[1] - 16 / 9) < 0.01, `16:9, got ${size}`);
  assert.equal(await out.evaluate(() => getComputedStyle(document.body).backgroundColor), "rgba(0, 0, 0, 0)", "clear ground");
  // Every part on the output is a picture (AT13 rule 6): inert, so not one control is reached by keyboard either.
  const parts = await out.locator("[data-clean] section.area").count();
  assert.ok(parts > 0 && (await out.locator("[data-clean] section.area[data-picture][inert]").count()) === parts, "every part a picture");
  for (let i = 0; i < 6; i++) await out.keyboard.press("Tab");
  assert.equal(await out.evaluate(() => !!document.activeElement?.closest("section.area")), false, "nothing on the output takes the keyboard");
  await box("Stream").locator("[data-output-screen]").waitFor({ timeout: 15000 });
  await shot(out, "clean-output");
  await shot(gm, "switchboard-output");
  await out.close();

  // On a phone the board is one column, without the drawn lines.
  await gm.setViewportSize({ width: 390, height: 844 });
  await gm.waitForTimeout(400);
  assert.equal(await gm.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), true, "no sideways scroll on a phone");
  await shot(gm, "switchboard-phone");

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("switchboard e2e: ok");
} finally {
  server.kill();
}
