// The class gallery's pre-drawn globes (V14i part 2): every class of every shipped atlas drawn once in the built shell,
// exactly as the gallery draws it live (a sample world from the kernel's class_sample, its True face, one turn as a
// strip), then written beside the base presentation pack as one `sse.class-pictures/1` set. The shipped atlases' sample
// worlds are fixed by their own examples, so the gallery shows these at once, on any seat and any device, for as long as
// the atlas's version matches; rerun this whenever an atlas, the kernel's surfaces or True's look changes.
// Usage: scripts/build-shell.sh first, then node scripts/build-class-pictures.mjs [--check].
// --check draws nothing: it reads the set, says what it holds, and fails when it was drawn with another look than the
// tree's (the gallery then draws live): run this script again to bring it up to date.
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { go } from "../tests/ui/nav.mjs";
import { lookFingerprint } from "./class-pictures-look.mjs";

const FILE = "packs/base-presentation/class-pictures.json";
const NOTE = "The class gallery's turning globes for the shipped atlases, drawn ahead of time by scripts/build-class-pictures.mjs from each class's sample world (kernel class_sample) in True, without clouds. Each sheet is one turn, frames side by side.";

if (process.argv.includes("--check")) {
  const set = JSON.parse(readFileSync(FILE, "utf8"));
  for (const a of set.atlases) {
    const ps = Object.values(a.pictures);
    console.log(`${a.pack} ${a.version}: ${ps.filter((p) => p.sheet).length} globes, ${ps.filter((p) => p.none).length} without (${a.frames} frames of ${a.size} px)`);
  }
  console.log(`${(readFileSync(FILE).length / 1048576).toFixed(1)} MB`);
  const now = lookFingerprint(process.cwd());
  if (set.look !== now) {
    console.log(`drawn with look ${set.look ?? "(none)"}, the tree's is ${now}: the gallery draws live until this script is run again`);
    process.exit(1);
  }
  console.log(`up to date (look ${now})`);
  process.exit(0);
}

const { chromium } = createRequire(import.meta.url)("playwright");
const port = 9500 + Math.floor(Math.random() * 90);
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  const browser = await chromium.launch();
  const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
  // Draw live, never from an earlier set.
  await page.addInitScript(() => localStorage.setItem("sse.class-pictures.live", "1"));
  page.on("console", (m) => /class picture/.test(m.text()) && console.log(m.text()));
  await page.goto(`http://localhost:${port}/#chart`);
  await page.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 30000 });
  await go(page, "reference");
  await page.locator('[data-page="classes"]').click();
  const classes = page.locator('[data-classes="offered"]');
  await classes.waitFor({ timeout: 30000 });
  const versions = await page.evaluate(async () => {
    const out = {};
    for (const u of ["packs/base-atlas.json", "packs/sse3-classic.json"]) {
      const r = await fetch(new URL(u, document.baseURI));
      if (r.ok) {
        const p = await r.json();
        out[p.id] = String(p.version ?? "");
      }
    }
    return out;
  });
  const atlases = [];
  const picks = await classes.locator("[data-atlas-pick]").evaluateAll((els) => els.map((e) => e.dataset.atlasPick));
  const only = process.argv.find((a) => a.startsWith("--atlas="))?.slice(8);
  for (const pack of picks.length ? picks : [await classes.getAttribute("data-atlas")]) {
    if (only && pack !== only) continue;
    if (picks.length) await classes.locator(`[data-atlas-pick="${pack}"]`).click();
    const version = versions[pack];
    if (version === undefined) throw new Error(`no version read for ${pack}`);
    const pictures = {};
    let size = 0;
    let frames = 0;
    for (const scheme of await classes.locator("[data-scheme]").evaluateAll((els) => els.map((e) => e.dataset.scheme))) {
      await classes.locator(`[data-scheme="${scheme}"]`).click();
      const sizer = classes.locator("[data-page-size]");
      if (await sizer.count()) await sizer.selectOption("48");
      for (;;) {
        const tiles = classes.locator(`[data-gallery="${scheme}"] .tile`);
        await tiles.first().waitFor();
        // Every tile on the page drawn, or said why not.
        await page.waitForFunction((s) => [...document.querySelectorAll(`[data-gallery="${s}"] .tile`)].every((t) => ["drawn", "none"].includes(t.getAttribute("data-picture-state") ?? "")), scheme, { timeout: 30 * 60000, polling: 1000 });
        const got = await tiles.evaluateAll(async (els) =>
          Promise.all(
            els.map(async (t) => {
              const id = t.dataset.class;
              const pic = t.querySelector("[data-tile-picture]");
              if (!pic) return { id, none: t.dataset.pictureWhy ?? "no picture" };
              const url = getComputedStyle(pic).backgroundImage.replace(/^url\("?|"?\)$/g, "");
              const blob = await (await fetch(url)).blob();
              const sheet = await new Promise((r) => {
                const f = new FileReader();
                f.onload = () => r(f.result);
                f.readAsDataURL(blob);
              });
              const img = await createImageBitmap(blob);
              return { id, sheet, caption: pic.dataset.caption ?? "", size: img.height, frames: Math.round(img.width / img.height) };
            }),
          ),
        );
        for (const g of got) {
          if (g.sheet) {
            pictures[g.id] = { sheet: g.sheet, caption: g.caption };
            size = g.size;
            frames = g.frames;
          } else pictures[g.id] = { none: g.none };
          console.log(`${pack} ${g.id}: ${g.sheet ? `${Math.round(g.sheet.length / 1024)} kB` : `none (${g.none})`}`);
        }
        const next = classes.locator("[data-page-next]");
        if (!(await next.count()) || (await next.isDisabled())) break;
        await next.click();
      }
    }
    atlases.push({ pack, version, size, frames, pictures });
  }
  // With --atlas, the other atlases are kept as they were (their look stamp is the set's, so redraw them all after a look change).
  if (only) {
    let was = [];
    try {
      was = JSON.parse(readFileSync(FILE, "utf8")).atlases ?? [];
    } catch {}
    atlases.unshift(...was.filter((a) => a.pack !== only));
  }
  writeFileSync(FILE, JSON.stringify({ schema: "sse.class-pictures/1", note: NOTE, look: lookFingerprint(process.cwd()), atlases }) + "\n");
  console.log(`wrote ${FILE}: ${(readFileSync(FILE).length / 1048576).toFixed(1)} MB`);
  await browser.close();
} finally {
  server.kill();
}
