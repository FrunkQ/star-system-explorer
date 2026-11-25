// The base postcard set (PC6, SSE4-postcards-design.md section 7): made in the built shell exactly as a GM makes
// one (a camera on a view, Take photograph, Keep as postcard, Send, Save file), then written into the base
// presentation pack as `sse.postcards/1`. Each card travels as a clip with its origin, so it draws live where it is
// opened, and carries its stamp so a held screen can show it without drawing.
// Usage: scripts/build-shell.sh first, then node scripts/build-postcard-set.mjs [--check].
// --check rebuilds nothing: it reads the pack's set and says whether every card reads and has a stamp.
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { go, more } from "../tests/ui/nav.mjs";

const PACK = "packs/base-presentation/pack.json";
const SET = { schema: "sse.postcards/1", id: "sse.base", title: "From the base set" };

// The places, in order. Each picks a world on the system chart (a moon inside its planet's chart), turns its globe (key presses on the globe: arrows
// turn by 15 degrees, shift by 45) and gives the card its name and caption. Grow the list with the trailer thread
// (STORYBOARDS.md section H; SHOTLIST.md): views other than the globe join as their stills land (PC2).
const PLACES = [
  { world: "Earth", keys: ["Shift+ArrowRight", "Shift+ArrowRight", "ArrowDown"], name: "Earth (sample), the day side", caption: "The sample save's Earth in True colour: its oceans, ice and its own generated continents." },
  { world: "Earth", keys: ["ArrowUp", "ArrowUp", "ArrowUp", "ArrowUp"], name: "Earth (sample), over the pole", caption: "Looking down on the north polar cap, where the kernel's own ice line puts it." },
  { world: "Moon", inside: "Earth", keys: ["Shift+ArrowLeft", "ArrowDown"], name: "Moon (sample)", caption: "The sample Moon: airless, cratered ground under a low sun." },
];

const pack = JSON.parse(readFileSync(PACK, "utf8"));
if (process.argv.includes("--check")) {
  const sets = pack.provides?.other?.postcard_sets ?? [];
  const set = sets.find((s) => s.id === SET.id);
  const bad = !set ? ["no base set"] : set.cards.filter((c) => !/^data:image\//.test(c.stamp ?? "") || c.world?.how !== "clip" || !c.world.origin).map((c) => c.name);
  console.log(bad.length ? `base set: missing or incomplete: ${bad.join(", ")}` : `base set: ${set.cards.length} postcards, each with a stamp and its origin`);
  process.exit(bad.length ? 1 : 0);
}

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  const browser = await chromium.launch();
  // Each place on a browser of its own, the sample save imported afresh: a page holds only so many drawing contexts.
  async function freshPage(sample = true) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
    const page = await ctx.newPage();
    await page.goto(base + "#chart");
    await page.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 30000 });
    if (!sample) return { ctx, page };
    await more(page, "Make a system");
    const dlg = page.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    return { ctx, page };
  }

  const cards = [];
  for (const [i, place] of PLACES.entries()) {
    const { ctx, page } = await freshPage(place.sample !== false);
    await go(page, "system");
    // A moon is on its planet's own chart: open that first.
    if (place.inside) {
      const host = page.locator(`.area-chart .body[aria-label^="${place.inside}"]`).first();
      await host.waitFor({ timeout: 15000 });
      await host.dispatchEvent("dblclick");
    }
    const body = page.locator(`.area-chart .body[aria-label^="${place.world}"]`).first();
    await body.waitFor({ timeout: 15000 });
    // The chart redraws as it settles after a load, so the world is picked by its own pointer event, not a click.
    await body.dispatchEvent("pointerdown", { pointerType: "mouse", isPrimary: true, button: 0 });
    await go(page, "body");
    console.log(`place ${i + 1}: ${place.name}`);
    await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']):not([data-frames='0'])", { state: "attached", timeout: 60000 }).catch(async (e) => {
      if (process.env.SET_DEBUG) await page.screenshot({ path: process.env.SET_DEBUG });
      throw e;
    });
    // In True: a postcard is the world as it looks, not the chart.
    await page.getByRole("button", { name: "True", exact: true }).first().click();
    await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']):not([data-frames='0'])", { state: "attached", timeout: 60000 });
    const globe = page.locator("[data-body] [data-lon]").first();
    await globe.focus();
    for (const k of place.keys) {
      await page.keyboard.press(k);
      await page.waitForTimeout(150);
    }
    await page.locator("[data-take-photo]").evaluate((b) => b.click());
    await page.locator("[data-photo-note]").waitFor({ timeout: 90000 });
    await page.locator("[data-keep-postcard]").click();
    await go(page, "postcards");
    const tiles = page.locator('[data-postcard-row="campaign"] [data-postcard-tile]');
    await tiles.first().click();
    await page.locator("[data-postcard-drawn]").waitFor({ state: "attached", timeout: 90000 });
    await page.locator("[data-postcard-turn]").click();
    await page.locator("[data-postcard-back]").getByRole("button", { name: "Send" }).click();
    const box = page.locator("[data-postcard-sendbox]");
    await box.locator("[data-postcard-whatgoes]").waitFor({ timeout: 30000 });
    await box.locator("[data-postcard-message-input]").fill("");
    const [download] = await Promise.all([page.waitForEvent("download"), box.locator("[data-postcard-file]").click()]);
    const card = JSON.parse(readFileSync(await download.path(), "utf8"));
    delete card.message;
    card.id = `sse.base.${i + 1}`;
    card.name = place.name;
    card.caption = place.caption;
    card.credit = ["SSE4"];
    if (!/^data:image\//.test(card.stamp ?? "")) throw new Error(`${place.name}: no stamp came with it`);
    if (!card.world?.origin) throw new Error(`${place.name}: no origin came with it`);
    cards.push(card);
    await ctx.close();
    console.log(`made ${card.name} (${Math.round(JSON.stringify(card).length / 1024)} KB)`);
  }
  await browser.close();
  const other = (pack.provides.other ??= {});
  other.postcard_sets = [...(other.postcard_sets ?? []).filter((s) => s.id !== SET.id), { ...SET, cards }];
  writeFileSync(PACK, `${JSON.stringify(pack, null, 1)}\n`);
  console.log(`wrote ${cards.length} postcards to ${PACK}`);
} finally {
  server.kill();
}
