// The stars over a landing site (Land, night) turn round the same pole as the sun, and the eye opens up in the dark.
import { test } from "node:test";
import assert from "node:assert/strict";
import { ADAPT_RANGE, adaptedExposure, starsOverSite } from "../src/night-sky.ts";
import { sunAt, sunDirection } from "../src/surface-sky.ts";
import { constant } from "../../ui-contract/src/index.ts";

const near = (a: number, b: number, e = 1e-9) => assert.ok(Math.abs(a - b) < e, `${a} against ${b}`);
const same: { to_galactic: [[number, number, number], [number, number, number], [number, number, number]] } = { to_galactic: [[1, 0, 0], [0, 1, 0], [0, 0, 1]] };
const base = { frame: same, tilt_deg: 23.4, orbit_phase_deg: 70, sun_hour_deg: 35, lat_deg: 40, host_lsun: 1, orbit_m: constant("au") };

test("a star where the sun is stands where the sun stands", () => {
  const phi = (base.orbit_phase_deg * Math.PI) / 180;
  const [s] = starsOverSite({ ...base, stars: [{ dir: [-Math.cos(phi), -Math.sin(phi), 0], luminosity_lsun: 1, distance_pc: 1, teff_k: 5800 }] });
  const at = sunAt({ tilt_deg: base.tilt_deg, orbit_phase_deg: base.orbit_phase_deg, spin_s: 86164, t_s: 0, lon_deg: 0 })!;
  const want = sunDirection(base.lat_deg, base.sun_hour_deg, at.declination_deg);
  for (let i = 0; i < 3; i++) near(s!.dir[i]!, want[i]!, 1e-9);
});

test("the star over the world's pole stands as high as the site's latitude, whatever the hour", () => {
  const e = (base.tilt_deg * Math.PI) / 180;
  for (const h of [-150, 0, 90]) {
    const [s] = starsOverSite({ ...base, sun_hour_deg: h, stars: [{ dir: [Math.sin(e), 0, Math.cos(e)], luminosity_lsun: 1, distance_pc: 1, teff_k: 5800 }] });
    near((Math.asin(s!.dir[2]) * 180) / Math.PI, base.lat_deg, 1e-9);
  }
});

test("a star's share is its light over the home star's at the world: a sun at 1 pc against one at 1 AU", () => {
  const [s] = starsOverSite({ ...base, stars: [{ dir: [0, 0, 1], luminosity_lsun: 1, distance_pc: 1, teff_k: 5800 }] });
  near(s!.share, (constant("au") / constant("pc")) ** 2, 1e-20);
});

test("the eye keeps the day's exposure by day and opens up at night, no further than its range", () => {
  const air = { tau_green: 0.1, radius_m: 6.371e6, scale_height_m: 8000 };
  near(adaptedExposure(6, { ...air, sun_z: Math.SQRT1_2 }), 6);
  const dusk = adaptedExposure(6, { ...air, sun_z: 0.05 });
  const night = adaptedExposure(6, { ...air, sun_z: -0.5 });
  assert.ok(dusk > 6 && night > dusk, `${dusk} ${night}`);
  assert.ok(night <= 6 * ADAPT_RANGE);
});
