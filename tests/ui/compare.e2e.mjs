// V14 in a real browser, against the built shell (scripts/build-shell.sh
// first). Usage: node tests/ui/compare.e2e.mjs [chromium firefox webkit] [--screens <dir>]
// The Sizes screen starts with Kestrel Reach's own bodies on one true scale;
// reference objects come in and go out; Readable (a log scale) says it is not to scale;
// after adding the sample Sol to the campaign its Earth can be pulled in
// from the campaign. On the chart, Measure gives the distance between two
// bodies. On a phone the screen menu reaches Sizes with no sideways scroll.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
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
    const gm = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
    gm.on("pageerror", (e) => errors.push(e.message));
    await gm.goto(base + "#chart");
    await gm.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    const h0 = await gm.locator("[data-state-hash]").getAttribute("data-state-hash");

    // Measure on the chart first: the star to Tarn is Tarn's orbit.
    await gm.getByRole("button", { name: "Measure" }).click();
    const measure = gm.getByRole("status", { name: "Measurement" });
    assert.match(await measure.innerText(), /pick two bodies/);
    await gm.locator('.area-chart .body[aria-label^="Kestrel"]').first().click();
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    assert.match(await measure.innerText(), /Kestrel to Tarn\s+Now 0\.52 AU \(77\.8 million km\), 4\.3 light minutes\./);
    await gm.locator('.area-chart .body[aria-label^="Vault"]').first().click();
    assert.match(await measure.innerText(), /Now pick a second body/);
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    assert.match(await measure.innerText(), /Vault to Tarn\s+Now .* AU .*Closest .* farthest/s);
    assert.equal(await gm.locator(".mline").count(), 1);
    await shot(gm, `measure-${name}`);
    await gm.getByRole("button", { name: "Measure" }).click();
    assert.equal(await gm.locator(".mline").count(), 0);

    // Sizes: Kestrel Reach's own bodies with a radius.
    await go(gm, "compare");
    const cmp = gm.getByRole("region", { name: "Size comparison" });
    await cmp.waitFor();
    const start = Number(await cmp.getAttribute("data-compare"));
    assert.ok(start >= 5, `starts with the system's bodies: ${start}`);
    const rows = cmp.getByRole("table", { name: "Sizes in the lineup" }).locator("tbody tr");
    assert.equal(await rows.count(), start);
    assert.match(await cmp.locator('tr[data-row$="body:3"]').innerText(), /Tarn\s+III · temperate ocean world\s+Kestrel\s+6,052 km\s+0\.95 × Earth\s+fixture/);

    // Reference objects in and out.
    await cmp.getByRole("button", { name: "Reference objects" }).click();
    const refs = cmp.getByRole("group", { name: "Add a reference object" });
    await refs.getByRole("button", { name: "Jupiter", exact: true }).click();
    await refs.getByRole("button", { name: "Typical neutron star", exact: true }).click();
    assert.equal(Number(await cmp.getAttribute("data-compare")), start + 2);
    assert.equal(await refs.getByRole("button", { name: "Jupiter", exact: true }).isDisabled(), true, "already in the lineup");
    assert.match(await cmp.locator('tr[data-row="ref:jupiter"]').innerText(), /Jupiter\s+gas giant\s+Reference\s+69,911 km\s+1\.00 × Jupiter\s+measured/);
    assert.equal(await cmp.locator('[data-item="ref:neutron-star"]').getAttribute("data-tiny"), "true", "a neutron star is under a pixel beside Jupiter");
    await shot(gm, `compare-${name}`);
    // Scale is the one view controls' (Alex, 23 Nov): the comparison opens at Real (true scale); Readable is a log scale.
    const scaleBar = gm.getByRole("group", { name: "Scale" });
    assert.equal(await scaleBar.getByRole("button", { name: "Real", exact: true }).getAttribute("aria-pressed"), "true");
    await scaleBar.getByRole("button", { name: "Readable", exact: true }).click();
    await gm.waitForSelector('[data-mode="log"]');
    assert.equal(await cmp.getAttribute("data-mode"), "log");
    assert.match(await cmp.locator("svg").getAttribute("aria-label"), /not to scale \(log\)/);
    assert.equal(await cmp.locator("[data-tiny]").count(), 0);
    await scaleBar.getByRole("button", { name: "Real", exact: true }).click();
    await gm.waitForSelector('[data-mode="true"]');
    await cmp.getByRole("button", { name: "Remove Typical neutron star" }).click();
    assert.equal(Number(await cmp.getAttribute("data-compare")), start + 1);

    // Clicking one of this system's bodies selects it for the card; sizes then read as multiples of it.
    await cmp.getByRole("button", { name: /^Tarn, / }).click();
    assert.match(await gm.locator(".area-card").innerText(), /Tarn/);
    assert.match(await cmp.locator('tr[data-row="ref:jupiter"]').innerText(), /\d+ × Tarn/);

    // The lineup survives a trip to another screen.
    await go(gm, "system");
    // Leaving gives the viewer's own scale back (Readable here); the comparison opens at Real again next time.
    assert.equal(await gm.locator("[data-view-controls]").getAttribute("data-scale"), "readable");
    await go(gm, "compare");
    assert.equal(await gm.locator("[data-view-controls]").getAttribute("data-scale"), "real");
    assert.equal(Number(await gm.getByRole("region", { name: "Size comparison" }).getAttribute("data-compare")), start + 1);

    // Add the sample Sol, come back to Kestrel Reach, and pull in Sol's Earth from the campaign.
    await go(gm, "system");
    await more(gm, "Make a system");
    const dlg = gm.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 8000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await gm.locator('.area-chart .body[aria-label^="Earth"]').first().waitFor({ timeout: 15000 });
    await gm.getByLabel("System on the chart").selectOption("");
    await gm.locator('.area-chart .body[aria-label^="Tarn"]').first().waitFor({ timeout: 8000 });
    await go(gm, "compare");
    const cmp2 = gm.getByRole("region", { name: "Size comparison" });
    await cmp2.getByRole("button", { name: "From the campaign" }).click();
    const camp = cmp2.getByRole("group", { name: "Add from the campaign" });
    await camp.getByRole("heading", { name: "Sol" }).waitFor({ timeout: 15000 });
    await camp.getByRole("button", { name: "Earth (sample)", exact: true }).click();
    assert.match(await cmp2.locator("tbody").innerText(), /Earth \(sample\)\n[^\n]*\n\s*Sol \(sample\)\s+6,371 km/);
    await cmp2.getByRole("button", { name: "Surprise me" }).click();
    assert.equal(Number(await cmp2.getAttribute("data-compare")), start + 3);
    assert.equal(await gm.locator("[data-state-hash]").getAttribute("data-state-hash") !== h0, true, "adding Sol wrote history");

    // Globes (Alex, 24 Nov): 3D, True and Wireframe stand the lineup up, each body turning on its own axis.
    const styleBar = gm.getByRole("group", { name: "View style" });
    await cmp2.getByRole("button", { name: "Reference objects" }).click();
    await cmp2.getByRole("group", { name: "Add a reference object" }).getByRole("button", { name: "Saturn", exact: true }).click().catch(() => {});
    for (const look of ["3d", "true", "wireframe"]) {
      await styleBar.getByRole("button", { name: { "3d": "3D", true: "True", wireframe: "Wireframe" }[look], exact: true }).click();
      // A browser with no GPU path (Firefox headless in CI) keeps the flat lineup and says so.
      await cmp2.locator(`canvas[data-lineup-3d="${look}"], [data-no-globes]`).first().waitFor({ timeout: 10000 });
      if (await cmp2.locator("[data-no-globes]").count()) {
        assert.equal(await cmp2.locator("canvas[data-lineup-3d]").count(), 0);
        continue;
      }
      assert.match(await cmp2.locator("[data-pace]").innerText(), /turns once every 40 seconds, its own way round/);
      await gm.waitForTimeout(look === "true" ? 4000 : 800);
      await shot(gm, `globes-${look}-${name}`);
    }
    if (!(await cmp2.locator("[data-no-globes]").count())) {
      await cmp2.getByRole("button", { name: "True pace" }).click();
      assert.match(await cmp2.locator("[data-pace]").innerText(), /True pace: one second is .* of turning; .*, the quickest/);
      await gm.getByRole("group", { name: "Seen from" }).getByRole("button", { name: "Above", exact: true }).click();
      await gm.waitForTimeout(500);
      await shot(gm, `globes-above-${name}`);
    }
    await styleBar.getByRole("button", { name: "Chart", exact: true }).click();
    assert.equal(await cmp2.locator("canvas[data-lineup-3d]").count(), 0, "Chart is the flat lineup again");

    // True at Readable: the system's own star stands as a globe, and it is the living star of the system's True (SG4b).
    if (!(await cmp2.locator("[data-no-globes]").count())) {
      await styleBar.getByRole("button", { name: "True", exact: true }).click();
      await scaleBar.getByRole("button", { name: "Readable", exact: true }).click();
      await gm.waitForFunction(() => Number(document.querySelector("canvas[data-lineup-3d]")?.getAttribute("data-live-stars")) > 0, null, { timeout: 20000 });
      await gm.waitForTimeout(1500);
      await shot(gm, `globes-live-star-${name}`);
      await scaleBar.getByRole("button", { name: "Real", exact: true }).click();
      await styleBar.getByRole("button", { name: "Chart", exact: true }).click();
    }

    // Phone.
    const phone = await (await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true })).newPage();
    phone.on("pageerror", (e) => errors.push(e.message));
    await phone.goto(base + "#chart");
    await phone.waitForSelector("[data-state-hash]:not([data-state-hash=''])", { timeout: 20000 });
    await go(phone, "compare");
    await phone.getByRole("region", { name: "Size comparison" }).waitFor();
    const wide = await phone.evaluate(() => document.documentElement.scrollWidth);
    assert.ok(wide <= 390, `no sideways scroll on a phone: ${wide}`);
    await shot(phone, `compare-phone-${name}`);

    await browser.close();
    assert.deepEqual(errors, [], `${name}: page errors`);
    console.log(`compare e2e ${name}: ok (${start} bodies to start)`);
  }
} finally {
  server.kill();
}
