// Stations built from their description (renderer station-design.ts), against the built shell
// (scripts/build-shell.sh first): add the Traveller pack, put a class A highport, a corsair base and a research
// station in the system, open each one's Picture and check the shape its words picked, its plans and its berths;
// on the Build desk a remix is kept on the construct as construct.design; the 3D model draws.
// Usage: node tests/ui/stations.e2e.mjs   (SSE_SHOTS=<dir> for screenshots)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk as chooseDesk, go } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 9700 + Math.floor(Math.random() * 90);
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shots = process.env.SSE_SHOTS;
if (shots) mkdirSync(shots, { recursive: true });
const shot = async (page, n) => shots && (await page.screenshot({ path: `${shots}/${n}.png` }));

try {
  const browser = await chromium.launch();
  const errors = [];
  const page = await browser.newPage({ viewport: { width: 1440, height: 1100 }, deviceScaleFactor: shots ? 2 : 1 });
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(`http://localhost:${port}/#chart`);
  await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await go(page, "packs");
  const packs = page.locator("[role=region][aria-label=Packs]");
  await packs.getByRole("button", { name: "Add the Traveller pack" }).click();
  await packs.locator("[aria-label=Proposal]").getByRole("button", { name: "Accept" }).click();
  await page.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sse.traveller"));
  await go(page, "system");
  await chooseDesk(page, "Build");
  await page.locator(".rail button", { hasText: "Ships and stations" }).first().click();
  const desk = page.locator("[role=region][aria-label='Ships and stations']");
  await desk.waitFor();
  for (const [tpl, name] of [["highport_a", "Tarn Highport"], ["corsair_base", "Black Rock"], ["research_station", "Quiet Lab"]]) {
    await desk.locator("[data-add-here]").click();
    await desk.locator(`[data-template="${tpl}"]`).click();
    await desk.locator("[data-name]").fill(name);
    await desk.locator("[data-add]").click();
    await desk.locator("[data-note]", { hasText: `${name} added` }).waitFor();
  }
  const open = async (name) => {
    await desk.locator("[data-construct]", { hasText: name }).click();
    const t = desk.locator("[data-picture-toggle]");
    if ((await t.getAttribute("aria-expanded")) !== "true") await t.click();
    const plans = desk.locator("[data-station-plans]");
    await plans.waitFor();
    return plans;
  };
  // The words pick the shape: a pirate haven is a hollowed rock, a lab a truss, a class A highport (hostel, concourse) a disc.
  for (const [name, shape] of [["Black Rock", "hollow_rock"], ["Quiet Lab", "truss"], ["Tarn Highport", "disc"]]) {
    const plans = await open(name);
    assert.equal(await plans.getAttribute("data-shape"), shape, `${name}'s shape`);
    assert.ok((await plans.locator("svg path").count()) > 3, "plans are drawn");
    await page.waitForTimeout(300);
    await shot(page, `station-${shape}`);
  }
  // The highport's words name its berths, the biggest opening up on a big station.
  const plans = desk.locator("[data-station-plans]");
  assert.match(await plans.locator(".words").innerText(), /Berths: \d+ for small craft, \d+ for ships/);
  // A ring for 50,000 at one gravity: switch shape, and the remix is kept on the construct.
  await plans.locator("[data-station-shape]").selectOption("ring");
  await page.waitForFunction(() => document.querySelector("[data-station-plans]")?.getAttribute("data-shape") === "ring");
  await shot(page, "station-ring");
  // Kept on the construct: open another station and come back, the ring is still there.
  await open("Quiet Lab");
  assert.equal(await (await open("Tarn Highport")).getAttribute("data-shape"), "ring", "the choice is kept as construct.design");
  // The 3D model.
  await plans.getByRole("group", { name: "Seen from" }).getByRole("button", { name: "Tilted", exact: true }).click();
  await plans.locator("[data-station-model]").waitFor();
  await page.waitForTimeout(1500);
  await shot(page, "station-ring-3d");
  assert.deepEqual(errors, []);
  await browser.close();
  console.log("stations e2e: ok");
} finally {
  server.kill();
}
