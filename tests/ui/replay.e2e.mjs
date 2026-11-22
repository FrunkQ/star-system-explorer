// The formation replay in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/replay.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Grow "first light", press "Watch it form": the replay opens over the page at
// stage 1. Each stage chip jumps to its stage, with the disc in stage 3, embryos
// in stage 4, the violent phase in stage 6 with an event in words, and the
// run's worlds at the end. Play moves time forward; Escape closes it. Watching
// never changes the state hash, and nothing is written.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { more } from "./nav.mjs";
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
  await page.waitForTimeout(400); // let the dialog's opening motion finish
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const errors = [];
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    const h0 = await hash();
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByRole("textbox", { name: "Seed" }).fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    const worlds = Number(await dlg.getByLabel("Formation result").getAttribute("data-worlds", { timeout: 20000 }));
    await dlg.locator("[data-watch-form]").click();
    const rp = gm.locator(".replay");
    await rp.waitFor();
    assert.equal(await rp.getAttribute("data-replay-stage"), "1", "opens at the first stage");
    const stage = async (n) => {
      await rp.locator(".stages button").nth(n - 1).click();
      assert.equal(await rp.getAttribute("data-replay-stage"), String(n), `chip ${n} jumps to stage ${n}`);
    };
    await stage(3);
    assert.equal(await rp.locator("[data-disc]").count(), 1, "the disc in stage 3");
    assert.equal(await rp.locator("[data-line]").count(), 5, "with its five condensation lines");
    await stage(4);
    assert.ok((await rp.locator("[data-replay-body]").count()) >= 1, "embryos in stage 4");
    await shot(gm, `replay-growth-${name}`);
    await stage(6);
    await gm.keyboard.press("ArrowRight");
    await gm.keyboard.press("ArrowRight");
    assert.match(await rp.locator("[data-replay-event]").innerText(), /impact|grazes|flings|chain|rogue|merge/i, "the violent phase in words");
    assert.equal(await rp.locator("[data-disc]").count(), 0, "the gas has gone");
    await shot(gm, `replay-violent-${name}`);
    await stage(9);
    assert.equal(await rp.locator("[data-replay-body]").count(), worlds, "the run's worlds at the end");
    await shot(gm, `replay-end-${name}`);
    await stage(2);
    const t0 = Number(await rp.getAttribute("data-replay-t"));
    await rp.locator("[data-replay-play]").click();
    await gm.waitForTimeout(1200);
    assert.ok(Number(await rp.getAttribute("data-replay-t")) > t0, "play moves time forward");
    await rp.locator("[data-replay-play]").click();
    await gm.keyboard.press("Escape");
    await gm.waitForTimeout(200);
    assert.equal(await gm.locator(".replay").count(), 0, "Escape closes the replay");
    assert.equal(await hash(), h0, "watching writes nothing");
    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`replay e2e ${name}: ok (${worlds} worlds)`);
  }
} finally {
  server.kill();
}
