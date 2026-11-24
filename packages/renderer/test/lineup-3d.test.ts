// The size comparison's turning globes (Alex, 24 Nov): each leans by its own tilt and turns its own way, at a pace
// that can be watched.
import { test } from "node:test";
import assert from "node:assert/strict";
import { circleSteps, globeFrame, graticule, ringEdge, SECONDS_PER_TURN, spinAt } from "../src/lineup-3d.ts";

const near = (a: number, b: number, e = 1e-9) => assert.ok(Math.abs(a - b) < e, `${a} against ${b}`);
const edge = { elevation: 0, yaw: 0 };

test("edge-on, an untilted globe's pole points up the screen; its tilt leans it sideways, across the view", () => {
  const [, , z] = globeFrame(0, edge, 0);
  near(z[0], 0), near(z[1], 1), near(z[2], 0);
  const t = (23.4 * Math.PI) / 180;
  const [, , zt] = globeFrame(t, edge, 0);
  near(zt[0], Math.sin(t)), near(zt[1], Math.cos(t)), near(zt[2], 0);
});

test("seen from above, the pole comes toward the viewer", () => {
  const [, , z] = globeFrame(0, { elevation: Math.PI / 2, yaw: 0 }, 0);
  near(z[2], 1);
});

test("the frame stays a true rotation at every tilt, view and turn", () => {
  for (const tilt of [0, 0.4, 1.7, 3.1])
    for (const view of [edge, { elevation: 0.55, yaw: 0.3 }, { elevation: -1.2, yaw: 2 }])
      for (const turn of [0, 1, 4]) {
        const [x, y, z] = globeFrame(tilt, view, turn);
        const dot = (a: number[], b: number[]) => a[0]! * b[0]! + a[1]! * b[1]! + a[2]! * b[2]!;
        near(dot(x, x), 1), near(dot(y, y), 1), near(dot(z, z), 1), near(dot(x, y), 0), near(dot(y, z), 0);
        // Right-handed: x × y = z.
        near(x[1] * y[2] - x[2] * y[1], z[0]), near(x[2] * y[0] - x[0] * y[2], z[1]);
      }
});

test("a globe turns on its pole, and one tilted past a right angle turns the other way as seen from its orbit's north", () => {
  // A point on the equator moves, seen from above, anticlockwise for Earth and clockwise for Venus.
  const above = { elevation: Math.PI / 2, yaw: 0 };
  const sweep = (tilt: number) => {
    const a = globeFrame(tilt, above, 0)[0];
    const b = globeFrame(tilt, above, 0.1)[0];
    return a[0] * b[1] - a[1] * b[0];
  };
  assert.ok(sweep(0.41) > 0, "prograde");
  assert.ok(sweep(3.096) < 0, "retrograde");
});

test("the pace: every body one turn in the same time, or in true proportion to the quickest", () => {
  near(spinAt(24, SECONDS_PER_TURN / 4, "same", 10), Math.PI / 2);
  near(spinAt(5832, SECONDS_PER_TURN / 4, "same", 10), Math.PI / 2, 1e-9);
  near(spinAt(10, SECONDS_PER_TURN / 4, "true", 10), Math.PI / 2);
  near(spinAt(20, SECONDS_PER_TURN / 4, "true", 10), Math.PI / 4);
  near(spinAt(-24, SECONDS_PER_TURN / 4, "same", 10), -Math.PI / 2);
  assert.equal(spinAt(undefined, 30, "same", 10), 0, "a day not known stands still");
});

test("lines stay on the globe and round to half a pixel", () => {
  const f = globeFrame(0.5, { elevation: 0.55, yaw: 0 }, 1);
  for (const l of graticule(f, [10, 20, -30], 50)) for (const p of l.points) near(Math.hypot(p[0] - 10, p[1] - 20, p[2] + 30), 50 * 1.004, 1e-6);
  assert.equal(graticule(f, [0, 0, 0], 50).filter((l) => l.kind === "equator").length, 1);
  for (const p of ringEdge(f, [0, 0, 0], 80)) {
    near(Math.hypot(...p), 80, 1e-6);
    near(p[0] * f[2][0] + p[1] * f[2][1] + p[2] * f[2][2], 0, 1e-6);
  }
  const R = 1e5;
  const n = circleSteps(R);
  assert.ok(R * (1 - Math.cos(Math.PI / n)) <= 0.5 || n === 2048);
});

test("a shaded globe's lines are trimmed to the side facing the viewer", async () => {
  const { nearSide } = await import("../src/lineup-3d.ts");
  const f = globeFrame(0, { elevation: 0.55, yaw: 0 }, 0.3);
  for (const g of graticule(f, [5, 5, 0], 40)) for (const run of nearSide(g.points, [5, 5, 0])) for (const p of run) assert.ok(p[2] >= 0);
  assert.equal(nearSide([[0, 0, -1], [1, 0, -1]], [0, 0, 0]).length, 0);
});
