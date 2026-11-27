// SC1: a star's larger cells, the kernel's supergranule scale. On a red supergiant the scale is wider than the star,
// so a few cells cover it and its face shows them; on the Sun there are thousands, faint and too small to see.
import { test } from "node:test";
import assert from "node:assert/strict";
import { bigCellContrast, BIG_CELL_DT_MAX, BIG_CELLS_LEAST, cellsAcross, churningBigCells } from "../src/star-churn.ts";
import { granuleCount } from "../src/star.ts";
import { drawStarNow } from "../src/star-now.ts";
import type { LiveStar } from "../src/live-star.ts";

const DAY = 86400;
const R_SUN = 6.957e8;
// The kernel's convection for a 20 solar-mass red supergiant at 9 Myr and the Sun (read_activity, 26 Nov).
const rsgConvection = { pressure_scale_height_m: 9.7e9, granule_size_m: 9.7e10, granule_lifetime_s: 96 * DAY, supergranule_size_m: 2.4e12, supergranule_lifetime_s: 24000 * DAY, speed_m_s: 6000, sound_speed_m_s: 6100, granules_across: 23.1, network_cells: 2.69 };
const sunConvection = { pressure_scale_height_m: 1.3e5, granule_size_m: 1.3e6, granule_lifetime_s: 330, supergranule_size_m: 3.25e7, supergranule_lifetime_s: 82600, speed_m_s: 2000, sound_speed_m_s: 8000, granules_across: 1050, network_cells: 5580 };
const star = (radius: number, teff: number, convection: Record<string, number>): LiveStar => ({
  address: "u:test/star:a",
  teff,
  mass: 2e30,
  radius,
  lum: 1e30,
  age_s: 1e15,
  activity: { rotation_period_s: 1e4 * DAY, convection },
  moment: { spot_coverage: 0, xray_w: 1e20 },
  flares: [],
});

test("the large cells' contrast follows their flow: half a per cent on the Sun, several on a red supergiant, capped", () => {
  const sun = bigCellContrast(sunConvection);
  const rsg = bigCellContrast(rsgConvection);
  assert.ok(sun > 0.002 && sun < 0.01, `Sun ${sun}`);
  assert.ok(rsg > 0.03 && rsg <= BIG_CELL_DT_MAX, `red supergiant ${rsg}`);
  assert.equal(bigCellContrast({ supergranule_size_m: 1e12, supergranule_lifetime_s: 1, sound_speed_m_s: 1 }), BIG_CELL_DT_MAX);
  assert.equal(bigCellContrast(null), 0);
});

test("a red supergiant's large cells are few; the count is the kernel's, never fewer than tile a sphere", () => {
  const few = churningBigCells("u:test/star:a", rsgConvection.network_cells, 0, rsgConvection.supergranule_lifetime_s);
  assert.equal(few.n, BIG_CELLS_LEAST);
  const many = churningBigCells("u:test/star:a", 400, 0, 1000);
  assert.equal(many.n, 400);
  assert.ok(Math.abs(granuleCount(cellsAcross(400)) - 400) <= 1, "cellsAcross inverts granuleCount");
});

/** The spread of the face's brightness over blocks a tenth of the disc wide: the large-scale pattern, not single pixels. */
function coarseSpread(face: { width: number; data: Uint8ClampedArray }): number {
  const n = face.width;
  const block = Math.floor(n / 10);
  const means: number[] = [];
  for (let by = 0; by + block <= n; by += block)
    for (let bx = 0; bx + block <= n; bx += block) {
      let sum = 0;
      let k = 0;
      for (let y = by; y < by + block; y++)
        for (let x = bx; x < bx + block; x++) {
          const o = (y * n + x) * 4;
          const dx = (x - n / 2) / (n * 0.46);
          const dy = (y - n / 2) / (n * 0.46);
          // The middle of the disc only, away from the limb's darkening.
          if (dx * dx + dy * dy > 0.25) continue;
          sum += face.data[o]! + face.data[o + 1]! + face.data[o + 2]!;
          k++;
        }
      if (k === block * block) means.push(sum / k);
    }
  const m = means.reduce((a, b) => a + b, 0) / means.length;
  return Math.sqrt(means.reduce((a, b) => a + (b - m) ** 2, 0) / means.length) / m;
}

test("a red supergiant's face shows a few huge cells; the Sun's middle stays even at the same size", () => {
  const ask = { t: 1e15, clock: DAY, span: DAY / 10, calm: true };
  const face = () => ({ width: 256, data: new Uint8ClampedArray(256 * 256 * 4) });
  const rsg = face();
  drawStarNow(rsg, null, star(1610 * R_SUN, 3200, rsgConvection), ask);
  const sun = face();
  drawStarNow(sun, null, star(R_SUN, 5772, sunConvection), ask);
  const big = coarseSpread(rsg);
  const even = coarseSpread(sun);
  // Absolute, not only a ratio: blocks a tenth of the disc across differ by several per cent on the supergiant.
  assert.ok(big > 0.05, `red supergiant coarse spread ${big}`);
  assert.ok(even < 0.01, `Sun coarse spread ${even}`);
});
