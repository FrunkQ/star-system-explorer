// Watch it form (U19): a grown run's nine stages replayed from its own record.
export default {
  id: "22-formation-replay",
  title: "Watch a system form, stage by stage",
  feature: "Formation replay (U19) over the F0-F9 pipeline",
  trailers: ["generation", "features", "intro"],
  async run(page, d) {
    await d.caption("Grow a system", "One seed, one run of the nine-stage formation pipeline.");
    await d.click(page.getByRole("button", { name: "More", exact: true }));
    await d.click(page.getByRole("group", { name: "More" }).getByRole("button", { name: "Make a system" }));
    const dlg = page.getByRole("dialog", { name: "Make a system" });
    await d.click(dlg.getByRole("tab", { name: "Grow a new system" }));
    const seed = dlg.getByRole("textbox", { name: "Seed" });
    await d.click(seed);
    await seed.fill("");
    await d.type("first light", 110);
    await d.click(dlg.getByRole("button", { name: "Grow it" }));
    await dlg.getByLabel("Formation result").waitFor({ timeout: 60000 });
    await d.hold(900);
    await d.click(dlg.locator("[data-watch-form]"));
    const rp = page.locator(".replay");
    await rp.waitFor();
    await d.caption(null);
    await d.hold(1500);
    const chips = rp.locator(".stages button");
    const words = {
      3: ["The disc", "Gas and dust round the young star, with its condensation lines."],
      4: ["Embryos grow", "Each embryo keeps a ledger of what it swept up."],
      6: ["The violent phase", "The gas is gone; orbits cross, worlds collide or are flung out."],
      9: ["What is left", "The run's own worlds, as the campaign will keep them."],
    };
    for (const n of [2, 3, 4, 5, 6, 7, 8, 9]) {
      await d.click(chips.nth(n - 1), { settle: 200 });
      if (words[n]) await d.caption(...words[n]);
      if (n === 6) {
        await page.keyboard.press("ArrowRight");
        await d.hold(700);
        await page.keyboard.press("ArrowRight");
      }
      await d.hold(words[n] ? 2600 : 1300);
    }
    await d.caption("Play", "Time runs forward through the record; nothing is re-simulated.");
    await d.click(chips.nth(2));
    await d.click(rp.locator("[data-replay-play]"));
    await d.hold(5000);
    await d.click(rp.locator("[data-replay-play]"));
    await d.hold(800);
  },
};
