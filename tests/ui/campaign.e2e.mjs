// U10 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/campaign.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Grow a system and add it to the campaign: the campaign is kept in this
// browser. Reload: the same state hash, the grown system still on the chart,
// a note saying it was reopened, and a player's screen joining it agrees by
// hash. Save campaign downloads it; Start fresh opens Kestrel Reach as
// shipped; Open campaign… with the downloaded file brings it back. A file
// that is not a save is refused and changes nothing.
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import assert from "node:assert/strict";
import { go, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
    const gm = await context.newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    const ready = async () => gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const kept = () => gm.evaluate(() => JSON.parse(localStorage.getItem("sse4.campaign") ?? "null"));
    const keptWith = (h) => gm.waitForFunction((x) => JSON.parse(localStorage.getItem("sse4.campaign") ?? "null")?.state_hash === x, h, { timeout: 15000 });
    await gm.goto(base + "#chart");
    await ready();
    const h0 = await hash();
    await keptWith(h0);
    assert.match(await gm.locator("[data-kept]").innerText(), /Kept \d\d:\d\d/);

    // Grow a system and add it.
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    await dlg.getByLabel("Formation result").waitFor({ timeout: 20000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing")?.includes("sys:@first-light"), null, { timeout: 15000 });
    const h1 = await hash();
    assert.notEqual(h1, h0);
    await keptWith(h1);
    const save = await kept();
    assert.equal(save.schema, "sse.campaign-save/1");
    assert.equal(save.root, "u:kestrel");

    // Reload: the same campaign.
    await gm.reload();
    await ready();
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") === h, h1, { timeout: 20000 });
    assert.match(await gm.locator("[data-kept-note]").innerText(), /Reopened the campaign kept .*: \d+ events\./);
    assert.ok((await gm.locator("[aria-label='System on the chart'] option").allInnerTexts()).some((t) => /first light/i.test(t)), "the grown system is still on the chart");
    await shot(gm, "u10-reopened");

    // A player's screen joins the reopened campaign and agrees.
    const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
    await gm.getByRole("button", { name: "Share with players" }).click();
    (await popup)?.close();
    const link = await gm.getAttribute("[data-players-link]", "data-players-link");
    const p = await context.newPage();
    p.on("pageerror", (e) => errors.push(`player: ${e.message}`));
    await p.goto(link);
    await p.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
    assert.equal(await p.getAttribute("footer[data-state-hash]", "data-state-hash"), await hash(), "the player agrees by state hash");
    await p.close();

    // Save campaign: a file.
    await more(gm, "Campaign…");
    const cd = gm.getByRole("dialog", { name: "Campaign" });
    const dl = gm.waitForEvent("download");
    await cd.getByRole("button", { name: "Save campaign" }).click();
    const file = await dl;
    assert.match(file.suggestedFilename(), /^sse-campaign-kestrel-\d{4}-\d\d-\d\d\.json$/);
    const path = join(tmpdir(), `u10-${port}.json`);
    await file.saveAs(path);
    assert.equal(JSON.parse(readFileSync(path, "utf8")).state_hash, h1);

    // Start fresh: Kestrel Reach as shipped.
    await cd.getByRole("button", { name: "Start fresh…" }).click();
    await shot(gm, "u10-campaign-dialog");
    await Promise.all([gm.waitForEvent("load"), cd.getByRole("button", { name: "Forget it and start fresh" }).click()]);
    await ready();
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") === h, h0, { timeout: 20000 });
    assert.equal(await gm.locator("[aria-label='System on the chart']").count(), 0, "no grown system after starting fresh");
    assert.equal(await gm.locator("[data-kept-note]").count(), 0);

    // A file that is not a save is refused.
    await more(gm, "Campaign…");
    const od = gm.getByRole("dialog", { name: "Campaign" });
    const junk = join(tmpdir(), `u10-junk-${port}.json`);
    writeFileSync(junk, JSON.stringify({ hello: "world" }));
    await od.getByLabel("Campaign file").setInputFiles(junk);
    await od.locator("[data-offered-problem]").waitFor();
    assert.match(await od.locator("[data-offered-problem]").innerText(), /cannot be opened here: not a campaign save/);

    // Open the downloaded file: the campaign is back.
    await od.getByLabel("Campaign file").setInputFiles(path);
    await od.locator("[data-offered]").waitFor();
    assert.match(await od.locator("[data-offered]").innerText(), /Kestrel, kept .*: \d+ events/);
    await shot(gm, "u10-open-campaign");
    await Promise.all([gm.waitForEvent("load"), od.getByRole("button", { name: "Open it" }).click()]);
    await ready();
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") === h, h1, { timeout: 20000 });
    assert.ok((await gm.locator("[aria-label='System on the chart'] option").allInnerTexts()).some((t) => /first light/i.test(t)));

    // A damaged kept save is set aside, and a fresh campaign opens.
    await gm.evaluate(() => {
      const s = JSON.parse(localStorage.getItem("sse4.campaign"));
      s.export.head = "0".repeat(s.export.head.length);
      localStorage.setItem("sse4.campaign", JSON.stringify(s));
    });
    await gm.reload();
    await ready();
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") === h, h0, { timeout: 20000 });
    assert.match(await gm.locator("[data-kept-note]").innerText(), /could not be reopened .* set aside/);
    assert.ok(await gm.evaluate(() => localStorage.getItem("sse4.campaign.set-aside") !== null), "the damaged save is kept aside");

    // V3 gap 27: the note offers the set-aside save back. Trying again with a save still damaged sets it aside again.
    await Promise.all([gm.waitForEvent("load"), gm.locator("[data-recover-try]").click()]);
    await ready();
    await gm.locator("[data-kept-note]").getByText(/set aside, not lost/).waitFor({ timeout: 20000 });
    assert.ok(await gm.evaluate(() => localStorage.getItem("sse4.campaign.set-aside") !== null), "still kept aside");

    // An open that never finished (the tab closed part way) is set aside on the next visit, not tried again blind.
    await gm.evaluate(() => localStorage.setItem("sse4.campaign.opening", new Date().toISOString()));
    await gm.reload();
    await ready();
    assert.match(await gm.locator("[data-kept-note]").innerText(), /did not finish/);
    assert.equal(await gm.evaluate(() => localStorage.getItem("sse4.campaign.opening")), null);

    // Your data, in the Campaign dialog: space used, the set-aside save, a diagnostic file without keys, clear all.
    await gm.evaluate(() => localStorage.setItem("sse4.agents.key.test", "sk-not-a-real-key"));
    await more(gm, "Campaign…");
    const yd = gm.locator("[data-your-data]");
    await yd.locator("[data-used]").getByText(/This browser keeps .* for SSE4/).waitFor();
    await yd.locator("[data-set-aside]").waitFor();
    await shot(gm, "u27-your-data");
    const [diag] = await Promise.all([gm.waitForEvent("download"), yd.locator("[data-diagnostic]").click()]);
    const dpath = `${tmpdir()}/sse4-diag-${name}.json`;
    await diag.saveAs(dpath);
    const dtext = readFileSync(dpath, "utf8");
    const d = JSON.parse(dtext);
    assert.equal(d.kind, "sse4-diagnostic");
    assert.ok(d.kernel && d.state_hash, "says what ran");
    assert.ok(!dtext.includes("sk-not-a-real-key"), "no agent keys in the diagnostic file");
    await yd.getByRole("button", { name: "Clear all data…" }).click();
    await Promise.all([gm.waitForEvent("load"), yd.locator("[data-clear-all]").click()]);
    await ready();
    assert.equal(await gm.evaluate(() => localStorage.getItem("sse4.agents.key.test")), null, "keys forgotten");
    assert.equal(await gm.evaluate(() => localStorage.getItem("sse4.campaign.set-aside")), null, "set-aside forgotten");
    assert.equal(await gm.locator("[data-kept-note]").count(), 0);

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: campaign kept, reopened, joined, downloaded, started fresh and opened from a file`);
  }
} finally {
  server.kill();
}
