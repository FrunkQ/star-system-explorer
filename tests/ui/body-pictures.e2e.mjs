// Pictures on the body card (Alex, 24 Nov: "same as a ship"), against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/body-pictures.e2e.mjs [--screens <dir>].
// A world's card offers its Globe and Map; a click on the world never builds its surface, a tap does (about a second),
// then the globe draws and the flat map shows the same ground (Earth, from the sample V3 save; Tarn, a fixture world,
// says it has no surface yet). A star shows its disc, in visible light or X-rays. On the Build desk the GM adds
// an own picture: the card opens on it from then on, on every desk, and no picture.* field shows among the facts.
// On a phone the picture fits the card.
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { desk, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
// A 64 by 48 PNG, made in the page (no file on disk): a two-colour test card.
const pngOf = (page) =>
  page.evaluate(async () => {
    const c = document.createElement("canvas");
    c.width = 64;
    c.height = 48;
    const g = c.getContext("2d");
    g.fillStyle = "#3a6ea5";
    g.fillRect(0, 0, 64, 48);
    g.fillStyle = "#e8c547";
    g.fillRect(16, 12, 32, 24);
    return c.toDataURL("image/png").split(",")[1];
  });

try {
  const browser = await chromium.launch();
  const context = await browser.newContext({ deviceScaleFactor: screens ? 2 : 1 });
  const errors = [];
  const gm = await context.newPage();
  await gm.setViewportSize({ width: 1440, height: 900 });
  gm.on("pageerror", (e) => errors.push(e.message));
  await gm.goto(base + "#chart");
  await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  const body = (n) => gm.locator(`.area-chart .body[aria-label^="${n}"]`).first();
  const pic = gm.locator(".area-card section.picture").first();

  // A fixture world: Globe and Map are offered, nothing is built by the click, and a tap says there is no surface yet.
  await body("Tarn").locator(".hit").click();
  await pic.waitFor({ timeout: 20000 });
  assert.equal(await pic.getAttribute("data-picture"), "globe", "a world opens on its globe");
  assert.deepEqual(await pic.locator("[data-show]").evaluateAll((b) => b.map((x) => x.textContent.trim())), ["Globe", "Map"]);
  await shot(gm, "1-world-card-before");
  await pic.locator("[data-draw-surface]").click();
  await pic.locator("[data-no-surface]").waitFor({ timeout: 20000 });

  // A real world (Earth, sample save): the tap builds it and the globe draws.
  await more(gm, "Make a system");
  const dlg = gm.getByRole("dialog", { name: "Make a system" });
  await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
  await dlg.getByRole("button", { name: "Try a sample save" }).click();
  await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
  await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
  await body("Earth").waitFor({ timeout: 15000 });
  await body("Earth").locator(".hit").click();
  const draw = pic.locator("[data-draw-surface]");
  await draw.waitFor({ timeout: 20000 });
  await draw.click();
  await pic.locator("[data-body-picture='globe'][data-built='yes'] .globe canvas").waitFor({ timeout: 30000 });
  await gm.waitForTimeout(600);
  await shot(gm, "2-world-globe");

  // The map: the same ground laid flat, not blank.
  await pic.locator("[data-show='map']").click();
  const map = pic.locator("canvas.map");
  await map.waitFor();
  const lit = await map.evaluate((c) => {
    const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
    let n = 0;
    for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 0 && d[i] + d[i + 1] + d[i + 2] > 30) n++;
    return n / (d.length / 4);
  });
  assert.ok(lit > 0.9, `the map is drawn (${Math.round(lit * 100)}% lit)`);
  await shot(gm, "3-world-map");

  // Back to the same world later: it is built, so it shows at once.
  await body("Sun").locator(".hit").click();
  await gm.waitForTimeout(300);
  await body("Earth").locator(".hit").click();
  await pic.locator("[data-built='yes']").waitFor({ timeout: 3000 });

  // A star: its disc.
  await body("Sun").locator(".hit").click();
  await gm.locator(".area-card section.picture[data-picture='disc'] [data-body-picture='disc']").waitFor({ timeout: 20000 });
  assert.equal(await gm.locator(".area-card [data-star-picture] dl").count(), 0, "the card shows the disc alone; its facts stay on the Body screen");
  await gm.locator(".area-card section.picture [data-show='x-ray']").click();
  await gm.locator(".area-card [data-body-picture='x-ray'] .disc[data-band='xray']").waitFor({ timeout: 20000 });
  await gm.waitForTimeout(600);
  await shot(gm, "4-star-xray");

  // Build: an own picture, kept and opened on from then on; picture fields never show as facts.
  await desk(gm, "Build");
  await body("Earth").locator(".hit").click();
  const file = pic.locator("[data-picture-file]");
  await file.waitFor({ timeout: 20000 });
  await file.setInputFiles({ name: "tarn.png", mimeType: "image/png", buffer: Buffer.from(await pngOf(gm), "base64") });
  await pic.locator("[data-own-picture]").waitFor({ timeout: 10000 }).catch(async (e) => {
    throw new Error(`no own picture; the panel says: ${await pic.innerText()}`, { cause: e });
  });
  assert.match(await pic.locator("[data-picture-note]").innerText(), /kept/i);
  await shot(gm, "5-build-own-picture");
  const labels = await gm.locator(".area-card dt, .area-card .label").allInnerTexts();
  assert.ok(!labels.some((l) => /picture/i.test(l)), `no picture field among the facts: ${labels.join(", ")}`);
  await desk(gm, "Explore");
  await body("Sun").locator(".hit").click();
  await gm.waitForTimeout(300);
  await body("Earth").locator(".hit").click();
  await pic.locator("[data-own-picture]").waitFor({ timeout: 10000 });
  assert.equal(await pic.locator("[data-picture-file]").count(), 0, "Explore adds no pictures");
  await shot(gm, "6-explore-opens-on-it");

  // A phone: the picture fits inside the card.
  const phone = await browser.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: screens ? 3 : 1 });
  phone.on("pageerror", (e) => errors.push(`phone: ${e.message}`));
  await phone.goto(base + "#chart");
  await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
  await phone.locator(`.area-chart .body[aria-label^="Tarn,"] .hit`).first().tap();
  const ppic = phone.locator("section.picture").first();
  await ppic.waitFor({ timeout: 20000 });
  await phone.waitForFunction(() => document.getAnimations().length === 0);
  const over = await ppic.evaluate((s) => [...s.querySelectorAll("*")].some((e) => e.getBoundingClientRect().right > innerWidth + 1));
  assert.ok(!over, "nothing in the picture runs off a phone screen");
  await shot(phone, "7-phone");

  assert.deepEqual(errors, []);
  console.log("body pictures: globe, map, star disc, own picture kept, phone fits");
  await browser.close();
} finally {
  server.kill();
}
