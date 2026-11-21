// Depth (top-level design 2.4): Curious says numbers in words, rounded honestly.
import { test } from "node:test";
import assert from "node:assert/strict";
import { curiousDisplay, curiousSentence, headline, isDepth, roughly } from "../src/depth.ts";

test("roughly: two significant figures, grouped", () => {
  assert.equal(roughly(0.524), "0.52");
  assert.equal(roughly(5772), "5,800");
  assert.equal(roughly(12.4), "12");
  assert.equal(roughly(0), "0");
});

test("Curious numbers: °C for temperatures a person could stand in, hours, days, years", () => {
  assert.equal(curiousDisplay({ display: "288 K", unit: "K" }), "about 15 °C");
  assert.equal(curiousDisplay({ display: "5772 K", unit: "K" }), "about 5,800 K");
  assert.equal(curiousDisplay({ display: "31.4 h", unit: "h" }), "about 31 hours");
  assert.equal(curiousDisplay({ display: "151 d", unit: "d" }), "about 150 days");
  assert.equal(curiousDisplay({ display: "4332 d", unit: "d" }), "about 12 years");
  assert.equal(curiousDisplay({ display: "0.52 AU", unit: "AU" }), "about 0.52 AU");
  assert.equal(curiousDisplay({ display: "Tarn" }), "Tarn", "text stays text");
  assert.equal(curiousDisplay({ display: "0.3" }), "about 0.3");
});

test("Curious sentence: the body's prose, else one made from what it is", () => {
  assert.equal(curiousSentence({ kind: "planet", prose: "Cold seas." }), "Cold seas.");
  assert.equal(curiousSentence({ kind: "planet", subtitle: "III · ice giant", parent: "s", orbit: { a_au: 19.2, phase_deg: 0 } }, "the Sun"), "An ice giant. It goes round the Sun at about 19 AU.");
  assert.equal(curiousSentence({ kind: "moon", subtitle: "rocky moon", parent: "p", orbit: { a_au: 0.002, phase_deg: 0 } }, "Earth"), "A rocky moon. It goes round Earth.");
  assert.equal(curiousSentence({ kind: "star" }), "A star.");
  assert.ok(isDepth("engine") && !isDepth("basic"));
});

test("Curious leads with orbit, warmth, day and year, then mass", () => {
  const f = ["a_au", "eccentricity", "mass_earth", "radius_earth", "albedo", "surface_k", "day_h", "year_d"].map((field) => ({ field }));
  assert.deepEqual(headline(f).map((x) => x.field), ["a_au", "surface_k", "day_h", "year_d"]);
  assert.deepEqual(headline([{ field: "albedo" }, { field: "mass_sun" }]).map((x) => x.field), ["mass_sun", "albedo"]);
});
