import { test } from "node:test";
import assert from "node:assert/strict";
import { layoutSystems, mapPoint, mapUnit, readCampaignSystems, mapScaleBar } from "../src/your-systems.ts";
import type { SystemResolved } from "../src/make.ts";

test("a place comes from {x, y, z} or [x, y, z]; anything else is no place", () => {
  assert.deepEqual(mapPoint({ x: 1, y: -2, z: 3 }), { x: 1, y: -2, z: 3 });
  assert.deepEqual(mapPoint({ x: 1, y: 2 }), { x: 1, y: 2, z: 0 });
  assert.deepEqual(mapPoint([4, 5]), { x: 4, y: 5, z: 0 });
  for (const bad of [null, undefined, "1,2", { x: "1", y: 2 }, [1], { x: NaN, y: 0 }]) assert.equal(mapPoint(bad), null);
});

test("layout keeps one scale on both axes, y up, a lone system in the middle", () => {
  const { marks, unitsPerPx } = layoutSystems(
    [
      { system: "a", position: { x: 0, y: 0, z: 0 } },
      { system: "b", position: { x: 10, y: 5, z: 2 } },
      { system: "c", position: null },
    ],
    200,
    20,
  );
  assert.equal(marks.length, 2, "the unplaced one is left out, never put somewhere made up");
  const [a, b] = marks;
  assert.equal(unitsPerPx, 10 / 160);
  assert.equal(a!.px, 20);
  assert.equal(b!.px, 180);
  assert.ok(b!.py < a!.py, "y up on the page");
  assert.equal(a!.py - b!.py, 80, "the same scale on y as on x");
  assert.equal(b!.pz, 32);
  const lone = layoutSystems([{ system: "s", position: { x: 7, y: 7, z: 0 } }], 200);
  assert.deepEqual([lone.marks[0]!.px, lone.marks[0]!.py], [100, 100]);
});

test("scale bar picks a round 1, 2 or 5", () => {
  assert.deepEqual(mapScaleBar(0.1, 100), { units: 10, px: 100 });
  assert.equal(mapScaleBar(0.37, 100).units, 50);
  assert.equal(mapScaleBar(0.013, 100).units, 1);
});

test("the loader reads each system back, counts it, and keeps one it cannot read by name", async () => {
  const kestrel: SystemResolved = {
    type: "system_resolved",
    system: "u/sys:1",
    source: "v3",
    facts: {
      u: { "v3.distanceUnit": "ly" },
      "u/sys:1": { "system.position_map_units": { x: 3, y: 4, z: 0 } },
      "u/sys:1/star:1": { "body.kind": "star", "body.mass": 2e30, "v3.name": "Kestrel" },
      "u/sys:1/planet:1": { "body.mass": 6e24, "orbit.semi_major_axis": 1.5e11, "v3.name": "Tarn" },
    },
    surfaces: [],
    tiles: [],
    refused: [],
  };
  const r = await readCampaignSystems({
    listSystems: async () => ({ ok: true, systems: [{ system: "u/sys:1", source: "v3", name: "Kestrel", addresses: [] }, { system: "u/sys:2", source: "facts", name: "Lost", addresses: [] }] }),
    resolvedSystem: async (s) => (s === "u/sys:1" ? { ok: true, result: kestrel } : { ok: false, code: "no", message: "gone" }),
  });
  assert.ok(r.ok);
  if (!r.ok) return;
  const [k, lost] = r.systems;
  assert.deepEqual(k!.position, { x: 3, y: 4, z: 0 });
  assert.equal(k!.unit, "ly");
  assert.equal(k!.summary.stars, 1);
  assert.equal(k!.summary.planets, 1);
  assert.equal(lost!.title, "Lost");
  assert.equal(lost!.position, null);
  assert.equal(mapUnit({ facts: {} }), null);
});

test("routes join systems from the record's links; links inside one system or to nowhere are left out", async () => {
  const { systemOf, systemRoutes, gridLines } = await import("../src/your-systems.ts");
  const sys = ["u/sys:1", "u/sys:2", "u/sys:10"];
  assert.equal(systemOf("u/sys:10/planet:3", sys), "u/sys:10", "sys:1 does not claim sys:10's bodies");
  assert.equal(systemOf("u/elsewhere", sys), null);
  const routes = systemRoutes(
    [
      { id: "l1", a: "u/sys:1/planet:1", b: "u/sys:2", name: "the Spinward Gate", one_way: true },
      { id: "l2", a: "u/sys:2/moon:1", b: "u/sys:1/planet:2", kind: "jump lane" },
      { id: "l3", a: "u/sys:10/planet:1", b: "u/sys:10/planet:2", kind: "tether" },
      { id: "l4", a: "u/sys:10", b: "u/void", kind: "lane" },
      { id: "l5", a: "u/sys:10", b: "u/sys:1", one_way: true, kind: "slipstream" },
    ],
    sys,
  );
  assert.equal(routes.length, 2);
  const [r12, r101] = routes;
  assert.deepEqual([r12!.a, r12!.b, r12!.one_way, r12!.words, r12!.links], ["u/sys:1", "u/sys:2", false, ["the Spinward Gate", "jump lane"], ["l1", "l2"]]);
  assert.deepEqual([r101!.a, r101!.b, r101!.one_way], ["u/sys:10", "u/sys:1", true], "a one-way route keeps its direction");
  assert.deepEqual(gridLines(100, 25), [0, 25, 50, 75, 100]);
  assert.deepEqual(gridLines(100, 0), []);
});
