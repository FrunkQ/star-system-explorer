// Grow a new system from a seed: formation run and its preview.
export default {
  id: "16-grow-system",
  title: "Grow a solar system from a cloud of gas",
  feature: "Formation run with preview (F0-F9, V13)",
  trailers: ["features", "intro"],
  async run(page, d) {
    await d.caption('Grow a system', 'From a cloud of gas and a seed: the worlds form, migrate and collide.');
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
    await d.hold(4000);
  },
};
