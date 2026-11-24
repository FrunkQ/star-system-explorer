// The handle sheet and the strip (ED1): the true inputs of a world and a star, their figures through the units
// engine, the strip's range and reference dots from the catalogued Solar System, and what followed a change.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { constant } from "../src/sheet.ts";
import { readSol } from "../src/sol.ts";
import { HANDLES, RESULTS, handleFigure, handleSubject, handlesFor, handleValue, fieldValue, referenceDots, resultsOf, scaleClass, stretchTo, stripDomain, stripPos, stripStep, stripValue, whatFollowed } from "../src/handles.ts";
import { FIGURE_KINDS } from "../src/figures.ts";

const root = new URL("../../../", import.meta.url);
const sol = readSol(JSON.parse(readFileSync(new URL("packs/base-sol/pack.json", root), "utf8")))!;
const AU = constant("au");

test("a world has six handles, a star its mass and spin, every kind known to the units engine", () => {
  assert.deepEqual(handlesFor("world").map((h) => h.id), ["orbit", "mass", "makeup", "liquids", "air", "spin"]);
  assert.deepEqual(handlesFor("star").map((h) => h.id), ["mass", "spin"]);
  for (const h of HANDLES) assert.ok(FIGURE_KINDS.includes(h.part.kind), `${h.on} ${h.id}: kind ${h.part.kind}`);
  for (const r of RESULTS) if (r.kind) assert.ok(FIGURE_KINDS.includes(r.kind), r.key);
  for (const h of HANDLES) for (const id of [...h.refs, ...(h.moon_refs ?? [])]) assert.ok(sol.bodies.some((b) => b.id === id), `${h.id}: no Sol body ${id}`);
  assert.equal(handleSubject("star"), "star");
  assert.equal(handleSubject("moon"), "world");
  assert.equal(handleSubject("belt"), null);
});

test("figures by scale class", () => {
  assert.equal(scaleClass("planet", constant("M_jup")), "giant");
  assert.equal(scaleClass("planet", constant("M_earth")), "planet");
  const mars = { "orbit.semi_major_axis": 1.524 * AU, "body.mass": 6.4169e23, "body.rotation_period": 88642, "body.obliquity": 0.4396, "body.surface_water_oceans": 0 };
  const [orbit, mass, , liquids, , spin] = handlesFor("world");
  assert.equal(handleFigure(orbit!, mars, "planet")!.main.text, "1.52 AU");
  assert.equal(handleFigure(mass!, mars, "planet")!.main.text, "0.107 M⊕");
  const s = handleFigure(spin!, mars, "planet")!;
  assert.equal(s.main.text, "24.6 h");
  assert.equal(s.also!.text, "25.2°");
  assert.equal(handleValue(liquids!, { "body.surface_water_oceans": 2 }), 2 * constant("M_ocean_earth"));
  assert.equal(fieldValue(liquids!, 2 * constant("M_ocean_earth")), 2);
  assert.equal(handleFigure(orbit!, {}, "planet"), null);
});

test("the strip spans the reference worlds and the value, and stretches for a typed value", () => {
  const orbit = handlesFor("world")[0]!;
  const dots = referenceDots(orbit, sol, "planet");
  assert.deepEqual(dots.map((d) => d.label), ["Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune"]);
  const d = stripDomain(orbit, 1.524 * AU, dots);
  assert.equal(d.scale, "log");
  assert.ok(d.lo < 0.39 * AU && d.hi > 30 * AU);
  for (const v of [0.5 * AU, AU, 10 * AU]) assert.ok(Math.abs(stripValue(d, stripPos(d, v)) / v - 1) < 1e-9);
  assert.ok(stripPos(d, AU) < stripPos(d, 1.524 * AU));
  const far = stretchTo(d, 500 * AU);
  assert.ok(far.hi > 500 * AU && stripPos(far, 500 * AU) < 1);
  const up = stripStep(d, AU, 1);
  assert.ok(up > AU && up < 1.2 * AU);
  assert.ok(stripStep(d, AU, 1, true) > up, "Shift steps further");
});

test("a strip that offers none: no air at its left end", () => {
  const air = handlesFor("world")[4]!;
  const d = stripDomain(air, 0, referenceDots(air, sol, "planet"));
  assert.equal(stripPos(d, 0), 0);
  assert.equal(stripValue(d, 0), 0);
  assert.ok(stripValue(d, 0.5) > 0);
});

test("what followed: moved results said by the units engine, labels and yes or no in words", () => {
  const surfaces = [{ address: "a", t_surface_now_k: 214, gravity_m_s2: 3.71, regime_label: "Cold desert", core: { dynamo_now: false }, steps: [1, 2] }];
  const before = resultsOf(surfaces, "a");
  assert.equal(before["core.dynamo_now"], false);
  assert.equal(before.steps, undefined, "arrays are left out");
  const after = { ...before, t_surface_now_k: 264.16, regime_label: "Temperate", gravity_m_s2: 3.711 };
  assert.deepEqual(whatFollowed(before, after, "planet"), [
    { label: "Surface", before: "214 K", after: "264 K" },
    { label: "Climate", before: "Cold desert", after: "Temperate" },
  ]);
});
