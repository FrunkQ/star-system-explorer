// Human-scale time: the ruler zooms to a year, steps by the week, then plays; worlds move by the kernel's orbits.
export default {
  id: "04-time-play",
  title: "Zoom the ruler to a year and play: the worlds move",
  feature: "Time ruler, campaign now, step and play (U11)",
  trailers: ["features"],
  async run(page, d) {
    const ruler = page.locator(".area-ruler");
    await d.hold(800);
    const show = ruler.locator("select").nth(0);
    await d.glide(show, 700);
    await show.selectOption("year");
    await d.hold(1200);
    const step = ruler.locator("select").nth(1);
    await d.glide(step, 600);
    await step.selectOption("week");
    await d.hold(400);
    const fwd = page.getByRole("button", { name: /^Forward one/ });
    await d.click(fwd);
    await d.click(fwd, { ms: 200 });
    await d.click(page.getByRole("button", { name: "Play", exact: true }));
    await d.glide({ x: 1450, y: 620 }, 1000);
    await d.hold(9000);
    await d.click(page.getByRole("button", { name: "Pause", exact: true }));
    await d.hold(1000);
  },
};
