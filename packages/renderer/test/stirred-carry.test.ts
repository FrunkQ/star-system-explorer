// The renderer's carrying (stirred-carry.ts) held to the kernel's (crates/sse-weather/src/stirred.rs) on the
// kernel's own tables and answers (tests/fixtures/weather/carry.json, WS5).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Carrier, frame, generations, latLon, patchPlace, unit, type CarryTable, type Generation } from "../src/stirred-carry.ts";

interface Fixture {
  neptune: { table: CarryTable; patch: { lat_deg: number; lon_deg: number; half_km: number; n: number; day: number; renewal_days: number; cells: [Generation, Generation][] } };
  saturn: { table: CarryTable; day: number; winds: { lat: number; lon: number; east: number; north: number }[] };
}
const fx = JSON.parse(readFileSync(new URL("../../../tests/fixtures/weather/carry.json", import.meta.url), "utf8")) as Fixture;

/** Great-circle distance, km. */
const km = (a: number, la1: number, lo1: number, la2: number, lo2: number) => {
  const p = unit(la1, lo1);
  const q = unit(la2, lo2);
  return (Math.acos(Math.min(1, p[0] * q[0] + p[1] * q[1] + p[2] * q[2])) * a) / 1000;
};

test("the winds match the kernel's to a millimetre a second: bands, the polygon, storms, their shields and the eddies", () => {
  const c = new Carrier(fx.saturn.table);
  assert.ok(fx.saturn.table.poles.some((p) => p.polygon_sides > 0), "Saturn's fixture has its polygon");
  assert.ok(fx.saturn.table.storms.length > 0);
  let worst = 0;
  for (const w of fx.saturn.winds) {
    const v = c.wind(unit(w.lat, w.lon), fx.saturn.day);
    const [e, n] = frame(w.lat, w.lon);
    const east = v[0] * e[0] + v[1] * e[1] + v[2] * e[2];
    const north = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
    const d = Math.hypot(east - w.east, north - w.north);
    worst = Math.max(worst, d);
    assert.ok(d < 1e-3, `at ${w.lat}, ${w.lon}: ${east}, ${north} against ${w.east}, ${w.north}`);
  }
  // Worst about 6e-4 m/s, where the shield's slopes are taken across 1e-4 degrees.
  assert.ok(worst < 1e-3);
});

test("the generations' ages and weights are the kernel's", () => {
  const t = fx.neptune.table;
  const p = fx.neptune.patch;
  p.cells.forEach((cell, k) => {
    const [lat] = patchPlace(t.radius_m, p.lat_deg, p.lon_deg, p.half_km, p.n, k % p.n, Math.floor(k / p.n));
    const g = generations(t, lat);
    assert.ok(Math.abs(g[0][0] - cell[0].age_days) < 1e-9 && Math.abs(g[0][1] - cell[0].weight) < 1e-9, `${k}`);
    assert.ok(Math.abs(g[1][0] - cell[1].age_days) < 1e-9, `${k}`);
  });
});

test("carried as the kernel carries it, each cell's cloud is read where the kernel reads it, to a kilometre", () => {
  const t = fx.neptune.table;
  const p = fx.neptune.patch;
  const c = new Carrier(t);
  p.cells.forEach((cell, k) => {
    const [lat, lon] = patchPlace(t.radius_m, p.lat_deg, p.lon_deg, p.half_km, p.n, k % p.n, Math.floor(k / p.n));
    const g = c.generationsAt(lat, lon);
    for (const i of [0, 1]) {
      const d = km(t.radius_m, g[i]!.lat_deg, g[i]!.lon_deg, cell[i]!.lat_deg, cell[i]!.lon_deg);
      assert.ok(d < 1, `cell ${k} generation ${i}: ${d} km off`);
      assert.ok(Math.abs(g[i]!.stretch_log2 - cell[i]!.stretch_log2) < 0.01, `cell ${k} generation ${i}: stretch ${g[i]!.stretch_log2} against ${cell[i]!.stretch_log2}`);
    }
  });
});

test("in the picture's fixed steps, each cloud lands where the kernel's does, well inside a picture's pixel", () => {
  // The picture follows fixed steps with no halving. A step's small error grows as the map stretches, so the bound
  // grows with it: a kilometre, and a kilometre more for each 64 times the stretch. Both are far inside a 1024-wide
  // face's pixel (about 150 km on Neptune).
  const t = fx.neptune.table;
  const p = fx.neptune.patch;
  const c = new Carrier(t);
  const pixel = (2 * Math.PI * t.radius_m) / 1000 / 1024;
  p.cells.forEach((cell, k) => {
    const [lat, lon] = patchPlace(t.radius_m, p.lat_deg, p.lon_deg, p.half_km, p.n, k % p.n, Math.floor(k / p.n));
    for (const g of cell) {
      const x = c.back(unit(lat, lon), t.day, g.age_days);
      const [rl, ro] = latLon(c.readPlace(x, t.day, g.age_days));
      const d = km(t.radius_m, rl, ro, g.lat_deg, g.lon_deg);
      const bound = Math.min(pixel, Math.max(1, 2 ** g.stretch_log2 / 64));
      assert.ok(d < bound, `cell ${k}, age ${g.age_days}: ${d} km off, bound ${bound} km`);
    }
  });
});
