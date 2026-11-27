// What is due on the Run desk's clock (TM3b), read from the shape of the kernel's happenings reply (TP6).
import { test } from "node:test";
import assert from "node:assert/strict";
import { NO_HAPPENINGS, checkHappenings, dueAhead, passedWords, type Happenings, type TimeMark } from "../src/due.ts";

const DAY = 86_400n;
const mark = (at_s: bigint, kind: string, rank = 1, source = "test"): TimeMark => ({ at_s, kind, subjects: [], rank, why: `${kind} at ${at_s}`, source });

test("due ahead: only marks after now and within the window, earliest first, placed along the line", () => {
  const now = 1_000_000_000n;
  const h: Happenings = {
    marks: [mark(now + 3n * DAY, "arrival"), mark(now - DAY, "eclipse"), mark(now, "ruling"), mark(now + DAY, "window"), mark(now + 9n * DAY, "arrival")],
    summarised: [],
  };
  const due = dueAhead(h, now, 4n * DAY);
  assert.deepEqual(due.map((d) => d.mark.kind), ["window", "arrival"]);
  assert.equal(due[0]!.ahead_s, DAY);
  assert.equal(due[0]!.x, 0.25);
  assert.equal(due[1]!.x, 0.75);
});

test("due ahead: a mark at the window's end is due; ties go to the higher rank, then a fixed order", () => {
  const now = 0n;
  const h: Happenings = { marks: [mark(DAY, "b", 1), mark(DAY, "a", 1), mark(DAY, "c", 5), mark(2n * DAY, "end")], summarised: [] };
  assert.deepEqual(dueAhead(h, now, 2n * DAY).map((d) => d.mark.kind), ["c", "a", "b", "end"]);
  assert.deepEqual(dueAhead(h, now, 2n * DAY, 2).map((d) => d.mark.kind), ["c", "a"]);
  assert.deepEqual(dueAhead(h, now, 0n), []);
  assert.deepEqual(dueAhead(NO_HAPPENINGS, now, DAY), []);
});

test("due ahead keeps whole seconds near a star's age (no doubles for absolute time)", () => {
  // 13.8 Gyr in seconds is past 2^53: one second apart must stay one second apart.
  const now = 435_485_000_000_000_000n;
  const h: Happenings = { marks: [mark(now + 2n, "flare"), mark(now + 1n, "flare")], summarised: [] };
  const due = dueAhead(h, now, 10n);
  assert.deepEqual(due.map((d) => d.ahead_s), [1n, 2n]);
});

test("passed words count listed and summarised marks together, commonest first", () => {
  const h: Happenings = {
    marks: [mark(1n, "eclipse"), mark(2n, "eclipse"), mark(3n, "arrival"), mark(4n, "eclipse")],
    summarised: [{ kind: "flare", count: 1200 }],
  };
  assert.equal(passedWords(h), "1200 flares, 3 eclipses, 1 arrival");
  assert.equal(passedWords(NO_HAPPENINGS), "");
  const many: Happenings = { marks: ["a", "b", "c", "d", "e", "f"].map((k, i) => mark(BigInt(i), k)), summarised: [] };
  assert.equal(passedWords(many), "1 a, 1 b, 1 c, 1 d, 2 more");
});

test("a happenings reply is checked: whole seconds, a kind, a why and a rank on every mark", () => {
  assert.deepEqual(checkHappenings({ marks: [mark(1n, "arrival")], summarised: [] }), []);
  assert.deepEqual(checkHappenings({ marks: [{ at_s: 1.5, kind: "", why: 3, rank: "x" }], summarised: 0 }), [
    "mark 0: at_s must be whole seconds",
    "mark 0: kind must be a word",
    "mark 0: why must be a sentence",
    "mark 0: rank must be a number",
    "summarised must be a list",
  ]);
  assert.deepEqual(checkHappenings(null), ["not a happenings reply"]);
});
