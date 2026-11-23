// True scale: this system's worlds, then Jupiter and a neutron star beside them; then log scale.
export default {
  id: "07-sizes",
  title: "How big is it, really? True scale, then log scale",
  feature: "Sizes and reference objects (V14)",
  trailers: ["features"],
  async run(page, d, nav) {
    await d.caption('How big is it, really?', 'Worlds side by side at true scale.');
    const entry = page.locator('.rail [data-go="compare"]').first();
    if (await entry.isVisible()) await d.click(entry);
    else await nav.go(page, "compare");
    const cmp = page.getByRole("region", { name: "Size comparison" });
    await cmp.waitFor();
    await d.hold(1800);
    await d.click(cmp.getByRole("button", { name: "Reference objects" }));
    const refs = cmp.getByRole("group", { name: "Add a reference object" });
    await d.click(refs.getByRole("button", { name: "Jupiter", exact: true }));
    await d.hold(1500);
    await d.click(refs.getByRole("button", { name: "Typical neutron star", exact: true }));
    await d.caption("Jupiter and a neutron star", "A star's core the size of a city, beside a giant planet.");
    await d.hold(1500);
    // Scale lives in the one set of view controls (23 Nov): Real is true scale, Readable a log scale.
    await d.caption("Readable", "A log scale, so the smallest still shows.");
    await d.click(page.getByRole("group", { name: "Scale" }).getByRole("button", { name: "Readable", exact: true }));
    await d.hold(2500);
  },
};
