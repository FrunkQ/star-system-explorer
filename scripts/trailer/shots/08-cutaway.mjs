// Cut a planet in half: shells from core to air, each with its reason.
export default {
  id: "08-cutaway",
  title: "Cut a planet in half",
  feature: "Cutaway from the kernel's world stack (V6)",
  trailers: ["features", "intro"],
  async run(page, d, nav) {
    await d.click(page.locator('.area-chart .body[aria-label^="Tarn"]').first());
    await d.click(page.locator('.rail [data-go="world"]').first());
    await nav.go(page, "cutaway");
    const cut = page.locator("[data-cutaway]");
    const btn = cut.getByRole("button", { name: "Tarn" });
    if (await btn.isVisible().catch(() => false)) await d.click(btn);
    await cut.locator("[data-shell]").first().waitFor({ timeout: 20000 });
    await d.hold(2000);
    const lines = cut.getByRole("list", { name: "Above, on and within Tarn" }).locator("li");
    const n = await lines.count();
    for (let i = 0; i < Math.min(n, 5); i++) {
      await d.glide(lines.nth(i), 500);
      await d.hold(600);
    }
    if (n > 5) await d.click(lines.nth(5).getByRole("button"));
    await d.hold(2500);
  },
};
