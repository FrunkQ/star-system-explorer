// Fixes from the GPU check of the star close-up (28 Nov 2026): speed, the limb, eruptions in words.
import { test } from "node:test";
import assert from "node:assert/strict";
import { srgb255, linear255 } from "../src/srgb.ts";
import { limbDarkeningAt } from "../src/star.ts";
import { drawGlow } from "../src/star-glow.ts";
import { drawStarNow } from "../src/star-now.ts";
import type { LiveStar } from "../src/live-star.ts";

const exact = (c: number) => 255 * (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);

test("the sRGB table is the curve to a fiftieth of a step, and decodes bytes exactly", () => {
  for (let i = 0; i <= 20000; i++) {
    const v = (i / 20000) ** 3;
    assert.ok(Math.abs(srgb255(v) - exact(v)) < 0.02, `${v}`);
  }
  assert.equal(srgb255(-1), 0);
  assert.equal(srgb255(2), 255);
  for (let b = 0; b < 256; b++) assert.ok(Math.abs(exact(linear255(b)) - b) < 1e-9);
});

test("limb darkening by colour: the Sun's green limb near a third of its centre, blue darker than red, hot stars darken less", () => {
  const g = limbDarkeningAt(0, 550e-9, 5772);
  assert.ok(g > 0.25 && g < 0.4, `${g}`);
  assert.equal(limbDarkeningAt(1, 550e-9, 5772), 1);
  assert.ok(limbDarkeningAt(0.2, 465e-9, 5772) < limbDarkeningAt(0.2, 550e-9, 5772));
  assert.ok(limbDarkeningAt(0.2, 550e-9, 5772) < limbDarkeningAt(0.2, 610e-9, 5772));
  assert.ok(limbDarkeningAt(0, 550e-9, 19000) > limbDarkeningAt(0, 550e-9, 5772));
  assert.ok(limbDarkeningAt(0, 550e-9, 3400) < limbDarkeningAt(0, 550e-9, 5772));
});

test("a kept glow is the same picture as a fresh one, and a flare draws it afresh", () => {
  const a = { width: 96, data: new Uint8ClampedArray(96 * 96 * 4) };
  const b = { width: 96, data: new Uint8ClampedArray(96 * 96 * 4) };
  drawGlow(a, { teff_k: 5772, disc_deg: 3, reach_deg: 20 });
  b.data.fill(7);
  drawGlow(b, { teff_k: 5772, disc_deg: 3, reach_deg: 20 });
  assert.deepEqual(b.data, a.data, "the kept picture, copied whole");
  drawGlow(b, { teff_k: 5772, disc_deg: 3, reach_deg: 20, boost: 3 });
  assert.notDeepEqual(b.data, a.data, "a flare's light draws a new one");
});

const DAY = 86400;
const R = 6.957e8;
const born = 1.4e17;
const sun = (prominences: unknown[] = []): LiveStar =>
  ({ address: "u:t/sys:0/star:0", teff: 5772, mass: 1.989e30, radius: R, lum: 3.828e26, age_s: born, activity: { rotation_period_s: 1e30 }, moment: { cycle_phase: 0.2, spot_coverage: 0, xray_w: 2e20 }, flares: [], prominences }) as unknown as LiveStar;

test("the Sun's face is round, so the view cuts its limb on the screen; an eruption is named with its speed and time in view", () => {
  const prom = { kind: "quiescent", born_s: born, life_s: 5 * DAY, lat_deg: 30, lon_deg: 90, tilt_deg: 10, height_m: 4e7, length_m: 2e8, thickness_m: 7e6, mass_kg: 1e12, temperature_k: 8000, share_visible: 1e-4, eruption: { at_s: born + 5 * DAY, speed_m_s: 4e5 } };
  const face = { width: 128, data: new Uint8ClampedArray(128 * 128 * 4) };
  const halo = { width: 160, data: new Uint8ClampedArray(160 * 160 * 4) };
  const ask = { clock: 0, span: 0, disc: 0.92, disc_deg: 13.5, reach_deg: 30, lat_deg: 0 };
  const quiet = drawStarNow(face, halo, sun([prom]), { ...ask, t: born + DAY });
  assert.equal(quiet.round, true);
  const t = born + 5 * DAY + 600;
  const now = drawStarNow(face, halo, sun([prom]), { ...ask, t });
  const words = now.words.join(" ");
  assert.match(words, /erupting at 400 km\/s \(out of the picture in \d+ minutes\)/);
  // A risen rope's light goes through the filter's log scale: a radius up it still shows well beside its standing self.
  const extra = (at: number) => {
    drawStarNow(face, halo, sun([]), { ...ask, t: at });
    const bare = Uint8ClampedArray.from(halo.data);
    drawStarNow(face, halo, sun([prom]), { ...ask, t: at });
    let sum = 0;
    for (let i = 3; i < bare.length; i += 4) sum += Math.max(0, halo.data[i]! - bare[i]!);
    return sum;
  };
  const ratio = extra(born + 5 * DAY + 1800) / extra(born + DAY);
  // (186 with the log scale; 108 when the fade was laid over the scale instead.)
  assert.ok(ratio > 150, `${ratio}`);
});
