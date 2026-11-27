// The Formation screen (F10, the reading half) in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/formation.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// Grow "first light" and add it to the campaign, then open "How it formed" for it: the chart draws a track for every
// body, the condensation lines, the gas disc and the upheaval; the Inherited panel has its rows. A stage button moves
// the time slice and the time ruler with it; the slice's handle takes the arrow keys. Pressing a world's track selects
// it, and its story shows (where its material came from, its events, how sure); a lost body has a story too. "Watch
// it form" plays the replay over the page. The star's end (F10c) is worked out after the chart is up and is a press
// away, past now, without moving the campaign's date. "How this was worked out" says what each part rests on. Kestrel Reach (written fact by fact) says it has no formation history and how to get one. Reading the history
// never changes the state hash.
import "./chart-first.mjs";
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
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(300);
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

    // Kestrel Reach was written fact by fact: no history, and the screen says how to get one.
    await gm.evaluate(() => (location.hash = location.hash.replace(/~.*$/, "") + "~formation"));
    const fs = gm.locator(".formation");
    await gm.waitForSelector(".formation[data-formation='none']", { timeout: 15000 });
    assert.match(await fs.innerText(), /No formation history/);
    assert.match(await fs.innerText(), /Grow a new system/);

    // Grow first light and add it.
    await gm.evaluate(() => (location.hash = "#chart"));
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Grow a new system" }).click();
    await dlg.getByLabel("Seed").fill("first light");
    await dlg.getByRole("button", { name: "Grow it" }).click();
    const worlds = Number(await dlg.getByLabel("Formation result").getAttribute("data-worlds", { timeout: 20000 }));
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.waitForFunction(() => document.querySelector("[aria-label='System on the chart']")?.getAttribute("data-viewing")?.includes("sys:@first-light"), null, { timeout: 15000 });
    const h0 = await hash();

    // How it formed, for the system on the chart.
    await gm.evaluate(() => (location.hash = location.hash.replace(/~.*$/, "") + "~formation"));
    await gm.waitForSelector(".formation[data-formation='run']", { timeout: 20000 });
    const tracks = gm.locator("[data-track]");
    assert.equal(await gm.locator("[data-track-end='lasts']").count(), worlds, "a lasting track for every world");
    assert.ok((await tracks.count()) > worlds, "and tracks for the bodies lost on the way");
    assert.equal(await gm.locator("[data-line]").count(), 5, "five condensation lines");
    assert.equal(await gm.locator("[data-gas]").count(), 1, "the gas disc");
    assert.equal(await gm.locator("[data-upheaval]").count(), 1, "the upheaval span");
    assert.ok((await gm.locator("[data-event]").count()) > 0, "events as diamonds");
    for (const key of ["star", "metals", "c_o", "disc", "gas", "cluster", "al26", "seed"]) assert.equal(await gm.locator(`[data-inherited='${key}']`).count(), 1, `inherited ${key}`);
    assert.match(await gm.locator("[data-inherited='seed']").innerText(), /first light/);
    assert.match(await gm.locator("[data-slice-words]").innerText(), /stage 9/, "the slice starts at now, in stage 9");
    // F10b: the story card opens on the heaviest world, with its material bar and its story; How this was worked out.
    const story = gm.locator("[data-story]");
    await story.waitFor();
    assert.equal(await gm.locator("[data-material] span").count() > 0, true, "a material bar");
    assert.ok((await gm.locator("[data-story-line]").count()) >= 2, "its story in lines");
    assert.match(await story.innerText(), /now at/);
    await gm.locator("[data-how-panel] summary").click();
    assert.equal(await gm.locator("[data-how]:visible").count(), 5, "how this was worked out, a press away, five rows");
    assert.equal(await gm.locator("[data-how='not yet']").count(), 1, "passing stars are honestly not yet");
    // F10c: the star's end, worked out after the chart is up: a close-up of the end, or words for a star that outlives it.
    await gm.waitForSelector("[data-fate], [data-fate-note]", { timeout: 90000 });
    const ends = await gm.locator("[data-fate]").count();
    if (ends) {
      assert.ok((await gm.locator("[data-fate-star]").count()) === 1, "the star swelling");
      assert.match(await gm.locator("[data-fate-words]").innerText(), /white dwarf|neutron star|black hole/);
      assert.equal(await gm.locator("[data-star-end]").count(), 1, "the end marked on the main chart");
      await gm.locator("[data-fate]").scrollIntoViewIfNeeded();
      await shot(gm, `formation-end-${name}`);
      await gm.locator(".stages").scrollIntoViewIfNeeded();
      // The star's end is a press away; the slice goes past now but the campaign's date stays.
      const ruler = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
      await gm.locator("[data-stage='end']").click();
      await gm.waitForFunction(() => document.querySelector("[data-stage='end']")?.getAttribute("aria-current") === "step");
      assert.match(await gm.locator("[data-slice-words]").innerText(), /looking ahead/);
      assert.equal(await hash(), ruler, "looking at the end writes nothing");
    } else assert.match(await gm.locator("[data-fate-note]").innerText(), /trillion years/);
    await shot(gm, `formation-now-${name}`);
    // Watch it form: the replay of the same record, over the page; Escape closes it.
    await gm.locator("[data-watch-form]").click();
    await gm.locator(".replay").waitFor();
    assert.equal(await gm.locator(".replay").getAttribute("data-replay-stage"), "1");
    await gm.keyboard.press("Escape");
    await gm.locator(".replay").waitFor({ state: "detached" });

    // A stage moves the slice and the ruler with it.
    const sliceAt = async () => Number(await gm.locator("[data-slice]").getAttribute("data-slice"));
    const before = await sliceAt();
    await gm.locator("[data-stage='6']").click();
    await gm.waitForFunction(() => document.querySelector("[data-stage='6']")?.getAttribute("aria-current") === "step");
    const at6 = await sliceAt();
    assert.ok(at6 < before / 10, `stage 6 moves the slice up (${before} to ${at6})`);
    assert.match(await gm.locator("[data-stage-text]").innerText(), /gas gone/i);
    // Moving the slice (and so the ruler) reads nothing again: the history is stored.
    await gm.waitForTimeout(800);
    assert.equal(await gm.locator(".formation").getAttribute("data-formation"), "run", "the chart stays up while the date moves");
    await shot(gm, `formation-violent-${name}`);
    // The handle takes the keys: down moves later.
    await gm.locator("[data-slice-handle]").focus();
    await gm.keyboard.press("ArrowDown");
    await gm.waitForFunction((t) => Number(document.querySelector("[data-slice]")?.getAttribute("data-slice")) > t * 1.2, at6);
    // A lasting world's track is a button: Enter on it selects the world and its story shows.
    const firstWorld = await gm.locator("[data-track-end='lasts']").first().getAttribute("data-track");
    await gm.locator("[data-track-end='lasts'] .hit").first().focus();
    await gm.keyboard.press("Enter");
    await gm.waitForSelector(`[data-track='${firstWorld}'].selected`, { timeout: 5000 });
    assert.equal(await gm.locator("[data-story]").getAttribute("data-story"), firstWorld, "the pressed world's story");
    // A body lost on the way has a story too.
    const lost = await gm.locator("[data-track-end='merged']").first().getAttribute("data-track");
    await gm.locator(`[data-track='${lost}'] .hit`).dispatchEvent("click");
    await gm.waitForFunction((id) => document.querySelector("[data-story]")?.getAttribute("data-story") === id, lost);
    assert.match(await gm.locator("[data-story]").innerText(), /merge/i);
    await shot(gm, `formation-story-${name}`);

    // Phone width: the chart first, then the panel; nothing spills sideways.
    await gm.setViewportSize({ width: 390, height: 844 });
    await gm.waitForTimeout(300);
    const spill = await gm.evaluate(() => document.querySelector(".formation").scrollWidth - document.querySelector(".formation").clientWidth);
    assert.ok(spill <= 1, `no sideways scroll on a phone (${spill})`);
    await shot(gm, `formation-phone-${name}`);

    assert.equal(await hash(), h0, "reading the history writes nothing");
    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`formation e2e ${name}: ok (${worlds} worlds, ${await tracks.count().catch(() => "?")} tracks)`);
  }
} finally {
  server.kill();
}
