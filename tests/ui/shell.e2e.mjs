// U0 and U1 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/shell.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Checks: keyboard-only selection and editing; the edit inked through the GM
// seat equals the same intent from an auto-approved player seat (event id and
// state hash); ruler keyboard steps; mode and theme keys; phone layout has no
// sideways scroll; screenshots at desktop and phone widths in both modes.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Session } from "../../packages/ui-contract/src/session.ts";
import { tableSeat } from "../../packages/ui-contract/src/seat.ts";
import { nodeKernel } from "../../packages/ui-contract/test/node-kernel.ts";

// Playwright is installed outside the lockfile (CI: npm install --no-save), as for the harness.
const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8090 + Math.floor(Math.random() * 500);
const url = `http://localhost:${port}/`;
const TARN = "u:kestrel/sys:1/body:3";
const NOW = 4_610_000_000;

const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

async function playerEvent() {
  const k = nodeKernel();
  const seat = { ...tableSeat(), id: "helm", label: "Helm", may_send: ["set_value"], auto_approve: [{ intent: "set_value", scope: ["u:kestrel/sys:1"] }] };
  const s = await Session.open(k.client, seat, { id: "kestrel", seed: "kestrel reach campaign" });
  const r = await s.send({ kind: "set_value", address: TARN, field: "mass_earth", value: 0.9, at_yr: NOW });
  await k.close();
  assert.equal(r.status, "applied");
  return r;
}

async function tabTo(page, test, max = 60) {
  for (let i = 0; i < max; i++) {
    await page.keyboard.press("Tab");
    if (await page.evaluate(test)) return i + 1;
  }
  throw new Error("could not reach the target with Tab");
}

const results = [];
let u2 = null;
const expected = await playerEvent();
try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    const browser = await engine.launch();
    const errors = [];

    // Desktop, keyboard only.
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(url);
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const tabsToTarn = await tabTo(page, () => document.activeElement?.getAttribute("aria-label")?.startsWith("Tarn"));
    await page.keyboard.press("Enter");
    await page.waitForSelector("section[aria-label='Tarn']");
    const tabsToMass = await tabTo(page, () => document.activeElement?.id && document.querySelector(`label[for='${document.activeElement.id}']`)?.textContent?.startsWith("Mass"), 20);
    await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
    await page.keyboard.type("0.9");
    await page.keyboard.press("Enter");
    const inked = await page.waitForSelector("[data-event-id]", { timeout: 10000 });
    const eventId = await inked.getAttribute("data-event-id");
    await page.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") === h, expected.state_hash, { timeout: 5000 });
    assert.equal(eventId, expected.event_id, "GM edit and auto-approved player intent give the same event");
    assert.match(await page.locator("section[aria-label='Tarn']").innerText(), /MASS\s*·\s*SET[\s\S]*0\.9/i);

    // U2: derived values follow edits; drag an orbit; undo and redo; why?
    const card = page.locator("section[aria-label='Tarn']");
    // A field's shown value, whether typed (input) or read-only (output).
    const fieldStartsWith = (prefix) =>
      page.evaluate((p) => {
        const f = [...document.querySelectorAll("section[aria-label='Tarn'] .field")].find((x) => x.querySelector("label")?.textContent.trim().toLowerCase().startsWith(p.toLowerCase()));
        const v = f?.querySelector("input, output");
        return v ? (v.value ?? v.textContent).trim() : "";
      }, prefix);
    await page.waitForFunction(() => /0\.9\b/.test(document.querySelector("section[aria-label='Tarn']")?.textContent ?? ""));
    const gravityAfterMass = await fieldStartsWith("Gravity");
    assert.notEqual(gravityAfterMass, "0.908 g", "gravity re-derived after the mass edit");
    const surface0 = parseFloat(await fieldStartsWith("Surface"));
    const tarn = page.locator("[role=button][aria-label^='Tarn'] circle.hit");
    const star = page.locator("[role=button][aria-label^='Kestrel'] circle.hit");
    const tb = await tarn.boundingBox();
    const sb = await star.boundingBox();
    const [tx, ty, sx, sy] = [tb.x + tb.width / 2, tb.y + tb.height / 2, sb.x + sb.width / 2, sb.y + sb.height / 2];
    await page.mouse.move(tx, ty);
    await page.mouse.down();
    for (let k = 1; k <= 10; k++) await page.mouse.move(tx + ((tx - sx) * 0.35 * k) / 10, ty + ((ty - sy) * 0.35 * k) / 10);
    await page.mouse.up();
    await page.waitForFunction(() => document.querySelectorAll("[data-event-id]").length >= 2, null, { timeout: 5000 });
    await page.waitForFunction((s0) => {
      const f = [...document.querySelectorAll("section[aria-label='Tarn'] .field")].find((x) => /^surface/i.test(x.textContent.trim()));
      return f && parseFloat(f.textContent.replace(/^\D+/, "")) < s0;
    }, surface0, { timeout: 5000 });
    const orbitDragged = parseFloat(await fieldStartsWith("Orbit"));
    assert.ok(orbitDragged > 0.52, `dragged orbit ${orbitDragged} AU`);
    const surface1 = parseFloat(await fieldStartsWith("Surface"));
    await page.keyboard.press("Control+z");
    const orbitIs = (want) => page.waitForFunction((w) => [...document.querySelectorAll("section[aria-label='Tarn'] input")].some((i) => i.value === w), want, { timeout: 5000 });
    await orbitIs("0.52 AU");
    await page.keyboard.press("Control+Shift+z");
    await orbitIs(`${orbitDragged} AU`);
    const events = await page.locator("[data-event-id]").count();
    assert.equal(events, 4, "mass edit, drag, undo and redo are four inked events");
    await card.locator(".field").filter({ hasText: /^surface/i }).getByRole("button", { name: /why/i }).click();
    const why = page.locator("[role=region][aria-label^='Why']");
    await why.locator(".formula").waitFor();
    const formula = await why.locator(".formula").innerText();
    await why.getByRole("tab", { name: "Science" }).click();
    const science = await why.locator("[role=tabpanel]").innerText();
    await why.getByRole("button", { name: "why?" }).first().click();
    await page.waitForFunction(() => document.querySelectorAll("[aria-label='Explanation trail'] button").length === 2);
    const trail = await why.locator("[aria-label='Explanation trail']").innerText();
    u2 = { gravityAfterMass, orbitDragged, surface: [surface0, surface1], formula, science: science.slice(0, 60), trail: trail.replace(/\s+/g, " ") };
    await why.getByRole("button", { name: "Close the explanation" }).click();

    // Ruler by keyboard.
    await page.keyboard.press("Escape");
    await page.locator("body").click({ position: { x: 5, y: 5 } }).catch(() => {});
    await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
    await page.keyboard.press("t");
    const before = Number(await page.getAttribute("[role=slider]", "aria-valuenow"));
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("PageDown");
    const after = Number(await page.getAttribute("[role=slider]", "aria-valuenow"));
    assert.equal(after, before + 100_000_000 - 1_000_000_000);

    // Modes and theme from the keyboard.
    await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
    await page.keyboard.press("v");
    await page.locator("[role=button][aria-label^='Tarn']").focus();
    await page.keyboard.press("Enter");
    assert.equal(await page.locator(".area-card input").count(), 0, "view mode has no inputs");
    await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
    await page.keyboard.press("n");
    assert.equal(await page.getAttribute("html", "data-theme"), "night-chart");
    await page.keyboard.press("?");
    assert.ok(await page.locator("dialog[open]").isVisible());
    await page.keyboard.press("Escape");
    await page.close();

    // Screens and phone layout, both modes.
    const phone = {};
    for (const theme of ["chart-room", "night-chart"]) {
      for (const [form, viewport] of [["desktop", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
        const ctx = await browser.newContext({ viewport, deviceScaleFactor: 1 });
        await ctx.addInitScript((t) => localStorage.setItem("sse4.theme", t), theme);
        const p = await ctx.newPage();
        p.on("pageerror", (e) => errors.push(e.message));
        await p.goto(url);
        await p.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
        await p.locator("[role=button][aria-label^='Tarn']").click();
        await p.waitForSelector("section[aria-label='Tarn']");
        if (form === "phone") {
          const w = await p.evaluate(() => document.scrollingElement.scrollWidth);
          phone[theme] = w;
          assert.ok(w <= 390, `phone layout scrolls sideways (${w}px)`);
          assert.ok(await p.locator("[role=slider]").isVisible(), "ruler visible on phone");
        }
        if (form === "desktop") {
          await p.locator("section[aria-label='Tarn'] .field").filter({ hasText: /^surface/i }).getByRole("button", { name: /why/i }).click();
          await p.locator("[role=region][aria-label^='Why'] .formula").waitFor();
        }
        if (screens && name === "chromium") {
          mkdirSync(screens, { recursive: true });
          await p.waitForTimeout(300);
          await p.screenshot({ path: `${screens}/${theme}-${form}.png`, fullPage: form === "phone" });
        }
        await ctx.close();
      }
    }
    await browser.close();
    assert.deepEqual(errors, [], "no page errors");
    results.push({ engine: name, tabsToTarn, tabsToMass, eventId, phoneWidth: phone, u2 });
    console.log(`U2 ${name}: ${JSON.stringify(u2)}`);
    console.log(`PASS ${name}: Tarn reached in ${tabsToTarn} tabs, Mass in ${tabsToMass}; event ${eventId.slice(0, 12)} matches the player seat; phone width ${JSON.stringify(phone)}`);
  }
} finally {
  server.kill();
}
