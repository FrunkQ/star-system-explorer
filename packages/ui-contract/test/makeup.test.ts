// The Makeup box's bulk bar (ED9 part 1): five shares by mass that always sum to one; kinds from the class atlas.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { makeupEdits, makeupKinds, makeupOf, retradeMakeup } from "../src/makeup.ts";

const earth = { "makeup.metal": 0.32, "makeup.rock": 0.68, "body.mass": 5.97e24 };
const sum = (m: { share: number }[]) => m.reduce((t, g) => t + g.share, 0);

test("the makeup reads the five parts in the bar's order, every part listed", () => {
  assert.deepEqual(makeupOf(earth).map((g) => [g.gas, g.share]), [["metal", 0.32], ["rock", 0.68], ["carbon", 0], ["ice", 0], ["gas", 0]]);
  assert.deepEqual(makeupOf({ "body.mass": 1 }), []);
});

test("setting one part trades it against the rest, which keep their proportions; the sum stays one", () => {
  const r = retradeMakeup(makeupOf(earth), "ice", 0.5);
  assert.ok(Math.abs(sum(r) - 1) < 1e-12);
  assert.deepEqual(r.map((g) => g.gas), ["metal", "rock", "carbon", "ice", "gas"]);
  const get = (p: string) => r.find((g) => g.gas === p)!.share;
  assert.equal(get("ice"), 0.5);
  assert.ok(Math.abs(get("metal") / get("rock") - 0.32 / 0.68) < 1e-9, "metal to rock as before");
  // One edit per field that moved; a part never set before goes in with null before it.
  const e = makeupEdits(earth, r);
  assert.deepEqual(e.map((x) => x.field), ["makeup.metal", "makeup.rock", "makeup.ice"]);
  assert.equal(e.find((x) => x.field === "makeup.ice")!.before, null);
});

test("a world with none set starts from Earth's bulk and writes all five", () => {
  const r = retradeMakeup([], "metal", 0.7);
  assert.ok(Math.abs(sum(r) - 1) < 1e-12);
  assert.equal(r.find((g) => g.gas === "metal")!.share, 0.7);
  assert.equal(makeupEdits({}, r).length, 5);
});

test("kinds come from the class atlas's named examples, one per top-level physical class", () => {
  const atlas = JSON.parse(readFileSync(new URL("../../../packs/base-atlas/pack.json", import.meta.url), "utf8"));
  const k = makeupKinds([atlas]);
  const labels = k.map((x) => x.label);
  for (const want of ["Like Mercury", "Like Venus", "Like Ceres", "Like Uranus", "Like Jupiter"]) assert.ok(labels.includes(want), `${want} in ${labels.join(", ")}`);
  assert.equal(new Set(labels).size, labels.length, "each example once");
  for (const x of k) assert.ok(Math.abs(sum(x.makeup) - 1) < 1e-9, `${x.label} sums to one`);
  assert.equal(k.find((x) => x.label === "Like Mercury")!.makeup.find((g) => g.gas === "metal")!.share, 0.7);
});
