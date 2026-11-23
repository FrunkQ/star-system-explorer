// The galaxy in 3D and True (V9t, GH9): arms, bar and bulge in light, scars from its mergers, then edge-on.
// Press an option of the view controls. HUD and Band may sit behind the "HUD on · Visible ▾" chip
// ([data-view-more]); when they do, the chip is opened on camera first.
async function pick(page, d, group, name) {
  const g = page.getByRole("group", { name: group });
  const chip = page.locator("[data-view-more]").first();
  if (!(await g.isVisible()) && (await chip.count())) await d.click(chip);
  await d.click(g.getByRole("button", { name, exact: true }));
}
export default {
  id: "26-galaxy-true",
  title: "The galaxy in 3D and True",
  feature: "Galaxy in 3D and True (V9t), collision marks (GH9)",
  trailers: ["intro", "features"],
  notes: "GPU only: film with --gpu on AC-ProArt.",
  async run(page, d) {
    await d.caption("Out to the galaxy", "Grown from its own merger history.");
    await d.click(page.locator('.place [data-place-step="galaxy"]').first());
    await page.locator(".gal").waitFor({ timeout: 60000 });
    await d.hold(2000);
    await d.caption("3D", "The disc as a diagram you can turn.");
    await pick(page, d, "View style", "3D");
    await d.hold(3500);
    await d.caption("True", "Arms, bar and bulge in light: old stars by mass, young stars where gas is forming them.");
    await pick(page, d, "View style", "True");
    await d.hold(4500);
    await d.caption("Edge-on", "Seen side on, the disc keeps the warp its mergers left.");
    await pick(page, d, "Seen from", "Edge-on");
    await d.hold(4000);
    await pick(page, d, "Seen from", "Above");
    await d.hold(2000);
  },
};
