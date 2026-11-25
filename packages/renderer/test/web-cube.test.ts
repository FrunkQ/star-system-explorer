import { test } from "node:test";
import assert from "node:assert/strict";
import { cubeEdges, densityLevel, galaxyLight, mixCss, webGlow, webLights, webMarks, webParcelLevels, webParcels } from "../src/web-cube.ts";

test("parcels sort by kind, sit in their own deformed cell, and drop those carried out of the cube", () => {
  const v = {
    centre: [10, 0, 0] as [number, number, number],
    extent: 8,
    scale_factor: 0.5,
    // 8 parcels (2 a side) on their starting grid; the last carried far outside.
    position: [8, -2, -2, 12, -2, -2, 8, 2, -2, 12, 2, -2, 8, -2, 2, 12, -2, 2, 8, 2, 2, 40, 2, 2],
    environment: ["void", "void", "sheet", "filament", "knot", "void", "sheet", "knot"],
  };
  const p = webParcels(v);
  assert.equal(p.void.length, 3);
  assert.equal(p.knot.length, 1, "the parcel 30 Mpc outside is left out");
  for (const k of ["void", "sheet", "filament", "knot"] as const)
    for (const q of p[k]) for (const c of q) assert.ok(Math.abs(c) <= 4 * 1.04, `inside the cube (${q})`);
  // Scattered inside its own cell, the same every time.
  assert.deepEqual(webParcels(v), p);
  const first = p.void[0]!;
  assert.ok(Math.abs(first[0] - -2) <= 2 && Math.abs(first[1] - -2) <= 2);
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

test("parcels are inked by their density today and drawn in the shape the step gave their cell", () => {
  assert.equal(densityLevel(2), 0, "spread out: under the mean");
  assert.equal(densityLevel(0.5), 1);
  assert.equal(densityLevel(0.2), 2);
  assert.equal(densityLevel(0.05), 3);
  assert.equal(densityLevel(-0.05), 3, "past crossing, the size of the room counts");
  // 27 parcels in a cube 30 across (cells of 10), squeezed to a tenth along x: a wall, thin in x and wide in y and z.
  const n = 27;
  const position: number[] = [];
  for (let k = 0; k < 3; k++) for (let j = 0; j < 3; j++) for (let i = 0; i < 3; i++) position.push((i - 1) * 1, (j - 1) * 10, (k - 1) * 10);
  const v = { centre: [0, 0, 0] as [number, number, number], extent: 30, scale_factor: 1, position, environment: Array.from({ length: n }, () => "sheet"), room: Array.from({ length: n }, () => 0.1) };
  const groups = webParcelLevels(v);
  assert.equal(groups.length, 1);
  assert.equal(groups[0]!.level, 3, "ten times the mean");
  const pts = groups[0]!.points;
  const spread = (k: number) => Math.max(...pts.map((p) => p[k]!)) - Math.min(...pts.map((p) => p[k]!));
  assert.ok(spread(0) < 3, `thin across the wall (${spread(0)})`);
  assert.ok(spread(1) > 20 && spread(2) > 20, "wide along it");
  // A grid it cannot read (not a cube number) falls back to the plain cell.
  assert.equal(webParcelLevels({ ...v, position: position.slice(0, 3 * 5), environment: v.environment.slice(0, 5), room: v.room.slice(0, 5) }).flatMap((g) => g.points).length > 0, true);
});

test("colours mix, and the big galaxies are marked by their halo", () => {
  assert.equal(mixCss("#000", "rgb(200, 100, 50)", 0.5), "rgb(100, 50, 25)");
  assert.equal(mixCss("#ffffff", "#000000", 2), "rgb(0, 0, 0)");
  const g = (m: number, x = 0) => ({ position: [x, 0, 0] as [number, number, number], halo_msun: m, stars_msun: m / 50, sfr_msun_yr: 0, quenched: true });
  const marks = webMarks([g(5e11), g(2e12), g(3e13), g(4e14), g(4e14, 90)], { centre: [0, 0, 0], extent: 100 });
  assert.deepEqual(marks.map((m) => [m.name, m.points.length]), [["galaxy", 1], ["group", 1], ["cluster", 1]]);
  assert.ok(marks[2]!.size > marks[0]!.size);
});
