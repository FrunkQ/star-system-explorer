// R1 acceptance: the governor holds its budget. Simulated devices (frame time
// = the plan's estimate × a device factor, with seeded jitter and a thermal
// slowdown) must settle at a level whose frames fit the budget, must not
// flicker between levels, and must climb back when the load lifts.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Governor, badge } from "../src/governor.ts";
import { estimateMs, maxLevel, plan, type RecipeDoc } from "../src/recipe.ts";
import { LIBRARY } from "../src/passes.ts";

const recipe: RecipeDoc = JSON.parse(readFileSync(new URL("../src/recipes/survey-relief.json", import.meta.url), "utf8"));
const PIXELS = 640 * 640; // a 320 px globe at device pixel ratio 2

/** A small seeded generator (mulberry32) for jitter. */
function rng(seed: number) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = seed;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function run(o: { tier: "canvas" | "gpu-basic"; factor: (frame: number) => number; frames: number; budget: number; seed?: number }) {
  const max = maxLevel(recipe, o.tier);
  const g = new Governor(max, { budget_ms: o.budget });
  const r = rng(o.seed ?? 7);
  const levels: number[] = [];
  const times: number[] = [];
  let changes = 0;
  for (let f = 0; f < o.frames; f++) {
    const p = plan(recipe, LIBRARY, o.tier, g.level);
    const ms = estimateMs(p, LIBRARY, PIXELS, { [p.tier]: o.factor(f) }) * (0.85 + 0.3 * r()) + (r() < 0.02 ? 40 : 0);
    times.push(ms);
    const ratio = Number.isFinite(g.ewma) ? g.ewma / estimateMs(p, LIBRARY, PIXELS) : null;
    if (g.sample(ms, (lv) => (ratio === null ? null : estimateMs(plan(recipe, LIBRARY, o.tier, lv), LIBRARY, PIXELS) * ratio))) changes++;
    levels.push(g.level);
  }
  return { levels, times, changes, g };
}

const median = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)]!;

test("a fast device keeps full quality", () => {
  const { levels, changes } = run({ tier: "canvas", factor: () => 0.3, frames: 600, budget: 33.3 });
  assert.equal(changes, 0);
  assert.equal(levels.at(-1), 0);
});

test("a slow device steps down until its frames fit the budget, then holds", () => {
  // About four times slower than the reference: full quality would take ~100 ms a frame.
  const { levels, times, changes } = run({ tier: "canvas", factor: () => 4, frames: 900, budget: 33.3 });
  const settled = times.slice(-300);
  assert.ok(median(settled) <= 33.3, `median ${median(settled).toFixed(1)} ms within 33.3`);
  assert.ok(levels.at(-1)! > 0, "it had to reduce");
  assert.ok(changes <= 6, `no flicker: ${changes} changes`);
  assert.equal(new Set(levels.slice(-300)).size, 1, "steady over the last 300 frames");
});

test("thermal throttling part way through: it steps down again, and climbs back when it cools", () => {
  // GPU estimates are small on the reference device; this phone is ~6x slower cool and ~20x when hot.
  const hot = (f: number) => (f < 300 ? 6 : f < 1500 ? 20 : 6);
  const { levels, times } = run({ tier: "gpu-basic", factor: hot, frames: 4000, budget: 33.3, seed: 11 });
  const before = levels[299]!;
  const during = levels[1499]!;
  assert.ok(during > before, `stepped down when hot (${before} to ${during})`);
  assert.ok(median(times.slice(1200, 1500)) <= 33.3, "held the budget while hot");
  assert.ok(levels.at(-1)! < during, `climbed back when cool (${during} to ${levels.at(-1)})`);
  assert.ok(median(times.slice(-300)) <= 33.3);
});

test("a device on the edge does not flicker: a step up that has to be taken back waits longer next time", () => {
  // Level 0 just over budget, level 1 well under: a naive governor would bounce.
  const { changes } = run({ tier: "canvas", factor: () => 0.52, frames: 6000, budget: 33.3, seed: 3 });
  assert.ok(changes <= 8, `${changes} changes in 6000 frames`);
});

test("the badge says nothing at full quality, and what was reduced and why otherwise", () => {
  assert.equal(badge({ reductions: [], ewma_ms: 10, budget_ms: 33.3 }).reduced, false);
  const b = badge({ reductions: ["Drawn at three-quarter resolution.", "Grid lines off."], ewma_ms: 48.2, budget_ms: 33.3, lift: "A faster device would lift it." });
  assert.equal(b.short, "Reduced: Grid lines off");
  assert.deepEqual(b.detail, ["Drawn at three-quarter resolution.", "Grid lines off.", "This device was taking about 48 ms a frame against a budget of 33 ms (30 a second)."]);
  assert.equal(b.lift, "A faster device would lift it.");
});
