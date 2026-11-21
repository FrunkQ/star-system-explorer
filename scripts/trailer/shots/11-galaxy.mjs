// Out to the galaxy: rings of history, the green band, a merger inked into the past.
export default {
  id: "11-galaxy",
  title: "Out to the galaxy, and ink a merger into its past",
  feature: "Galaxy chart and history edits (U6, G6)",
  trailers: ["features", "intro"],
  async run(page, d, nav) {
    await d.click(page.locator('.rail [data-go="explore"]').first());
    const tab = page.locator("[data-section-tab=galaxy]");
    if (await tab.isVisible().catch(() => false)) await d.click(tab);
    await page.waitForSelector(".gal [data-ring]", { timeout: 60000 });
    const gal = page.locator(".gal");
    await gal.locator("[data-green]").waitFor({ timeout: 60000 });
    await d.hold(2500);
    await d.click(gal.locator("[data-layer=supernovae]"));
    await d.hold(1500);
    await gal.locator("[data-ink-kind]").selectOption("merger");
    await gal.locator("[data-ink-t]").fill("8");
    await gal.locator("[data-ink-ratio]").selectOption({ label: "equal (1 : 1)" });
    await d.click(gal.locator("[data-ink-add]"));
    await gal.locator("[data-inked]").waitFor({ timeout: 60000 });
    await d.hold(3500);
  },
};
