// True on the system chart (V10a to V10e): worlds in their own surfaces lit by their star, then the bands and Mag.
// Press an option of the view controls. HUD and Band may sit behind the "HUD on · Visible ▾" chip
// ([data-view-more]); when they do, the chip is opened on camera first.
async function pick(page, d, group, name) {
  const g = page.getByRole("group", { name: group });
  const chip = page.locator("[data-view-more]").first();
  if (!(await g.isVisible()) && (await chip.count())) await d.click(chip);
  await d.click(g.getByRole("button", { name, exact: true }));
}
export default {
  id: "25-true-system",
  title: "The Solar System in True, HUD on, then in other light",
  feature: "True on the system chart, living star, bands through True, Mag at system scale (V10a to V10e)",
  trailers: ["intro", "features"],
  path: "",
  waitFor: '[data-door="sol"]:not([disabled])',
  async setup(page) {
    await page.locator('[data-door="sol"]').click();
    await page.locator('.area-chart .body[aria-label^="Earth,"]').first().waitFor({ timeout: 20000 });
  },
  notes: "The catalogued Solar System (SOL1). GPU only: film with --gpu on AC-ProArt. In the cloud True has no GPU tier and films blank.",
  async run(page, d) {
    await d.caption("True", "The Solar System as it would look: each world lit by the Sun, the Sun in its own colour.");
    await pick(page, d, "View style", "True");
    await d.hold(4500);
    await pick(page, d, "Seen from", "Tilted");
    await d.hold(3500);
    await d.caption("HUD", "Names, orbits and outlines over the true picture, like a heads-up display.");
    await pick(page, d, "HUD", "On");
    await d.hold(3500);
    await d.caption("Infrared", "Each body glows by its own heat.");
    await pick(page, d, "Band", "Infrared");
    await d.hold(3500);
    await d.caption("X-ray", "The star's corona and flares; the worlds go dark.");
    await pick(page, d, "Band", "X-ray");
    await d.hold(3500);
    await d.caption("Mag", "Every world's magnetic field lines, and the wind's bubble round it.");
    await pick(page, d, "Band", "Mag");
    await d.hold(5000);
    await pick(page, d, "Band", "Visible");
    await d.hold(1500);
    // A click on any body flies in to its close-up (24 Nov), with its rings and moons.
    const saturn = page.locator("[aria-roledescription='3D chart'] .pin").filter({ hasText: /^Saturn/ }).first();
    if (await saturn.isVisible().catch(() => false)) {
      await d.caption("Fly in", "Click any world and the camera flies to it: Saturn, its rings and moons.");
      await d.click(saturn);
      await d.hold(5000);
    }
  },
};
