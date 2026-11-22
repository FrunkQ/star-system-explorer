// The Mag layer (MF14b) on the catalogued Earth: its measured dipole and the wind's bubble.
export default {
  id: "24-mag-earth",
  title: "Mag: the catalogued Earth's field and the wind's bubble",
  feature: "Mag layer on the Body screen (MF14b part 1), Solar System fields fixture (MF14a)",
  trailers: ["features", "physics"],
  notes: "The field is measured (MF14a fixture); the catalogued Earth's surface is not modelled yet, so the caption says so. Cloud takes run without WebGPU (the globe would film blank and the panel stalls).",
  browserArgs: ["--disable-webgl", "--disable-webgl2", "--disable-3d-apis", "--disable-gpu"],
  path: "",
  waitFor: '[data-door="sol"]:not([disabled])',
  async setup(page, d, nav) {
    if (!process.argv.includes("--gpu")) {
      // No GPU in the cloud: hide WebGPU so the Body screen takes its canvas tier, then reload into the sheet.
      await page.addInitScript(() => Object.defineProperty(Navigator.prototype, "gpu", { get: () => undefined }));
      await page.reload();
      await page.locator('[data-door="sol"]:not([disabled])').waitFor({ timeout: 20000 });
      await d.ready(); // the reload dropped the pointer and caption card
    }
    await page.locator('[data-door="sol"]').click();
    const b = page.locator('.area-chart .body[aria-label^="Earth,"]').first();
    await b.waitFor({ timeout: 20000 });
    await b.locator(".hit").click();
    await nav.go(page, "body");
    await page.locator("[data-mag-toggle]:visible").first().waitFor({ timeout: 20000 });
    await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']) canvas", { timeout: 60000 });
    await page.waitForTimeout(1500);
  },
  async run(page, d) {
    await d.caption("Earth, from the catalogue", "The Solar System pack: measured orbit, size and field. Its surface is not modelled yet.");
    await d.hold(1500);
    await d.caption("Mag", "Field lines from the measured dipole, and where it holds the solar wind off.");
    await d.click(page.locator("[data-mag-toggle]:visible").first());
    await page.waitForSelector('[data-mag="lines"]', { timeout: 45000 });
    await d.hold(3500);
    const mag = page.locator("[data-mag]");
    for (const name of [/Field out/, /bubble/, /Axes and tilt/]) {
      await d.click(mag.getByRole("button", { name }).first());
      await d.hold(2400);
    }
    await d.hold(1500);
  },
};
