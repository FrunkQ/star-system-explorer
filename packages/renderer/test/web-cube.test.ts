import { test } from "node:test";
import assert from "node:assert/strict";
import { cubeEdges, galaxyLight, webGlow, webLights, webParcels } from "../src/web-cube.ts";

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

test("galaxies shine by their stars and star formation, blue when young", () => {
  const old = galaxyLight({ stars_msun: 1e11, sfr_msun_yr: 0 });
  const young = galaxyLight({ stars_msun: 1e9, sfr_msun_yr: 5 });
  assert.ok(Math.abs(old.l_sun - 4e10) < 1, "old light, 0.4 L☉ per M☉");
  assert.equal(old.colour_k, 4800);
  assert.ok(young.colour_k > 12000, "a starburst is blue");
  assert.equal(galaxyLight({ stars_msun: 0, sfr_msun_yr: 0 }).l_sun, 0);
});

test("galaxies group by brightness and colour, the faint drop below the shown decades, the far-flung left out", () => {
  const cube = { centre: [0, 0, 0] as [number, number, number], extent: 100 };
  const g = (x: number, stars: number, sfr = 0) => ({ position: [x, 0, 0] as [number, number, number], halo_msun: stars * 30, stars_msun: stars, sfr_msun_yr: sfr, quenched: sfr === 0 });
  const groups = webLights([g(0, 1e12), g(10, 1e11), g(20, 1e10, 1), g(-30, 1e6), g(80, 1e12)], cube);
  const pts = groups.flatMap((x) => x.points);
  assert.equal(pts.length, 3, "the 1e6 galaxy is too faint, the one at 80 Mpc moved out");
  const bright = groups.find((x) => x.points.some((p) => p[0] === 0))!;
  const dim = groups.find((x) => x.points.some((p) => p[0] === 10))!;
  assert.ok(bright.glow > dim.glow && bright.size > dim.size);
  const blue = groups.find((x) => x.points.some((p) => p[0] === 20))!;
  assert.ok(blue.colour_k > bright.colour_k);
  assert.deepEqual(webLights([g(0, 1e12), g(10, 1e11)], cube, 0.5).flatMap((x) => x.points).map((p) => p[0]).sort((a, b) => a - b), [0, 5]);
  assert.deepEqual(webLights([], cube), []);
});
