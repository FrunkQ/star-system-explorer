// The Hazard map's Profile layout (XO18): the stretched scale, ticks, labels that would overprint, distance words.
import { test } from "node:test";
import assert from "node:assert/strict";
import { distanceWords, labelled, radiusTicks, xOf } from "../../packages/views/hazard/src/profile.ts";

const map = { from_m: 7e7, to_m: 7e9 };

test("distance runs on a stretched scale: each factor of ten takes the same width", () => {
  assert.equal(xOf(map, 7e7, 0, 200), 0);
  assert.equal(xOf(map, 7e9, 0, 200), 200);
  assert.ok(Math.abs(xOf(map, 7e8, 0, 200) - 100) < 1e-9);
  // Outside the ends it sits on the ends.
  assert.equal(xOf(map, 1, 0, 200), 0);
  assert.equal(xOf(map, 1e12, 0, 200), 200);
});

test("ticks are 1, 2 and 5 of each power of ten radii, inside the span", () => {
  const t = radiusTicks(map).map((d) => d / 7e7);
  assert.deepEqual(t, [1, 2, 5, 10, 20, 50, 100]);
});

test("marks closer than the gap keep their tick but lose the label", () => {
  const l = labelled([{ x: 0 }, { x: 10 }, { x: 60 }]);
  assert.deepEqual(l.map((m) => m.show), [true, false, true]);
});

test("distances are said in radii, or in au round a star far out", () => {
  assert.equal(distanceWords(4.2e8, 7e7), "6.00 radii");
  assert.equal(distanceWords(1.84e9, 7e7), "26.3 radii");
  assert.equal(distanceWords(2.27e11, 6.957e8), "1.52 au");
});
