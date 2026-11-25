import { test } from "node:test";
import assert from "node:assert/strict";
import { systemSummary, summaryWords } from "../src/system-summary.ts";

const sol = [
  { label: "Sun", kind: "star" },
  { label: "Earth", kind: "planet", tags: ["life:complex", "ocean"] },
  { label: "Jupiter", kind: "planet" },
  { label: "Saturn", kind: "planet" },
  { label: "Saturn's rings", kind: "belt", parent: "saturn", ring: { inner_m: 7e7, outer_m: 1.4e8 } },
  { label: "Moon", kind: "moon", parent: "earth" },
  { label: "Io", kind: "moon", parent: "jupiter" },
  { label: "Ceres", kind: "dwarf_planet" },
  { label: "Main belt", kind: "belt" },
  { label: "ISS", kind: "station", parent: "earth" },
  { label: "Warm zone", kind: "zone" },
];

test("counts each kind once and says it in one line", () => {
  const s = systemSummary(sol);
  assert.deepEqual([s.stars, s.planets, s.moons, s.minor, s.belts, s.rings, s.craft], [1, 3, 2, 1, 1, 1, 1]);
  assert.deepEqual(s.life, ["Earth"]);
  assert.deepEqual(s.remarkable, ["1 ringed world"]);
  assert.equal(summaryWords(s), "1 star · 3 planets · 2 moons · 1 minor body · 1 belt · 1 ship or station · life on Earth");
});

test("notes multiple stars, remnants and honesty marks; nothing is guessed for unknown kinds", () => {
  const s = systemSummary([
    { kind: "star" },
    { kind: "remnant" },
    { kind: "brown_dwarf" },
    { kind: "planet", stamps: ["authored", "impossible"] },
    { kind: "planet", stamps: ["fictional"] },
    { kind: "wormhole" },
  ]);
  assert.equal(s.stars, 3);
  assert.equal(s.planets, 2);
  assert.deepEqual(s.remarkable, ["a 3-star system", "a dead star (a remnant)", "1 brown dwarf", "1 body marked impossible", "1 body marked fictional"]);
});

test("reads picker items by their chips when there is no kind", () => {
  const s = systemSummary([{ kinds: ["Stars"] }, { kinds: ["Stars"] }, { kinds: ["Planets"] }, { kinds: ["Moons"] }, { kinds: ["Ships and stations"] }, { kinds: ["Planets", "Life"], label: "Tarn" }]);
  assert.equal(summaryWords(s), "2 stars · 2 planets · 1 moon · 1 ship or station · life on Tarn");
  assert.deepEqual(s.remarkable, ["a binary star"]);
});

test("an empty system says so; many living worlds are counted, not listed", () => {
  assert.equal(summaryWords(systemSummary([])), "Nothing charted yet");
  const many = ["A", "B", "C"].map((label) => ({ label, kind: "planet", tags: ["biosphere"] }));
  assert.match(summaryWords(systemSummary(many)), /life on 3 worlds$/);
});
