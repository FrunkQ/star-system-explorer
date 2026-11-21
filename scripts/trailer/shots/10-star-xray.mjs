// The Sun drawn from physics, then the same star in X-rays.
export default {
  id: "10-star-xray",
  title: "See it in X-rays: the same star, a different object",
  feature: "Stars drawn from physics, Visible and X-ray (R5a, O10)",
  trailers: ["features", "intro"],
  async setup(page, d, nav) {
    await nav.more(page, "Make a system");
    const dlg = page.getByRole("dialog", { name: "Make a system" });
    await dlg.getByRole("tab", { name: "Import a V3 save" }).click();
    await dlg.getByRole("button", { name: "Try a sample save" }).click();
    await dlg.getByLabel("Import report").waitFor({ timeout: 20000 });
    await dlg.getByRole("button", { name: "Add this system to the campaign" }).click();
    await page.locator('.area-chart .body[aria-label^="Sun"]').first().click({ timeout: 20000 });
    await nav.go(page, "body");
    await page.locator("[data-star] [data-band]").waitFor({ timeout: 30000 });
  },
  async run(page, d) {
    await d.caption('The Sun, from physics', 'Spots, activity and flares worked out from its mass and spin.');
    await d.hold(2500);
    await d.click(page.locator("[data-star]").getByRole("button", { name: "X-ray" }));
    await d.caption("The same star in X-rays", "What shows is the million-degree corona, not the surface.");
    await page.waitForFunction(() => document.querySelector("[data-star] [data-band]")?.getAttribute("data-band") === "xray");
    await d.hold(3500);
  },
};
