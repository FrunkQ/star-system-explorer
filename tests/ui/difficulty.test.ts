// A game system's difficulty numbers beside SSE4's words (Alex, 23 Nov): exact dice odds, the step nearest a hazard's
// odds for a typical operator, the word table as a fallback, the skill for the role, and nothing without a ladder.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { chanceToMeet, gameDifficulty, difficultyLadderOf, stepTarget } from "../../packages/ui-contract/src/difficulty.ts";

const TRAVELLER = JSON.parse(readFileSync(new URL("../../packs/traveller/pack.json", import.meta.url), "utf8"));
const near = (a: number, b: number) => assert.ok(Math.abs(a - b) < 1e-12, `${a} against ${b}`);

test("dice odds are exact", () => {
  const d = { count: 2, sides: 6 };
  near(chanceToMeet(d, 8), 15 / 36);
  near(chanceToMeet(d, 2), 1);
  near(chanceToMeet(d, 13), 0);
  near(chanceToMeet(d, 8, 1), 21 / 36);
  near(chanceToMeet({ count: 1, sides: 20 }, 15), 6 / 20);
});

test("the Traveller pack's ladder: targets, odds, skills and words", () => {
  const l = difficultyLadderOf([{ provides: { other: {} } }, TRAVELLER])!;
  assert.ok(l);
  assert.deepEqual(l.steps.map((s) => stepTarget(l, s)), [2, 4, 6, 8, 10, 12, 14]);
  // A pilot facing a 1-in-2 chance of things going wrong: a typical operator (DM +1) meets Difficult (10+) 1 time in 6,
  // Average (8+) 21 times in 36, so Average is nearest.
  const a = gameDifficulty(l, { p_wrong: 0.5, role: "pilot" })!;
  assert.equal(a.step, "Average");
  assert.equal(a.words, "Piloting, Average 8+");
  assert.equal(a.basis, "odds");
  // A near-certain failure is the hardest step; a safe act is the easiest.
  assert.equal(gameDifficulty(l, { p_wrong: 0.99 })!.step, "Formidable");
  assert.equal(gameDifficulty(l, { p_wrong: 0 })!.step, "Simple");
  // Words map by target; a role the pack does not know shows no skill.
  const w = gameDifficulty(l, { word: "hard", role: "cook" })!;
  assert.equal(w.words, "Difficult 10+");
  assert.equal(w.basis, "words");
  assert.equal(w.skill, undefined);
  assert.equal(gameDifficulty(l, { word: "unheard of" }), null);
});

test("without a ladder in the stack, nothing is shown", () => {
  assert.equal(difficultyLadderOf([{ provides: { other: {} } }]), null);
  assert.equal(gameDifficulty(null, { p_wrong: 0.3 }), null);
});
