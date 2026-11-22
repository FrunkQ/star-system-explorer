// The Reference screen, part 1 (matter-pack design section 8, MP10): Constants and Units from the kernel's data
// sheet, in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/reference.e2e.mjs [--screens <dir>]
// From the Workshop's rail: the Constants page shows the sheet's groups in order with what each is; the two pure
// numbers at the top; every theoretical row carries its stamp and its note at Curious, Science and Engine; a row
// opens its card with the source linked; Engine adds ids. Units list each against its SI unit (a parsec in metres,
// worked out from the sheet). Night chart keeps the marks; on a phone nothing runs off the side, rows wrap to two
// lines and the card rises from the foot. Reading writes nothing.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { depth, go, more } from "./nav.mjs";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9700 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/#chart`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
const shot = async (page, n) => {
  await page.waitForTimeout(400);
  if (screens) (mkdirSync(screens, { recursive: true }), await page.screenshot({ path: `${screens}/${n}.png`, fullPage: false }));
};

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base);
    await gm.waitForFunction(() => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash"), null, { timeout: 20000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    await go(gm, "reference");
    const ref = gm.locator('[data-reference="constants"]');
    await ref.waitFor({ timeout: 20000 });

    // The sheet's groups, in order, each with what it is.
    const groups = await ref.locator("[data-group]").evaluateAll((els) => els.map((e) => e.dataset.group));
    assert.deepEqual(groups.slice(0, 4), ["nature", "convention", "factor", "cosmos"]);
    assert.ok((await ref.locator('[data-group="cosmos"] .about').innerText()).length > 40);
    assert.match(await ref.locator("[data-dimensionless]").innerText(), /1 \/ 137\.036/);

    // Theoretical rows say so, with their note, at every depth.
    for (const d of ["Curious", "Science", "Engine"]) {
      await depth(gm, d);
      const marked = ref.locator('[data-mark="theoretical"]');
      const n = await marked.count();
      assert.ok(n >= 4, `${d}: ${n} theoretical rows`);
      assert.equal(await marked.locator(".stamp").count(), n, `${d}: every theoretical row is stamped`);
      assert.equal(await marked.locator("[data-mark-note]").count(), n, `${d}: every theoretical row has its note`);
      assert.match(await ref.locator('[data-row="w0"] [data-mark-note]').innerText(), /would change the picture/);
      assert.equal(await ref.locator(".id").count() > 0, d === "Engine", `${d}: ids only at Engine`);
    }
    await depth(gm, "Science");
    assert.match(await ref.locator('[data-row="c"] .how').innerText(), /exact/);
    assert.ok((await ref.locator("[data-group-notes] li").count()) >= 4, "cosmos notes");
    assert.match(await ref.locator("[data-group-notes]").innerText(), /Clowe 2006/);

    // A row opens its card, with the source linked; Escape closes it and gives focus back.
    await ref.locator('[data-row="G"] .row').click();
    const card = gm.locator('[data-card="G"]');
    await card.waitFor();
    await card.locator("[data-source]").first().waitFor({ timeout: 10000 });
    assert.match(await card.locator("[data-source] a").first().getAttribute("href"), /^https:\/\/doi\.org\/10\./);
    assert.match(await card.innerText(), /Measured, ±/);
    if (vp === "phone") {
      const box = await card.boundingBox();
      assert.ok(box && box.y + box.height >= viewport.height - 2, "the card is a sheet from the foot");
    }
    await shot(gm, `constants-card-${vp}`);
    await gm.keyboard.press("Escape");
    await card.waitFor({ state: "detached" });
    assert.equal(await gm.evaluate(() => document.activeElement?.closest("[data-row]")?.getAttribute("data-row") ?? document.activeElement?.outerHTML.slice(0, 120)), "G");

    // The theoretical card.
    await ref.locator('[data-row="Omega_c"] .row').click();
    await gm.locator('[data-card="Omega_c"] [data-card-mark-note]').waitFor();
    await ref.locator('[data-group="cosmos"]').scrollIntoViewIfNeeded();
    await shot(gm, `constants-cosmos-${vp}`);
    await gm.keyboard.press("Escape");

    // Units against their SI unit.
    await gm.locator('[data-page="units"]').click();
    const units = gm.locator('[data-reference="units"]');
    await units.waitFor();
    assert.match(await units.locator('[data-unit="pc"] .val').innerText(), /^1 pc = 3\.085 677 581 × 10¹⁶ m$/);
    assert.match(await units.locator('[data-unit="m"] .val').innerText(), /SI/);
    await shot(gm, `units-${vp}`);

    if (vp === "phone") {
      await gm.locator('[data-page="constants"]').click();
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `phone overflows by ${over} px`);
      const row = await gm.locator('[data-row="G"] .row').boundingBox();
      assert.ok(row && row.height >= 44, "phone rows are tall enough to tap and wrap to two lines");
      await shot(gm, `constants-${vp}`);
    } else {
      await gm.locator('[data-page="constants"]').click();
      await shot(gm, `constants-${vp}`);
      await more(gm, /Night chart/);
      const marked = gm.locator('[data-mark="theoretical"] .stamp').first();
      await marked.waitFor();
      await gm.locator('[data-group="cosmos"]').scrollIntoViewIfNeeded();
      await shot(gm, "constants-night-desk");
    }
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "reading the sheet writes nothing");
    assert.deepEqual(errors, []);
    await browser.close();
  }
  console.log("reference e2e: ok");
} finally {
  server.kill();
}
