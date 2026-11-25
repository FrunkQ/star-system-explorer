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
    // The inner planets crowd at system scale, so a click on the pin's centre can land on a
    // neighbour (25 Nov GPU take: no fly-in). Glide there, then click the pin element itself.
    const flyTo = async (name, next) => {
      await d.glide(pin(page, name));
      await pin(page, name).click({ timeout: 8000 }).catch(() => pin(page, name).dispatchEvent("click"));
      if (next && !(await pin(page, next).isVisible().catch(() => false))) {
        await pin(page, next).waitFor({ state: "visible", timeout: 6000 }).catch(() => pin(page, name).dispatchEvent("click"));
      }
    };
    await flyTo("Earth", "Moon");
    await d.hold(2000);
    await flyTo("Moon");
    // 25 Nov GPU take: the surface arrives about 3 s after the fly-in, and HUD Off pulled the
    // camera back to a small dot, so the HUD stays as it is and the shot waits for the surface.
    // Detail ladder thread: the finer tiles (1024 a face on WebGPU) need about 30 s, left alone.
    // The editor cuts this wait; the relief shows best along the day/night line.
    await d.hold(30000);
    await d.caption("The Moon", "Craters and highlands at true scale, lit by the Sun: relief drawn from the kernel's own surface.");
    await d.hold(4800);
    await d.caption(null);
    await d.hold(2400);
  },
};
