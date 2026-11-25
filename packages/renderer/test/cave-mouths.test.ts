// Lava tube openings on the ground (third audit, TRUE6): the kernel's fallen roofs cut down to its passage floor.
import { test } from "node:test";
import assert from "node:assert/strict";
import { caveCut, caveOpenings, nearestOpening, openingLine } from "../src/cave-mouths.ts";

// A tube running east from the site: roof standing, then one fallen segment (a skylight), then standing again.
const survey = {
  passages: [
    {
      id: "tube-1",
      speculative: false,
      nodes: [
        { x_m: 0, y_m: 0, ground_m: 100, floor_m: 60 },
        { x_m: 200, y_m: 0, ground_m: 98, floor_m: 58 },
        { x_m: 300, y_m: 0, ground_m: 96, floor_m: 56 },
        { x_m: 500, y_m: 0, ground_m: 94, floor_m: 54 },
      ],
      segments: [
        { from: 0, to: 1, width_m: 40, roofed: true, open: null },
        { from: 1, to: 2, width_m: 40, roofed: false, open: "skylight" },
        { from: 2, to: 3, width_m: 40, roofed: true, open: null },
      ],
    },
  ],
};

test("only fallen roof opens the ground", () => {
  const o = caveOpenings(survey);
  assert.equal(o.length, 1);
  assert.equal(o[0]!.kind, "skylight");
  assert.equal(o[0]!.depth_a_m, 40);
  assert.deepEqual(caveOpenings(null), []);
  assert.deepEqual(caveOpenings({ passages: "x" }), []);
});

test("inside the opening the ground falls to the passage floor; over standing roof it does not", () => {
  const o = caveOpenings(survey);
  assert.equal(caveCut(o, 250, 0), 40);
  assert.equal(caveCut(o, 100, 0), 0, "over the standing roof");
  assert.equal(caveCut(o, 250, 30), 0, "beyond the rim");
  // The broken edge is a steep wall: halfway down within a metre of the rim.
  const lip = caveCut(o, 250, 19);
  assert.ok(lip > 10 && lip < 40, `${lip}`);
});

test("the nearest opening is named with its size, depth, distance and bearing", () => {
  const o = caveOpenings(survey);
  const n = nearestOpening(o, 250, -520)!;
  assert.equal(Math.round(n.d_m), 500);
  assert.equal(Math.round(n.bearing_deg), 0);
  assert.match(openingLine(n), /skylight 40 m across, 40 m deep, 500 m at 0°/);
  assert.match(openingLine(nearestOpening(o, 250, 0)), /^over a skylight/);
  assert.equal(nearestOpening([], 0, 0), null);
});
