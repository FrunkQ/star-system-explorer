// The stepper on the Run desk (TM7, three desks design 3.2): every level drawn, Out and In.
import { test } from "node:test";
import assert from "node:assert/strict";
import { stepperLine, stepIn, stepOut } from "../src/stepper.ts";
import { PLACE_SCALES, placeSteps, type Place } from "../src/place.ts";
import { partShows } from "../src/desk.ts";

const names = { system: "Kestrel" };

test("every level is drawn; the ones not visited read as the level's word", () => {
  const here: Place = { scale: "system", system: "" };
  const line = stepperLine(placeSteps(here, null, names));
  assert.deepEqual(line.map((s) => s.scale), [...PLACE_SCALES]);
  assert.deepEqual(line.map((s) => s.label), ["Cosmic web", "Galaxies nearby", "Galaxy", "Kestrel", "World"]);
  assert.deepEqual(line.map((s) => s.unvisited), [false, false, false, false, true]);
  assert.equal(line.find((s) => s.current)?.scale, "system");
});

test("a world the trail remembers is pencilled with its name, and In goes back to it", () => {
  const world: Place = { scale: "world", system: "", world: "u:k/b:tarn" };
  const here: Place = { scale: "system", system: "" };
  const steps = placeSteps(here, world, { system: "Kestrel", world: "Tarn" });
  const line = stepperLine(steps);
  assert.deepEqual(line.at(-1), { scale: "world", label: "Tarn", place: world, current: false, below: true, unvisited: false });
  assert.deepEqual(stepIn(steps, here, null), world);
});

test("In: the thing selected at a system, the level's way in above it, nothing at a world", () => {
  const sys: Place = { scale: "system", system: "s1" };
  assert.equal(stepIn(placeSteps(sys, null, names), sys, null), null, "a system with nothing selected");
  assert.deepEqual(stepIn(placeSteps(sys, null, names), sys, "u:k/b:ember"), { scale: "world", system: "s1", world: "u:k/b:ember" });
  const web: Place = { scale: "web", system: "" };
  assert.deepEqual(stepIn(placeSteps(web, null, names), web, null), { scale: "cluster", system: "" });
  const galaxy: Place = { scale: "galaxy", system: "" };
  assert.deepEqual(stepIn(placeSteps(galaxy, null, names), galaxy, null), { scale: "system", system: "" });
  const w: Place = { scale: "world", system: "", world: "u:k/b:tarn" };
  assert.equal(stepIn(placeSteps(w, null, names), w, "u:k/b:tarn"), null, "Site is not a level yet");
});

test("Out: the level above, none at the top", () => {
  const sys: Place = { scale: "system", system: "" };
  assert.equal(stepOut(placeSteps(sys, null, names))?.scale, "galaxy");
  const web: Place = { scale: "web", system: "" };
  assert.equal(stepOut(placeSteps(web, null, names)), null);
});

test("the stepper is the Run desk's form of the place line", () => {
  assert.equal(partShows("stepper", "run"), true);
  assert.equal(partShows("stepper", "explore"), false);
  assert.equal(partShows("stepper", "build"), false);
});
