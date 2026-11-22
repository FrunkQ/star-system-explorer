// The Reference screen (MP10): the master data sheet's constants, and the periodic table coloured by origin.
export default {
  id: "23-reference-elements",
  title: "The data sheet: constants and where the elements came from",
  feature: "Reference screen, Constants and Elements (MP10 parts 1 and 2)",
  trailers: ["features", "physics"],
  async run(page, d, nav) {
    await d.caption("One cited sheet", "Every constant the physics reads, with its source.");
    const entry = page.locator('.rail [data-go="reference"]').first();
    if (!(await entry.isVisible())) await d.click(page.locator('.rail [data-go="workshop"]').first());
    await d.click(entry);
    const ref = page.locator('[data-reference="constants"]');
    await ref.waitFor({ timeout: 20000 });
    await d.hold(1500);
    await d.click(ref.locator('[data-row="G"] .row'));
    await page.locator('[data-card="G"] [data-source]').first().waitFor({ timeout: 10000 });
    await d.hold(2500);
    await page.keyboard.press("Escape");
    await d.hold(400);
    await d.caption("The elements, by origin", "Coloured by what made them: the Big Bang, stars, supernovae, merging neutron stars.");
    await d.click(page.locator('[data-page="elements"]'));
    const els = page.locator('[data-elements="ready"]');
    await els.waitFor({ timeout: 20000 });
    await d.hold(2500);
    await nav.depth(page, "Curious");
    await d.caption("Gold", "Its card tells the story, and how sure we are.");
    await d.click(els.locator('.table [data-el="au"]'));
    await page.locator('[data-element-card="au"]').waitFor();
    await d.hold(4000);
    await page.keyboard.press("Escape");
    await d.hold(400);
    await d.caption("Iron", "From exploding white dwarfs and the cores of massive stars.");
    await d.click(els.locator('.table [data-el="fe"]'));
    await page.locator('[data-element-card="fe"]').waitFor();
    await d.hold(3500);
    await page.keyboard.press("Escape");
  },
};
