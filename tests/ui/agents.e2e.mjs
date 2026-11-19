// A1 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/agents.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// With no key every agent says it is not linked and the chart works as ever.
// Link a provider (a local mock with a key), check it, link Support, and ask
// Tars: the answer arrives with its cost. The key travels only to that
// provider and is stored only in its own entry, never in the settings; the state hash never moves.
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
const port = 8450 + Math.floor(Math.random() * 200);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const KEY = "sk-e2e-secret-4242";
const provider = await mockProvider({ key: KEY, priced: true, context: 16000 });
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const engine = { chromium, firefox, webkit }[name];
    const browser = await engine.launch();
    const errors = [];
    const gm = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    gm.on("pageerror", (e) => errors.push(e.message));
    const carried = [];
    gm.on("request", (r) => {
      const h = r.headers().authorization ?? "";
      if (h.includes(KEY) || (r.postData() ?? "").includes(KEY)) carried.push(r.url());
    });
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const h0 = await gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    await gm.locator(".area-chart [role=button]", { hasText: "Tarn" }).first().click();

    await gm.getByRole("button", { name: "Agents" }).click();
    const dlg = gm.getByRole("dialog", { name: "Agents" });
    await dlg.getByRole("tab", { name: "Tars" }).click();
    assert.ok(await dlg.locator("[data-agent-off=tars]").isVisible(), "no key: Tars says it is not linked");
    assert.ok(await dlg.getByRole("button", { name: "Ask" }).isDisabled());

    await dlg.getByRole("tab", { name: "Link models" }).click();
    await dlg.getByLabel("Provider kind").selectOption("other");
    await dlg.getByLabel("Provider name").fill("Test provider");
    await dlg.getByLabel("Provider address").fill(provider.base);
    await dlg.getByLabel("Provider key").fill(KEY);
    await dlg.getByRole("button", { name: "Add and check" }).click();
    await dlg.getByRole("button", { name: "2 models" }).waitFor();
    await dlg.getByLabel("Support provider").selectOption("other");
    assert.equal(await dlg.getByLabel("Support model").inputValue(), "mock/small");
    await shot(gm, "a1-link");

    await dlg.getByRole("tab", { name: "Tars" }).click();
    await dlg.getByLabel("Ask Tars").fill("What is Tarn like?");
    assert.match(await dlg.locator("[data-estimate]").innerText(), /Before sending: about \d+ tokens in/);
    await dlg.getByRole("button", { name: "Ask" }).click();
    await dlg.locator("[data-answer]").waitFor();
    assert.match(await dlg.locator("[data-answer]").innerText(), /Mock answer to: What is Tarn like\?/);
    assert.match(await dlg.getByRole("status").innerText(), /120 tokens in, 12 out, \$0\.0001/);
    // Tars was told about the selected body.
    const sent = JSON.parse(provider.seen.find((x) => x.path.endsWith("/chat/completions")).body);
    assert.match(sent.messages[0].content, /Selected: Tarn/);
    await shot(gm, "a1-tars");

    // The key went only to the provider, is stored only in its own entry, never in the settings, and the state hash did not move.
    assert.ok(carried.length >= 2 && carried.every((u) => u.startsWith(provider.base)), `key carried only to the provider: ${carried.join(", ")}`);
    const stored = await gm.evaluate((k) => Object.keys(localStorage).filter((x) => localStorage.getItem(x).includes(k)), KEY);
    assert.deepEqual(stored, ["sse4.agents.key.other"], "the key has its own entry; the settings never hold it");
    assert.equal(await gm.locator("[data-state-hash]").getAttribute("data-state-hash"), h0);
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: no key works; Tars answered through the provider; key only to ${provider.base} and only in its own entry`);
  }
} finally {
  server.kill();
  await provider.close();
}
