// Grow a system and read how it formed: the nine-stage pipeline's dated events, then add it to the campaign.
export default {
  id: "21-how-it-formed",
  title: "Grow a system, then read how it formed",
  feature: "Formation pipeline history (F0-F9), make a system (V13)",
  trailers: ["generation", "features"],
  async run(page, d) {
    await d.caption("Grow a system", "One seed, one run of the nine-stage formation pipeline.");
    await d.click(page.getByRole("button", { name: "More", exact: true }));
    await d.click(page.getByRole("group", { name: "More" }).getByRole("button", { name: "Make a system" }));
    const dlg = page.getByRole("dialog", { name: "Make a system" });
    await d.click(dlg.getByRole("tab", { name: "Grow a new system" }));
    const seed = dlg.getByLabel("Seed");
    await d.click(seed);
    await seed.fill("");
    await d.type("trailer-1", 110);
    await d.click(dlg.getByRole("button", { name: "Grow it" }));
    await dlg.getByLabel("Formation result").waitFor({ timeout: 60000 });
    await d.hold(1500);
    const how = dlg.locator("summary", { hasText: "How it formed" });
    await d.caption("How it formed", "Dated events from the disc onward, each kept as the campaign's history.");
    await d.click(how);
    const list = how.locator("xpath=..").locator("li");
    const n = Math.min(await list.count(), 12);
    for (let i = 0; i < n; i += 2) {
      await d.glide(list.nth(i), 350);
      await d.hold(450);
    }
    await d.hold(1500);
  },
};
