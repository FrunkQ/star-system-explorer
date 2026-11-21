// Bring in a V3 save (Earth), open its globe drawn from its own air and ground.
export default {
  id: "09-globe-photo",
  title: "A world in true colour, from its own air and ground",
  feature: "V3 import, Body globe on render recipes, air from kernel facts (V13, R1, R2a)",
  trailers: ["features", "intro"],
  notes: "HOLD (Alex 21 Nov): no clouds, air or ice caps yet, the V3 sample is not a modelled Earth, surface mix looks wrong (passed to the physics stream), and the cutaway is missing. Re-shoot when those land; never caption the sample as Earth. Without a GPU the WebGPU globe films blank; the cloud take uses the 2D canvas tier. Re-shoot with --gpu on a real GPU for the hero take.",
  async setup(page, d, nav) {
    await nav.more(page, "Make a system");
    const dlg = page.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 20000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await page.locator('.area-chart .body[aria-label^="Earth"]').first().waitFor({ timeout: 20000 });
  },
  async run(page, d, nav) {
    await d.caption('A world in true colour', 'Drawn from its own air and ground.');
    await d.hold(600);
    await d.click(page.locator('.area-chart .body[aria-label^="Earth"]').first().locator(".hit"));
    await d.hold(600);
    await d.click(page.locator('.rail [data-go="world"]').first());
    await nav.go(page, "body");
    await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']) canvas", { timeout: 60000 });
    if (!process.argv.includes("--gpu")) {
      const sel = page.getByLabel("Draw on");
      if (!(await sel.isVisible().catch(() => false))) await page.locator("[data-badge]").click();
      await sel.selectOption("canvas");
      await page.waitForTimeout(1500);
    }
    const g = page.locator("[data-body] [data-tier]").first();
    await d.hold(1500);
    await d.glide(g, 800);
    const b = await g.boundingBox();
    await page.mouse.down();
    await d.glide({ x: b.x + b.width * 0.85, y: b.y + b.height / 2 }, 3000);
    await page.mouse.up();
    await d.hold(3000);
  },
};
