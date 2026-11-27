import { test } from "node:test";
import assert from "node:assert/strict";
import { drawLiveStar, flareProfile, flaresAt, readLiveStar, spotsOf, turnAt, type LiveStar, type StarReader } from "../src/live-star.ts";

const DAY = 86400;
const sun = (over: Partial<LiveStar> = {}): LiveStar => ({
  address: "sys/star",
  teff: 5772,
  mass: 1.989e30,
  radius: 6.957e8,
  lum: 3.828e26,
  age_s: 1e9,
  activity: { rotation_period_s: 25.4 * DAY, cycle: { period_s: 11 * 365.25 * DAY } },
  moment: { cycle_phase: 0.2, spot_coverage: 0.002, xray_w: 2e20 },
  flares: [{ at_s: 1e9 + 3600, rise_s: 600, decay_s: 1800, peak_w: 4e20, energy_j: 5e24 }],
  ...over,
});

test("a flare rises in a straight line to its peak, then falls away exponentially and ends", () => {
  const f = sun().flares[0]!;
  assert.equal(flareProfile(f, f.at_s - 700), 0);
  assert.equal(flareProfile(f, f.at_s - 300), 0.5);
  assert.equal(flareProfile(f, f.at_s), 1);
  assert.ok(Math.abs(flareProfile(f, f.at_s + 1800) - Math.exp(-1)) < 1e-12);
  assert.equal(flareProfile(f, f.at_s + 1800 * 4), 0, "gone once below a fiftieth");
});

test("the star turns once in its own rotation period and flares sit over its spots at their strength now", () => {
  const s = sun();
  // The star's frame turns once a spin from age zero (SG7: the frame the kernel places regions in).
  assert.equal(turnAt(s, 0), 0);
  assert.ok(Math.abs(turnAt(s, 25.4 * DAY * 0.25) - 90) < 1e-6);
  assert.ok(Math.abs(((turnAt(s, s.age_s + 25.4 * DAY * 0.25) - turnAt(s, s.age_s) + 360) % 360) - 90) < 1e-3);
  assert.equal(turnAt(sun({ activity: {} }), s.age_s + DAY), 0, "no spin known: no turn");
  const spots = spotsOf(s);
  assert.equal(flaresAt(s, spots, s.age_s).length, 0, "quiet before the flare");
  const at = flaresAt(s, spots, s.flares[0]!.at_s);
  assert.equal(at.length, 1);
  assert.equal(at[0]!.strength, 2, "peak power over the star's X-ray power");
  if (spots.length) assert.equal(at[0]!.lat_deg, spots[0]!.lat_deg);
});

test("drawn at a moment: a disc that changes as the star turns, the same picture for the same moment", () => {
  const s = sun({ moment: { cycle_phase: 0.25, spot_coverage: 0.02, xray_w: 2e20 } });
  const pic = (t: number) => {
    const o = { width: 64, data: new Uint8ClampedArray(64 * 64 * 4) };
    drawLiveStar(o, s, t);
    return o.data;
  };
  const a = pic(s.age_s);
  assert.equal(a[(32 * 64 + 32) * 4 + 3], 255, "the middle is on the disc");
  assert.equal(a[3], 0, "the corner is space");
  assert.deepEqual(pic(s.age_s), a);
  assert.notDeepEqual(pic(s.age_s + 5 * DAY), a, "spots carried round with the spin");
});

test("read from the kernel: facts and activity, or why there is nothing true to draw", async () => {
  const reader = (facts: Record<string, Record<string, unknown>>): StarReader => ({
    resolvedSystem: async () => ({ ok: true, result: { facts: facts as never } }),
    readActivity: async (ask) => ({ ok: true, stars: [{ activity: { regime: "solar" }, moment: { xray_w: 1 }, flares: { flares: [] }, asked: ask } as never] }),
  });
  const got = await readLiveStar(reader({ sys: { "system.age": 1e17 }, "sys/s": { "star.effective_temperature": 5772, "body.mass": 2e30, "body.radius": 7e8 } }), "sys", "sys/s");
  assert.ok(typeof got !== "string");
  assert.equal(got.age_s, 1e17);
  assert.equal(got.activity.regime, "solar");
  assert.match(String(await readLiveStar(reader({}), "sys", "sys/s")), /nothing true to draw/);
});

test("a picture every span of star time shows a flare that peaked between pictures, however brief", () => {
  const s = sun({ flares: [{ at_s: 1e9 + 100, rise_s: 10, decay_s: 20, peak_w: 2e20, energy_j: 1e24 }] });
  const spots = spotsOf(s);
  assert.equal(flaresAt(s, spots, 1e9 + 2000).length, 0, "long gone at that instant");
  assert.equal(flaresAt(s, spots, 1e9 + 2000, 2160)[0]!.strength, 1, "but it peaked within the last picture's span");
});
