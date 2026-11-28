import { test } from "node:test";
import assert from "node:assert/strict";
import { regionShare, spotsFromRegions, type Region, type Regions } from "../src/star-regions.ts";
import { flaresAt, spotsAt, type LiveStar } from "../src/live-star.ts";

const DAY = 86400;
const region = (over: Partial<Region> = {}): Region => ({
  address: "sys/star/region:0.0",
  born_s: 1e9,
  grow_s: 5 * DAY,
  life_s: 50 * DAY,
  peak_share: 2.5e-4,
  peak_msh: 500,
  lat_deg: 20,
  lon_deg: 100,
  tilt_deg: -11,
  leading_sign: 1,
  separation_deg: 6,
  ...over,
});
const set = (regions: Region[]): Regions => ({ regions, rotation_period_s: 25.4 * DAY, differential: [14.713, -2.396, -1.787], leader_share: 0.6 });

test("a region's spots grow fast, decay steadily and are gone after its life", () => {
  const g = region();
  assert.equal(regionShare(g, g.born_s - 1), 0);
  assert.ok(Math.abs(regionShare(g, g.born_s + 5 * DAY) - 2.5e-4) < 1e-12, "at its peak");
  assert.ok(Math.abs(regionShare(g, g.born_s + 27.5 * DAY) - 1.25e-4) < 1e-12, "half way down");
  assert.equal(regionShare(g, g.born_s + 51 * DAY), 0);
  assert.equal(spotsFromRegions(set([g]), g.born_s + 60 * DAY).length, 0);
});

test("each region is a pair: the leader bigger, ahead in the spin and nearer the equator in both hemispheres", () => {
  for (const [lat, tilt] of [[20, -11], [-20, 11]]) {
    const [lead, follow] = spotsFromRegions(set([region({ lat_deg: lat, tilt_deg: tilt })]), 1e9 + 10 * DAY);
    assert.ok(lead!.radius_deg > follow!.radius_deg);
    assert.ok(Math.abs(lead!.lat_deg) < Math.abs(follow!.lat_deg), `${lead!.lat_deg} ${follow!.lat_deg}`);
    assert.ok(((lead!.lon_deg - follow!.lon_deg + 540) % 360) - 180 > 0, "leader ahead");
  }
});

test("the live star's spots are its regions' and a flare goes off at its region's site", () => {
  const s: LiveStar = {
    address: "sys/star",
    teff: 5772,
    mass: 1.989e30,
    radius: 6.957e8,
    lum: 3.828e26,
    age_s: 1e9 + 10 * DAY,
    activity: { rotation_period_s: 25.4 * DAY },
    moment: { spot_coverage: 0.002, xray_w: 2e20 },
    flares: [{ at_s: 1e9 + 10 * DAY, rise_s: 60, decay_s: 600, peak_w: 4e20, energy_j: 1e24, site: { region: "sys/star/region:0.0", lat_deg: 20, lon_deg: 101 } }],
    regions: set([region()]),
  };
  assert.equal(spotsAt(s, s.age_s).length, 2);
  const [f] = flaresAt(s, spotsAt(s, s.age_s), s.age_s);
  assert.equal(f!.lat_deg, 20);
  assert.equal(f!.lon_deg, 101);
});
