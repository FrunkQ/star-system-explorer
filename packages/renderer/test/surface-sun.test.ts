// Land part 2: the sun in a world's sky from its tilt, its place on its orbit and its spin.
import { test } from "node:test";
import assert from "node:assert/strict";
import { sunAt } from "../src/index.ts";

const DAY = 86400;
const YEAR = 365.25 * DAY;
const SIDEREAL = DAY / (1 + DAY / YEAR);

test("an Earth-like world: solstice declinations and a 24-hour solar day", () => {
  const at = (phase: number, t = 0, lon = 0) => sunAt({ tilt_deg: 23.44, orbit_phase_deg: phase, spin_s: SIDEREAL, orbit_s: YEAR, t_s: t, lon_deg: lon })!;
  assert.ok(Math.abs(at(180).declination_deg - 23.44) < 1e-9, "the sun over the northern tropic");
  assert.ok(Math.abs(at(0).declination_deg + 23.44) < 1e-9, "and the southern");
  assert.ok(Math.abs(at(90).declination_deg) < 1e-9, "an equinox");
  assert.ok(Math.abs(at(0).solar_day_s - DAY) < 1, `solar day ${at(0).solar_day_s}`);
  assert.ok(Math.abs(at(0, 0, 0).hour_deg) < 1e-9, "noon at longitude 0 at time 0");
  assert.ok(Math.abs(at(0, DAY / 4, 0).hour_deg - 90) < 1e-6, "six hours on");
  assert.ok(Math.abs(at(0, 0, 90).hour_deg - 90) < 1e-9, "90° east is six hours later");
  assert.ok(Math.abs(at(0, DAY / 2, 0).hour_deg) === 180, "midnight");
});

test("a locked world keeps its hour; a retrograde spin runs the other way; no spin, no clock", () => {
  const locked = sunAt({ tilt_deg: 0, orbit_phase_deg: 30, spin_s: 27.3 * DAY, orbit_s: 27.3 * DAY, t_s: 12345 * DAY, lon_deg: 20 })!;
  assert.equal(locked.hour_deg, 20);
  assert.equal(locked.solar_day_s, Infinity);
  const venus = sunAt({ tilt_deg: 177.4, orbit_phase_deg: 0, spin_s: -243.0 * DAY, orbit_s: 224.7 * DAY, t_s: 0, lon_deg: 0 })!;
  assert.ok(Math.abs(venus.solar_day_s / DAY - 116.75) < 0.2, `Venus's solar day ${venus.solar_day_s / DAY} d`);
  assert.equal(sunAt({ tilt_deg: 10, orbit_phase_deg: 0, t_s: 0, lon_deg: 0 }), null);
});
