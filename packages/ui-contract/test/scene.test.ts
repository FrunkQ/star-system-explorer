// Frames of the system chart: orbits are local to their parent
// (handover/rendering-rules.md), so a moon is listed under its planet,
// never under the star.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { childrenOf, frameFor, primaryOf, systemOrder, type SystemScene } from "../src/index.ts";

const scene: SystemScene = JSON.parse(readFileSync(new URL("../../shell/src/documents/kestrel.scene.json", import.meta.url), "utf8"));
const named = (n: string) => scene.subjects.find((s) => s.label === n)!.address;

test("the star's frame holds only its planets", () => {
  const star = primaryOf(scene)!;
  const kids = childrenOf(scene, star.address);
  assert.deepEqual(kids.map((s) => s.label), ["Ember", "Sable", "Tarn", "Vault", "Hollow"]);
  assert.ok(kids.every((s) => s.kind === "planet"));
});

test("a planet's frame holds its moons, nearest first", () => {
  assert.deepEqual(childrenOf(scene, named("Vault")).map((s) => s.label), ["Hasp", "Rivet", "Latch"]);
  assert.deepEqual(childrenOf(scene, named("Tarn")).map((s) => s.label), ["Tarn a"]);
  assert.deepEqual(childrenOf(scene, named("Ember")), []);
});

test("tree order puts each planet's moons straight after it", () => {
  const order = systemOrder(scene).map((s) => s.label);
  assert.equal(order[0], primaryOf(scene)!.label);
  const v = order.indexOf("Vault");
  assert.deepEqual(order.slice(v, v + 4), ["Vault", "Hasp", "Rivet", "Latch"]);
  assert.equal(order.length, scene.subjects.filter((s) => s.kind === "star" || s.orbit).length);
});

test("a moon is drawn in its planet's frame, a planet in the star's", () => {
  assert.equal(frameFor(scene, named("Hasp")), named("Vault"));
  assert.equal(frameFor(scene, named("Vault")), null);
  assert.equal(frameFor(scene, primaryOf(scene)!.address), null);
  assert.equal(frameFor(scene, null), null);
});
