// V3 gap 26 in a real browser, against the built shell (scripts/build-shell.sh
// first): a seat's tabletop link shows its view inside the tabletop the GM
// listed, with nothing round it, and says hello to that tabletop only; the
// same link inside a tabletop the GM did not list is refused, with a reason.
// The tabletop is played by a page on 127.0.0.1 framing the app on localhost.
// Usage: node tests/ui/embed.e2e.mjs
import { spawn } from "node:child_process";
import assert from "node:assert/strict";
import { desk as chooseDesk } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 8900 + Math.floor(Math.random() * 300);
const base = `http://localhost:${port}/`;
const tabletop = `http://127.0.0.1:${port}`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  // The tabletop and the app are different sites; the test keeps their storage unpartitioned so the framed player can reach the GM's room in this one browser.
  const browser = await chromium.launch({ args: ["--disable-features=ThirdPartyStoragePartitioning,PartitionedBroadcastChannel"] });
  const context = await browser.newContext();
  const errors = [];
  const gm = await context.newPage({ viewport: { width: 1440, height: 1000 } });
  gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await chooseDesk(gm, "Build");
  const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
  await gm.getByRole("button", { name: "Share with players" }).click();
  (await popup)?.close();
  await gm.getByRole("button", { name: "More seats" }).click();
  const panel = gm.getByRole("dialog", { name: "Seats" });
  // A wildcard is refused at the form, with the reason.
  await panel.getByLabel("Name", { exact: true }).fill("Wall");
  await panel.getByLabel("Fog of war", { exact: false }).uncheck();
  await panel.getByLabel("Tabletops it may sit inside").fill("https://*.example.com");
  await panel.getByRole("button", { name: "Add seat" }).click();
  assert.match(await panel.getByRole("alert").innerText(), /no wildcards/);
  await panel.getByLabel("Tabletops it may sit inside").fill(tabletop);
  await panel.getByRole("button", { name: "Add seat" }).click();
  await panel.locator("[data-seat-embed='wall']").waitFor();
  const wall = await panel.locator("[data-seat-embed='wall']").getAttribute("href");
  assert.match(wall, /seat=wall&embed=1$/);
  await panel.getByLabel("Name", { exact: true }).fill("Crew");
  // V3 gap 25: a brand on the seat's screens.
  await panel.getByLabel("Brand on its screens").fill("Vesh Navy");
  await panel.getByRole("button", { name: "Add seat" }).click();
  await panel.locator("[data-seat-link='crew']").waitFor();
  const crew = `${await panel.locator("[data-seat-link='crew']").getAttribute("href")}&embed=1`;

  // The crew's own page (not framed) wears the brand.
  const crewPage = await context.newPage();
  crewPage.on("pageerror", (e) => errors.push(`crew: ${e.message}`));
  await crewPage.goto(crew);
  assert.equal(await crewPage.locator(".page").getAttribute("data-embed"), "page");
  await crewPage.locator("[data-brand]").waitFor({ timeout: 30000 });
  assert.equal((await crewPage.locator("[data-brand]").innerText()).trim(), "Vesh Navy");
  await crewPage.close();

  // The tabletop: a page on another site that frames the link and logs what it hears.
  const host = await context.newPage({ viewport: { width: 1280, height: 900 } });
  host.on("pageerror", (e) => errors.push(`tabletop: ${e.message}`));
  await host.goto(`${tabletop}/players.html?tabletop`);
  const frame = async (src) => {
    await host.evaluate((src) => {
      window.heard = [];
      addEventListener("message", (e) => window.heard.push({ origin: e.origin, data: e.data }));
      document.documentElement.innerHTML = `<body style="margin:0"><iframe id="v" src="${src}" style="width:900px;height:700px;border:0"></iframe></body>`;
    }, src);
    const f = host.frameLocator("#v");
    await f.locator(".page[data-embed='embedded'], .page[data-embed='refused']").waitFor({ timeout: 30000 });
    return f;
  };
  const f = await frame(wall);
  assert.equal(await f.locator(".page").getAttribute("data-embed"), "embedded");
  assert.equal(await f.locator("header.bar").count(), 0, "no page chrome inside the tabletop");
  assert.equal(await f.locator(".soundbar").count(), 0);
  await f.locator("footer[data-link='live']").waitFor({ state: "attached", timeout: 30000 });
  await host.waitForFunction(() => window.heard.some((m) => m.data?.source === "sse4" && m.data.type === "ready"));
  const ready = await host.evaluate(() => window.heard.find((m) => m.data?.type === "ready"));
  assert.equal(ready.origin, base.slice(0, -1));
  assert.equal(ready.data.seat, "Wall");
  // A click in the view tells the tabletop what is in focus; the tabletop's own ask moves focus back.
  const focused = (n) => host.waitForFunction((n) => window.heard.some((m) => m.data?.type === "focus" && m.data.name === n), n);
  await f.getByRole("button", { name: "Tarn", exact: true }).first().click();
  await focused("Tarn");
  const tarn = await host.evaluate(() => window.heard.find((m) => m.data?.type === "focus" && m.data.name === "Tarn").data.address);
  await f.getByRole("button", { name: "Vault", exact: true }).first().click();
  await focused("Vault");
  await host.evaluate(() => (window.heard = []));
  await host.evaluate((a) => document.getElementById("v").contentWindow.postMessage({ source: "vtt", type: "focus", address: a }, "*"), tarn);
  await focused("Tarn");

  // The seat that lists no tabletop is refused there, and says why.
  const g = await frame(crew);
  assert.equal(await g.locator(".page").getAttribute("data-embed"), "refused");
  assert.match(await g.getByRole("alert").innerText(), /has not allowed this seat inside a tabletop/);
  assert.ok(!(await host.evaluate(() => window.heard.some((m) => m.data?.source === "sse4"))), "a refused view says nothing to the tabletop");

  assert.deepEqual(errors, []);
  console.log(`PASS chromium: tabletop link embedded at ${tabletop}, ready heard from ${ready.origin}, focus both ways; unlisted seat refused; brand shown`);
  await browser.close();
} finally {
  server.kill();
}
