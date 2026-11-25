// RK12 mount on the system chart: rings are placed with the chart's own log scale, so a ring point at the holder's
// place lands on the holder's mark, and the fixture's rings round Tarn grow outward in px as they do in metres.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { polar, radius } from "../../packages/chart-grammar/src/scale.ts";
import { rangeRings, ringPoints, type RangeEffect } from "../../packages/ui-contract/src/index.ts";
import { chartProject, holderAt } from "../../packages/views/system-chart/src/range-place.ts";

const scale = { inner_au: 0.05, outer_au: 60, r_min: 20, r_max: 400 };
const rr = (au: number) => radius(au, scale);
const project = chartProject(500, 400, rr);

test("a point at the holder's place lands where the chart draws the holder", () => {
  for (const orbit of [{ a_au: 1, phase_deg: 30 }, { a_au: 5.2, phase_deg: 200 }, { a_au: 0.4, phase_deg: -75 }]) {
    const [x, y] = project(...holderAt(orbit));
    const [ex, ey] = polar(500, 400, rr(orbit.a_au), orbit.phase_deg);
    assert.ok(Math.abs(x - ex) < 1e-6 && Math.abs(y - ey) < 1e-6, `${orbit.a_au} AU`);
  }
  assert.deepEqual(holderAt(null), [0, 0]);
});

test("the fixture's rings round a world grow outward on the chart, in order", () => {
  const fx = JSON.parse(readFileSync(new URL("../../packages/ui-contract/test/fixtures/range-effects.json", import.meta.url), "utf8"));
  const effects: RangeEffect[] = fx.effects;
  const rings = rangeRings(effects[0]!, "Laser");
  assert.ok(rings.length >= 1, "the fixture gives rings");
  const at = holderAt({ a_au: 1, phase_deg: 0 });
  const [hx, hy] = project(...at);
  let last = 0;
  for (const g of rings) {
    // The far side (away from the centre) of each ring, in px from the holder.
    const far = ringPoints([at[0], at[1], 0], g.radius_m, 4)[0]!;
    const [x, y] = project(far[0], far[1]);
    const d = Math.hypot(x - hx, y - hy);
    assert.ok(d >= last, `${g.kind} lies outside the last ring`);
    last = d;
  }
});
