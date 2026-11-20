// A2 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/orac.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Orac on a mock provider: its first draft asks for a value the kernel
// rejects; SSE sends the kernel's words back; the fixed draft passes. Only the
// passing draft has an Accept button, the failing one never appears with one,
// and nothing changes (no request filed, state hash unchanged) until Accept.
// Accepting files Orac's request on the Physics screen, validated, for the GM
// to adopt there.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mockProvider } from "../../packages/agents/test/mock-provider.mjs";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8650 + Math.floor(Math.random() * 200);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const CLIMATE = "sse.base.planet-climate";
const fence = (o) => "```json\n" + JSON.stringify(o) + "\n```";
const draft = (after) => fence({ note: "More heat kept on the day side.", make: [{ kind: "physics", title: "Hold day-side heat", rationale: "Tidally locked worlds carry less heat to the night side.", doc: { flow: CLIMATE, edits: [{ node: "teq", param: "redistribution", after }] } }] });
const view = (tiers) => fence({ note: "A view for a ship with only long-range sensors.", make: [{ kind: "data_view", title: "Orbits only", doc: { id: "orbits-only", label: "Orbits only", show_tiers: tiers, tags: { mode: "none" }, unknown_style: "omit" } }] });
// Scripted by what Orac was last told: a first draft out of range, the fix after the kernel's words come back.
// The second request (a data view) first shows a GM-only tier, then is fixed after Player data's check answers.
const provider = await mockProvider({
  context: 32000,
  replies: (messages) => {
    const last = messages.at(-1).content;
    if (/GM-only facts/.test(last)) return view(["obvious"]);
    if (/long-range sensors/.test(messages[1].content)) return view(["obvious", "secret"]);
    return /allowed range/.test(last) ? draft(0.5) : draft(7);
  },
});
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    provider.seen.length = 0; // one mock serves every engine: count from this engine's start
    const browser = await engine.launch();
    const errors = [];
    const gm = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const h0 = await gm.locator("[data-state-hash]").getAttribute("data-state-hash");

    await gm.getByRole("button", { name: "Agents" }).click();
    const dlg = gm.getByRole("dialog", { name: "Agents" });
    await dlg.getByRole("tab", { name: "Orac" }).click();
    assert.ok(await dlg.locator("[data-agent-off=orac]").isVisible(), "no model: Orac says so");
    assert.ok(await dlg.getByRole("button", { name: "Draft" }).isDisabled());

    await dlg.getByRole("tab", { name: "Link models" }).click();
    await dlg.getByLabel("Provider kind").selectOption("other");
    await dlg.getByLabel("Provider address").fill(provider.base);
    await dlg.getByRole("button", { name: "Add and check" }).click();
    await dlg.getByRole("button", { name: "2 models" }).waitFor();
    await dlg.getByLabel("Building provider").selectOption("other");

    await dlg.getByRole("tab", { name: "Orac" }).click();
    await dlg.getByLabel("Ask Orac").fill("Tidally locked worlds should keep more heat on their day side.");
    assert.match(await dlg.locator("[data-orac-estimate]").innerText(), /First try: about \d+ tokens in/);
    await dlg.getByRole("button", { name: "Draft" }).click();
    await dlg.locator("[data-orac-result]").waitFor({ timeout: 30000 });
    assert.equal(await dlg.locator("[data-orac-result]").getAttribute("data-orac-result"), "ok", await dlg.locator("[data-orac-result]").innerText());
    const tries = await dlg.getByRole("list", { name: "Orac's tries" }).innerText();
    assert.match(tries, /Try 1: SSE found 1 problem, sent back/);
    assert.match(tries, /Try 2: passed every check/);
    // The kernel's words reached Orac.
    const bodies = provider.seen.filter((x) => x.path.endsWith("/chat/completions")).map((x) => JSON.parse(x.body).messages);
    assert.match(bodies[1].at(-1).content, /outside the allowed range/);
    // Only the passing draft, with Accept; no failed card; nothing filed yet.
    assert.equal(await dlg.locator("[data-candidate=failed]").count(), 0);
    const card = dlg.locator("[data-candidate=passed]");
    assert.equal(await card.count(), 1);
    assert.match(await card.innerText(), /teq\.redistribution 0\.25 → 0\.5/);
    assert.equal(await gm.locator("[data-state-hash]").getAttribute("data-state-hash"), h0);
    await shot(gm, "a2-orac");
    await card.getByRole("button", { name: "Accept" }).click();
    await card.getByText("Filed on the Physics screen").waitFor();
    await dlg.evaluate((d) => d.close());

    // On the Physics screen, Orac's request is there, validated, waiting for the GM to adopt.
    await gm.keyboard.press("p");
    const req = gm.locator(".area-card").getByText("Hold day-side heat").first();
    await req.waitFor({ timeout: 10000 });
    assert.equal(await gm.locator("[data-state-hash]").getAttribute("data-state-hash"), h0, "nothing adopted by Orac");
    await shot(gm, "a2-physics");

    // Part 2: a data view. Player data's own check refuses a GM-only tier; the fix is applied only on Accept.
    await gm.getByRole("group", { name: "Screen" }).getByRole("button", { name: "Player data" }).click();
    const dv = gm.locator("[role=region][aria-label='Player data']");
    await dv.waitFor();
    assert.ok(!(await dv.innerText()).includes("Orbits only"));
    await gm.getByRole("button", { name: "Agents" }).click();
    await dlg.getByRole("tab", { name: "Orac" }).click();
    await dlg.getByLabel("Ask Orac").fill("A data view for a ship with only long-range sensors.");
    await dlg.getByRole("button", { name: "Draft" }).click();
    await dlg.locator("[data-orac-result=ok]").waitFor({ timeout: 30000 });
    assert.match(await dlg.getByRole("list", { name: "Orac's tries" }).innerText(), /Try 1: SSE found 1 problem, sent back/);
    const vcard = dlg.locator("[data-candidate=passed]");
    assert.match(await vcard.innerText(), /Orbits only: shows obvious, tags none, unknowns left out/);
    assert.ok(!(await dv.innerText()).includes("Orbits only"), "nothing applied before Accept");
    await vcard.getByRole("button", { name: "Accept" }).click();
    await dlg.getByText("Applied to the campaign's player data").waitFor();
    await dlg.evaluate((d) => d.close());
    await gm.waitForFunction(() => /Orbits only/.test(document.querySelector("[role=region][aria-label='Player data']")?.textContent ?? ""), null, { timeout: 10000 });
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: Orac's rejected draft fixed in the loop; only the passing one offered; filed on Physics after Accept; state hash unchanged; a data view applied only on Accept`);
  }
} finally {
  server.kill();
  await provider.close();
}
