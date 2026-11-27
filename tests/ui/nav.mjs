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
  // On the Run desk the rail is folded to one button at the head of the view controls (three desks design 3.1): open it
  // before each press. Its Workshop door is in More.
  const unfold = async () => {
    const fold = page.locator("[data-rail-fold]").first();
    if ((await fold.count()) && (await fold.isVisible()) && (await fold.getAttribute("aria-expanded")) !== "true") await fold.click();
  };
  const target = page.locator(`.rail [data-go="${id}"]:visible`).first();
  const open = async () => {
    await unfold();
    if (!(await target.isVisible())) return false;
    if ((await target.getAttribute("aria-pressed")) !== "true") await target.click();
    return true;
  };
  if (await open()) return;
  const up = page.locator('.rail [data-go="back"]:visible, .place [data-go="system"]').first();
  await unfold();
  if (await up.isVisible()) await up.click();
  if (await open()) return;
  for (const door of ['.rail [data-go="world"]', '.rail [data-go="workshop"]']) {
    await unfold();
    const d = page.locator(`${door}:visible`).first();
    if (await d.isVisible()) await d.click();
    else if (door.includes("workshop")) await more(page, "Workshop");
    else continue;
    if (await open()) return;
    await unfold();
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
    await g.waitFor({ state: "visible", timeout: 15000 });
  }
  await g.getByRole("button", { name: option, exact: true }).click();
}

/**
 * The date a test's pages read off the device's clock (RW17 part 2: the Solar System's door opens on the device's
 * present). Pinned so the Solar System lies the same way on every run; time still flows from it. Call once on a
 * launched browser: every context and page it opens afterwards starts its clock here.
 */
export const SKY_DATE_MS = Date.UTC(2026, 10, 26, 12, 0, 0);
export function pinClock(browser, at = SKY_DATE_MS) {
  const script = `(() => { const R = Date; const off = ${at} - R.now(); class D extends R { constructor(...a) { if (a.length) super(...a); else super(R.now() + off); } static now() { return R.now() + off; } } globalThis.Date = D; })();`;
  const newContext = browser.newContext.bind(browser);
  browser.newContext = async (o) => {
    const c = await newContext(o);
    await c.addInitScript(script);
    return c;
  };
  const newPage = browser.newPage.bind(browser);
  browser.newPage = async (o) => {
    const p = await newPage(o);
    await p.addInitScript(script);
    return p;
  };
  return browser;
}
