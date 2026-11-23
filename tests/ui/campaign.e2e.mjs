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
import { createHash } from "node:crypto";
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

    // The one Open door (V3 gap 28) names any file by what is inside it.
    await more(gm, "Campaign…");
    const od = gm.getByRole("dialog", { name: "Campaign" });
    const give = (name, body) => od.getByLabel("File", { exact: true }).setInputFiles({ name, mimeType: "application/octet-stream", buffer: Buffer.from(body) });
    await give("hello.json", JSON.stringify({ hello: "world" }));
    await od.locator("[data-opened='unknown']").getByText(/not one SSE4 recognises/).waitFor();
    const other = JSON.parse(readFileSync(path, "utf8"));
    await give("from-elsewhere.json", JSON.stringify({ ...other, root: "u:elsewhere", export: { ...other.export, root: "u:elsewhere" } }));
    assert.match(await od.locator("[data-offered-problem]").innerText(), /cannot be opened here: it belongs to u:elsewhere/);
    await give("world.ubox", "PK");
    await od.locator("[data-opened='other-app']").getByRole("link", { name: "Open the converter" }).waitFor();
    await give("p.json", JSON.stringify({ format: "sse.pack/1", id: "base-physics", version: "9.9.9" }));
    await od.locator("[data-opened='pack'] [data-pack-problem]").waitFor();
    // A paper's pack (ND6) says what it cites, what it fudges and whether it reproduces the paper, before it is added.
    await give("paper.json", readFileSync("packages/ui-contract/test/fixtures/paper.pack.json"));
    assert.match(await od.locator("[data-opened='pack'] [data-pack-report='test.paper']").innerText({ timeout: 20000 }), /2 sources, 2 fudges, 1 of 1 check reproduced, citing Planetary fact sheet and A made-up fit/);
    await give("starmap.json", readFileSync("packages/shell/public/samples/synthetic-sol.json"));
    await od.locator("[data-opened='v3-save']").getByText(/Star System Explorer 3 save/).waitFor();
    await od.getByRole("button", { name: "Bring it in" }).click();
    const make = gm.getByRole("dialog", { name: "Make a system" });
    await make.getByText(/1 system, 3 bodies/).waitFor({ timeout: 20000 });
    await make.getByRole("button", { name: "Close" }).click();

    // Open the downloaded file: the campaign is back.
    await more(gm, "Campaign…");
    await give("kestrel-save.json", readFileSync(path));
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

    // Opened from an Explorers item page (?explorers=<listing URL>): the bytes are checked against the listing first.
    const packBytes = readFileSync("packages/shell/src/documents/sample.warm-worlds.pack.json");
    const pack = JSON.parse(packBytes.toString());
    const sum = createHash("sha256").update(packBytes).digest("hex");
    const listing = { sha256: sum, source: { site: "explorers", title: "Warm worlds", url: "https://explorers.starsystemx.com/i/warm", creditChain: ["Ann", "Bo"] }, pack: { id: pack.id, version: pack.version } };
    await context.route("https://explorers.starsystemx.com/api/v4/**", (r) => {
      const u = r.request().url();
      if (u.includes("/api/v4/nothing")) return r.fulfill({ status: 404, body: "" });
      if (u.endsWith("/file")) return r.fulfill({ status: 200, body: packBytes, headers: { "x-sse-sha256": sum, "access-control-allow-origin": "*", "access-control-expose-headers": "x-sse-sha256" } });
      const l = u.includes("tampered") ? { ...listing, sha256: "0".repeat(64) } : listing;
      return r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(l), headers: { "access-control-allow-origin": "*" } });
    });
    const openWith = async (slug) => {
      await gm.goto(`${base}?explorers=${encodeURIComponent(`https://explorers.starsystemx.com/api/v4/${slug}`)}#chart`);
      await ready();
    };
    await openWith("nothing");
    await gm.locator("[data-explorers-note]").getByText(/no V4 shelf/).waitFor({ timeout: 20000 });
    assert.ok(!gm.url().includes("explorers="), "the link leaves the address bar");
    await openWith("tampered");
    await gm.locator("[data-explorers-note]").getByText(/sha256 differs/).waitFor({ timeout: 20000 });
    await openWith("warm-1");
    const ed = gm.getByRole("dialog", { name: "Campaign" });
    await ed.locator("[data-from-explorers]").getByText(/Warm worlds.*Made by Ann, then Bo/).waitFor({ timeout: 20000 });
    await ed.locator("[data-pack-check]").waitFor({ timeout: 20000 });
    await shot(gm, "u28-from-explorers");
    await ed.getByRole("button", { name: "Add it to this campaign" }).click();
    await ed.getByText(/^Added\./).waitFor({ timeout: 20000 });

    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`${name}: campaign kept, reopened, joined, downloaded, started fresh and opened from a file`);
  }
} finally {
  server.kill();
}
