// Age the system on a branch: the star swells and dies; then throw the future away.
export default {
  id: "05-look-ahead",
  title: "Look ahead: age the system until its star dies",
  feature: "Look ahead on a branch, star ageing (U4, T1, T2)",
  trailers: ["intro", "features"],
  async run(page, d) {
    await d.hold(800);
    await d.click(page.getByRole("button", { name: "Look ahead…" }));
    const panel = page.getByRole("group", { name: "Look ahead" });
    const age = panel.getByLabel("Target age in Gyr");
    await d.click(age);
    await age.fill("");
    await d.type("40", 200);
    await d.click(panel.getByRole("button", { name: "Look ahead", exact: true }));
    const scrub = panel.getByLabel("Age shown");
    await scrub.waitFor({ timeout: 60000 });
    await d.hold(800);
    // Slide the scrubber end to end so the chart ages on screen.
    const box = await scrub.boundingBox();
    await d.glide({ x: box.x + 4, y: box.y + box.height / 2 }, 600);
    await page.mouse.down();
    await d.glide({ x: box.x + box.width - 4, y: box.y + box.height / 2 }, 6000);
    await page.mouse.up();
    await d.hold(1200);
    await d.click(page.locator('.area-chart .body[aria-label^="Kestrel"]').first());
    await d.hold(2800);
  },
};
