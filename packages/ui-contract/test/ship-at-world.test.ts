// A ship at a world (kernel ship_at_world): the words a card puts its answer in.
import { test } from "node:test";
import assert from "node:assert/strict";
import { chanceWords, verdictWords, worstDangers, type WorldCheck } from "../src/ship-at-world.ts";

test("words chances as odds, never dice targets", () => {
  assert.equal(chanceWords(0), "very unlikely");
  assert.equal(chanceWords(0.25), "about 1 in 4");
  assert.equal(chanceWords(0.6), "about 60 in 100");
  assert.equal(chanceWords(0.999), "near certain");
});
test("names every verdict, including the ones that are only physics", () => {
  assert.equal(verdictWords("dangerous"), "past a limit");
  assert.equal(verdictWords("not-here"), "not possible here");
});
test("puts the worst dangers first and leaves out the ones within limits", () => {
  const d = (id: string, level: "ok" | "caution" | "danger", p: number) => ({ id, subject: "hull", role: "pilot", figure: 1, limit: 1, unit: "g", ratio: 1, level, chance: { p, basis: "" }, if_wrong: "", why: "" });
  const c: WorldCheck = { id: "aerocapture", words: "Aerocapture", verdict: "dangerous", why: [], dangers: [d("a", "ok", 0), d("b", "caution", 0.2), d("c", "danger", 0.9)] };
  assert.deepEqual(worstDangers(c).map((x) => x.id), ["c", "b"]);
});
