// Copy and paste on the Build desk (top-level design 8.2, U20 part 1), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/clip.e2e.mjs [--screens <dir>]
// On Build, select Tarn and press Copy: the clipboard holds an sseClip 2 with Tarn and its moon. Pasting it shows
// where it would land, orbiting the star at its own distance. On Explore, a paste says to switch to Build. A V3
// clip is recognised; text that is not a clip is refused in words. Copying and reading a paste never change the
// state hash. "Paste here" places Tarn and its moon as one change: both join the chart, one Undo takes both away and
// Redo brings them back; the pasted copy can be cut, and Undo puts it back (U20 part 2).
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk, more } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};
// A paste as the browser delivers it, with the text on the event (no clipboard permission needed).
const paste = (page, text) =>
  page.evaluate((t) => {
    const dt = new DataTransfer();
    dt.setData("text/plain", t);
    document.querySelector(".stage")?.dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true }));
  }, text);

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const context = await browser.newContext({ viewport });
    await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: base });
    const gm = await context.newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const hash = () => gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    const h0 = await hash();

    // Explore has no Copy; Build does, on the tool bar (the desk) or in More (the phone), once a body is selected.
    const phone = vp === "phone";
    const copy = phone ? gm.locator("[data-more-copy]") : gm.getByRole("button", { name: "Copy", exact: true });
    assert.equal(await copy.count(), 0, "no Copy on Explore");
    await desk(gm, "Build");
    if (phone) await gm.getByRole("button", { name: "More", exact: true }).click();
    assert.ok(await copy.isDisabled(), "Copy waits for a selection");
    if (phone) await gm.getByRole("button", { name: "More", exact: true }).click();
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().locator(".hit").click();
    if (phone) await gm.getByRole("button", { name: "More", exact: true }).click();
    await copy.click();
    const note = gm.locator("[data-clip-note]");
    await note.waitFor();
    assert.match(await note.innerText(), /Copied Tarn, with a moon/);
    const text = await gm.evaluate(() => navigator.clipboard.readText());
    const clip = JSON.parse(text);
    assert.equal(clip.sseClip, 2);
    assert.equal(clip.generation, "v4");
    assert.deepEqual(clip.nodes.map((n) => n.kind), ["planet", "moon"]);
    assert.ok(clip.events.some((e) => e.node === "n0" && e.field === "orbit.semi_major_axis"));
    assert.ok(clip.packs.length >= 1, "names the packs it was made under");
    await shot(gm, `clip-copied-${vp}`);

    // Paste on Build: where it would land (the phone pastes from More).
    if (phone) await more(gm, /^Paste/);
    else await paste(gm, text);
    const dlg = gm.locator("dialog[data-paste]");
    await dlg.waitFor();
    assert.match(await dlg.locator("h2").innerText(), /Paste Tarn, with a moon/);
    assert.match(await dlg.innerText(), /orbiting Kestrel at 0\.52 AU, as copied/, "a planet goes round the star, not round the selected planet");
    assert.equal(await dlg.locator("[data-paste-missing]").count(), 0, "made under this campaign's own packs");
    await shot(gm, `clip-paste-${vp}`);
    await dlg.getByRole("button", { name: "Close" }).click();

    // A clip naming a pack this campaign lacks says so.
    await paste(gm, JSON.stringify({ ...clip, packs: [...clip.packs, { id: "traveller", version: "1.0.0" }] }));
    await dlg.waitFor();
    assert.match(await dlg.locator("[data-paste-missing]").innerText(), /traveller 1\.0\.0/);
    await dlg.getByRole("button", { name: "Close" }).click();

    // A V3 clip and plain text: refused in words.
    await paste(gm, JSON.stringify({ sseClip: 1, root: "a", nodes: [] }));
    assert.match(await note.innerText(), /Star System Explorer 3/);
    await paste(gm, "hello");
    assert.match(await note.innerText(), /does not hold/);

    assert.equal(await hash(), h0, "copying and reading a paste change nothing");

    // Paste here: the copy joins the chart as one change, at a new address.
    const copyOf = gm.locator('.area-chart .body[data-address="u:kestrel/sys:1/body:6"]');
    const present = (yes) => (yes ? copyOf.waitFor() : copyOf.waitFor({ state: "detached" }));
    await paste(gm, text);
    await dlg.waitFor();
    await dlg.locator("[data-paste-place]").click();
    await present(true);
    assert.match(await note.innerText(), /Pasted Tarn, with a moon, orbiting Kestrel/);
    assert.match((await copyOf.getAttribute("aria-label")) ?? "", /^Tarn, Added by hand/);
    const h1 = await hash();
    assert.notEqual(h1, h0, "a paste is history");
    await shot(gm, `clip-pasted-${vp}`);
    // One Undo takes it away; Redo brings it back.
    await gm.keyboard.press("Control+z");
    await present(false);
    await gm.keyboard.press("Control+Shift+z");
    await present(true);
    // The sample's own Tarn cannot be cut; the pasted one can, and Undo puts it back.
    await gm.locator('.area-chart .body[data-address="u:kestrel/sys:1/body:3"] .hit').click();
    if (phone) await gm.getByRole("button", { name: "More", exact: true }).click();
    assert.equal(await gm.locator("[data-more-cut]").count(), 0, "the sample's own body is not cut");
    if (phone) await gm.getByRole("button", { name: "More", exact: true }).click();
    await copyOf.locator(".hit").click();
    if (phone) await more(gm, /^Cut Tarn/);
    else await gm.getByRole("button", { name: "Cut", exact: true }).click();
    await present(false);
    assert.match(await note.innerText(), /Cut Tarn, with a moon/);
    await gm.keyboard.press("Control+z");
    await present(true);

    // On Explore a paste says where it works.
    await desk(gm, "Explore");
    await paste(gm, text);
    assert.match(await note.innerText(), /Build desk/);
    assert.equal(await dlg.evaluate((d) => d.open), false);

    // Phone: nothing runs off the side.
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
    }
    assert.deepEqual(errors, []);
    await browser.close();
  }
  console.log("clip e2e: ok");
} finally {
  server.kill();
}
