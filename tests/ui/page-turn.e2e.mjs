// Turning a paper page (AT15a, atlas design answer 5 and 14A note 5), in a real browser. No build needed: the file is
// plain TypeScript with no imports, so its types are stripped and it runs on a bare page.
// Usage: node tests/ui/page-turn.e2e.mjs [--shots <dir>]
// curlTurn: the old page peels from its bottom outer corner, the new page readable beneath, then the copies go; a tap
// finishes it; going back the previous page comes down over this one. By hand: a drag carries the fold, past a third it
// turns, short it settles back and the page is put back; a press on a button inside is the button's. Reduced motion:
// a plain change.
import "./chart-first.mjs";
import { mkdirSync, readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire, stripTypeScriptTypes } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const src = stripTypeScriptTypes(readFileSync("packages/chart-grammar/src/page-turn.ts", "utf8")).replace(/^export /gm, "");
const shotsAt = process.argv.indexOf("--shots");
const shots = shotsAt > 0 ? process.argv[shotsAt + 1] : null;
if (shots) mkdirSync(shots, { recursive: true });

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 900, height: 760 }, deviceScaleFactor: shots ? 2 : 1 });
const page = await ctx.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
const html = `<!doctype html><body style="margin:0;background:#cfc8b8;font:16px Georgia,serif">
<div id="host" style="padding:40px"><article id="p" data-deliver-item style="width:600px;height:640px;background:#f4efe2;padding:32px;box-sizing:border-box;overflow:hidden">
<h1 id="t">Page 1</h1><p>${"The sheet lifts from its corner. ".repeat(40)}</p><button id="b">Print</button></article></div></body>`;
await page.setContent(html);
await page.addScriptTag({
  content: `${src}
window.n = 1; window.steps = [];
window.goTo = (d) => { steps.push(d); n += d; document.getElementById("t").textContent = "Page " + n; };
window.hand = pageTurn(document.getElementById("p"), { sheet: true, onstep: (d) => goTo(d) });`,
});
const state = () =>
  page.evaluate(() => ({
    title: document.getElementById("t").textContent,
    copies: document.querySelectorAll("[data-page-curl='copy']").length,
    flaps: document.querySelectorAll("[data-page-curl='flap']").length,
    steps: window.steps.slice(),
    delivering: document.getElementById("p").dataset.delivering ?? null,
  }));
const copyTitle = () => page.evaluate(() => [...document.querySelectorAll("[data-page-curl='copy']")].sort((a, b) => Number(a.style.zIndex) - Number(b.style.zIndex)).map((c) => c.querySelector("h1").textContent));
const shot = async (name) => shots && page.screenshot({ path: `${shots}/${name}.png` });

// A timed turn forward: halfway, the old page's copy lies on top, clipped by the fold, the new page beneath.
await page.evaluate(() => (window.turning = curlTurn(document.getElementById("p"), 1, () => goTo(1), { ms: 1400 })));
await page.waitForTimeout(700);
let s = await state();
assert.equal(s.title, "Page 2", "the new page is already beneath");
assert.equal(s.copies, 1);
assert.equal(s.flaps, 1);
assert.equal(s.delivering, "page_turn");
assert.deepEqual(await copyTitle(), ["Page 1"], "the copy on top is the old page");
const mid = await page.evaluate(() => ({
  clip: document.querySelector("[data-page-curl='copy']").style.clipPath,
  flap: document.querySelector("[data-curl='flap']").getAttribute("points"),
  shadow: document.querySelector("[data-curl='shadow']").getAttribute("filter"),
}));
assert.match(mid.clip, /^polygon\(/, "the old page is clipped along the fold");
assert.ok(mid.flap.split(" ").length >= 3, "the lifted part is drawn");
assert.match(mid.shadow, /url\(#/, "it casts a blurred shadow");
await shot("1-forward-halfway");
await page.evaluate(() => window.turning.done);
s = await state();
assert.equal(s.copies + s.flaps, 0, "the copies go at the end");
assert.equal(s.delivering, null);

// A tap finishes a turn at once.
await page.evaluate(() => (window.turning = curlTurn(document.getElementById("p"), 1, () => goTo(1), { ms: 5000 })));
await page.waitForTimeout(150);
await page.mouse.click(880, 740);
await page.evaluate(() => window.turning.done);
s = await state();
assert.equal(s.title, "Page 3");
assert.equal(s.copies + s.flaps, 0, "a tap finishes it");

// Back: the page coming back lies on top and comes down over this one, which lies beneath as a copy.
await page.evaluate(() => (window.turning = curlTurn(document.getElementById("p"), -1, () => goTo(-1), { ms: 1400 })));
await page.waitForTimeout(500);
s = await state();
assert.equal(s.title, "Page 2");
assert.deepEqual(await copyTitle(), ["Page 3", "Page 2"], "this page beneath, the one coming back on top (by stacking)");
await shot("2-back-coming-down");
await page.evaluate(() => window.turning.done);
assert.equal((await state()).copies, 0);

// By hand: a drag carries the fold, and the page beneath is already the next one.
const drag = async (from, to, mid) => {
  await page.mouse.move(from, 420);
  await page.mouse.down();
  for (let i = 1; i <= 10; i++) await page.mouse.move(from + ((to - from) * i) / 10, 420 + i * 0.3, { steps: 1 });
  await page.waitForTimeout(60);
  if (mid) await mid();
  await page.mouse.up();
  await page.waitForTimeout(700);
};
await page.evaluate(() => (window.steps = []));
await drag(600, 300, async () => {
  const m = await state();
  assert.equal(m.title, "Page 3", "the next page is beneath while the fold is carried");
  assert.equal(m.copies, 1);
  assert.ok(Number((await page.evaluate(() => document.querySelector("[data-page-curl='flap']").dataset.u)) ?? 0) > 0.2, "the fold follows the finger");
  await shot("3-carried-by-hand");
});
s = await state();
assert.deepEqual(s.steps, [1]);
assert.equal(s.title, "Page 3");
assert.equal(s.copies + s.flaps, 0);

// A short slow drag settles back and puts the page back.
await page.mouse.move(600, 420);
await page.mouse.down();
for (let i = 1; i <= 6; i++) (await page.mouse.move(600 - i * 10, 420), await page.waitForTimeout(60));
await page.mouse.up();
await page.waitForTimeout(700);
s = await state();
assert.deepEqual(s.steps, [1, 1, -1], "stepped while carried, then put back");
assert.equal(s.title, "Page 3");
assert.equal(s.copies + s.flaps, 0);

// Back by hand.
await drag(100, 460);
s = await state();
assert.deepEqual(s.steps.slice(3), [-1]);
assert.equal(s.title, "Page 2");

// A button inside keeps its press.
const b = await page.locator("#b").boundingBox();
await page.mouse.move(b.x + 5, b.y + 5);
await page.mouse.down();
await page.mouse.move(b.x - 300, b.y + 5, { steps: 8 });
await page.mouse.up();
await page.waitForTimeout(300);
assert.equal((await state()).steps.length, 4);

// Reduced motion: a plain change, nothing drawn.
await page.emulateMedia({ reducedMotion: "reduce" });
await page.evaluate(() => (window.turning = curlTurn(document.getElementById("p"), 1, () => goTo(1))));
s = await state();
assert.equal(s.copies + s.flaps, 0);
await page.evaluate(() => window.turning.done);
assert.equal((await state()).title, "Page 3");

// Like the Atlas: the page sits centred in a static sheet inside a positioned box with an offset, the swap replaces
// the page element (a keyed block) with a taller one, and going back the page fills in a little late. The copies must
// lie exactly where the pages stand, and the flap fit the copy on top.
await page.emulateMedia({ reducedMotion: "no-preference" });
await page.goto("about:blank");
await page.setContent(`<!doctype html><body style="margin:0;background:#cfc8b8;font:16px Georgia,serif">
<div style="display:grid;grid-template-columns:300px 1fr;gap:30px;padding:30px"><nav>Contents</nav>
<div id="box" style="position:relative;padding:12px"><div id="sheet" style="padding:0 20px">
<article class="pg" style="max-width:520px;margin:0 auto;background:#f4efe2;padding:24px;height:300px"><h1>Page 1</h1></article>
</div></div></div></body>`);
await page.addScriptTag({
  content: `${src}
window.mk = (n, h) => { const a = document.createElement("article"); a.className = "pg"; a.style.cssText = "max-width:520px;margin:0 auto;background:#f4efe2;padding:24px;height:" + h + "px"; a.innerHTML = "<h1>Page " + n + "</h1>"; return a; };
window.replace = (n, h, late) => { const sheet = document.getElementById("sheet"); const old = sheet.querySelector("article.pg"); const a = mk(n, late ? 300 : h); old.replaceWith(a); if (late) setTimeout(() => { a.style.height = h + "px"; a.querySelector("h1").textContent = "Page " + n + " (filled)"; }, 120); };`,
});
const rects = () =>
  page.evaluate(() => {
    const r = (el) => (el ? (({ x, y, width, height }) => ({ x: Math.round(x), y: Math.round(y), w: Math.round(width), h: Math.round(height) }))(el.getBoundingClientRect()) : null);
    const copies = [...document.querySelectorAll("[data-page-curl='copy']")].sort((a, b) => Number(a.style.zIndex) - Number(b.style.zIndex));
    return { real: r(document.querySelector("#sheet > article.pg:not([data-page-curl])")), copies: copies.map((c) => ({ ...r(c), text: c.textContent.trim() })), flap: r(document.querySelector("[data-page-curl='flap']")) };
  });
const old = (await rects()).real;
await page.evaluate(() => (window.turning = curlTurn(document.querySelector("#sheet > article.pg"), 1, () => replace(2, 520), { ms: 1400 })));
await page.waitForTimeout(400);
let r = await rects();
assert.deepEqual({ ...r.copies[0], text: undefined }, { ...old, text: undefined }, "forward: the old page's copy lies exactly where it stood");
assert.equal(r.copies[0].text, "Page 1");
assert.deepEqual(r.flap, old, "the flap fits the copy");
assert.equal(r.real.h, 568, "the taller new page is beneath");
await shot("4-offset-taller-forward");
await page.evaluate(() => window.turning.done);
const tall = (await rects()).real;
await page.evaluate(() => (window.turning = curlTurn(document.querySelector("#sheet > article.pg"), -1, () => replace(1, 300, true), { ms: 1400 })));
await page.waitForTimeout(500);
r = await rects();
assert.deepEqual({ ...r.copies[0], text: undefined }, { ...tall, text: undefined }, "back: this page's copy lies beneath where it stood");
assert.equal(r.copies[1].text, "Page 1 (filled)", "the page coming back is copied again once it has filled in");
assert.deepEqual({ ...r.copies[1], text: undefined }, { ...r.real, text: undefined }, "and lies exactly over the real page");
assert.deepEqual(r.flap, r.real, "the flap fits it");
await shot("5-offset-back-late");
await page.evaluate(() => window.turning.done);
r = await rects();
assert.equal(r.copies.length + (r.flap ? 1 : 0), 0);
assert.equal(await page.evaluate(() => document.getElementById("sheet").style.position), "", "the sheet's own position is put back");

assert.deepEqual(errors, []);
await browser.close();
console.log("page turn e2e: ok");
