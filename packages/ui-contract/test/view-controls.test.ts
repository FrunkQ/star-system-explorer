// The standard view controls (rule 12): every option always listed, the gaps quiet with reasons, a flat chart turning
// into 3D, the kept choice read back safely.
import { test } from "node:test";
import assert from "node:assert/strict";
import { flatChart, pickSeen, pickStyle, readViewChoice, viewOptions, WHY_TRUE_HELD, type ViewControlsSpec } from "../src/view-controls.ts";

const system: ViewControlsSpec = { styles: { chart: true, "3d": true, true: WHY_TRUE_HELD }, flat: true };

test("every option is listed in the same order; what a view cannot do says why", () => {
  const o = viewOptions(system, { style: "chart", seen: "tilted" });
  assert.deepEqual(o.styles.map((x) => x.label), ["Chart", "3D", "True", "Wireframe"]);
  assert.deepEqual(o.seen.map((x) => x.label), ["Above", "Tilted", "Edge-on", "Free"]);
  assert.equal(o.styles[2]!.unavailable, WHY_TRUE_HELD);
  assert.match(o.styles[3]!.unavailable!, /not drawn here yet/);
  assert.deepEqual(o.shown, { style: "chart", seen: "above" }, "a flat chart is seen from above");
  assert.ok(o.seen.every((x) => !x.unavailable), "turning a flat chart is offered when it has 3D");
});

test("turning a flat chart makes it 3D; a fixed view on 3D stays 3D", () => {
  const c = pickSeen(system, { style: "chart", seen: "tilted" }, "edge");
  assert.deepEqual(c, { style: "3d", seen: "edge" });
  assert.deepEqual(pickSeen(system, c, "free"), { style: "3d", seen: "free" });
  assert.deepEqual(pickSeen(system, { style: "chart", seen: "tilted" }, "above"), { style: "chart", seen: "tilted" });
  assert.deepEqual(pickStyle(c, "chart"), { style: "chart", seen: "edge" });
});

test("a style a view cannot draw falls back to Chart, and the choice is kept for the next view", () => {
  const galaxy = flatChart();
  const o = viewOptions(galaxy, { style: "3d", seen: "free" });
  assert.deepEqual(o.shown, { style: "chart", seen: "above" });
  assert.ok(o.seen.slice(1).every((x) => x.unavailable), "a flat-only view cannot be turned");
  assert.equal(o.turnable, false);
  assert.deepEqual(viewOptions(system, { style: "3d", seen: "free" }).shown, { style: "3d", seen: "free" });
});

test("a kept choice is read back safely", () => {
  assert.deepEqual(readViewChoice('{"style":"3d","seen":"edge"}'), { style: "3d", seen: "edge" });
  assert.deepEqual(readViewChoice('{"style":"sepia","seen":7}'), { style: "chart", seen: "tilted" });
  assert.deepEqual(readViewChoice("oops"), { style: "chart", seen: "tilted" });
});
