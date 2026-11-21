// The same chart room, in Night chart.
export default {
  id: "17-night-chart",
  title: "Night chart",
  feature: "Chart room and Night chart themes (U1)",
  trailers: ["features"],
  async run(page, d) {
    await d.caption('Night chart', 'The same chart room, for a dark table.');
    await d.hold(1000);
    await d.click(page.getByRole("button", { name: "More", exact: true }));
    await d.click(page.getByRole("group", { name: "More" }).getByRole("button", { name: "Night chart" }));
    await d.glide({ x: 1300, y: 600 }, 800);
    await d.hold(1500);
    await d.click(page.locator('.area-chart .body[aria-label^="Vault"]').first());
    await d.hold(3000);
  },
};
