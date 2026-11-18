// U5 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/setup.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Packs (PackSetup board): the live stack, Check stack, a sample fictional
// pack previewed value by value, accepted, and its labels on the card.
// Screens (Compose board): the view stack, a player screen drawn live from
// its document, an invalid parameter caught, Orac's sample screen request
// validated and accepted.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8300 + Math.floor(Math.random() * 300);
const url = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const squash = (t) => t.replace(/\s+/g, " ").trim();

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(url);
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    const stateHash = await page.getAttribute("[data-state-hash]", "data-state-hash");
    const screenBtn = (n) => page.getByRole("group", { name: "Screen" }).getByRole("button", { name: n });

    // Packs.
    await screenBtn("Packs").click();
    const packs = page.locator("[role=region][aria-label=Packs]");
    await packs.getByRole("button", { name: "Check stack" }).click();
    await packs.locator(".verdict").getByText("Validated").waitFor();
    assert.deepEqual(await packs.locator("[data-pack]").evaluateAll((els) => els.map((e) => e.dataset.pack)), ["sse.base-physics"]);
    await packs.getByRole("button", { name: "Try the sample pack" }).click();
    const proposal = packs.locator("[aria-label=Proposal]");
    await proposal.getByText("Validated").waitFor();
    const rows = (await proposal.locator("tbody tr").allInnerTexts()).map(squash);
    assert.ok(rows.some((r) => /^Tarn · surface/.test(r) && /fictional/.test(r)), rows.join(" | "));
    if (screens && name === "chromium") (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/packs-desktop.png` }));
    await proposal.getByRole("button", { name: "Accept" }).click();
    await page.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sample.warm-worlds"));
    assert.equal(await page.getAttribute("[data-state-hash]", "data-state-hash"), stateHash, "stacking packs is not an authored edit");
    await screenBtn("Chart").click();
    await page.locator("[role=button][aria-label^='Tarn']").click();
    const tarn = page.locator("section[aria-label='Tarn']");
    await tarn.locator(".stamp", { hasText: "Fictional" }).waitFor();
    const surface = squash(await tarn.locator(".field").filter({ hasText: /^surface/i }).locator("label").innerText());
    assert.match(surface, /fictional/i);

    // Screens.
    await screenBtn("Screens").click();
    const comp = page.locator("[role=region][aria-label='Screens composer']");
    await comp.locator("[data-view='sse.view.card'] article").waitFor();
    assert.equal(await comp.locator(".preview [data-view]").count(), 6, "the Bridge screen's six items drawn");
    // A bad parameter is caught by the screen checker, not drawn.
    const firstParams = comp.locator(".items li").nth(2).locator("input");
    await firstParams.fill('{"style": "dial"}');
    await firstParams.press("Enter");
    await comp.locator(".problems").getByText(/style: must be one of/).waitFor();
    await firstParams.fill('{"field": "ocean.fraction", "style": "gauge", "range": [0, 1]}');
    await firstParams.press("Enter");
    await comp.locator(".preview [role=meter]").first().waitFor();
    // Orac's sample request: validated, then accepted.
    const orac = comp.locator("[aria-label='Orac change request']");
    await orac.getByText("Validated").waitFor();
    if (screens && name === "chromium") await page.screenshot({ path: `${screens}/screens-desktop.png` });
    await orac.getByRole("button", { name: "Accept" }).click();
    await comp.getByRole("heading", { name: "Bridge approach" }).waitFor();
    assert.equal(await comp.locator(".preview [data-view]").count(), 4);
    await comp.getByRole("button", { name: "Show document" }).click();
    assert.match(await comp.locator("pre.doc").innerText(), /"id": "sse.screen.players.bridge-approach"/);
    await browser.close();
    assert.deepEqual(errors, [], "no page errors");
    console.log(`U5 ${name}: packs preview ${rows.length} rows, e.g. ${rows.find((r) => r.startsWith("Tarn · surface"))}; card "${surface}"`);
    console.log(`PASS ${name}`);
  }
} finally {
  server.kill();
}
