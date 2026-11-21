// V4 acceptance in a real browser, against the built shell
// (scripts/build-shell.sh first). Usage:
//   node tests/ui/p2p.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The GM shares the session; a desktop window and a phone-sized window join
// the room, each running its own kernel; after the GM moves Tarn's orbit, all
// three agree by state hash; when the GM's page goes, the players show the
// read-only banner on the last state.
// (Scanned, data-view grants, version mismatch and snapshot repair are proved in ui-contract/test/p2p.test.ts.)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { desk as chooseDesk } from "./nav.mjs";
import { createRequire } from "node:module";

const { chromium, firefox, webkit } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
// --relay <url>: run against the Worker (e.g. `wrangler dev`) and join players through its relay.
const ri = args.indexOf("--relay");
const relay = ri >= 0 ? args.splice(ri, 2)[1] : null;
const engines = args.length ? args : ["chromium"];
const port = 8900 + Math.floor(Math.random() * 300);
const base = relay ?? `http://localhost:${port}/`;
const server = relay ? { kill() {} } : spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png` }));
};

try {
  for (const name of engines) {
    const browser = await { chromium, firefox, webkit }[name].launch();
    const context = await browser.newContext();
    const errors = [];
    const gm = await context.newPage({ viewport: { width: 1440, height: 1000 } });
    gm.setViewportSize({ width: 1440, height: 1000 });
    gm.on("pageerror", (e) => errors.push(`gm: ${e.message}`));
    await gm.goto(base);
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])");
    await chooseDesk(gm, "Build");
    const gmHash = () => gm.getAttribute("[data-state-hash]", "data-state-hash");

    // V10: a session that is not shared shows no seat UI at all.
    assert.equal(await gm.getByRole("button", { name: /seats/i }).count(), 0, "no seat UI before sharing");

    // Share: the players' link names a room.
    const popup = context.waitForEvent("page", { timeout: 3000 }).catch(() => null);
    await gm.getByRole("button", { name: "Share with players" }).click();
    (await popup)?.close();
    const link = await gm.getAttribute("[data-players-link]", "data-players-link");
    assert.match(link, /players\.html\?join=[a-z0-9]+$/);

    const join = async (label, viewport) => {
      const p = await context.newPage();
      await p.setViewportSize(viewport);
      p.on("pageerror", (e) => errors.push(`${label}: ${e.message}`));
      await p.goto(relay ? `${link}&via=relay` : link);
      await p.waitForSelector("footer[data-link='live'][data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
      return p;
    };
    const desk = await join("desk", { width: 1280, height: 900 });
    if (relay) assert.equal(await desk.getAttribute("footer[data-via]", "data-via"), "relay", "joined through the site's relay");
    const phone = await join("phone", { width: 390, height: 844 });
    const playerHash = (p) => p.getAttribute("footer[data-state-hash]", "data-state-hash");
    assert.equal(await playerHash(desk), await gmHash(), "desk agrees with the GM on joining");
    assert.equal(await playerHash(phone), await gmHash(), "phone agrees with the GM on joining");
    await gm.waitForSelector("[data-players='2']");
    assert.equal(await phone.getByRole("group", { name: "Sensor data" }).count(), 0, "a joined screen has no stand-in Scanned control");

    // The GM moves Tarn's orbit (keyboard) and the players follow.
    const before = await gmHash();
    await gm.locator("[role=button][aria-label^='Tarn']").focus();
    await gm.keyboard.press("PageUp");
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, before);
    const after = await gmHash();
    for (const p of [desk, phone]) await p.waitForFunction((h) => document.querySelector("footer[data-state-hash]")?.getAttribute("data-state-hash") === h, after, { timeout: 15000 });
    await shot(phone, "p2p-phone-live");
    await shot(desk, "p2p-desk-live");

    // V5: the desk asks for a name; nothing changes until the GM approves; then everyone follows.
    await desk.getByLabel("Position not known").getByRole("button", { name: "Tarn" }).click();
    await desk.getByLabel(/Suggest a name for Tarn/).fill("Haven");
    await desk.getByRole("button", { name: "Ask the game master" }).click();
    await desk.waitForSelector("[data-own-request='waiting']");
    const req = gm.getByRole("group", { name: "Players' requests" });
    await req.waitFor();
    assert.match(await req.innerText(), /name of Tarn → "Haven"/);
    assert.equal(await gmHash(), after, "a request alone changes nothing");
    await shot(gm, "p5-gm-request");
    await req.getByRole("button", { name: "Approve" }).click();
    await desk.waitForSelector("[data-own-request='approved']", { timeout: 15000 });
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, after);
    const named = await gmHash();
    let discovered = "";
    let orun = null;
    for (const p of [desk, phone]) await p.waitForFunction((h) => document.querySelector("footer[data-state-hash]")?.getAttribute("data-state-hash") === h, named, { timeout: 15000 });

    // V10: the GM adds a fog-of-war seat; its windows see only what it has discovered.
    await gm.getByRole("button", { name: "More seats" }).click();
    const panel = gm.getByRole("dialog", { name: "Seats" });
    assert.match(await panel.locator("[data-seats='1']").innerText(), /Everyone sees: Long-range sensors/);
    await panel.getByLabel("Name", { exact: true }).fill("Empire: Vesh");
    await panel.getByRole("button", { name: "Add seat" }).click();
    await panel.locator("[data-seats='2']").waitFor();
    // Adding a fog-of-war seat chronicles its starting view (one event); the table's scene does not change.
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, named);
    const seated = await gmHash();
    const veshLink = await panel.locator("[data-seat-link='empire-vesh']").getAttribute("href");
    assert.match(veshLink, /join=[a-z0-9]+&seat=empire-vesh$/);
    const veshWindow = async (label) => {
      const p = await context.newPage();
      await p.setViewportSize({ width: 1280, height: 900 });
      p.on("pageerror", (e) => errors.push(`${label}: ${e.message}`));
      await p.goto(relay ? `${veshLink}&via=relay` : veshLink);
      await p.waitForSelector("footer[data-link='live'][data-audience='empire-vesh']", { timeout: 30000 });
      return p;
    };
    const vesh1 = await veshWindow("vesh-1");
    const vesh2 = await veshWindow("vesh-2");
    const shows = (p) => p.locator("main").innerText();
    for (const p of [vesh1, vesh2]) assert.doesNotMatch(await shows(p), /Haven|Vault/, "Vesh starts knowing nothing");
    assert.match(await shows(desk), /Vault/, "the table still sees what everyone has been shown");
    await panel.getByLabel(/Tarn known to Empire: Vesh/).check();
    await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, seated);
    discovered = await gmHash();
    for (const p of [vesh1, vesh2, desk, phone]) await p.waitForFunction((h) => document.querySelector("footer[data-state-hash]")?.getAttribute("data-state-hash") === h, discovered, { timeout: 15000 });
    for (const p of [vesh1, vesh2]) {
      await p.waitForFunction(() => /Haven/.test(document.querySelector("main")?.innerText ?? ""), null, { timeout: 15000 });
      assert.doesNotMatch(await shows(p), /Vault/, "only Vesh's own discovery");
    }
    await shot(vesh1, "v10-vesh-window");

    // V10 part 2: a sealed seat runs no kernel and fetches no seed, packs or recorded facts; the GM sends finished scenes.
    await panel.getByLabel("Name", { exact: true }).fill("Empire: Orun");
    await panel.getByLabel(/Sealed: its screens/).check();
    await panel.getByRole("button", { name: "Add seat" }).click();
    await panel.locator("[data-seats='3']").waitFor();
    const orunLink = await panel.locator("[data-seat-link='empire-orun']").getAttribute("href");
    assert.match(orunLink, /players\.html\?sealed=[a-z0-9]+&seat=empire-orun$/);
    assert.ok(!orunLink.includes(new URL(link).searchParams.get("join")), "a sealed seat's link does not name the shared room");
    orun = await context.newPage();
    await orun.setViewportSize({ width: 1280, height: 900 });
    orun.on("pageerror", (e) => errors.push(`orun: ${e.message}`));
    const fetched = [];
    orun.on("request", (r) => fetched.push(r.url()));
    await orun.goto(relay ? `${orunLink}&via=relay` : orunLink);
    await orun.waitForSelector("footer[data-link='live'][data-sealed][data-audience='empire-orun']", { timeout: 30000 });
    assert.doesNotMatch(await shows(orun), /Haven|Vault/, "Orun starts knowing nothing");
    const orunScene0 = await orun.getAttribute("footer", "data-scene-hash");
    await panel.getByLabel(/Vault known to Empire: Orun/).check();
    await orun.waitForFunction((h) => document.querySelector("footer")?.getAttribute("data-scene-hash") !== h, orunScene0, { timeout: 15000 });
    await orun.waitForFunction(() => /Vault/.test(document.querySelector("main")?.innerText ?? ""), null, { timeout: 15000 });
    assert.doesNotMatch(await shows(orun), /Haven/, "only Orun's own discovery");
    const leaks = fetched.filter((u) => /\.wasm|player-facts|\/packs\/|kernel/.test(u));
    assert.deepEqual(leaks, [], "a sealed screen fetches no kernel, packs or recorded facts");
    await shot(orun, "v10-sealed-window");
    await shot(gm, "v10-gm-seats");
    await panel.getByRole("button", { name: "Close" }).click();

    // Phone width: no sideways scroll.
    const pw = await phone.evaluate(() => document.scrollingElement.scrollWidth);
    assert.ok(pw <= 390, `phone scrolls sideways (${pw}px)`);

    // The GM leaves: the players go read-only with a banner, on the last state.
    const last = await gmHash();
    for (const p of [desk, phone]) await p.waitForFunction((h) => document.querySelector("footer[data-state-hash]")?.getAttribute("data-state-hash") === h, last, { timeout: 15000 });
    const orunLast = await orun.getAttribute("footer", "data-scene-hash");
    await gm.close({ runBeforeUnload: true });
    await orun.waitForSelector("[role=status][aria-label='Game master'][data-link='offline']", { timeout: 15000 });
    assert.equal(await orun.getAttribute("footer", "data-scene-hash"), orunLast, "the sealed seat keeps its last scene");
    for (const p of [desk, phone]) {
      await p.waitForSelector("[role=status][aria-label='Game master'][data-link='offline']", { timeout: 15000 });
      assert.equal(await playerHash(p), last, "the last state stays on screen");
    }
    assert.match(await phone.getByRole("status", { name: "Game master" }).innerText(), /away[\s\S]*read only/i);
    await shot(phone, "p2p-phone-offline");
    assert.deepEqual(errors, []);
    assert.equal(await desk.getByRole("button", { name: "Ask the game master" }).isDisabled(), true, "no requests while the GM is away");
    console.log(`V4/V5 ${name}${relay ? " (relay)" : ""}: desk and phone joined; GM edit replayed (state ${after.slice(0, 10)}); request approved (state ${named.slice(0, 10)}); Vesh seat in two windows sees only its discovery (state ${discovered.slice(0, 10)}); sealed Orun seat shown only its discovery with no kernel; GM drop shows read-only`);
    await browser.close();
  }
  console.log("PASS");
} finally {
  server.kill();
}
