// ED3 part 2: geared dragging and light magnetism on the chart (editing design section 4, question 10).
import { test } from "node:test";
import assert from "node:assert/strict";
import { gearing, nearestCatch, resonanceCatches, roundFigures } from "../src/index.ts";

test("gearing: all of the move on the line, less off it, a tenth with Shift", () => {
  assert.equal(gearing(0, false), 1);
  assert.equal(gearing(80, false), 0.5);
  assert.ok(Math.abs(gearing(0, true) - 0.1) < 1e-12);
  assert.ok(gearing(400, false) < 0.2);
});

test("round figures near a value, in the unit shown", () => {
  const au = roundFigures(1.52, 1, "AU");
  assert.ok(au.some((c) => c.at === 1.5 && c.says === "a round 1.5 AU"));
  assert.ok(au.some((c) => c.at === 2 && c.says === "a round 2 AU"));
  // A moon's orbit, shown in km, catches at round thousands of km.
  const km = roundFigures(384400 / 149597870.7, 149597870.7, "km");
  assert.ok(km.some((c) => c.says === "a round 400,000 km"));
});

test("resonance catches: Pluto's place in step with Neptune", () => {
  const c = resonanceCatches(39.5, [{ label: "Neptune", a: 30.07 }, { label: "Uranus", a: 19.2 }]);
  assert.equal(c.length, 1);
  assert.equal(c[0]!.says, "3:2 with Neptune");
  assert.ok(Math.abs(c[0]!.at - 30.07 * 1.5 ** (2 / 3)) < 1e-9);
});

test("the nearest catch within reach on the line, or none", () => {
  const px = (v: number) => v * 100;
  const cs = [
    { at: 1.5, says: "a round 1.5 AU" },
    { at: 1.53, says: "2:1 with Earth" },
  ];
  assert.equal(nearestCatch(152.6, cs, px)?.says, "2:1 with Earth");
  assert.equal(nearestCatch(150.5, cs, px)?.says, "a round 1.5 AU");
  assert.equal(nearestCatch(170, cs, px), null);
});
