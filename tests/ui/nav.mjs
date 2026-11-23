// Getting around the GM shell in browser tests (top-level design: the place line, the instrument rail, the Workshop,
// More). `go(page, id)` opens a screen by its id from the rail, going up, down or into the Workshop as needed;
// `more(page, name)` presses an entry of the More menu.

/** Open the screen `id` (system, body, cutaway, compare, life, route, voyages, window, stories, galaxy, cluster, web, physics, packs, screens, data, symbols, styles). */
export async function go(page, id) {
  // The scales above a system are steps on the place line (U24): web, cluster (galaxies nearby), galaxy. Press the
  // step when it shows, else climb by the parent step ("‹", which is all a phone shows).
  if (["web", "cluster", "galaxy"].includes(id)) {
    for (let i = 0; i < 8; i++) {
      if (await page.locator(`.place [data-place="${id}"]`).count()) return;
      const step = page.locator(`.place [data-place-step="${id}"]`).first();
      const up = page.locator(".place .step.up").first();
      const back = page.locator('.rail [data-go="back"]').first();
      if (await step.isVisible()) await step.click();
      else if (await up.isVisible()) await up.click();
      else if (await back.isVisible()) await back.click();
      else break;
    }
    throw new Error(`no way up to ${id}`);
  }
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

/** Choose a depth: "Curious", "Science" or "Engine". A fresh browser opens at Curious. In the bar on a wide screen, in More otherwise. */
export async function depth(page, name) {
  if ((await page.locator(".app").getAttribute("data-depth")) === name.toLowerCase()) return;
  const bar = page.getByRole("group", { name: "Depth", exact: true });
  if (await bar.isVisible()) await bar.getByRole("button", { name, exact: true }).click();
  else {
    const toggle = page.getByRole("button", { name: "More", exact: true });
    if ((await toggle.getAttribute("aria-expanded")) !== "true") await toggle.click();
    await page.getByRole("group", { name: "Depth in More" }).getByRole("button", { name, exact: true }).click();
    await toggle.click();
  }
  await page.waitForFunction((d) => document.querySelector(".app")?.getAttribute("data-depth") === d, name.toLowerCase());
}

/**
 * Press one option of the view controls by its switch and name: `view(page, "Band", "Mag")`, `view(page, "HUD", "Off")`.
 * HUD and Band are folded into the "Show" chip; it is opened when needed, and a choice closes it.
 */
export async function view(page, group, option, scope = page.locator(".viewbar [data-view-controls], [data-view-controls]").first()) {
  let g = scope.getByRole("group", { name: group, exact: true });
  const folded = !(await g.isVisible());
  if (folded) {
    await scope.locator("[data-view-more]").click();
    g = scope.getByRole("group", { name: group, exact: true });
  }
  await g.getByRole("button", { name: option, exact: true }).click();
}
