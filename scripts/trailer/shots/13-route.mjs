// How long to the next world, and will the crew survive it? Courses compared, one charted.
export default {
  id: "13-route",
  title: "Plan a voyage: courses compared by fuel, time and dose",
  feature: "Route, the astrogation console (V8, T3)",
  trailers: ["features"],
  async run(page, d) {
    await d.click(page.locator('.rail [data-go="route"]').first());
    const cards = page.locator("[data-course]");
    await cards.first().waitFor({ timeout: 60000 }).catch(() => {});
    await d.hold(2500);
    const n = await cards.count();
    if (n > 1) await d.click(cards.nth(1).getByRole("button").first());
    await d.hold(3500);
  },
};
