// Stand on a world and look up.
export default {
  id: "14-sky-from-here",
  title: "What does the sky look like from there?",
  feature: "Window / Sky from here (V8 part 2a)",
  trailers: ["features", "intro"],
  async run(page, d) {
    await d.click(page.locator('.rail [data-go="window"]').first());
    await d.hold(5000);
  },
};
