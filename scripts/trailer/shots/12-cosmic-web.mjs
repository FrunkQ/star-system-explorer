// The cosmic web: a slice of foam, played from early times to today.
export default {
  id: "12-cosmic-web",
  title: "Zoom out until galaxies are dots: the cosmic web grows",
  feature: "Cosmic web foam with time play (V9)",
  trailers: ["intro", "features"],
  async run(page, d) {
    await d.caption('The cosmic web', 'Gravity draws matter into filaments, from the first billion years to today.');
    await d.click(page.locator('.rail [data-go="explore"]').first());
    await d.click(page.locator("[data-section-tab=foam]"));
    const foam = page.locator(".foam");
    await foam.locator("[data-lede]").waitFor({ timeout: 60000 });
    await foam.locator("[data-time]").fill("1");
    await foam.locator("[data-time]").dispatchEvent("change");
    await d.hold(1500);
    await d.click(foam.locator("[data-play]"));
    await d.glide({ x: 1700, y: 900 }, 800);
    await d.hold(9000);
  },
};
