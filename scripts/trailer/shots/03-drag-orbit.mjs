// Drag a world to a new orbit; its surface temperature follows. Then undo.
export default {
  id: "03-drag-orbit",
  title: "Drag a world outward and watch it cool",
  feature: "Drag orbits, derived climate, undo (U2)",
  trailers: ["features"],
  async setup(page) {
    await page.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    await page.locator("section[aria-label='Tarn']").waitFor();
  },
  async run(page, d) {
    await d.hold(800);
    const tarn = page.locator("[role=button][aria-label^='Tarn'] circle.hit").first();
    const star = await page.locator("[role=button][aria-label^='Kestrel'] circle.hit").first().boundingBox();
    const t = await tarn.boundingBox();
    const [dx, dy] = [t.x - star.x, t.y - star.y];
    await d.drag(tarn, dx * 0.45, dy * 0.45, 2200);
    await d.hold(2000);
    await page.keyboard.press("Control+z");
    await d.hold(1800);
  },
};
