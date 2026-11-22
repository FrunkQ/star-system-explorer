// Climate "Why?" on the Body screen: the regime, belts of temperature and rain, and what the kernel assumed.
export default {
  id: "20-climate-why",
  title: "Why is the climate like this? Belts, reasons and assumptions",
  feature: "Climate Why? on the Body screen (U17, W7a)",
  trailers: ["physics", "features"],
  notes: "Filmed on the V3 sample planet (labelled 'Earth (sample)' by the app); never caption it as Earth. The globe in frame is not hero quality (see 09, on hold).",
  async setup(page, d, nav) {
    await nav.more(page, "Make a system");
    const dlg = page.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 20000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await page.locator('.area-chart .body[aria-label^="Earth"]').first().locator(".hit").click({ timeout: 20000 });
    await nav.go(page, "body");
    await page.waitForSelector("[data-body] [data-tier]:not([data-tier='']) canvas", { timeout: 60000 });
  },
  async run(page, d) {
    await d.caption("Why is the climate like this?", "A sample planet, imported from a version 3 save.");
    await d.hold(1500);
    await d.click(page.locator("[data-climate-toggle]"));
    const why = page.locator("[data-climate-why]");
    await why.waitFor({ timeout: 30000 });
    await d.caption("Belts of heat and rain", "Six 30° belts, worked out by the kernel's climate.");
    await d.glide(why.locator("[data-climate-belts]"), 900);
    await d.hold(3000);
    await d.caption("Its reasons, and what it assumed", "Every assumption listed, including values carried over from the old save.");
    await d.glide(why.locator("[data-climate-assumed]"), 900);
    await d.hold(3500);
  },
};
