// U5 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/setup.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Packs (PackSetup board): the live stack, Check stack, a sample fictional
// pack previewed value by value, accepted, and its labels on the card.
// Player data (PlayerData board): the live scene for everyone, an edited
// disclosure applied and put back, a look through another view (the
// Scanned button has gone; the eye on a card replaces it, known.e2e).
// Screens (Compose board): the view stack, a player screen drawn live from
// its document, an invalid parameter caught, Orac's sample screen request
// validated and accepted. V3: screens opened from the campaign's documents
// and from a file, a bad one refused, and a composed screen handed to the
// players page, which also loads a report by name.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
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
    await page.goto(url + "#chart");
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    const stateHash = await page.getAttribute("[data-state-hash]", "data-state-hash");

    // Packs.
    await go(page, "packs");
    const packs = page.locator("[role=region][aria-label=Packs]");
    await packs.getByRole("button", { name: "Check stack" }).click();
    await packs.locator(".verdict").getByText("Validated").waitFor();
    assert.deepEqual(await packs.locator("[data-pack]").evaluateAll((els) => els.map((e) => e.dataset.pack)), ["kestrel.player-data", "sse.base-manifest", "sse.base-reference", "sse.base-time", "sse.base-physics"]);
    const proposal = packs.locator("[aria-label=Proposal]");
    // ND6: a paper's pack shows its sources, fudges and reproduction checks before anyone accepts it.
    await packs.locator("#pack-file").setInputFiles("packages/ui-contract/test/fixtures/paper.pack.json");
    const paperReport = proposal.locator("[data-pack-report='test.paper']");
    await paperReport.waitFor({ timeout: 20000 });
    assert.equal(await paperReport.getAttribute("data-reproduced"), "true");
    assert.equal(await paperReport.locator("[data-pack-source]").count(), 2);
    assert.equal(await paperReport.locator("[data-pack-fudge]").count(), 2);
    assert.match(squash(await paperReport.locator("[data-check='A world like ours']").innerText()), /✓ A world like ours.*published 9\.8 m\/s², this pack 9\.8\d* m\/s²; .* off, the pack allows 1 %/);
    if (screens && name === "chromium") (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/packs-paper-report.png` }));
    await proposal.getByRole("button", { name: "Cancel" }).click();
    await packs.getByRole("button", { name: "Try the sample pack" }).click();
    await proposal.getByText("Validated").waitFor();
    const rows = (await proposal.locator("tbody tr").allInnerTexts()).map(squash);
    assert.ok(rows.some((r) => /^Tarn · surface/.test(r) && /fictional/.test(r)), rows.join(" | "));
    if (screens && name === "chromium") (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/packs-desktop.png` }));
    await proposal.getByRole("button", { name: "Accept" }).click();
    await page.waitForFunction(() => [...document.querySelectorAll("[data-pack]")].some((e) => e.dataset.pack === "sample.warm-worlds"));
    await page.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, stateHash);
    await go(page, "system");
    await page.locator("[role=button][aria-label^='Tarn']").click();
    const tarn = page.locator("section[aria-label='Tarn']");
    await tarn.locator(".stamp", { hasText: "Fictional" }).waitFor();
    const surface = squash(await tarn.locator(".field").filter({ hasText: /^surface/i }).locator("label").innerText());
    assert.match(surface, /fictional/i);

    // Player data.
    await go(page, "data");
    const dv = page.locator("[role=region][aria-label='Player data']");
    const pcard = dv.locator("[data-view='sse.view.card'] article");
    const massOf = async () => squash(await pcard.locator("[data-field='mass_kg'] dd").innerText());
    await pcard.waitFor();
    assert.equal(await pcard.getAttribute("aria-label"), "Tarn");
    assert.equal(await massOf(), "not yet scanned");
    const beforeData = await page.getAttribute("[data-state-hash]", "data-state-hash");
    await dv.locator("details.disc summary").click();
    await dv.getByLabel("Tier of mass_kg").selectOption("obvious");
    await dv.getByText("1 unapplied: mass_kg scanned to obvious.").waitFor();
    // The draft is previewed before it is applied (scene with a draft observe setup; nothing recorded).
    await page.waitForFunction(() => /Earth/.test(document.querySelector("[aria-label='Player data'] [data-field='mass_kg'] dd")?.textContent ?? ""));
    assert.equal(await page.getAttribute("[data-state-hash]", "data-state-hash"), beforeData, "a draft preview records nothing");
    await dv.getByRole("button", { name: "Apply to the campaign" }).click();
    await page.waitForFunction(() => /Earth/.test(document.querySelector("[aria-label='Player data'] [data-field='mass_kg'] dd")?.textContent ?? ""));
    const massApplied = await massOf();
    await page.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, beforeData);
    if (screens && name === "chromium") (await dv.evaluate((e) => e.scrollTo(0, 0)), await page.screenshot({ path: `${screens}/data-views-desktop.png` }));
    await dv.getByRole("button", { name: "Put back" }).click();
    await page.waitForFunction(() => /not yet scanned/.test(document.querySelector("[aria-label='Player data'] [data-field='mass_kg'] dd")?.textContent ?? ""));
    await dv.getByLabel("Through").selectOption({ label: "Survey on file" });
    await page.waitForFunction(() => /rare earths/.test(document.querySelector("[aria-label='Player data'] [data-field='resources'] dd")?.textContent ?? ""));
    await dv.getByLabel("Through").selectOption({ label: "their own grants" });
    // The Scanned button has gone: the eye on a card shows the table one thing (known.e2e).
    assert.equal(await dv.getByRole("button", { name: /Scanned button/ }).count(), 0);

    // Screens.
    await go(page, "screens");
    const comp = page.locator("[role=region][aria-label='Screens composer']");
    await comp.locator("[data-view='sse.view.card'] article").waitFor();
    assert.equal(await comp.locator(".preview [data-view]").count(), 6, "the Bridge screen's six items drawn");
    // A bad parameter is caught by the screen checker, not drawn.
    const firstParams = comp.locator(".items li").nth(2).locator(".pl input");
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
    // V3: the board's screen loads from its document. The campaign's survey report (page layout, through the survey view, items bound to Tarn).
    await comp.getByLabel("Open a campaign screen").selectOption({ label: "players.survey.screen.json" });
    await comp.getByRole("heading", { name: "Tarn survey report" }).waitFor();
    await comp.locator(".preview [data-layout='page']").waitFor();
    assert.equal(await comp.locator(".preview [data-view]").count(), 3, "the report's three items");
    assert.match(await comp.locator(".preview [data-view='sse.view.text']").innerText(), /Tarn/);
    assert.doesNotMatch(await comp.locator(".preview [data-view='sse.view.text']").innerText(), /Vault/, "bound to Tarn only");
    assert.match(await comp.locator(".preview [data-field='interior.core_fraction']").innerText(), /\d/, "seen through the survey view");
    // A document opened from a file; a broken one is refused with reasons.
    const fileDoc = { schema: "sse.screen/1", id: "sse.screen.players.flow-test", label: "Tiles", audience: "players", layout: "flow", theme: "chart", symbology: "sse.symbology.diagrammatic", vocab: "plain", items: [{ view: "sse.view.card", area: "1" }, { view: "sse.view.readout", area: "2", params: { field: "mass_kg" } }] };
    await comp.locator("input[type=file]").setInputFiles({ name: "tiles.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify(fileDoc)) });
    await comp.getByRole("heading", { name: "Tiles" }).waitFor();
    await comp.locator(".preview [data-layout='flow']").waitFor();
    await comp.locator("input[type=file]").setInputFiles({ name: "bad.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify({ ...fileDoc, label: "Bad", data_view: "spies" })) });
    await comp.locator(".problems").getByText(/no data view spies/).waitFor();
    // Handed to the players page, which loads it from the document.
    await comp.locator("input[type=file]").setInputFiles({ name: "tiles.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify(fileDoc)) });
    const [pp] = await Promise.all([page.context().waitForEvent("page"), comp.getByRole("button", { name: "Show on the players page" }).click()]);
    await pp.locator("[data-screen-id='sse.screen.players.flow-test'][data-layout='flow']").waitFor();
    assert.equal(await pp.locator("[data-screen-id] [data-view]").count(), 2);
    const report = await browser.newPage({ viewport: { width: 1280, height: 900 } });
    report.on("pageerror", (e) => errors.push(e.message));
    await report.goto(`${url}players.html?screen=players.survey.screen.json`);
    await report.locator("[data-screen-id='sse.screen.players.tarn-survey'][data-layout='page']").waitFor();
    assert.match(await report.locator(".kicker").innerText(), /seen through survey/i);
    if (screens && name === "chromium") await report.screenshot({ path: `${screens}/report-desktop.png`, fullPage: true });
    await browser.close();
    assert.deepEqual(errors, [], "no page errors");
    console.log(`U5 ${name}: data views: mass applied as obvious reads "${massApplied}", put back, survey and Scanned checked`);
    console.log(`U5 ${name}: packs preview ${rows.length} rows, e.g. ${rows.find((r) => r.startsWith("Tarn · surface"))}; card "${surface}"`);
    console.log(`PASS ${name}`);
  }
} finally {
  server.kill();
}
