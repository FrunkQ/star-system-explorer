// Find tonight's story: sifters over the universe, hits with evidence.
export default {
  id: "15-stories",
  title: "Find tonight's story",
  feature: "Story finder on the sift op (U8)",
  trailers: ["features", "intro"],
  async run(page, d) {
    await d.click(page.locator('.rail [data-go="stories"]').first());
    await d.hold(3000);
    const hit = page.locator("[data-hit]").first();
    if (await hit.isVisible().catch(() => false)) await d.click(hit);
    await d.hold(3000);
  },
};
