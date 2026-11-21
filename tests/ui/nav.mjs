// Getting around the GM shell in browser tests (top-level design: the place line, the instrument rail, the Workshop,
// More). `go(page, id)` opens a screen by its id from the rail, going up, down or into the Workshop as needed;
// `more(page, name)` presses an entry of the More menu.

/** Open the screen `id` (system, body, cutaway, compare, life, route, window, stories, explore, physics, packs, screens, data, symbols, styles). */
export async function go(page, id) {
  const target = page.locator(`.rail [data-go="${id}"]`).first();
  const open = async () => {
    if (!(await target.isVisible())) return false;
    if ((await target.getAttribute("aria-pressed")) !== "true") await target.click();
    return true;
  };
  if (await open()) return;
  const up = page.locator('.rail [data-go="back"], .place [data-go="system"]').first();
  if (await up.isVisible()) await up.click();
  if (await open()) return;
  for (const door of ['.rail [data-go="world"]', '.rail [data-go="workshop"]']) {
    await page.locator(door).first().click();
    if (await open()) return;
    await up.click();
  }
  throw new Error(`no rail entry for screen ${id}`);
}

/** Press an entry of the More menu by its accessible name ("Campaign…", "Make a system", "Night chart", "Workshop"). */
export async function more(page, name) {
  const toggle = page.getByRole("button", { name: "More", exact: true });
  if ((await toggle.getAttribute("aria-expanded")) !== "true") await toggle.click();
  await page.getByRole("group", { name: "More" }).getByRole("button", { name }).click();
}

/** Choose a desk: "Explore" (look), "Build" (change) or "Run" (the table). A fresh browser opens on Explore. */
export async function desk(page, name) {
  const b = page.getByRole("group", { name: "Desk" }).getByRole("button", { name, exact: true });
  if ((await b.getAttribute("aria-pressed")) !== "true") await b.click();
}
