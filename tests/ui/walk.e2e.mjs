// The stage 1 test drive (handover/design/SSE4-coming-together.md, section 3), in a real browser against the built
// shell (scripts/build-shell.sh first), on a desktop and on a phone. Usage: node tests/ui/walk.e2e.mjs [--screens <dir>]
//
// One walk a tester can do: open the Solar System, pick Earth and ask why, change a world on the Build desk and see a
// derived value follow, open it as a globe, its magnetic field and a cutaway, run time, go out to the cosmic web and
// come straight back down, and look up an element. Each step counts the presses it took (clicks, taps and choices
// from a list; typing a value is not counted) against the most it may take, so a later change that buries a common
// step fails here. A step that needs more than one press is listed in handover/test-drive.md as a rough edge.
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { view, pinClock } from "./nav.mjs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)("playwright");
const args = process.argv.slice(2);
const si = args.indexOf("--screens");
const screens = si >= 0 ? args.splice(si, 2)[1] : null;
const port = 9600 + Math.floor(Math.random() * 90);
const root = `http://localhost:${port}/`;
const server = spawn(process.execPath, ["scripts/serve.mjs", "packages/shell/dist", String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));

// The most presses each step may take. One is the aim for anything common (Alex, 23 Nov 2026: "most common things a
// click away"); a step allowed more is a known rough edge in handover/test-drive.md.
const BUDGET = {
  "open the Solar System": 1,
  "pick a world": 1,
  "ask why": 1,
  "change a world": 2, // the Build desk, then the value (typed)
  "see what followed": 1,
  "add a world": { desktop: 4, phone: 5 }, // pick the star, Add here (in More on a phone), press where on the chart (the ghost, ED4), then one of what forms there
  "open the globe": 1,
  "its magnetic field": 2, // Band is folded with HUD into the Show chip (changed far less than style, seen from, scale)
  "cut it open": 1,
  "run time": 1,
  "out to the cosmic web": { desktop: 1, phone: 2 }, // a phone's line shows one step up: the list beside the name, then the web
  "straight back down": 1,
  "look up an element": { desktop: 1, phone: 2 }, // Look up, then type its name and Enter (a phone presses the box first)
};

async function walk(browser, vp) {
  const phone = vp === "phone";
  const page = await (await browser.newContext(phone ? { viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, hasTouch: true } : { viewport: { width: 1440, height: 900 } })).newPage();
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  // Presses: trusted pointer presses, and a choice made from a list (a select's change).
  await page.addInitScript(() => {
    window.__presses = 0;
    addEventListener("pointerdown", (e) => e.isTrusted && window.__presses++, true);
    addEventListener("change", (e) => e.isTrusted && e.target instanceof HTMLSelectElement && window.__presses++, true);
  });
  const presses = () => page.evaluate(() => window.__presses);
  const shot = async (n) => {
    if (!screens) return;
    await page.waitForTimeout(400);
    mkdirSync(screens, { recursive: true });
    await page.screenshot({ path: `${screens}/walk-${vp}-${n}.png` });
  };
  const counted = [];
  async function step(name, fn) {
    const before = await presses();
    await fn();
    const n = (await presses()) - before;
    counted.push({ step: name, presses: n });
    const most = typeof BUDGET[name] === "number" ? BUDGET[name] : BUDGET[name][vp];
    assert.ok(n <= most, `${vp}: "${name}" took ${n} presses, at most ${most}`);
    await shot(name.replace(/\W+/g, "-"));
  }
  // A press waits for its target to hold still first: pressed while the layout is still moving, the browser driver
  // finds something else under the point, cancels that press and presses again, and the counter sees both (CI, 26 Nov:
  // "change a world" took 3 on a loaded runner). Waiting changes no budget; it only stops one press counting twice.
  const still = async (loc) => {
    await loc.waitFor();
    let was = "";
    for (let same = 0, i = 0; same < 3 && i < 100; i++) {
      const b = await loc.boundingBox();
      const now = b ? [b.x, b.y, b.width, b.height].map(Math.round).join(",") : "";
      same = now && now === was ? same + 1 : 0;
      was = now;
      await page.waitForTimeout(50);
    }
  };
  const tap = async (loc) => (await still(loc), phone ? loc.tap() : loc.click());
  const rail = (id) => page.locator(`.rail [data-go="${id}"]`).first();
  const card = (name) => page.locator(`section[aria-label='${name}']`);
  const field = (name, label) => card(name).locator(".field").filter({ hasText: new RegExp(`^${label}`, "i") });

  await page.goto(root);
  await step("open the Solar System", async () => {
    await tap(page.locator('[data-door="sol"]'));
    await page.waitForFunction(() => document.querySelector(".place h1.title")?.textContent?.trim() === "The Solar System", null, { timeout: 30000 });
    await page.locator(".area-chart [role=button][aria-label^='Earth']").first().waitFor({ timeout: 30000 });
  });
  const hash0 = await page.locator("[data-state-hash]").first().getAttribute("data-state-hash");

  await step("pick a world", async () => {
    await tap(page.locator(".area-chart [role=button][aria-label^='Earth']").first());
    await card("Earth").waitFor();
  });
  await step("ask why", async () => {
    await tap(field("Earth", "orbit").getByRole("button", { name: /why/i }));
    await page.getByText(/real catalogue/).first().waitFor();
  });

  // Change Mars: the Build desk, then type a nearer orbit. Its surface, worked out by the physics, warms.
  await tap(page.locator(".area-chart [role=button][aria-label^='Mars']").first());
  await card("Mars").waitFor();
  const surfaceK = async () => {
    await page.getByRole("group", { name: "Depth", exact: true }).count();
    return (await card("Mars").innerText()).match(/(-?\d+(?:\.\d+)?)\s*(K|°C)/g);
  };
  await step("change a world", async () => {
    const b = page.getByRole("group", { name: "Desk" }).getByRole("button", { name: "Build", exact: true });
    await tap(b);
    const orbit = field("Mars", "orbit").locator("input");
    await tap(orbit);
    await orbit.fill("1.0");
    await orbit.press("Enter");
    await field("Mars", "orbit").getByText(/set/i).first().waitFor({ timeout: 15000 });
  });
  await step("see what followed", async () => {
    const surface = field("Mars", "surface");
    // Warmer than about 230 K: in kelvin, or in °C as a world's surface is said (ED1a).
    await page.waitForFunction(() => /2[3-9]\d K|3\d\d K|(^|[^\d])−?([0-3]?\d|4\d)(\.\d)? °C/.test(document.querySelector("section[aria-label='Mars']")?.textContent ?? ""), null, { timeout: 20000 });
    await tap(surface.getByRole("button", { name: /why/i }));
    await page.locator(".area-card").getByText(/./).first().waitFor();
  });
  assert.ok((await surfaceK())?.length, "Mars's surface is shown");

  // Add a world by hand: what this orbit would make, from formation runs round the Sun (V3 gap 2). One Undo.
  await step("add a world", async () => {
    await tap(page.locator(".area-chart [role=button][aria-label^='Sun']").first());
    if (phone) {
      await tap(page.getByRole("button", { name: "More", exact: true }));
      await tap(page.locator("[data-more-add]"));
    } else await tap(page.getByRole("button", { name: "Add here" }));
    // The ghost comes first (ED4): press on the chart where it should go, a little inside Mars.
    await page.locator("[data-add-note]").waitFor({ timeout: 10000 });
    const sb = await page.locator(".area-chart [role=button][aria-label^='Sun'] .hit").first().boundingBox();
    const mb = await page.locator(".area-chart [role=button][aria-label^='Mars,'] .hit").first().boundingBox();
    const [px, py] = [sb.x + sb.width / 2 + (mb.x - sb.x) * 0.85, sb.y + sb.height / 2 + (mb.y - sb.y) * 0.85];
    if (phone) await page.touchscreen.tap(px, py);
    else await page.mouse.click(px, py);
    const add = page.locator("dialog[data-add]");
    await add.locator("[data-forms-here] li, [data-forms-nothing]").first().waitFor({ timeout: 30000 });
    await tap(add.locator("[data-forms-here] li button").first());
    await page.locator("[data-clip-note]").getByText(/^Added New world, orbiting Sun/).waitFor({ timeout: 20000 });
    await page.locator(".area-chart [role=button][aria-label^='New world']").first().waitFor({ timeout: 20000 });
  });
  await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
  await page.keyboard.press("Control+z");
  await page.waitForFunction(() => !document.querySelector(".area-chart [role=button][aria-label^='New world']"), null, { timeout: 15000 });
  assert.notEqual(await page.locator("[data-state-hash]").first().getAttribute("data-state-hash"), hash0, "the edit is in the universe");

  // The world itself: back to Earth, then its globe, its field and a cut through it.
  await tap(page.locator(".area-chart [role=button][aria-label^='Earth']").first());
  await card("Earth").waitFor();
  await step("open the globe", async () => {
    await tap(page.getByRole("button", { name: "Open Earth ›" }));
    await page.locator("[data-body] [data-tier]").first().waitFor({ timeout: 30000 });
    await page.waitForFunction(() => Number(document.querySelector("[data-body] [data-tier]")?.getAttribute("data-frames") ?? 0) > 0, null, { timeout: 30000 });
  });
  await step("its magnetic field", async () => {
    await view(page, "Band", "Mag");
    await page.getByText(/Field \d+(\.\d+)? µT at the equator/).waitFor({ timeout: 20000 });
  });
  await step("cut it open", async () => {
    await tap(rail("cutaway"));
    await page.getByText("Earth: cutaway").waitFor({ timeout: 20000 });
    await page.getByText(/Inner core/).first().waitFor();
  });

  await step("run time", async () => {
    const now = page.locator(".area-ruler .now");
    const t0 = await now.innerText();
    await tap(page.locator(".area-ruler").getByRole("button", { name: "Play" }));
    await page.waitForFunction((t) => document.querySelector(".area-ruler .now")?.textContent !== t, t0, { timeout: 10000 });
  });
  await tap(page.locator(".area-ruler").getByRole("button", { name: "Pause" }));

  await step("out to the cosmic web", async () => {
    // A desktop has the step on the line. A phone shows only the step above, so it opens the picker beside the name and
    // picks the web from the levels above at its top: a press to open it, and a press on the row.
    const web = page.locator('.place [data-place-step="web"]');
    if (await web.isVisible()) await tap(web);
    else {
      await tap(page.locator(".place [data-picker-chip]").first());
      // The list grows out of the chip; a person's second tap lands once it has settled.
      await page.waitForFunction(() => document.querySelector("[data-picker-pop]")?.getAnimations().length === 0, null, { timeout: 5000 });
      await tap(page.locator('[data-picker-pop] [data-pick="up:web"]'));
    }
    await page.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "web", null, { timeout: 30000 });
  });
  await step("straight back down", async () => {
    await tap(page.locator('.rail [data-go="back"]'));
    await page.waitForFunction(() => document.querySelector(".place h1.title")?.getAttribute("data-place") === "world", null, { timeout: 30000 });
    assert.equal((await page.locator(".place h1.title").innerText()).trim(), "Earth");
  });

  await step("look up an element", async () => {
    await tap(rail("reference"));
    const find = page.locator("[data-ref-find]");
    // The screen slides in; a person's tap on the box lands once it has settled (mid-slide it hit the page tabs).
    if (phone) {
      await find.scrollIntoViewIfNeeded();
      await page.waitForFunction(
        () =>
          new Promise((done) => {
            const box = () => document.querySelector("[data-ref-find]")?.getBoundingClientRect();
            const a = box();
            setTimeout(() => {
              const b = box();
              const top = b && document.elementFromPoint(b.x + b.width / 2, b.y + b.height / 2);
              done(!!a && !!b && a.x === b.x && a.y === b.y && !!top?.closest("[data-ref-find]") && document.getAnimations().length === 0);
            }, 300);
          }),
        null,
        { timeout: 8000, polling: 200 },
      );
      await tap(find);
    }
    else await page.waitForFunction(() => document.activeElement?.hasAttribute("data-ref-find"), null, { timeout: 20000 });
    await page.keyboard.type("iron");
    await page.locator('[data-ref-hit="elements:fe"]').waitFor({ timeout: 20000 });
    await page.keyboard.press("Enter");
    await page.locator('[data-element-card="fe"]').waitFor({ timeout: 20000 });
  });

  // Nothing runs off the side of a phone, anywhere on the walk's last screen.
  if (phone) {
    const over = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    assert.ok(over <= 1, `phone: nothing runs off the side (${over}px)`);
  }
  assert.deepEqual(errors, []);
  return counted;
}

try {
  const browser = pinClock(await chromium.launch());
  const all = {};
  for (const vp of ["desktop", "phone"]) all[vp] = await walk(browser, vp);
  await browser.close();
  console.log(`walk e2e: ok\n${JSON.stringify(all)}`);
} finally {
  server.kill();
}
