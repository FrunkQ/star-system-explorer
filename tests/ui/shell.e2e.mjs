// U0 and U1 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/shell.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Checks: keyboard-only selection and editing; U3: open a flow from a why,
// change a parameter, an invalid value never reaches Adopt, diff and effect,
// adopt, badge on the card; the edit inked through the GM
// seat equals the same intent from an auto-approved player seat (event id and
// state hash); ruler keyboard steps; mode and theme keys; phone layout has no
// sideways scroll; screenshots at desktop and phone widths in both modes.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk, go, more } from "./nav.mjs";
import { createRequire } from "node:module";
import { Session } from "../../packages/ui-contract/src/session.ts";
import { decide, gmSeat, tableSeat } from "../../packages/ui-contract/src/seat.ts";
import { nodeKernel } from "../../packages/ui-contract/test/node-kernel.ts";
import { enterFacts } from "../../packages/ui-contract/src/data-views.ts";
import { readFileSync } from "node:fs";
const doc = (p) => JSON.parse(readFileSync(new URL(`../../${p}`, import.meta.url), "utf8"));

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
  const packs = [doc("packs/base-physics/pack.json"), doc("packages/shell/src/documents/kestrel.player-data.pack.json")];
  // The campaign opens with the GM's recorded player-facing facts, as the shell does (GM work: they include disclosure grants).
  const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" }, packs);
  assert.equal(await enterFacts(s, doc("packages/shell/src/documents/kestrel.player-facts.json")), null);
  // The helm's intent is inside its auto-approve scope, so the GM's session applies it without asking (V10: the GM is the single writer).
  const intent = { kind: "set_value", address: TARN, field: "mass_earth", value: 0.9, at_yr: NOW };
  assert.deepEqual(decide(seat, intent), { action: "apply" });
  const r = await s.send(intent);
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
let u3 = null;
let planet = null;
const expected = await playerEvent();
try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    const browser = await engine.launch();
    const errors = [];

    // Desktop, keyboard only.
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(url + "#chart");
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await desk(page, "Build");
    // The campaign opens with its recorded player-facing facts, inked at year 0; count edits after them.
    await page.waitForFunction(() => document.querySelectorAll("[data-event-id]").length > 0);
    const opening = await page.locator("[data-event-id]").count();
    const tabsToTarn = await tabTo(page, () => document.activeElement?.getAttribute("aria-label")?.startsWith("Tarn"));
    await page.keyboard.press("Enter");
    await page.waitForSelector("section[aria-label='Tarn']");
    // From Tarn the way to Mass passes the Build tool bar (Undo, Redo, Copy, Keyboard) and the card's fields before it,
    // each field with its "Known to" eye and popup button once the kernel answers known_to (SV1).
    const tabsToMass = await tabTo(page, () => document.activeElement?.id && document.querySelector(`label[for='${document.activeElement.id}']`)?.textContent?.startsWith("Mass"), 36);
    await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
    await page.keyboard.type("0.9");
    await page.keyboard.press("Enter");
    const inked = await page.waitForSelector(`[data-event-id='${expected.event_id}']`, { timeout: 10000 });
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
    // A shown figure as a number: a surface reads in °C (ED1a), so it can be below zero, written with a true minus.
    const num = (t) => parseFloat((t.replace(/−/g, "-").replace(/,/g, "").match(/-?\d+(?:\.\d+)?/) ?? ["NaN"])[0]);
    await page.waitForFunction(() => /0\.9\b/.test(document.querySelector("section[aria-label='Tarn']")?.textContent ?? ""));
    const gravityAfterMass = await fieldStartsWith("Gravity");
    assert.notEqual(gravityAfterMass, "0.908 g", "gravity re-derived after the mass edit");
    const surface0 = num(await fieldStartsWith("Surface"));
    const tarn = page.locator("[role=button][aria-label^='Tarn'] circle.hit");
    const star = page.locator("[role=button][aria-label^='Kestrel'] circle.hit");
    const tb = await tarn.boundingBox();
    const sb = await star.boundingBox();
    const [tx, ty, sx, sy] = [tb.x + tb.width / 2, tb.y + tb.height / 2, sb.x + sb.width / 2, sb.y + sb.height / 2];
    await page.mouse.move(tx, ty);
    await page.mouse.down();
    for (let k = 1; k <= 10; k++) await page.mouse.move(tx + ((tx - sx) * 0.35 * k) / 10, ty + ((ty - sy) * 0.35 * k) / 10);
    await page.mouse.up();
    await page.waitForFunction(() => (n) => document.querySelectorAll("[data-event-id]").length >= n + 2, opening, { timeout: 5000 });
    await page.waitForFunction((s0) => {
      const f = [...document.querySelectorAll("section[aria-label='Tarn'] .field")].find((x) => /^surface/i.test(x.textContent.trim()));
      const t = (f?.querySelector("input, output")?.value ?? f?.querySelector("output")?.textContent ?? "").replace(/−/g, "-").replace(/,/g, "");
      return f && parseFloat((t.match(/-?\d+(?:\.\d+)?/) ?? ["NaN"])[0]) < s0;
    }, surface0, { timeout: 5000 });
    const orbitDragged = num(await fieldStartsWith("Orbit"));
    assert.ok(orbitDragged > 0.52, `dragged orbit ${orbitDragged} AU`);
    const surface1 = num(await fieldStartsWith("Surface"));
    await page.keyboard.press("Control+z");
    const orbitIs = (want) => page.waitForFunction((w) => [...document.querySelectorAll("section[aria-label='Tarn'] input")].some((i) => i.value === w), want, { timeout: 5000 });
    await orbitIs("0.52 AU");
    await page.keyboard.press("Control+Shift+z");
    await orbitIs(`${orbitDragged} AU`);
    const events = await page.locator("[data-event-id]").count();
    assert.equal(events - opening, 4, "mass edit, drag, undo and redo are four inked events");
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

    // U3: open the flow from the why, change a parameter, see the diff, adopt, see the badge.
    const hashBeforePhysics = await page.getAttribute("[data-state-hash]", "data-state-hash");
    await why.getByRole("button", { name: "Open the flow" }).click();
    const editor = page.locator("section[aria-label='Flow editor']");
    await editor.waitFor();
    assert.equal(await editor.getByRole("button", { name: /^Node teq/ }).getAttribute("aria-pressed"), "true", "opened at the node that published the value");
    const surfChip = editor.locator("[data-kind='planet.surface_temperature'] .val");
    await page.waitForFunction(() => /\d/.test(document.querySelector("[data-kind='planet.surface_temperature'] .val")?.textContent ?? ""));
    const chipBefore = (await surfChip.innerText()).trim();
    await editor.locator("[aria-label^='Explain node']").getByRole("tab", { name: "Engine" }).click();
    const engineLens = await editor.locator("[aria-label^='Explain node'] [role=tabpanel]").innerText();
    assert.match(engineLens, /redistribution=2\.5e-1/, "engine lens names the parameter in force");
    const param = editor.locator("#p-teq-redistribution");
    await param.fill("7");
    await param.press("Enter");
    const request = page.locator(".area-card [role=region][aria-label^='Request']");
    await request.getByText("Rejected").waitFor();
    assert.ok(await request.getByRole("button", { name: "Adopt" }).isDisabled(), "an invalid change never reaches Adopt");
    await param.fill("0.5");
    await param.press("Enter");
    await request.getByText("Validated").waitFor();
    const diff = (await request.locator("table.diff tbody").innerText()).replace(/\s+/g, " ").trim();
    assert.equal(diff, "teq.redistribution 0.25 0.5");
    const effect = (await request.locator("table.effect tr.moved").allInnerTexts()).map((t) => t.replace(/\s+/g, " ").trim());
    assert.ok(effect.some((t) => t.startsWith("surface temperature")), `effect rows: ${effect}`);
    await request.getByRole("button", { name: "Adopt" }).click();
    await page.waitForFunction(() => document.querySelector("[data-stack-hash]")?.getAttribute("data-physics") === "campaign");
    await page.waitForFunction((b) => (document.querySelector("[data-kind='planet.surface_temperature'] .val")?.textContent ?? "").trim() !== b, chipBefore);
    const chipAfter = (await surfChip.innerText()).trim();
    // Adopting physics is chronicled (a stack.adopt event), so the state hash moves.
    await page.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hashBeforePhysics);
    await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
    await page.keyboard.press("p");
    await page.locator("[role=button][aria-label^='Tarn']").click();
    const tarnCard = page.locator("section[aria-label='Tarn']");
    await tarnCard.locator(".stamp", { hasText: "Changed physics" }).waitFor();
    const surfaceLabel = await tarnCard.locator(".field").filter({ hasText: /^surface/i }).locator("label").innerText();
    assert.match(surfaceLabel, /changed physics/i);
    u3 = { chipBefore, chipAfter, diff, effect, badge: surfaceLabel.replace(/\s+/g, " ") };

    // Planet frame: a planet's moons orbit it, not the star (handover/rendering-rules.md).
    const chartSvg = page.locator(".area-chart svg");
    assert.equal(await chartSvg.locator("[role=button][aria-label^='Hasp']").count(), 0, "moons are not drawn in the star's frame");
    assert.equal(await page.getAttribute("[role=button][aria-label^='Vault']", "data-moons"), "3", "Vault shows it has three moons");
    await page.locator("[role=button][aria-label^='Vault']").click();
    await page.getByRole("button", { name: "Moons of Vault ›" }).click();
    await page.waitForFunction(() => document.querySelector(".area-chart svg")?.getAttribute("aria-label") === "Chart of Vault and its moons");
    const moonNames = await chartSvg.locator(".body").evaluateAll((g) => g.map((x) => x.getAttribute("aria-label").split(",")[0]));
    assert.deepEqual(moonNames, ["Vault", "Hasp", "Rivet", "Latch"], "the planet at the centre and its moons about it");
    assert.match(await page.locator(".area-chart").innerText(), /in kilometres/);
    await chartSvg.locator("[role=button][aria-label^='Hasp']").click();
    const haspCard = page.locator("section[aria-label='Hasp']");
    await haspCard.waitFor();
    const haspOrbit = () => haspCard.locator(".field").filter({ hasText: /^orbit/i }).locator("input").inputValue();
    assert.equal(await haspOrbit(), "412 thousand km", "a moon's orbit said by the units engine (ED1a)");
    // The chart settles after the card opens (the frame's grow and any re-layout): measure only once the marks are still.
    const still = async (sel) => {
      let last = null;
      for (let k = 0; k < 40; k++) {
        const b = await chartSvg.locator(sel).boundingBox();
        if (b && last && Math.abs(b.x - last.x) < 0.5 && Math.abs(b.y - last.y) < 0.5) return b;
        last = b;
        await page.waitForTimeout(100);
      }
      return last;
    };
    const vb = await still("[role=button][aria-label^='Vault'] circle.hit");
    const hb = await still("[role=button][aria-label^='Hasp'] circle.hit");
    const hashBeforeDrag = await page.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    const [hx, hy, vx, vy] = [hb.x + hb.width / 2, hb.y + hb.height / 2, vb.x + vb.width / 2, vb.y + vb.height / 2];
    await page.mouse.move(hx, hy);
    await page.mouse.down();
    for (let k = 1; k <= 8; k++) await page.mouse.move(hx + ((hx - vx) * 0.2 * k) / 8, hy + ((hy - vy) * 0.2 * k) / 8);
    await page.mouse.up();
    // Letting go is one change: wait for the kernel to take it (the state hash moves), then for the card to say it.
    await page.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hashBeforeDrag, { timeout: 20000 });
    await haspCard.locator(".field").filter({ hasText: /^orbit/i }).locator("input").waitFor();
    await page.waitForFunction(() => {
      const i = [...document.querySelectorAll("section[aria-label='Hasp'] .field")].find((x) => /^orbit/i.test(x.textContent.trim()))?.querySelector("input");
      const km = i && /km$/.test(i.value) ? parseFloat(i.value.replace(/,/g, "")) * (/thousand/.test(i.value) ? 1e3 : /million/.test(i.value) ? 1e6 : 1) : 0;
      return km > 412000;
    }, null, { timeout: 20000 });
    const haspDragged = await haspOrbit();
    const orbitInput = haspCard.locator(".field").filter({ hasText: /^orbit/i }).locator("input");
    await orbitInput.fill("500,000 km");
    await orbitInput.press("Enter");
    await page.waitForFunction(() => [...document.querySelectorAll("section[aria-label='Hasp'] input")].some((i) => i.value === "500 thousand km"), null, { timeout: 5000 });
    await page.getByRole("button", { name: "‹ Kestrel" }).click();
    await page.waitForFunction(() => document.querySelector(".area-chart svg")?.getAttribute("aria-label")?.startsWith("System chart"));
    await page.locator("section[aria-label='Vault']").waitFor();
    planet = { moonNames, haspDragged };

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
    assert.equal(await page.locator("dialog[open] [data-help-page]").getAttribute("data-help-page"), "keys", "? opens Help on the keyboard page");
    await page.keyboard.press("Escape");
    // The Help door (V3 gap 31): More shows a dot until What's new has been opened on this device.
    await page.getByRole("button", { name: /^More/ }).click();
    assert.ok(await page.locator("[data-more-help] .news-dot").isVisible(), "More's Help shows unseen news");
    await page.locator("[data-more-help]").click();
    await page.locator("dialog[open] [data-help-page='new'] [data-news]").first().waitFor();
    await page.getByRole("dialog").getByRole("button", { name: "Getting started" }).click();
    await page.locator("dialog[open] [data-help-sol]").waitFor();
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => !document.querySelector("dialog[open]") && !document.querySelector(".news-dot"), null, { timeout: 5000 });
    await page.close();

    // Screens and phone layout, both modes.
    const phone = {};
    for (const theme of ["chart-room", "night-chart"]) {
      for (const [form, viewport] of [["desktop", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
        const ctx = await browser.newContext({ viewport, deviceScaleFactor: 1 });
        await ctx.addInitScript((t) => localStorage.setItem("sse4.theme", t), theme);
        const p = await ctx.newPage();
        p.on("pageerror", (e) => errors.push(e.message));
        await p.goto(url + "#chart");
        await p.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
        await desk(p, "Build");
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
        // The physics screen with a draft change, at both widths.
        await go(p, "physics");
        const ed = p.locator("section[aria-label='Flow editor']");
        await ed.waitFor();
        await ed.getByRole("button", { name: /^Node teq/ }).click();
        await ed.locator("#p-teq-redistribution").fill("0.5");
        await ed.locator("#p-teq-redistribution").press("Enter");
        await p.locator(".area-card").getByText("Validated").waitFor();
        if (form === "phone") {
          const w = await p.evaluate(() => document.scrollingElement.scrollWidth);
          assert.ok(w <= 390, `physics screen scrolls sideways on a phone (${w}px)`);
        }
        if (screens && name === "chromium") {
          await p.waitForTimeout(300);
          await p.screenshot({ path: `${screens}/${theme}-physics-${form}.png`, fullPage: form === "phone" });
        }
        await ctx.close();
      }
    }
    await browser.close();
    assert.deepEqual(errors, [], "no page errors");
    results.push({ engine: name, tabsToTarn, tabsToMass, eventId, phoneWidth: phone, u2, u3 });
    console.log(`U2 ${name}: ${JSON.stringify(u2)}`);
    console.log(`U3 ${name}: ${JSON.stringify(u3)}`);
    console.log(`Planet frame ${name}: ${JSON.stringify(planet)}`);
    console.log(`PASS ${name}: Tarn reached in ${tabsToTarn} tabs, Mass in ${tabsToMass}; event ${eventId.slice(0, 12)} matches the player seat; phone width ${JSON.stringify(phone)}`);
  }
} finally {
  server.kill();
}
