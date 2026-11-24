import { test } from "node:test";
import assert from "node:assert/strict";
import { LIKE_THE_MILKY_WAY, cellsAround, findHomes, homeFits } from "../src/home-finder.ts";
import type { CellHalo, HaloCell } from "../src/galaxy.ts";

const halo = (key: string, mass: number, last: number | null, n: CellHalo["neighbour"]): CellHalo => ({ address: `u:k/cell:0.0.0/halo:${key}`, key, mass_msun: mass, position_mpc: [0, 0, 0], half_mass_gyr: 5, spin: 0.04, mergers: 3, major_mergers: last === null ? 0 : 1, last_major_gyr: last, neighbour: n });
const bound = (d: number) => ({ address: "u:k/cell:0.0.0/halo:9", distance_mpc: d, bound: true, approach_gyr: 4.5 });
const cell = (index: [number, number, number], haloes: CellHalo[]): HaloCell => ({ address: "c", index, side_mpc: 10, origin_mpc: [0, 0, 0], delta_l: 0, mass_msun: 0, halo_mass_msun: 0, total: haloes.length, above_min: haloes.length, min_mass_msun: 1e10, haloes, bins: [], now_gyr: 13.8, model: [] });

test("a Milky Way like home: the right mass, quiet for 8 Gyr, a bound neighbour within 1 Mpc", () => {
  const mw = LIKE_THE_MILKY_WAY;
  const yes = homeFits(halo("1", 1.2e12, 3.5, bound(0.78)), mw, 13.8);
  assert.ok(yes);
  assert.match(yes.join("; "), /quiet for 10\.3 Gyr.*bound neighbour 0\.78 Mpc/);
  assert.equal(homeFits(halo("2", 5e12, 3.5, bound(0.78)), mw, 13.8), null, "too heavy");
  assert.equal(homeFits(halo("3", 1e12, 9, bound(0.78)), mw, 13.8), null, "a major merger 4.8 Gyr ago");
  assert.equal(homeFits(halo("4", 1e12, 3, null), mw, 13.8), null, "no neighbour");
  assert.equal(homeFits(halo("5", 1e12, 3, { ...bound(0.7), bound: false }), mw, 13.8), null, "a neighbour passing, not bound");
  assert.ok(homeFits(halo("6", 1e12, null, bound(0.5)), mw, 13.8)?.includes("no major merger on record"));
  assert.ok(homeFits(halo("7", 1e12, 9, null), { ...mw, quiet_gyr: 0, neighbour: "alone" }, 13.8), "alone and busy, when that is the wish");
});

test("homes across cells, the one nearest the middle of the mass range first", () => {
  const found = findHomes([cell([0, 0, 0], [halo("a", 6e11, 2, bound(0.9)), halo("b", 3e13, 2, bound(0.9))]), cell([1, 0, 0], [halo("c", 1e12, 2, bound(0.4))])], LIKE_THE_MILKY_WAY);
  assert.deepEqual(found.map((f) => [f.halo.key, f.cell]), [["c", [1, 0, 0]], ["a", [0, 0, 0]]]);
});

test("cells around one: itself first, then nearest shells", () => {
  const c = cellsAround([5, 0, -2], 1);
  assert.equal(c.length, 27);
  assert.deepEqual(c[0], [5, 0, -2]);
  assert.deepEqual(c.slice(1, 7).map((x) => Math.abs(x[0] - 5) + Math.abs(x[1]) + Math.abs(x[2] + 2)), [1, 1, 1, 1, 1, 1], "faces before edges and corners");
  assert.equal(new Set(c.map(String)).size, 27);
});
