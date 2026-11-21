// A3 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/minerva.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Minerva on a mock provider writes for the table ("everyone"), which has not
// scanned Tarn. Her first draft invents a number: it is sent back and never
// shown. The fix is shown, labelled narrative. The facts she was given hold
// nothing the table has not been shown, and SSE never sends the GM's secret
// note in any call.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
import { createRequire } from "node:module";
import { mockProvider } from "../../packages/agents/test/mock-provider.mjs";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8850 + Math.floor(Math.random() * 200);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const fence = (o) => "```json\n" + JSON.stringify(o) + "\n```";
const story = (text) => fence({ note: "n", make: [{ kind: "narrative", title: "Tarn", doc: { text } }] });
const provider = await mockProvider({ context: 32000, replies: (messages) => (/not in the facts/.test(messages.at(-1).content) ? story("Tarn turns beneath blue-white cloud. What lies below is not yet scanned.") : story("Tarn lies 7 light-minutes out.")) });
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
    await gm.getByRole("button", { name: "Ask", exact: true }).click();
    const dlg = gm.getByRole("dialog", { name: "Agents" });
    await dlg.getByRole("tab", { name: "Link models" }).click();
    await dlg.getByLabel("Provider kind").selectOption("other");
    await dlg.getByLabel("Provider address").fill(provider.base);
    await dlg.getByRole("button", { name: "Add and check" }).click();
    await dlg.getByRole("button", { name: "2 models" }).waitFor();
    await dlg.getByLabel("Describing provider").selectOption("other");

    await dlg.getByRole("tab", { name: "Minerva" }).click();
    await dlg.getByLabel("Audience").selectOption("everyone");
    const facts = await dlg.locator("[data-facts]").textContent({ timeout: 20000 });
    assert.match(facts, /Tarn: .*Not known to them: .*not yet scanned/);
    assert.ok(!/pirate/i.test(facts), "the table's facts hold no secret");
    await dlg.getByLabel("What about").fill("the approach to Tarn");
    await dlg.getByRole("button", { name: "Write" }).click();
    await dlg.locator("[data-minerva-result]").waitFor({ timeout: 30000 });
    assert.equal(await dlg.locator("[data-minerva-result]").getAttribute("data-minerva-result"), "ok", await dlg.locator("[data-minerva-result]").innerText());
    const text = await dlg.locator("[data-narrative=everyone]").innerText();
    assert.match(text, /Narrative · for Everyone/);
    assert.match(text, /Tarn turns beneath blue-white cloud/);
    assert.ok(!/7 light-minutes/.test(await dlg.innerText()), "the draft with an invented number was never shown");
    const bodies = provider.seen.filter((x) => x.path.endsWith("/chat/completions")).map((x) => JSON.parse(x.body).messages);
    assert.equal(bodies.length, 2);
    for (const msgs of bodies) for (const m of msgs) if (m.role !== "assistant") assert.ok(!/pirate|north pole/i.test(m.content), "SSE never sent the GM's secret");
    assert.match(bodies[1].at(-1).content, /not in the facts you were given: 7/);
    assert.equal(await gm.locator("[data-state-hash]").getAttribute("data-state-hash"), h0);
    await shot(gm, "a3-minerva");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: Minerva wrote for the table from its own facts; invented number sent back and never shown; no secret sent`);
  }
} finally {
  server.kill();
  await provider.close();
}
