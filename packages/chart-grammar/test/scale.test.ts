import { test } from "node:test";
import assert from "node:assert/strict";
import { fitScale, radius, scaleRings } from "../src/scale.ts";

test("scale rings: a solid ring each power of ten, 2 and 5 between when there is room", () => {
  // Mercury to Neptune on a 500 px radius.
  const s = fitScale([0.39, 30], 34, 500);
  const rings = scaleRings(s, 700);
  const majors = rings.filter((x) => x.major).map((x) => x.units);
  assert.deepEqual(majors, [1, 10, 100]);
  assert.ok(rings.some((x) => x.units === 0.5 && !x.major));
  assert.ok(rings.some((x) => x.units === 2 && !x.major) && rings.some((x) => x.units === 5));
  // Every ring lies on the chart, outward in order.
  const rs = rings.map((x) => radius(x.au, s));
  assert.ok(rs.every((r, i) => r >= s.r_min - 1e-9 && r <= 700 && (i === 0 || r > rs[i - 1]!)));
  // Cramped: decades only.
  const small = scaleRings(fitScale([0.39, 30], 22, 90), 120);
  assert.ok(small.every((x) => x.major));
});

test("scale rings in another unit: kilometres round a planet (the Galilean moons)", () => {
  const KM_PER_AU = 149_597_870.7;
  const s = fitScale([421_700 / KM_PER_AU, 1_882_700 / KM_PER_AU], 34, 300);
  const rings = scaleRings(s, 400, 1 / KM_PER_AU).filter((x) => x.major);
  assert.ok(rings.some((x) => x.units === 1_000_000), JSON.stringify(rings.map((x) => x.units)));
  assert.ok(rings.every((x) => Math.log10(x.units) % 1 === 0));
});
