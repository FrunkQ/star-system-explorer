// True on the system chart (V10a to V10e): worlds in their own surfaces lit by their star, then the bands and Mag.
const vc = (page, group, name) => page.getByRole("group", { name: group }).getByRole("button", { name, exact: true });
export default {
  id: "25-true-system",
  title: "The system in True, then in other light",
  feature: "True on the system chart, living star, bands through True, Mag at system scale (V10a to V10e)",
  trailers: ["intro", "features"],
  notes: "GPU only: film with --gpu on AC-ProArt. In the cloud True has no GPU tier and films blank.",
  async run(page, d) {
    await d.caption("True", "The same chart, drawn as it would look: each world its own surface, lit by its star.");
    await d.click(vc(page, "View style", "True"));
    await d.hold(4500);
    await d.click(vc(page, "Seen from", "Tilted"));
    await d.hold(3500);
    await d.caption("Infrared", "Each body glows by its own heat.");
    await d.click(vc(page, "Band", "Infrared"));
    await d.hold(3500);
    await d.caption("X-ray", "The star's corona and flares; the worlds go dark.");
    await d.click(vc(page, "Band", "X-ray"));
    await d.hold(3500);
    await d.caption("Mag", "Every world's magnetic field, and the wind's bubble round it.");
    await d.click(vc(page, "Band", "Mag"));
    await d.hold(4000);
    await d.click(vc(page, "Band", "Visible"));
    await d.hold(1500);
  },
};
