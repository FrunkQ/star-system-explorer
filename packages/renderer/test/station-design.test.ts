// Stations from their description (Alex, 24 Nov): one shape at every size, docks by size, the megastructure line read
// off the physics, the same recipe the same station anywhere.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { designStation, hoopNeed, pickShape, stationPackData, stationViews } from "../src/station-design.ts";
import { stationMesh } from "../src/station-mesh.ts";
import { constant } from "../../ui-contract/src/index.ts";

const ships = JSON.parse(readFileSync(new URL("../../../packs/base-ships/pack.json", import.meta.url), "utf8")).provides.other;
const tech = JSON.parse(readFileSync(new URL("../../../packs/base-tech/pack.json", import.meta.url), "utf8")).provides.other;
const d = stationPackData(ships, tech)!;
const build = (population: number, shape: string, more: Record<string, unknown> = {}, seed = 1) => designStation({ name: "Test", population, ...more }, d.rules, d.shapes, d.limits, { shape, seed });

test("about two dozen shapes, on every place, each building without error", () => {
  const ids = Object.keys(d.shapes);
  assert.ok(ids.length >= 24, `${ids.length} shapes`);
  for (const place of ["space", "ground"]) assert.ok(ids.some((i) => d.shapes[i]!.place === place), place);
  for (const id of ids) {
    const l = build(1000, id, { place: d.shapes[id]!.place });
    assert.equal(l.shape, id);
    assert.ok(l.parts.length > 0 && Number.isFinite(l.size_m) && l.size_m > 0, id);
    const m = stationMesh(l, { segments: 12, panel_m: 4 });
    assert.ok(m.triangles > 0, `${id} has a model`);
    const v = stationViews(l, [560, 240]);
    assert.ok(v.top.paths.length > 0 && v.side.paths.every((p) => !p.d.includes("NaN")), `${id} plans`);
  }
});

test("a 30-person ring and a 20-million ring are the same shape, one far bigger with more bolt-ons and berths", () => {
  const small = build(30, "ring");
  const big = build(2e7, "ring");
  assert.equal(small.shape, big.shape);
  assert.ok(big.size_m > 100 * small.size_m, `${small.size_m} m against ${big.size_m} m`);
  assert.ok(big.parts.length > small.parts.length, "more parts");
  const berths = (l: typeof small) => l.docks.reduce((a, x) => a + x.count, 0);
  assert.ok(berths(big) > 100 * berths(small));
  assert.ok(big.docks.length > small.docks.length, "bigger berths open up on a bigger station");
});

test("spin: a few people get the smallest radius a person can bear; a crowd sets its own", () => {
  const small = build(3, "ring");
  const g = constant("g0");
  const omega = (d.limits.spin_rpm * 2 * Math.PI) / 60;
  assert.ok(Math.abs(small.spin!.radius_m - g / omega ** 2) < 1e-6);
  assert.ok(Math.abs(small.spin!.rpm - d.limits.spin_rpm) < 1e-9);
  const big = build(2e7, "ring");
  assert.ok(big.spin!.rpm < d.limits.spin_rpm);
  assert.ok(Math.abs(big.spin!.g - 1) < 1e-9);
});

test("the megastructure line is the hull's strength: ordinary steel holds a small ring, not a 20-million one", () => {
  assert.equal(build(30, "ring").mega, null);
  const big = build(2e7, "ring");
  assert.ok(big.mega, "a 20-million ring is a megastructure");
  assert.ok(!big.mega!.fictional && big.mega!.needs, "a stronger material could carry it");
  // Where the line sits: the ring radius at which the hull needs more than the best built material.
  const ordinary = d.limits.strength.find((m) => m.mark === "built")!.v;
  assert.ok(hoopNeed(constant("g0"), big.spin!.radius_m, d.rules) > ordinary);
  // Past the carbon bond: fictional.
  const absurd = build(1e15, "ring");
  assert.ok(absurd.mega?.fictional, "past every material it is marked fictional");
});

test("a tower on the ground: its height against what its material stands", () => {
  assert.equal(build(1e4, "arcology", { place: "ground" }).mega, null);
  assert.ok(build(1e9, "arcology", { place: "ground" }).mega, "a tower for a billion is past steel");
});

test("its own gravity: a heavy enough construct is a body", () => {
  const l = build(500, "fortress_ball", { mass_kg: 1e14 });
  assert.match(l.mega?.why ?? "", /own gravity/);
});

test("the description picks the shape", () => {
  const pick = (words: string, tags: string[] = [], place = "space") => pickShape({ name: "x", population: 100, words, tags, place: place as "space" }, d.shapes);
  assert.equal(pick("a hollowed rock with an unlisted berth and no questions asked"), "hollow_rock");
  assert.equal(pick("a ring station where any ship may berth"), "ring");
  assert.equal(pick("a small, isolated laboratory"), "truss");
  assert.equal(pick("an old derelict hulk"), "derelict_ship");
  assert.equal(pick("an arcology tower", [], "ground"), "arcology");
  assert.equal(pick("", [], "ground"), "dome", "no words: the first shape for the place");
});

test("the same recipe gives the same station; a remix changes only the arrangement", () => {
  const a = build(5000, "cluster", { modules: ["labs", "hangar"] }, 3);
  const b = build(5000, "cluster", { modules: ["labs", "hangar"] }, 3);
  assert.deepEqual(a, b);
  const c = build(5000, "cluster", { modules: ["labs", "hangar"] }, 4);
  assert.equal(c.volume_m3, a.volume_m3);
  assert.deepEqual(c.docks, a.docks);
  assert.ok(a.parts.some((p) => p.label === "labs") && a.parts.some((p) => p.label === "hangar"), "each listed module is a bolt-on");
});
