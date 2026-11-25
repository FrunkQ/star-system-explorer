// "Type it all in" (ED13): every input of a body as a row; a pasted column or named rows; what it understood.
import { test } from "node:test";
import assert from "node:assert/strict";
import { handlesFor } from "../src/handles.ts";
import { readSheet, sheetRows, spreadPaste } from "../src/sheet-entry.ts";
import { constant } from "../src/sheet.ts";

const rows = sheetRows(handlesFor("world"), "science");
const mars = { "orbit.semi_major_axis": 1.524 * constant("au"), "body.mass": 6.417e23, "orbit.eccentricity": 0.0934, "body.rotation_period": 88642 };

test("a world's rows: the handles, then their box parts; no bar, no place-now ring", () => {
  const ids = rows.map((r) => r.id);
  assert.ok(ids.includes("orbit") && ids.includes("mass") && ids.includes("air") && ids.includes("spin"));
  assert.ok(ids.includes("orbit.shape") && ids.includes("orbit.lean"), ids.join(" "));
  assert.ok(!ids.includes("makeup"), "the makeup bar waits for its box");
  assert.ok(!ids.includes("orbit.now"), "the place now is read off the date");
});

test("a paste: named rows go to their row, other lines down from where it started", () => {
  const t = spreadPaste(rows, "Mass\t0.2 M⊕\nShape\t0.05\n\n");
  assert.deepEqual(t, { mass: "0.2 M⊕", "orbit.shape": "0.05" });
  const col = spreadPaste(rows, "1.2 AU\n7e23 kg");
  assert.equal(col[rows[0]!.id], "1.2 AU");
  assert.equal(col[rows[1]!.id], "7e23 kg");
});

test("what it understood, in the body's units, and what it could not read", () => {
  const r = readSheet(rows, { orbit: "1.3 AU", mass: "0.2 Earth masses", spin: "banana", "orbit.shape": "0.0934" }, mars, "planet", "science");
  assert.deepEqual(r.edits.map((e) => e.field), ["orbit.semi_major_axis", "body.mass"], "an unchanged value writes nothing");
  assert.ok(Math.abs(r.edits[0]!.value - 1.3 * constant("au")) < 1);
  assert.match(r.understood[0]!.after, /1\.3 AU/);
  assert.equal(r.problems.length, 1);
  assert.equal(r.problems[0]!.row, "spin");
});

test("the system sheet: a pasted table, columns matched by name with a bracketed unit, bodies by name", async () => {
  const { readTable } = await import("../src/sheet-entry.ts");
  const bodies = [
    { address: "a/mars", label: "Mars", facts: mars, cls: "planet" },
    { address: "a/earth", label: "Earth", facts: { "body.mass": 5.972e24 }, cls: "planet" },
  ];
  const t = "Name\tOrbit (AU)\tMass\tColour\nMars\t1.6\t0.2 M⊕\tred\nEarth\t1\t-\nVulcan\t0.1\t";
  const r = readTable(t, () => rows, bodies, "science");
  assert.deepEqual(r.columns.map((c) => c.row), ["orbit", "mass", null]);
  assert.deepEqual(r.edits.map((e) => `${e.address} ${e.field}`), ["a/mars orbit.semi_major_axis", "a/mars body.mass", "a/earth orbit.semi_major_axis"]);
  assert.ok(Math.abs(r.edits[2]!.value - constant("au")) < 1);
  assert.ok(r.problems.some((p) => /Colour/.test(p.where)));
  assert.ok(r.problems.some((p) => /Vulcan/.test(p.where)));
});
