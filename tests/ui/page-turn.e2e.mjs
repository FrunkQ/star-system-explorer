// Turning a paper page by hand (AT15a, atlas design answer 5), in a real browser. No build needed: the action is plain
// TypeScript with no imports, so its types are stripped and it runs on a bare page.
// Usage: node tests/ui/page-turn.e2e.mjs
// A drag to the left carries the sheet about its left edge; let go past a third and it steps forward; a short drag
// settles back flat and does not step; a drag to the right steps back; a press on a button inside is the button's.
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire, stripTypeScriptTypes } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const src = stripTypeScriptTypes(readFileSync("packages/chart-grammar/src/page-turn.ts", "utf8")).replace(/^export /gm, "");

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.setContent(`<!doctype html><body style="margin:0"><article id="p" style="width:600px;height:500px;margin:40px;background:#f4efe2">A page <button id="b">Print</button></article></body>`);
await page.addScriptTag({ content: `${src}\nwindow.steps = []; pageTurn(document.getElementById("p"), { sheet: true, onstep: (d) => steps.push(d) });` });
const drag = async (from, to, mid) => {
  await page.mouse.move(from, 300);
  await page.mouse.down();
  for (let i = 1; i <= 10; i++) await page.mouse.move(from + ((to - from) * i) / 10, 300 + i * 0.3, { steps: 1 });
  if (mid) await mid();
  await page.mouse.up();
  await page.waitForTimeout(260);
};
const steps = () => page.evaluate(() => window.steps.slice());

// Forward: the sheet lifts and follows about its left edge, then turns.
await drag(560, 300, async () => {
  const t = await page.evaluate(() => getComputedStyle(document.getElementById("p")).transformOrigin);
  assert.match(t, /^0px /, "it turns about its left edge");
  assert.notEqual(await page.evaluate(() => document.getElementById("p").style.transform), "");
});
assert.deepEqual(await steps(), [1]);
assert.equal(await page.evaluate(() => document.getElementById("p").style.transform), "", "flat again after the turn");
// A short slow drag settles back and does not step.
await page.mouse.move(560, 300);
await page.mouse.down();
for (let i = 1; i <= 6; i++) (await page.mouse.move(560 - i * 10, 300), await page.waitForTimeout(60));
await page.mouse.up();
await page.waitForTimeout(260);
assert.deepEqual(await steps(), [1]);
// Back.
await drag(100, 420);
assert.deepEqual(await steps(), [1, -1]);
// A button inside keeps its press.
const b = await page.locator("#b").boundingBox();
await page.mouse.move(b.x + 5, b.y + 5);
await page.mouse.down();
await page.mouse.move(b.x - 300, b.y + 5, { steps: 8 });
await page.mouse.up();
assert.deepEqual(await steps(), [1, -1]);

assert.deepEqual(errors, []);
await browser.close();
console.log("page turn e2e: ok");
