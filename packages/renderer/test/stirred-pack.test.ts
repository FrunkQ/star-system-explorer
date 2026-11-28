// The carry table filed for the GPU (stirred-pack.ts): every eddy that reaches a place at a date is in the list the
// shader reads there, found the way the shader finds it (stirred-gpu.ts `eddySum`), in each row's turning frame.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { unit, wrap, type CarryTable } from "../src/stirred-carry.ts";
import { CELL_DEG, COLS, DAY_S, GRID_D, ROWS, W, packCarry } from "../src/stirred-pack.ts";

const fx = JSON.parse(readFileSync(new URL("../../../tests/fixtures/weather/carry.json", import.meta.url), "utf8")) as { neptune: { table: CarryTable }; saturn: { table: CarryTable } };

/** The list the shader reads at a place and date (days from the table's start), as `eddySum` finds it. */
function listAt(p: ReturnType<typeof packCarry>, lat: number, lon: number, day: number): Set<number> {
  const raw = Math.floor(day / GRID_D);
  if (raw < 0 || raw > p.bins) return new Set();
  const bin = Math.min(raw, p.bins - 1);
  const row = Math.min(Math.max(Math.floor((lat + 90) / CELL_DEG), 0), ROWS - 1);
  const turned = lon - p.rates[row]! * (day - bin * GRID_D);
  const col = Math.min(Math.max(Math.floor((wrap(turned) + 180) / CELL_DEG), 0), COLS - 1);
  const c = bin * ROWS * COLS + row * COLS + col;
  const first = p.grid[c * 2]!;
  const n = p.grid[c * 2 + 1]!;
  return new Set(Array.from(p.list.subarray(first, first + n)));
}

for (const name of ["neptune", "saturn"] as const) {
  test(`${name}: every eddy reaching a place is in its cell's list`, () => {
    const t = fx[name].table;
    const p = packCarry(t);
    assert.ok(p.list.length / W < 8192, "the list fits a GPU's texture");
    const a = t.radius_m;
    let checked = 0;
    let s = 1;
    const rnd = () => ((s = (s * 16807) % 2147483647) / 2147483647);
    for (let k = 0; k < 4000; k++) {
      const lat = Math.asin(2 * rnd() - 1) * (180 / Math.PI);
      const lon = rnd() * 360 - 180;
      const day = t.day0 + rnd() * (t.day1 - t.day0);
      const here = listAt(p, lat, lon, day - t.day0);
      const pp = unit(lat, lon);
      t.eddies.forEach((e, idx) => {
        if (!(day >= e.born && day < e.ends)) return;
        const clon = wrap(e.lon_deg + ((e.drift_m_s * (day - e.born) * DAY_S) / (a * Math.max(Math.cos((e.lat_deg * Math.PI) / 180), 1e-6))) * (180 / Math.PI));
        const c = unit(e.lat_deg, clon);
        const d = Math.acos(Math.min(1, pp[0] * c[0] + pp[1] * c[1] + pp[2] * c[2])) * a;
        if (d > 3 * 0.5 * e.size_km * 1000) return;
        checked++;
        assert.ok(here.has(idx), `${name}: eddy ${idx} reaches ${lat.toFixed(2)}, ${lon.toFixed(2)} on day ${day.toFixed(2)} but is not in its list`);
      });
    }
    assert.ok(checked > 1000, `${checked} reaches checked`);
  });
}
