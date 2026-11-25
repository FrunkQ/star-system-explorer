// Sizes as turning globes (V14g): the Solar System's lineup stood up in 3D, True and Wireframe, each turning its own way.
async function pick(page, d, group, name) {
  const g = page.getByRole("group", { name: group });
  const chip = page.locator("[data-view-more]").first();
  if (!(await g.isVisible()) && (await chip.count())) await d.click(chip);
  await d.click(g.getByRole("button", { name, exact: true }));
}
export default {
  id: "27-sizes-globes",
  title: "The Solar System side by side, as turning globes",
  feature: "Sizes as globes in 3D, True and Wireframe (V14g)",
  trailers: ["intro", "features"],
  notes: "Best on a GPU (AC-ProArt). Without one, the lineup stays flat and says so.",
  path: "",
  waitFor: '[data-door="sol"]:not([disabled])',
  async setup(page, d, nav) {
    await page.locator('[data-door="sol"]').click();
    await page.locator('.area-chart .body[aria-label^="Earth,"]').first().waitFor({ timeout: 20000 });
    await nav.go(page, "compare");
    await page.getByRole("region", { name: "Size comparison" }).waitFor();
  },
  async run(page, d) {
    const cmp = page.getByRole("region", { name: "Size comparison" });
    await d.caption("How big is it, really?", "The Sun and its planets side by side.");
    await pick(page, d, "Scale", "Readable"); // Real leaves the small worlds under a pixel beside the Sun
    await d.hold(2000);
    await d.caption("3D", "The lineup stands up, each world turning on its own axis.");
    await pick(page, d, "View style", "3D");
    await cmp.locator('canvas[data-lineup-3d="3d"], [data-no-globes]').first().waitFor({ timeout: 15000 });
    await d.hold(2100);
    await d.caption("True", "Each world in its own colours and light.");
    await pick(page, d, "View style", "True");
    await d.hold(3000);
    await d.caption("Wireframe", "The same globes as lines.");
    await pick(page, d, "View style", "Wireframe");
    await d.hold(2100);
    await pick(page, d, "View style", "True");
    await d.caption("True pace", "Every world turning at its real rate, relative to the quickest.");
    await d.click(cmp.getByRole("button", { name: "True pace" }));
    await d.hold(3000);
  },
};
