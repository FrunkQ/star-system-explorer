// The small-body designer and lumpy small bodies (RW16a), in a real browser against the built shell
// (scripts/build-shell.sh first). Usage: node tests/ui/small-designer.e2e.mjs [--screens <dir>]
// Open the Solar System; the Workshop lists "Small bodies". The designer turns a body in True; a contact binary spun
// past its barrier is kept and says what it would take; a comet close to its star is awake. Nothing is written.
// (Phobos on the Body globe is not checked here: in the RW16a run its surface did not finish building in headless Chromium.)
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const base = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

try {
  for (const [vp, viewport] of [["desk", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const browser = await chromium.launch();
    const gm = await (await browser.newContext({ viewport, deviceScaleFactor: screens ? 2 : 1 })).newPage();
    const errors = [];
    gm.on("pageerror", (e) => errors.push(e.message));
    const shot = async (n, sel = "[data-small-designer]") => {
      if (screens) (mkdirSync(screens, { recursive: true }), await gm.locator(sel).first().screenshot({ path: `${screens}/${n}-${vp}.png` }));
    };
    await gm.goto(base);
    await gm.waitForSelector('[data-door="sol"]:not([disabled])', { timeout: 20000 });
    await gm.locator('[data-door="sol"]').click();
    await gm.locator(".area-chart .body").first().waitFor({ timeout: 20000 });

    // The Workshop door at the foot of the rail (once the system has settled), then its "Small bodies" entry.
    await gm.waitForTimeout(1500);
    await gm.locator('.rail [data-go="workshop"]').first().click();
    await gm.waitForTimeout(800);
    await gm.locator('.rail [data-go="small-designer"]').first().click();
    await gm.waitForSelector("[data-small-read]", { timeout: 30000 });
    const hash0 = await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash");
    const turned = async () => {
      await gm.waitForFunction(() => {
        const c = document.querySelector("[data-turn] canvas");
        if (!c) return false;
        const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
        let lit = 0;
        for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 30) lit++;
        return lit > 500;
      }, null, { timeout: 45000 });
      await gm.waitForTimeout(800);
    };
    await turned();
    const read = () => gm.locator("[data-small-read]").innerText();
    assert.match(await read(), /across/);
    await shot("asteroid");

    // A contact binary spun once an hour as a rubble pile: kept, and it says what it would take.
    await gm.getByRole("radio", { name: "Contact binary" }).click();
    await gm.getByRole("button", { name: "Rubble pile", exact: true }).click();
    const spin = gm.locator("label.row", { hasText: "Spin" }).locator("input");
    // 1 h on the log slider from 0.2 h to 1,000 h.
    await spin.evaluate((el) => {
      el.value = String(Math.log(1 / 0.2) / Math.log(1000 / 0.2));
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await gm.waitForSelector("[data-takes]", { timeout: 20000 });
    const takes = await gm.locator("[data-takes]").innerText();
    assert.match(takes, /fly apart/, takes);
    assert.match(await read(), /two lobes touching/);
    await turned();
    await shot("contact-fast");

    // A comet close to its star wakes.
    await gm.getByRole("radio", { name: "Comet" }).click();
    await gm.waitForFunction(() => /awake/.test(document.querySelector("[data-activity]")?.textContent ?? ""), null, { timeout: 20000 });
    await turned();
    await shot("comet");

    assert.equal(await gm.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "the designer writes nothing");

    assert.deepEqual(errors, [], errors.join("\n"));
    await browser.close();
  }
  console.log("small designer: ok");
} finally {
  server.kill();
}
