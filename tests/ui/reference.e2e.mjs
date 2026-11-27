// The Reference screen, part 1 (matter-pack design section 8, MP10): Constants and Units from the kernel's data
// sheet, in a real browser against the built shell (scripts/build-shell.sh first).
// Usage: node tests/ui/reference.e2e.mjs [--screens <dir>]
// From the Workshop's rail: the Constants page shows the sheet's groups in order with what each is; the two pure
// numbers at the top; every theoretical row carries its stamp and its note at Curious, Science and Engine; a row
// opens its card with the source linked; Engine adds ids. Units list each against its SI unit (a parsec in metres,
// worked out from the sheet). Night chart keeps the marks; on a phone nothing runs off the side, rows wrap to two
// lines and the card rises from the foot. Names and Classes (the class gallery) read the shipped packs. Reading writes
// nothing.
import "./chart-first.mjs";
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
    await ref.locator("[data-group-notes] a[href^='https://doi.org/']").first().waitFor({ timeout: 10000 });

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
    // How figures are said (ED1a): the units engine's table, the default reading marked.
    const orbitRow = units.locator('[data-figures-said] [data-said="length|*.orbit"]');
    assert.match(await orbitRow.innerText(), /AU/);
    assert.match(await orbitRow.locator(".chosen").innerText(), /AU/, "the first reading is the default");
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
    // Elements: the table by origin, one legend, a card with the story at each depth.
    await gm.locator('[data-page="elements"]').click();
    const els = gm.locator('[data-elements="ready"]');
    await els.waitFor({ timeout: 20000 });
    assert.equal(await els.locator(".table [data-el]").count(), 118);
    assert.equal(await els.locator("[data-origin-legend] li").count(), 7);
    const inks = await els.locator("[data-origin-legend] .sw").evaluateAll((s) => new Set(s.map((x) => getComputedStyle(x).backgroundColor)).size);
    assert.equal(inks, 7, "seven origin colours, all different");
    const goldBar = await els.locator('[data-el="au"] .bar').evaluate((b) => getComputedStyle(b).backgroundImage);
    assert.match(goldBar, /linear-gradient/);
    await shot(gm, `elements-${vp}`);
    await depth(gm, "Curious");
    await els.locator('.table [data-el="au"]').click({ force: vp === "phone" });
    const au = gm.locator('[data-element-card="au"]');
    await au.waitFor();
    assert.match(await au.locator("[data-shares]").innerText(), /Merging neutron stars, about 94 parts in 100/);
    assert.match(await au.locator("[data-story]").innerText(), /^Almost all gold was made in about a second/);
    assert.match(await au.locator("[data-argued]").innerText(), /How sure is this\?.*argued/s);
    assert.equal(await au.locator(".facts").count(), 0, "no figures at Curious");
    await depth(gm, "Science");
    await gm.locator('[data-element-card="au"] .facts').waitFor();
    assert.match(await au.locator("[data-story]").innerText(), /r-process/);
    assert.match(await au.innerText(), /Atomic weight\s+196\.966 57/);
    await au.locator("[data-source]").first().waitFor({ timeout: 10000 });
    if (vp === "phone") {
      assert.ok((await els.locator("[data-strip]").count()) === 7, "a strip of seven full cells");
      assert.ok(await els.locator('[data-strip="au"]').isVisible());
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `elements overflow by ${over} px`);
    }
    await shot(gm, `elements-gold-${vp}`);
    await gm.keyboard.press("Escape");
    await au.waitFor({ state: "detached" });
    if (vp === "desk") {
      await els.locator('.table [data-el="fe"]').click();
      await gm.locator('[data-element-card="fe"]').waitFor();
      await shot(gm, "elements-iron-night-desk");
      await gm.keyboard.press("Escape");
    }
    // Names (NM6), under "Names and classes": no pack carries names yet, so the base names pack is shown from a trial.
    await gm.locator('[data-page="names"]').click();
    const names = gm.locator('[data-names="offered"]');
    await names.waitFor({ timeout: 20000 });
    assert.match(await names.locator("[data-names-defaults]").innerText(), /Designation\s+Catalogue/);
    await names.locator('[data-scheme="vesh"]').click();
    const drawn = names.locator('[data-sample="vesh"] [aria-label="Sample names"]');
    await drawn.locator("li").nth(11).waitFor({ timeout: 20000 });
    const first = await drawn.getAttribute("data-digest");
    assert.match(await drawn.locator("li").first().innerText(), /say [A-Z]/);
    await names.locator("[data-hints]").waitFor();
    await names.locator("[data-names-more]").click();
    await gm.waitForFunction((d) => document.querySelector('[data-sample="vesh"] [aria-label="Sample names"]')?.getAttribute("data-digest") !== d, first);
    await names.locator('[data-scheme="english"]').click();
    await names.locator('[data-sample="english"] li .means').first().waitFor({ timeout: 20000 });
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `names overflow by ${over} px`);
    }
    await shot(gm, `names-${vp}`);
    // Classes (the class gallery), under "Names and classes": no pack carries classes, so the shipped atlases are shown.
    await gm.locator('[data-page="classes"]').click();
    const classes = gm.locator('[data-classes="offered"]');
    await classes.waitFor({ timeout: 20000 });
    assert.equal(await classes.getAttribute("data-atlas"), "sse.base-atlas", "SSE4's own atlas first");
    assert.deepEqual(await classes.locator("[data-scheme]").evaluateAll((els) => els.map((e) => e.dataset.scheme)), ["physical", "surface", "air", "habitability", "star"]);
    const tiles = classes.locator('[data-gallery="physical"] .tile');
    // V14i part 2: the gallery a page at a time, 12 by default, then 24 or 48 (Alex, 24 Nov).
    await tiles.nth(11).waitFor({ timeout: 20000 });
    assert.equal(await tiles.count(), 12, "a page of 12");
    assert.match(await classes.locator("[data-page-shown]").innerText(), /^1 to 12 of \d+$/);
    await classes.locator("[data-page-next]").click();
    assert.match(await classes.locator("[data-page-shown]").innerText(), /^13 to /);
    await classes.locator("[data-page-size]").selectOption("48");
    await tiles.nth(14).waitFor({ timeout: 20000 });
    assert.ok((await tiles.count()) >= 15, "the physical scheme's base classes are tiles");
    // Each tile wears its turning globe, or its mark until one is drawn (or where none can be: a small body, a giant).
    assert.equal((await tiles.locator("svg.icon").count()) + (await tiles.locator("[data-tile-picture]").count()), await tiles.count(), "every tile has its mark or its globe");
    // SSE4's atlas classes have marks of their own (scripts/add-atlas-marks.py): no two physical tiles draw alike.
    const looks = await tiles.locator("svg.icon").evaluateAll((els) => els.map((e) => e.innerHTML));
    assert.equal(new Set(looks).size, looks.length, "every physical class draws its own mark");
    await classes.locator('[data-class="gas-giant/ammonia-clouds"]').click();
    const ccard = gm.locator('[data-class-card="gas-giant/ammonia-clouds"]');
    await ccard.waitFor();
    assert.match(await ccard.locator("[data-bands]").innerText(), /equilibrium temperature up to 150 K/);
    assert.match(await ccard.locator("[data-examples]").innerText(), /Jupiter/);
    assert.match(await ccard.innerText(), /A kind of gas giant/);
    await shot(gm, `classes-${vp}`);
    await gm.keyboard.press("Escape");
    await ccard.waitFor({ state: "detached" });
    // V14i: a class opened is drawn from a sample world the kernel makes for it on a what-if branch, in True; its tile
    // then wears the picture. Nothing reaches the campaign (the state hash is checked at the end).
    if (vp === "desk") {
      await classes.locator('[data-class="iron-rich"]').click();
      const pic = gm.locator('[data-class-card="iron-rich"] [data-picture]');
      await gm.waitForFunction(() => ["drawn", "none"].includes(document.querySelector('[data-class-card="iron-rich"] [data-picture]')?.getAttribute("data-picture") ?? ""), null, { timeout: 60000 });
      assert.equal(await pic.getAttribute("data-picture"), "drawn", await pic.innerText());
      assert.match(await pic.innerText(), /after Mercury/);
      await classes.locator('[data-class="iron-rich"] [data-tile-picture]').waitFor({ state: "attached" });
      // The shipped atlas's globes come pre-drawn while the build's look is the one they were drawn with.
      const baked = await classes.locator('[data-class="iron-rich"] [data-tile-picture]').getAttribute("data-baked");
      console.log(`class globes: iron-rich ${baked === "true" ? "pre-drawn" : "drawn live"}`);
      // A small body's sample has no size yet, so it keeps its mark and says why.
      await gm.waitForFunction(() => ["drawn", "none"].includes(document.querySelector('[data-class="small-body"]')?.getAttribute("data-picture-state") ?? ""), null, { timeout: 60000 });
      assert.equal(await classes.locator('[data-class="small-body"]').getAttribute("data-picture-state"), "none");
      await shot(gm, "classes-picture-desk");
      await gm.keyboard.press("Escape");
    }
    await classes.locator("[data-classes-find]").fill("rings");
    await classes.locator('[data-hit="physical/ringed"]').click();
    await gm.locator('[data-class-card="physical/ringed"]').waitFor();
    await gm.keyboard.press("Escape");
    await classes.locator('[data-scheme="habitability"]').click();
    await classes.locator('[data-class="habitable/shirt-sleeve"]').click();
    await gm.locator('[data-class-card="habitable/shirt-sleeve"] [data-misses]').waitFor();
    await gm.keyboard.press("Escape");
    // SSE 3's classic list, which the charts marked first.
    await classes.locator('[data-atlas-pick="sse.sse3-classic"]').click();
    await gm.locator('[data-atlas="sse.sse3-classic"] [data-gallery="sse3-classic"] .tile').first().waitFor();
    // Its 61 or more classes, a page at a time.
    assert.ok(Number((await classes.locator("[data-page-shown]").innerText()).split(" of ")[1]) >= 61, "SSE 3's classic classes are all in the gallery");
    if (vp === "phone") {
      const over = await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      assert.ok(over <= 0, `classes overflow by ${over} px`);
    }
    assert.equal(await gm.locator("[data-classes-add]").count(), 0, "adding an atlas waits for the Build desk");
    // Parts (AT13), under "This setting": the parts of a player screen with the controls a screen can switch.
    await gm.locator('[data-page="parts"]').click();
    const parts = gm.locator("[data-parts]");
    await parts.waitFor();
    assert.deepEqual(await parts.locator('[data-part="sse.view.table"] [data-part-control]').evaluateAll((els) => els.map((e) => e.dataset.partControl)), ["sort", "select"]);
    assert.match(await parts.locator('[data-part="sse.view.text"]').innerText(), /No controls/);
    if (vp === "phone") assert.ok((await gm.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)) <= 0, "parts fit a phone");
    await shot(gm, `parts-${vp}`);
    await gm.locator('[data-page="names"]').click();
    await names.waitFor();
    assert.equal(await gm.locator("[data-names-add]").count(), 0, "adding a pack waits for the Build desk");
    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "reading the sheet writes nothing");
    if (vp === "desk") {
      await gm.getByRole("button", { name: "Build", exact: true }).click();
      await names.locator("[data-names-add]").click();
      await gm.locator('[data-names="stack"]').waitFor({ timeout: 20000 });
      await gm.waitForFunction((h) => document.querySelector("[data-state-hash]")?.getAttribute("data-state-hash") !== h, hash0);
      await gm.locator('[data-names="stack"] [data-sample] li').nth(11).waitFor({ timeout: 20000 });
      // Names here for the whole system, from the Names page: name every planet in Kestrel in Oluen.
      const here = gm.locator("[data-names-here='u:kestrel/sys:1']");
      await here.waitFor({ timeout: 20000 });
      await here.locator("[data-names-here-kind]").selectOption("planet");
      await here.locator('[data-names-here-layer="common"]').selectOption("oluen");
      await here.locator("[data-names-here-note]").getByText(/Common names for planets in Kestrel now come from Oluen/).waitFor({ timeout: 20000 });
      await shot(gm, "names-here-desk");
      await here.locator('[data-names-here-layer="common"]').selectOption("-");
      await here.locator("[data-names-here-note]").getByText(/follow the setting above again/).waitFor({ timeout: 20000 });
      // The name card: every name a body has, by layer, with where it came from.
      await go(gm, "system");
      await gm.locator("[role=button][aria-label^='Tarn']").click();
      await gm.locator("[data-names-open]").click();
      const card = gm.locator("[data-name-card='u:kestrel/sys:1/body:3']");
      await card.locator('[data-layer="designation"][data-source="drawn"]').waitFor({ timeout: 20000 });
      assert.match(await card.locator('[data-layer="common"]').innerText(), /^Tarn\s+typed/);
      await shot(gm, "name-card-desk");
      // On the Build desk: type a designation, put it back, and name everything beneath the star in Vesh.
      const desig = () => card.locator('[data-layer="designation"] .name').innerText();
      const drawnDesig = await desig();
      await card.locator('[data-name-type="designation"]').click();
      await card.locator('[data-name-input="designation"]').fill("Kestrel Survey 3");
      await card.locator('[data-name-input="designation"]').press("Enter");
      await card.locator('[data-layer="designation"][data-source="typed"]').waitFor({ timeout: 20000 });
      assert.equal(await desig(), "Kestrel Survey 3");
      await card.locator('[data-name-restore="designation"]').click();
      await card.locator('[data-layer="designation"][data-source="drawn"]').waitFor({ timeout: 20000 });
      assert.equal(await desig(), drawnDesig, "putting it back shows the drawn name again");
      await card.locator('[data-name-here="designation"]').selectOption("vesh");
      await card.locator("[data-name-note]").getByText(/take designation names from Vesh/).waitFor({ timeout: 20000 });
      await gm.waitForFunction(() => /drawn from Vesh/.test(document.querySelector('[data-name-card] [data-layer="designation"]')?.textContent ?? ""), null, { timeout: 20000 });
      // Layer choice in lenses: a lens of the GM's own that labels the chart by designation shows Tarn's Vesh designation.
      const vesh = await desig();
      await gm.getByRole("combobox", { name: "Lens", exact: true }).selectOption("__make");
      const dlg = gm.getByRole("dialog", { name: "Lens editor" });
      await dlg.getByLabel("Lens name").fill("Survey names");
      await dlg.getByLabel("Names on the chart").selectOption("designation");
      await dlg.getByRole("button", { name: "Save lens" }).click();
      await gm.waitForFunction(() => document.querySelector("main.stage")?.getAttribute("data-lens") === "mine.survey-names");
      await dlg.getByRole("button", { name: "Close" }).click();
      await gm.waitForFunction((n) => !!document.querySelector(`[role=button][aria-label^="${n}"]`), vesh, { timeout: 20000 });
      assert.equal(await gm.locator("[role=button][aria-label^='Tarn']").count(), 0, "the chart is labelled by designation");
      await shot(gm, "names-lens-desk");
    }
    assert.deepEqual(errors, []);
    await browser.close();
  }
  console.log("reference e2e: ok");
} finally {
  server.kill();
}
