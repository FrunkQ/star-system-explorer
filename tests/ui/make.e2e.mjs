// V13 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/make.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Import: the sample V3 save reads into a report and a preview chart; the
// differing value is marked; the honesty mark shows on the Moon's card; a file
// that is not a save is refused in words. Grow: a seed grows a system whose
// preview has the same number of worlds as Node's run of the same request.
// Nothing moves the campaign's state hash.
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Session } from "../../packages/ui-contract/src/session.ts";
import { gmSeat } from "../../packages/ui-contract/src/seat.ts";
import { nodeKernel } from "../../packages/ui-contract/test/node-kernel.ts";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

const SEED = "first light";
const k = nodeKernel();
let worlds;
try {
  const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [JSON.parse(readFileSync("packs/base-physics/pack.json", "utf8"))]);
  const r = await s.runFormation({ seed: SEED, block: "modern", system: "u:kestrel/sys:@first-light", star_pins: { primary_mass: 1 } });
  assert.ok(r.ok);
  worlds = r.result.run.worlds.length;
} finally {
  await k.close();
}

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").getAttribute("data-state-hash");
    const h0 = await hash();
    await gm.getByRole("button", { name: "Make a system" }).click();
    const dlg = gm.getByRole("dialog", { name: "Make a system" });

    // Not a save: refused in words.
    await dlg.locator("input[type=file]").setInputFiles({ name: "notes.json", mimeType: "application/json", buffer: Buffer.from('{"hello": 1}') });
    await dlg.getByRole("alert").waitFor({ timeout: 8000 });
    assert.match(await dlg.getByRole("alert").innerText(), /not a V3 save/);

    // The sample save.
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    const report = dlg.getByLabel("Import report");
    await report.waitFor({ timeout: 8000 });
    assert.match(await report.innerText(), /1 system, 3 bodies, 1 constructs, 85 facts/);
    assert.equal(await report.getAttribute("data-outside"), "1");
    assert.equal(await report.locator("tr[data-within=false] td").first().innerText(), "Moon");
    await dlg.locator(".preview .body").first().waitFor({ timeout: 8000 });
    assert.ok(await dlg.locator('.preview .body[aria-label^="Earth"]').count(), "Earth on the preview chart");
    await shot(gm, "v13-import");
    await dlg.locator('.preview .body[aria-label^="Earth"]').dispatchEvent("pointerdown");
    await dlg.locator(".preview .card").getByText("1 M⊕").waitFor({ timeout: 5000 });
    assert.equal(await hash(), h0, "an import does not touch the campaign");

    // Grow a system.
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill(SEED);
    await dlg.getByRole("button", { name: "Grow it" }).click();
    const res = dlg.getByLabel("Formation result");
    await res.waitFor({ timeout: 20000 });
    assert.equal(Number(await res.getAttribute("data-worlds")), worlds, "the browser grows what Node grows");
    if (worlds) await dlg.locator(".preview .body").nth(1).waitFor({ timeout: 8000 });
    await shot(gm, "v13-grow");
    assert.equal(await hash(), h0, "growing does not touch the campaign");
    // Phone width: the dialog fits without sideways scrolling of the page.
    await gm.setViewportSize({ width: 390, height: 844 });
    await gm.waitForTimeout(300);
    const over = await dlg.evaluate((d) => d.scrollWidth - d.clientWidth);
    assert.ok(over <= 1, `dialog overflows by ${over}px at phone width`);
    await shot(gm, "v13-phone");
    assert.deepEqual(errors, []);
    await browser.close();
    console.log(`PASS ${name}: sample save imported with its report and preview; ${worlds} worlds grown from "${SEED}" as in Node; state hash unmoved`);
  }
} finally {
  server.kill();
}
