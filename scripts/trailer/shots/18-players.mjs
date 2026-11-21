// The players' screen: only what they have scanned.
export default {
  id: "18-players",
  title: "Players see only what they have scanned",
  feature: "Player view and scenes (V1, U5, V10)",
  trailers: ["intro", "features"],
  path: "players.html",
  waitFor: "body",
  async run(page, d) {
    await d.caption("The players' screen", 'They see only what they have scanned. The GM holds the whole chart.');
    await d.hold(3000);
    const scanned = page.getByRole("group", { name: "Sensor data" }).getByRole("button", { name: "Scanned" });
    if (await scanned.isVisible().catch(() => false)) await d.click(scanned);
    await d.hold(3000);
  },
};
