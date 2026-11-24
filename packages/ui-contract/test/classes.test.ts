// The class gallery (V3 gaps item 35): class atlases read from pack data into schemes, trees, bands in words and
// example worlds. Pure: no kernel.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { classNumber, classBandWords, className, classTree, classesOnStack, findClass, findClasses, needsWords } from "../src/classes.ts";
import type { PackDoc } from "../src/physics.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const physics: PackDoc = json("packs/base-physics/pack.json");
const atlasPack: PackDoc = json("packs/base-atlas/pack.json");
const classic: PackDoc = json("packs/sse3-classic/pack.json");

test("a stack with no atlas has no classes; the SSE4 atlas has its four schemes", () => {
  assert.deepEqual(classesOnStack([physics]).schemes, []);
  const a = classesOnStack([physics, atlasPack]);
  assert.deepEqual(
    a.schemes.map((s) => s.id),
    ["physical", "surface", "air", "habitability"],
  );
  assert.equal(a.schemes[0]!.question, "What kind of object is this?");
  assert.equal(a.schemes.reduce((n, s) => n + s.classes.length, 0), 69);
  assert.equal(a.axes.get("teq")?.unit, "K");
});

test("a scheme's base classes form a tree under their parents, modifiers apart", () => {
  const a = classesOnStack([atlasPack]);
  const t = classTree(a.schemes.find((s) => s.id === "physical")!);
  const at = (id: string) => t.base.findIndex((n) => n.cls.id === id);
  assert.ok(at("gas-giant") >= 0 && at("gas-giant/ammonia-clouds") > at("gas-giant"), "a child follows its parent");
  assert.equal(t.base[at("gas-giant/ammonia-clouds")]!.depth, 1);
  assert.equal(t.base[at("gas-giant")]!.depth, 0);
  assert.ok(t.modifiers.some((c) => c.id === "physical/ringed"));
  assert.ok(t.base.every((n) => n.cls.kind !== "modifier"));
  assert.equal(new Set(t.base.map((n) => n.cls.id)).size, t.base.length, "each class once");
});

test("bands read in words with their units and the fringe they fade out over", () => {
  const a = classesOnStack([atlasPack]);
  const amm = findClass(a, "gas-giant/ammonia-clouds")!;
  assert.equal(className(amm), "Ammonia clouds");
  assert.equal(classBandWords(amm.region[0]!, a.axes), "equilibrium temperature up to 150 K (fades out to 170 K)");
  assert.equal(classBandWords(amm.region[0]!, a.axes, false), "equilibrium temperature up to 150 K");
  assert.deepEqual(
    amm.examples.filter((e) => e.in).map((e) => e.name),
    ["Jupiter", "Saturn"],
  );
  const ringed = findClass(a, "physical/ringed")!;
  assert.equal(classBandWords(ringed.region[0]!, a.axes), "has rings");
  const rogue = findClass(a, "physical/rogue")!;
  assert.equal(classBandWords(rogue.region[0]!, a.axes), "what it orbits: none");
  const sleeve = findClass(a, "habitable/shirt-sleeve")!;
  assert.ok(sleeve.examples.some((e) => !e.in), "near misses are kept apart from examples");
  const temperate = findClass(a, "surface/temperate-ocean")!;
  assert.match(classBandWords(temperate.region.find((b) => b.axis === "liquid_fraction")!, a.axes), /^surface liquid cover 10% to 90% \(fades out from 5% and to 95%\)$/);
  assert.ok(temperate.gate.length, "the surface scheme's gate is carried");
  assert.equal(findClass(a, "physical/terraformed")!.evidence, "fictional");
  assert.equal(needsWords("authored:settlement"), "a settlement the GM or a pack puts there");
});

test("numbers in bands stay short", () => {
  assert.equal(classNumber(150), "150");
  assert.equal(classNumber(0.0045), "0.0045");
  assert.equal(classNumber(1e-15), "10⁻¹⁵");
  assert.equal(classNumber(2.5e7), "2.5×10⁷");
});

test("SSE 3's classic list reads as one scheme; find looks in names, summaries and example worlds", () => {
  const c = classesOnStack([classic]);
  assert.deepEqual(
    c.schemes.map((s) => s.id),
    ["sse3-classic"],
  );
  assert.equal(c.schemes[0]!.classes.length, 72);
  const a = classesOnStack([atlasPack]);
  assert.ok(findClasses(a, "jupiter").some((x) => x.id === "gas-giant/ammonia-clouds"));
  assert.ok(findClasses(a, "rings").some((x) => x.id === "physical/ringed"));
  assert.deepEqual(findClasses(a, "  "), []);
  // A later pack's class of the same id replaces the earlier.
  const later: PackDoc = { ...atlasPack, id: "later", provides: { other: { classes: [{ id: "rocky", scheme: "physical", names: { plain: "stony world" } }] } } };
  const both = classesOnStack([atlasPack, later]);
  assert.equal(findClass(both, "rocky")!.names.plain, "stony world");
  assert.equal(findClass(both, "rocky")!.pack, "later");
  assert.equal(both.schemes.reduce((n, s) => n + s.classes.length, 0), 69);
});
