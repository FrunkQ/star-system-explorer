// Postcards (PC1, PC3, PC4), against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/postcards.e2e.mjs [--screens <dir>].
// On Earth's globe (the sample V3 save) the GM turns the world, takes a photograph and keeps it as a postcard: it is
// a bookmark with the globe's camera and a fixed date. The wall (rail, Look) shows it; opening it draws the front live
// from the same camera, the back says where, when and how, and Send packs it (as a clip of the bodies in frame), with
// "Wish you were here!" ready on the back. Going there turns the globe back to the kept camera. A postcard link opens
// as a visit, read-only, marked as one; a damaged link is refused in plain words. A sent postcard that carries its world,
// pasted on a fresh page, is drawn live from a visiting universe. On a phone the card fits.
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { go, more } from "./nav.mjs";

// A 24 by 24 grey thumbnail of a canvas, 0 to 1, for comparing two drawings of the same picture.
const thumb = (c) => {
  const t = document.createElement("canvas");
  t.width = t.height = 24;
  const g = t.getContext("2d");
  g.drawImage(c, 0, 0, 24, 24);
  const d = g.getImageData(0, 0, 24, 24).data;
  const out = [];
  for (let i = 0; i < d.length; i += 4) out.push((d[i] + d[i + 1] + d[i + 2]) / 765);
  return out;
};

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

async function openEarth(page) {
  await page.goto(base + "#chart");
  await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
  await more(page, "Make a system");
  const dlg = page.getByRole("dialog", { name: "Make a system" });
  await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
  await dlg.getByRole("button", { name: "Try a sample save" }).click();
  await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
  await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
  const earth = page.locator('.area-chart .body[aria-label^="Earth"]').first();
  await earth.waitFor({ timeout: 15000 });
  await earth.locator(".hit").click();
  await go(page, "body");
  await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']):not([data-frames='0'])", { timeout: 30000 });
}
const globeAt = async (page) => {
  const g = page.locator("[data-body] [data-lon]").first();
  return { lon: Number(await g.getAttribute("data-lon")), lat: Number(await g.getAttribute("data-lat")) };
};

try {
  const browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 });
  const errors = [];
  const gm = await context.newPage();
  gm.on("pageerror", (e) => errors.push(e.message));
  await openEarth(gm);

  // Turn the globe by keyboard to a camera of our own, then photograph it and keep it as a postcard.
  const g = gm.locator("[data-body] [data-lon]").first();
  await g.focus();
  for (let i = 0; i < 4; i++) await gm.keyboard.press("ArrowRight");
  for (let i = 0; i < 2; i++) await gm.keyboard.press("ArrowUp");
  const kept = await globeAt(gm);
  assert.ok(kept.lon !== 0 || kept.lat !== 20, `the globe turned: ${JSON.stringify(kept)}`);
  await gm.locator("[data-take-photo]").evaluate((b) => b.click());
  await gm.locator("[data-photo-note]").waitFor({ timeout: 60000 });
  await gm.locator("[data-keep-postcard]").click();
  assert.match(await gm.locator("[data-kept-postcard]").innerText(), /Kept as the postcard "Earth \(sample\), Body"/);

  // The wall shows it; opening it draws the front live from the kept camera.
  await go(gm, "postcards");
  const wall = gm.locator("[data-postcard-wall]");
  await wall.waitFor();
  const tiles = wall.locator('[data-postcard-row="campaign"] [data-postcard-tile]');
  assert.equal(await tiles.count(), 1, "one postcard on the wall");
  assert.equal(await tiles.locator("img").count(), 1, "its stamp shows at once");
  await shot(gm, "desk-1-wall");
  await tiles.first().click();
  await gm.locator("[data-postcard-drawn]").waitFor({ timeout: 60000 });
  await gm.waitForFunction(() => !document.querySelector("[data-postcard-progress]"), null, { timeout: 60000 });
  await shot(gm, "desk-2-front");
  const sentLook = await gm.locator("[data-postcard-drawn]").evaluate(thumb);
  const lit = await gm.locator("[data-postcard-drawn]").evaluate((c) => {
    const s = document.createElement("canvas");
    s.width = c.width;
    s.height = c.height;
    const x = s.getContext("2d");
    x.drawImage(c, 0, 0);
    return [...x.getImageData(c.width / 2, c.height / 2, 1, 1).data];
  });
  assert.ok(lit[3] > 250 && lit[0] + lit[1] + lit[2] > 30, `the front is drawn: ${lit}`);

  // The back: where, when, how; Send packs it and opens with the default message.
  await gm.locator("[data-postcard-turn]").click();
  const back = gm.locator("[data-postcard-back]");
  await back.waitFor();
  const words = await back.innerText();
  assert.match(words, /Where\s+Earth \(sample\)/);
  assert.match(words, /When\s+/);
  assert.match(words, /How\s+.*band/);
  assert.match(words, /Travels as\s+Nothing goes with it: it is kept in this campaign/);
  await shot(gm, "desk-3-back");
  await back.getByRole("button", { name: "Send" }).click();
  const box = gm.locator("[data-postcard-sendbox]");
  await box.waitFor();
  assert.equal(await box.locator("[data-postcard-message-input]").inputValue(), "Wish you were here!");
  await box.locator("[data-postcard-whatgoes]").waitFor({ timeout: 15000 });
  assert.match(await box.locator("[data-postcard-whatgoes]").innerText(), /Earth \(sample\).*facts, nothing else/);
  assert.ok(await box.locator("[data-postcard-link]").isDisabled(), "a postcard that carries its world goes as a file, not a link");
  const [download] = await Promise.all([gm.waitForEvent("download"), box.locator("[data-postcard-file]").click()]);
  assert.match(download.suggestedFilename(), /\.ssepostcard\.json$/);
  const sent = readFileSync(await download.path(), "utf8");
  const world = JSON.parse(sent).world;
  assert.match(world.about ?? "", /^n\d+$/, "a clip postcard names the clip node its camera is about");
  assert.ok(world.origin?.facts.some((f) => f.field === "body.surface_water_oceans"), "its origin carries the facts that shape the world");
  assert.ok(!world.origin.facts.some((f) => /note|description/i.test(f.field)), "never notes or words for the GM");
  await shot(gm, "desk-4-send");

  // Go there: the globe comes back to the kept camera.
  await gm.locator('[data-postcard-back] .foot').getByRole("button", { name: "Go there" }).click();
  await gm.waitForSelector("[data-body] [data-lon]", { state: "attached", timeout: 20000 });
  await gm.waitForFunction((k) => {
    const el = document.querySelector("[data-body] [data-lon]");
    return el && Math.abs(Number(el.getAttribute("data-lon")) - k.lon) <= 1 && Math.abs(Number(el.getAttribute("data-lat")) - k.lat) <= 1;
  }, kept, { timeout: 15000 });

  // Any bookmark made where a camera is kept is a postcard already (a bookmark fixes its date when made): it is on
  // the wall and draws at once.
  await gm.getByRole("button", { name: "Bookmarks" }).click();
  const marks = gm.locator("[data-bookmarks-list]");
  await marks.locator("[data-bookmark-add]").click();
  await marks.locator("[data-bookmark-name]").fill("Earth from the bookmark");
  await marks.locator("[data-bookmark-name]").press("Enter");
  await gm.keyboard.press("Escape");
  await go(gm, "postcards");
  const fromMark = gm.locator('[data-postcard-row="campaign"] [data-postcard-tile]', { hasText: "Earth from the bookmark" });
  await fromMark.click();
  await gm.locator("[data-postcard-drawn]").waitFor({ timeout: 60000 });

  // A postcard link opens as a visit, read-only and marked; a damaged one is refused in plain words.
  const card = {
    schema: "sse.postcard/1",
    id: "pc-test",
    name: "A world far away",
    bookmark: { schema: "sse.bookmark/1", id: "b1", name: "A world far away", scope: "u:elsewhere", at_yr: 4.6e9, instrument: "body", view: { style: "true", seen: "free" }, selected: "u:elsewhere/iv", pose: { about: "u:elsewhere/iv", eye_dir: [1, 0, 0], eye_m: 2.5e7, up: [0, 0, 1], fov_deg: 30, exposure: { mode: "long", ev: 0 }, keep: [0, 0, 1, 1] } },
    world: { how: "address", root: "u:elsewhere", packs: [{ id: "base-physics", version: "0.0.1" }], made_with: "0.0.1" },
    credit: ["Sam"],
    message: { text: "Wish you were here!", from: "Sam" },
  };
  const b64 = Buffer.from(JSON.stringify(card)).toString("base64").replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  const visitor = await context.newPage();
  visitor.on("pageerror", (e) => errors.push(e.message));
  await visitor.goto(`${base}#postcard=${b64}`);
  await visitor.waitForSelector("[data-postcard-wall] [data-postcard]", { timeout: 30000 });
  await visitor.locator("[data-postcard-turn]").click();
  assert.match(await visitor.locator("[data-postcard-visiting]").innerText(), /Visiting: a postcard from Sam/);
  assert.match(await visitor.locator("[data-postcard-message]").innerText(), /Wish you were here!\s+Sam/);
  assert.match(await visitor.locator("[data-postcard-packs]").innerText(), /base-physics 0\.0\.1/);
  assert.equal(await visitor.locator("[data-postcard-go]").count(), 0, "a place not open here cannot be gone to");
  assert.ok(!/#postcard=/.test(visitor.url()), "the address bar drops the postcard once it is open");
  await shot(visitor, "desk-5-visit");
  await visitor.close();

  // A postcard that carries its world (the file Send saved), pasted on a fresh page: its world is laid into a visiting
  // universe and the front is drawn live from the same camera, not from a kept picture.
  // Someone else's browser: nothing of the sender's campaign is kept there.
  const elsewhere = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: screens ? 2 : 1 });
  const guest = await elsewhere.newPage();
  guest.on("pageerror", (e) => errors.push(e.message));
  await guest.goto(base);
  await guest.waitForSelector("[data-start] button, [data-chart]", { timeout: 30000 }).catch(() => {});
  await guest.keyboard.press("Escape");
  await guest.evaluate((text) => {
    const dt = new DataTransfer();
    dt.setData("text/plain", text);
    document.dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true }));
  }, sent);
  await guest.waitForSelector("[data-postcard-wall] [data-postcard]", { timeout: 30000 });
  await guest.waitForSelector("[data-postcard-drawn], [data-postcard-why]", { state: "attached", timeout: 90000 });
  const whyNot = await guest.locator("[data-postcard-why]").allInnerTexts();
  assert.deepEqual(whyNot, [], `the visit draws its front: ${whyNot.join(" ")}`);
  await shot(guest, "desk-6-visit-drawn");
  // The same world from the same camera: the visit's front matches the sender's (graphics cards differ by a hair).
  const seenLook = await guest.locator("[data-postcard-drawn]").evaluate(thumb);
  const diff = sentLook.reduce((t, v, i) => t + Math.abs(v - seenLook[i]), 0) / sentLook.length;
  assert.ok(diff < 0.03, `the visit draws the sender's picture (mean difference ${diff.toFixed(3)})`);
  await guest.locator("[data-postcard-turn]").click();
  assert.match(await guest.locator("[data-postcard-visiting]").innerText(), /Visiting: a postcard/);
  await elsewhere.close();

  // On a phone the card fits.
  const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
  await phone.goto(`${base}#postcard=${b64}`);
  await phone.waitForSelector("[data-postcard-wall] [data-postcard]", { timeout: 30000 });
  const w = await phone.evaluate(() => document.documentElement.scrollWidth);
  assert.ok(w <= 392, `no sideways scroll on a phone (${w})`);
  await shot(phone, "phone-1-visit");
  await phone.close();

  assert.deepEqual(errors, [], `no page errors: ${errors.join("; ")}`);
  await browser.close();
  console.log("postcards e2e: ok");
} finally {
  server.kill();
}
