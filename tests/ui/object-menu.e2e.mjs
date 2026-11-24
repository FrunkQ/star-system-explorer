// The menu on any object (V3 gap 38), in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/object-menu.e2e.mjs [--screens <dir>]
// A right-click on a body of the system chart opens its menu from the body, with the name and what it orbits. The
// items follow the desk: Explore looks and sends to the table, Build changes, Run plans voyages and landings. An item
// that cannot be done now is greyed with the reason underneath; the foot names what other desks offer and switches
// desk. Arrow keys move, Escape closes and gives the focus back. Open goes to the world. On a phone a held press opens
// the menu as a sheet along the bottom. The picker's rows have the same menu, and "#" in its search finds a tag.
// Opening and closing menus writes nothing.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9500 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(300);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
const ready = (page) => page.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 30000 });
const ids = (page) => page.locator("[data-object-menu] [data-action]").evaluateAll((b) => b.map((x) => x.getAttribute("data-action")));

try {
  const browser = await chromium.launch();
  const errors = [];
  const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 })).newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(root + "#chart");
  await ready(gm);
  const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
  await desk(gm, "Explore");

  // The picker on the place line: "#" finds a tag ("#ocean" finds Tarn, tagged class:planet/ocean), and a right-click
  // on a row opens that body's menu, as on the chart.
  await gm.locator(".place [data-picker-chip]").click();
  const pop = gm.locator("[data-picker-pop]");
  await pop.locator("[data-picker-find]").fill("#ocean");
  assert.deepEqual((await pop.locator(".tree .name").allInnerTexts()).map((t) => t.trim()), ["Tarn"]);
  // Light up (V3 gap 41): the tag's holders are marked on the chart until turned off from the chip on the line.
  const tarnAddr = await pop.locator(".tree [data-pick-row]").first().getAttribute("data-pick-row");
  const glow = () => gm.evaluate((a) => getComputedStyle(document.querySelector(`.body[data-address="${a}"]`)).filter, tarnAddr);
  assert.equal(await glow(), "none");
  await pop.locator("[data-picker-light]").click();
  assert.match(await glow(), /drop-shadow/);
  assert.equal((await gm.locator(".place [data-lit-chip]").innerText()).replace("×", "").trim(), "#ocean");
  await shot(gm, "light-up-ocean");
  await gm.keyboard.press("Escape");
  await gm.keyboard.press("Escape");
  await gm.locator(".place [data-lit-chip]").click();
  assert.equal(await glow(), "none");
  await gm.locator(".place [data-picker-chip]").click();
  await pop.locator("[data-picker-find]").fill("");
  const row = pop.locator(".tree [data-pick-row][data-address]").nth(1);
  const rowAddress = await row.getAttribute("data-address");
  await row.click({ button: "right" });
  await gm.locator(`[data-object-menu="${rowAddress}"]`).waitFor({ timeout: 5000 });
  await gm.keyboard.press("Escape");
  await gm.locator("[data-object-menu]").waitFor({ state: "detached", timeout: 3000 });
  if (await pop.isVisible()) await gm.keyboard.press("Escape");

  // A planet on the chart: the first body with an orbit and a name.
  const body = gm.locator('.body[data-address*="body:"]').nth(2);
  await body.waitFor({ timeout: 30000 });
  const address = await body.getAttribute("data-address");
  await body.click({ button: "right" });
  const menu = gm.locator("[data-object-menu]");
  await menu.waitFor({ timeout: 5000 });
  assert.equal(await menu.getAttribute("data-object-menu"), address);
  const name = (await menu.locator(".head .name").innerText()).trim();
  assert.match(await menu.locator(".head .kicker").innerText(), /orbits /, "the head says what it orbits");
  // The right-click also selects it, as a press does.
  assert.equal(await gm.locator(`.body[data-address="${address}"]`).getAttribute("aria-pressed"), "true");
  const explore = await ids(gm);
  assert.deepEqual(explore.slice(0, 3), ["open", "describe", "link"]);
  assert.ok(explore.includes("send") && explore.includes("copy"), `Explore offers the table and Copy (${explore})`);
  assert.ok(!explore.includes("cut") && !explore.includes("add"), "nothing on Explore changes the universe");
  // Not shared yet: Show the players is greyed, with the reason in words.
  const send = menu.locator('[data-action="send"]');
  assert.equal(await send.getAttribute("aria-disabled"), "true");
  assert.match(await send.innerText(), /Share with the players first/);
  assert.match(await menu.locator('[data-other-desk="build"]').innerText(), /^On Build: Paste here/);
  // The keyboard: the first item has the focus; arrows move; Escape closes.
  assert.equal(await gm.evaluate(() => document.activeElement?.getAttribute("data-action")), "open");
  await gm.keyboard.press("ArrowDown");
  assert.equal(await gm.evaluate(() => document.activeElement?.getAttribute("data-action")), "describe");
  await shot(gm, "menu-explore-desk");
  await gm.keyboard.press("Escape");
  await menu.waitFor({ state: "detached", timeout: 3000 });

  // The foot of the menu switches desk, and the menu reopens with that desk's items.
  await body.click({ button: "right" });
  await menu.locator('[data-other-desk="build"]').click();
  assert.equal(await gm.getByRole("group", { name: "Desk" }).getByRole("button", { name: "Build", exact: true }).getAttribute("aria-pressed"), "true");
  await body.click({ button: "right" });
  const build = await ids(gm);
  assert.ok(["copy", "paste", "cut"].every((x) => build.includes(x)), `Build offers changing (${build})`);
  assert.ok(!build.includes("send"), "the table is not a Build item");
  const cut = menu.locator('[data-action="cut"]');
  assert.equal(await cut.getAttribute("aria-disabled"), "true", "the sample's own bodies cannot be cut");
  assert.match(await cut.innerText(), /Only a body added to this campaign/);
  // A greyed item does nothing when pressed.
  await cut.dispatchEvent("click");
  assert.equal(await menu.count(), 1);
  await shot(gm, "menu-build-desk");
  await gm.keyboard.press("Escape");

  // Run: ships and landings.
  await desk(gm, "Run");
  await body.click({ button: "right" });
  const run = await ids(gm);
  assert.ok(run.includes("voyage") && run.includes("land"), `Run offers voyages and landings (${run})`);
  // Open goes down to the world.
  await menu.locator('[data-action="open"]').click();
  await gm.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "world", null, { timeout: 20000 });
  assert.equal((await gm.locator(".place h1.title").innerText()).trim(), name);
  await desk(gm, "Explore");

  // Night chart.
  await more(gm, /Night chart/);
  await gm.goto(root + "#chart");
  await ready(gm);
  // System 3D: a right-click on a body's pin, or on the canvas beside it, opens the same menu; the view does not fly.
  await gm.getByRole("button", { name: "3D", exact: true }).click();
  const s3d = gm.locator("[aria-roledescription='3D chart']");
  await s3d.waitFor();
  const pin = s3d.locator(`[data-body="${address}"]`);
  await pin.waitFor({ timeout: 20000 });
  const frame0 = await s3d.getAttribute("data-frame");
  await pin.click({ button: "right" });
  await gm.locator(`[data-object-menu="${address}"]`).waitFor({ timeout: 5000 });
  await gm.keyboard.press("Escape");
  await gm.locator("[data-object-menu]").waitFor({ state: "detached", timeout: 3000 });
  const pb3 = await pin.boundingBox();
  await gm.mouse.click(pb3.x + pb3.width / 2 + 14, pb3.y + pb3.height / 2 + 2, { button: "right" });
  await gm.locator(`[data-object-menu="${address}"]`).waitFor({ timeout: 5000 });
  assert.equal(await s3d.getAttribute("data-frame"), frame0, "a right-click never flies");
  await shot(gm, "menu-system-3d");
  await gm.keyboard.press("Escape");
  await gm.getByRole("button", { name: "3D", exact: true }).locator("xpath=..").getByRole("button", { name: "Chart", exact: true }).click();

  await gm.locator(`.body[data-address="${address}"]`).click({ button: "right" });
  await menu.waitFor();
  await shot(gm, "menu-night-desk");
  await gm.keyboard.press("Escape");
  assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "menus write nothing");

  // Phone: a held press opens the menu as a sheet along the bottom.
  const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, deviceScaleFactor: screens ? 2 : 1 })).newPage();
  phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
  await phone.goto(root + "#chart");
  await ready(phone);
  const pb = phone.locator(`.body[data-address="${address}"]`);
  await pb.waitFor({ timeout: 30000 });
  const box = await pb.locator("circle.hit").boundingBox();
  const at = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  await pb.dispatchEvent("pointerdown", { pointerType: "touch", isPrimary: true, pointerId: 7, clientX: at.x, clientY: at.y, button: 0 });
  await phone.waitForTimeout(700);
  await pb.dispatchEvent("pointerup", { pointerType: "touch", isPrimary: true, pointerId: 7, clientX: at.x, clientY: at.y });
  const sheet = phone.locator("[data-object-menu].sheet");
  await sheet.waitFor({ timeout: 3000 });
  const r = await sheet.boundingBox();
  assert.ok(Math.abs(r.y + r.height - 844) <= 2 && r.width >= 388, "the sheet sits along the bottom, full width");
  const tall = await sheet.locator('[data-action="open"]').boundingBox();
  assert.ok(tall.height >= 44, "touch targets are at least 44 px");
  const overflow = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(overflow <= 1, `nothing runs off the side (${overflow}px)`);
  await shot(phone, "menu-phone-sheet");
  // A press outside puts it away.
  await phone.mouse.click(200, 120);
  await sheet.waitFor({ state: "detached", timeout: 3000 });

  assert.deepEqual(errors, []);
  await browser.close();
  console.log("object menu e2e: ok");
} finally {
  server.kill();
}
