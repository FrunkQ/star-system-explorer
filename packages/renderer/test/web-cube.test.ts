import { test } from "node:test";
import assert from "node:assert/strict";
import { cubeEdges, webGlow, webParcels } from "../src/web-cube.ts";

test("parcels sort by kind, sit in their own cell, and drop those carried out of the cube", () => {
  const v = {
    centre: [10, 0, 0] as [number, number, number],
    extent: 8,
    scale_factor: 0.5,
    // 8 parcels (2 a side); the last carried far outside.
    position: [9, -2, -2, 12, -2, -2, 9, 2, -2, 12, 2, -2, 9, -2, 2, 12, -2, 2, 9, 2, 2, 40, 2, 2],
    environment: ["void", "void", "sheet", "filament", "knot", "void", "sheet", "knot"],
  };
  const p = webParcels(v);
  assert.equal(p.void.length, 3);
  assert.equal(p.knot.length, 1, "the parcel 30 Mpc outside is left out");
  for (const k of ["void", "sheet", "filament", "knot"] as const)
    for (const q of p[k]) for (const c of q) assert.ok(Math.abs(c) <= 4 * 1.04 + 2, `inside the cube (${q})`);
  // Scattered inside its own cell, the same every time.
  assert.deepEqual(webParcels(v), p);
  const first = p.void[0]!;
  assert.ok(Math.abs(first[0] - -1) <= 2 && Math.abs(first[1] - -2) <= 2);
  // Real shrinks by the factor given.
  const half = webParcels(v, 0.5);
  assert.ok(Math.abs(half.void[0]![0] - first[0] * 0.5) < 1e-9);
});

test("the cube's edges and the light per parcel", () => {
  assert.equal(cubeEdges(1).length, 6);
  assert.ok(webGlow(1000) > webGlow(100000), "fewer parcels, each brighter");
  assert.ok(webGlow(1) <= 0.9 && webGlow(1e9) >= 0.08);
});
