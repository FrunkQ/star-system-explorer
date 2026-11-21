// Edit a world in place and ask "why?": derived values follow, the formula and its science open.
export default {
  id: "02-edit-and-why",
  title: "Change a world, then ask why",
  feature: "Edit in place, derived values, Why? with Science (U2, U3)",
  trailers: ["features", "intro"],
  async setup(page) {
    await page.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    await page.locator("section[aria-label='Tarn']").waitFor();
  },
  async run(page, d) {
    const card = page.locator("section[aria-label='Tarn']");
    await d.hold(800);
    const mass = card.locator(".field").filter({ hasText: /^mass/i }).locator("input").first();
    await d.click(mass);
    await page.keyboard.press("Control+A");
    await d.type("0.6", 160);
    await page.keyboard.press("Enter");
    await d.hold(1500);
    const why = card.locator(".field").filter({ hasText: /^surface/i }).getByRole("button", { name: /why/i });
    await d.click(why);
    const panel = page.locator("[role=region][aria-label^='Why']");
    await panel.locator(".formula").waitFor();
    await d.hold(1800);
    await d.click(panel.getByRole("tab", { name: "Science" }));
    await d.hold(2500);
    await d.click(panel.getByRole("button", { name: "why?" }).first());
    await d.hold(2500);
  },
};
