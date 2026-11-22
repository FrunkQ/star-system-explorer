// The start sheet (U21): what a first visit sees, then the Solar System door.
export default {
  id: "00-start-sheet",
  title: "The start sheet, then open the Solar System",
  feature: "Start sheet (U21), catalogued Solar System (SOL1)",
  trailers: ["intro", "features"],
  path: "",
  waitFor: '[data-door="sol"]:not([disabled])',
  async run(page, d) {
    await d.caption("Start here", "The app opens on a sheet: doors in, and the things it can do.");
    await d.hold(2200);
    const verbs = page.locator("[data-start] .atlas li");
    const n = Math.min(await verbs.count(), 6);
    for (let i = 0; i < n; i++) {
      await d.glide(verbs.nth(i), 300);
      await d.hold(250);
    }
    await d.caption("Open the Solar System", "The catalogued Sun, planets, Moon, Ceres and Pluto, from measured values.");
    await d.click(page.locator('[data-door="sol"]'));
    await page.locator('.area-chart .body[aria-label^="Earth,"]').first().waitFor({ timeout: 20000 });
    await d.hold(2500);
  },
};
