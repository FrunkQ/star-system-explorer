// Beauty shot (list H1): the Moon in True with relief lighting (VP3, VP6): craters at true scale, lit by the Sun.
async function pick(page, d, group, name) {
  const g = page.getByRole("group", { name: group });
  const chip = page.locator("[data-view-more]").first();
  if (!(await g.isVisible()) && (await chip.count())) await d.click(chip);
  await d.click(g.getByRole("button", { name, exact: true }));
}
const pin = (page, name) => page.locator("[aria-roledescription='3D chart'] .pin").filter({ hasText: new RegExp(`^${name}`) }).first();
export default {
  id: "28-moon-relief",
  title: "The Moon in True, its craters lit by the Sun",
  feature: "System True close-up with the texture ladder and relief normals (VP3, VP6)",
  trailers: ["intro", "beauty"],
  notes: "GPU only (AC-ProArt). The Moon is the catalogued one; its relief comes from the kernel's tiles at true scale.",
  path: "",
  waitFor: '[data-door="sol"]:not([disabled])',
  async setup(page, d) {
    await page.locator('[data-door="sol"]').click();
    await page.locator('.area-chart .body[aria-label^="Earth,"]').first().waitFor({ timeout: 20000 });
  },
  async run(page, d) {
    await pick(page, d, "View style", "True");
    await pick(page, d, "HUD", "On");
    await d.hold(1500);
    await d.caption("Earth, then its Moon", "Click a world and the camera flies in.");
    await d.click(pin(page, "Earth"));
    await d.hold(2500);
    await d.click(pin(page, "Moon"));
    await d.hold(1500);
    await pick(page, d, "HUD", "Off");
    await d.caption("The Moon", "Craters and highlands at true scale, lit by the Sun: relief drawn from the kernel's own surface.");
    await d.hold(8000);
    await d.caption(null);
    await d.hold(4000);
  },
};
