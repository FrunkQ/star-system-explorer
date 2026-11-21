// The chart room opens: a system on a paper chart, a hover peek, a card that grows from the world chosen.
export default {
  id: "01-chart-open",
  title: "The chart room: peek, pick, the card opens from the world",
  feature: "System chart (U1, U2), tactility (U12)",
  trailers: ["intro", "features"],
  async run(page, d) {
    await d.hold(1200);
    await d.glide(page.locator('.area-chart .body[aria-label^="Vault"]').first(), 1200);
    await d.hold(900);
    await d.glide(page.locator('.area-chart .body[aria-label^="Ember"]').first(), 900);
    await d.hold(700);
    await d.click(page.locator('.area-chart .body[aria-label^="Tarn"]').first(), { ms: 900 });
    await page.locator("section[aria-label='Tarn']").waitFor();
    await d.hold(2500);
  },
};
