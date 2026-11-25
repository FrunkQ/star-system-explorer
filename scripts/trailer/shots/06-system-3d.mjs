// Flip the chart to 3D, then into a planet's own frame for its moons.
export default {
  id: "06-system-3d",
  title: "From the flat chart to 3D, then down to the moons",
  feature: "System 3D, nested frames (V6, planet chart)",
  trailers: ["features", "intro"],
  notes: "Without a GPU the WebGPU view films blank; in the cloud it runs on the Canvas tier (no WebGL). Re-shoot with --gpu on a real GPU.",
  browserArgs: ["--disable-webgl", "--disable-webgl2", "--disable-3d-apis", "--disable-gpu"],
  async run(page, d) {
    await d.caption('The system in 3D', 'Drawn on the GPU, with numbers relative to the camera so nothing jitters.');
    await d.hold(800);
    await d.click(page.getByRole("button", { name: "3D", exact: true }));
    const view = page.locator("[aria-roledescription='3D chart']");
    await view.waitFor();
    await d.hold(2500);
    await d.glide({ x: 1100, y: 500 }, 600);
    await page.mouse.down();
    await d.glide({ x: 800, y: 420 }, 2500);
    await page.mouse.up();
    await d.hold(1000);
    // A click on a world flies in to its close-up, in its own frame with its moons (24 Nov).
    const pin = view.locator(".pin").filter({ hasText: /^Vault/ }).first();
    await d.caption("Into a planet's own frame", "Click a world and the camera flies in: its moons orbit it, not the star.");
    await d.click((await pin.count()) ? pin : view.locator(".pin").nth(4));
    await d.hold(3500);
  },
};
