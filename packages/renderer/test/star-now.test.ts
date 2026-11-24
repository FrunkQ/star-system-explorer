import { test } from "node:test";
import assert from "node:assert/strict";
import { glarePsf, glareProfile, glowReach, drawGlow, GLOW_REACH_DEG } from "../src/star-glow.ts";
import { churningGranules, granuleBlur, limbLumps } from "../src/star-churn.ts";
import { addEjections, ejectionShare, ejectionsAt, flareBoost, patchRadiusDeg, shownFlares, FLASHES_PER_SECOND_MAX } from "../src/star-events.ts";
import { drawStar, granuleCount } from "../src/star.ts";
import { flarePlace, turnAt, type Flare, type LiveStar } from "../src/live-star.ts";
import { drawStarNow } from "../src/star-now.ts";

const DAY = 86400;
const R_SUN = 6.957e8;
const sun = (over: Partial<LiveStar> = {}): LiveStar => ({
  address: "sys/star",
  teff: 5772,
  mass: 1.989e30,
  radius: R_SUN,
  lum: 3.828e26,
  age_s: 1e9,
  activity: { rotation_period_s: 25.4 * DAY, cycle: { period_s: 11 * 365.25 * DAY } },
  moment: { cycle_phase: 0.2, spot_coverage: 0.002, xray_w: 2e20 },
  flares: [],
  ...over,
});
const flare = (at_s: number, energy_j: number, over: Partial<Flare> = {}): Flare => ({ at_s, rise_s: 30, decay_s: 300, peak_w: energy_j / 315, energy_j, ...over });

test("the glow is the CIE glare equation: steep near the star, under white by 5°, a hundredth by 20°", () => {
  // CIE 146:2002 at 1° for a 25-year-old, p = 0.5: 10 + (5 + 0.05)(1 + (25/62.5)^4) + 0.00125.
  const age = 1 + (25 / 62.5) ** 4;
  assert.ok(Math.abs(glarePsf(1) - (10 + 5.05 * age + 0.00125)) < 1e-9);
  assert.ok(glarePsf(0.01) === glarePsf(0.1), "held at the equation's smallest angle");
  const p = glareProfile(0.27, 96);
  for (let k = 1; k < p.length; k++) assert.ok(p[k]! <= p[k - 1]! + 1e-12, "falls away from the star");
  const white = glowReach(p, 1);
  assert.ok(white > 3 && white < 6, `outshines a lit white surface out to ${white}°`);
  assert.ok(Math.PI * glarePsf(GLOW_REACH_DEG, undefined, false) < 0.06, "faint at the edge of the picture");
  // A flare that doubles the star's light pushes the glow further out.
  assert.ok(glowReach(p, 1, GLOW_REACH_DEG, 2) > white);
});

test("the glow picture is clear over the disc, brightest at its edge, nothing at the square's edge", () => {
  const o = { width: 64, data: new Uint8ClampedArray(64 * 64 * 4) };
  drawGlow(o, { teff_k: 5772, disc_deg: 2 });
  const at = (x: number, y: number) => o.data[(y * 64 + x) * 4 + 3]!;
  assert.equal(at(32, 32), 0, "the disc is drawn by the face, not the glow");
  assert.ok(at(32 + 4, 32) > at(32 + 16, 32), "brighter near the star");
  assert.equal(at(0, 0), 0);
  assert.ok(at(63, 32) < 8, "faded to almost nothing at the edge");
});

test("cells churn: the same moment gives the same cells; a moment later most have barely moved and a few are new", () => {
  const life = 600;
  const a = churningGranules("sys/star", 20, 1e6, life);
  assert.deepEqual(churningGranules("sys/star", 20, 1e6, life).pts, a.pts);
  assert.equal(a.n, granuleCount(20));
  const dt = life / 20;
  const b = churningGranules("sys/star", 20, 1e6 + dt, life);
  let jumped = 0;
  for (let i = 0; i < a.n; i++) {
    const d = Math.hypot(a.pts[i * 3]! - b.pts[i * 3]!, a.pts[i * 3 + 1]! - b.pts[i * 3 + 1]!, a.pts[i * 3 + 2]! - b.pts[i * 3 + 2]!);
    if (d > 0.1 * a.spacing) jumped++;
    assert.ok(a.amp![i]! >= 0 && a.amp![i]! <= 1);
  }
  // About dt/life of the cells are reborn elsewhere; the rest drift a little.
  assert.ok(jumped / a.n < 0.12, `${jumped} of ${a.n} jumped`);
  assert.ok(jumped > 0, "some cells were reborn");
});

test("a picture spanning many cell lives shows their average: contrast falls as one over the square root", () => {
  assert.equal(granuleBlur(600, 60), 1);
  assert.ok(Math.abs(granuleBlur(600, 2400) - 0.5) < 1e-12);
});

test("the limb is lifted by the cells by the scale height: round on the Sun, lumpy on a red supergiant", () => {
  // The Sun: H about 150 km on 696,000 km.
  assert.ok(limbLumps({ pressure_scale_height_m: 1.5e5 }, R_SUN) < 3e-4);
  assert.equal(limbLumps(null, R_SUN), 0);
  // A Betelgeuse-like star, about 900 solar radii with a scale height of a few hundredths of it.
  const radius = 900 * R_SUN;
  const lumps = limbLumps({ pressure_scale_height_m: 0.03 * radius }, radius);
  const edge = (lumpy: number) => {
    const n = 128;
    const o = { width: n, data: new Uint8ClampedArray(n * n * 4) };
    drawStar(o, { teff_k: 3600, spots: [], lon_deg: 0, lat_deg: 7, light: "visible", corona_h: 0.1, granules: churningGranules("sys/bg", 6, 0, 1e7), lumps: lumpy }, 0.8);
    // The disc's edge along 64 bearings: the furthest opaque pixel from the centre.
    const radii: number[] = [];
    for (let k = 0; k < 64; k++) {
      const a = (k / 64) * 2 * Math.PI;
      let r = 0;
      for (let s = 0; s < n / 2; s += 0.25) {
        const x = Math.floor(n / 2 + s * Math.cos(a));
        const y = Math.floor(n / 2 + s * Math.sin(a));
        if (o.data[(y * n + x) * 4 + 3]! > 127) r = s;
      }
      radii.push(r);
    }
    const mean = radii.reduce((p, q) => p + q, 0) / radii.length;
    return Math.sqrt(radii.reduce((p, q) => p + (q - mean) ** 2, 0) / radii.length) / mean;
  };
  assert.ok(edge(0) < 0.01, "round without lumps");
  assert.ok(edge(lumps) > 0.01, `lumpy: ${edge(lumps)}`);
});

test("never more than three flashes a second, each lasting a third of a second; nothing flashes under reduced motion", () => {
  const clock = DAY / 4;
  const t0 = 1e9;
  // Twenty flares within one second of the screen's clock.
  const flares = Array.from({ length: 20 }, (_, i) => flare(t0 + (i * clock) / 20, 1e24 * (1 + i)));
  const s = sun({ flares });
  const onsets = new Set<number>();
  for (let step = 0; step <= 40; step++) {
    const t = t0 - clock * 0.1 + (step * clock) / 20;
    for (const f of shownFlares(s, t, clock)) onsets.add(f.index);
  }
  assert.ok(onsets.size <= FLASHES_PER_SECOND_MAX + 1, `${onsets.size} flares began within about a second`);
  // A brief flare (a 300 s fall is a fiftieth of a screen frame) stays up for at least a third of a second.
  const one = sun({ flares: [flare(t0, 1e25)] });
  const third = clock / FLASHES_PER_SECOND_MAX;
  assert.ok(shownFlares(one, t0 + 0.9 * third, clock).length === 1, "still showing near a third of a second on");
  assert.equal(shownFlares(one, t0, clock, true).length, 0, "reduced motion");
  assert.ok(flareBoost(one, [{ flare: { ...one.flares[0]!, light: { parts: [{ kind: "thermal", t_k: 1e4, area_m2: 1e18 }] } }, index: 0, k: 1 }]) > 1.1);
});

test("a flare's patch is the kernel's area on the star; with an ejection it goes off where the ejection leaves", () => {
  // An X-class solar flare: about 3e13 m² of 10,000 K (0.25°).
  const r = patchRadiusDeg(2.8e13, R_SUN);
  assert.ok(r > 0.2 && r < 0.3, `${r}`);
  const s = sun();
  const f = flare(s.age_s + DAY, 1e26, { ejection: { speed_m_s: 1e6, mass_kg: 1e13, direction: [0, 1, 0], half_width_rad: Math.PI / 6 } });
  const at = flarePlace(s, f)!;
  assert.ok(Math.abs(at.lat_deg) < 1e-9);
  // Seen at the peak the place on the turning star lies at the ejection's longitude in the viewer's frame (90°).
  assert.ok(Math.abs(((at.lon_deg - turnAt(s, f.at_s) + 360) % 360) - 90) < 1e-9);
});

test("an ejection's front leaves at its speed, fades as the fourth power of distance and is hidden behind the star", () => {
  const s = sun();
  const f = flare(s.age_s, 1e26, { ejection: { speed_m_s: 1e6, mass_kg: 1e13, direction: [1, 0, 0], half_width_rad: Math.PI / 6 } });
  const live = sun({ flares: [f] });
  assert.equal(ejectionsAt(live, s.age_s - 1, 50).length, 0, "not before the flare");
  const later = ejectionsAt(live, s.age_s + R_SUN / 1e6, 50)[0]!;
  assert.ok(Math.abs(later.r - 2) < 1e-9, "one stellar radius out after R/v");
  const far = ejectionsAt(live, s.age_s + (3 * R_SUN) / 1e6, 50)[0]!;
  assert.ok(Math.abs(ejectionShare(later) / ejectionShare(far) - 16) < 1e-6, "twice as far, a sixteenth as bright");
  assert.ok(ejectionShare(later) < 1e-7, "far fainter than the star to the eye");
  // Drawn: the cone points along x in the star's frame, which is the viewer's right at longitude 90°.
  const o = { width: 64, data: new Uint8ClampedArray(64 * 64 * 4) };
  addEjections(o, [{ ...later, lon_deg: 90, lat_deg: 0 }], 4, [1, 1, 1], 0);
  const lit = (x0: number, x1: number) => {
    let v = 0;
    for (let y = 0; y < 64; y++) for (let x = x0; x < x1; x++) v += o.data[(y * 64 + x) * 4 + 3]!;
    return v;
  };
  assert.ok(lit(40, 64) > 0, "seen on the right");
  assert.equal(lit(0, 24), 0, "nothing on the left");
});

test("the star now: face, glow and words together, the same picture for the same moment", () => {
  const s = sun({ activity: { rotation_period_s: 25.4 * DAY, convection: { granules_across: 8, granule_lifetime_s: 1e5, granule_size_m: 1e8, pressure_scale_height_m: 0.02 * R_SUN, supergranule_size_m: 1e9, supergranule_lifetime_s: 1e6, speed_m_s: 2000, sound_speed_m_s: 7000, network_cells: 10 } }, flares: [flare(1e9 + 60, 1e27, { ejection: { speed_m_s: 2e6, mass_kg: 1e14, direction: [0, 1, 0], half_width_rad: 0.5 } })] });
  const draw = () => {
    const face = { width: 64, data: new Uint8ClampedArray(64 * 64 * 4) };
    const halo = { width: 64, data: new Uint8ClampedArray(64 * 64 * 4) };
    const got = drawStarNow(face, halo, s, { t: 1e9 + 2000, clock: DAY / 4, span: DAY / 40, disc_deg: 1 });
    return { face: face.data, halo: halo.data, got };
  };
  const a = draw();
  assert.deepEqual(draw().face, a.face);
  assert.equal(a.got.shown.length, 1);
  assert.equal(a.got.ejections.length, 1);
  assert.ok(a.got.words.some((w) => w.includes("coronagraph")));
  assert.ok(a.got.words.some((w) => w.includes("glare")));
  assert.ok(a.halo.some((v) => v > 0));
});

test("spots drift by latitude: the equator keeps pace with the turn, high latitudes fall behind (VP-S4)", async () => {
  const { rateShare, spotsAt, spotsOf } = await import("../src/live-star.ts");
  assert.equal(rateShare(0), 1);
  // Snodgrass and Ulrich at 60°: 14.713 − 2.396 × 0.75 − 1.787 × 0.5625 = 11.91°/day, against 14.71 at the equator.
  assert.ok(Math.abs(rateShare(60) * 14.713 - 11.91) < 0.01);
  const s = sun({ moment: { cycle_phase: 0.2, spot_coverage: 0.02, xray_w: 2e20 } });
  const now = spotsOf(s);
  assert.deepEqual(spotsAt(s, s.age_s), now, "at the star's present, as the kernel places them");
  const later = spotsAt(s, s.age_s + 25.4 * DAY);
  now.forEach((sp, i) => {
    const lag = (((sp.lon_deg - later[i]!.lon_deg) % 360) + 360) % 360;
    assert.ok(lag >= 0 && lag < 90, "a spot falls behind, never ahead");
    if (Math.abs(sp.lat_deg) > 20) assert.ok(lag > 1, `a spot at ${sp.lat_deg.toFixed(0)}° lags ${lag.toFixed(1)}°`);
  });
});

test("the hidden-disc corona: a millionth of the disc, streamers along the equator when quiet, round at the peak (VP-S3)", async () => {
  const { baumbach, coronaShape, cycleActivity, drawCorona } = await import("../src/star-corona.ts");
  assert.ok(baumbach(1) > 3e-6 && baumbach(1) < 5e-6, "about four millionths at the limb");
  assert.ok(baumbach(2) < baumbach(1) / 50);
  assert.ok(coronaShape(Math.PI / 2, 0) < 0.3 && coronaShape(0, 0) === 1, "quiet: dark poles");
  assert.equal(coronaShape(Math.PI / 2, 1), 1, "peak: round");
  // Phase 0.25 from the cycle's minimum is cycle_phase 0 in the kernel's convention (minimum at 0.75).
  const quiet = sun({ activity: { rotation_period_s: 25.4 * DAY, cycle: { period_s: 11 * 365.25 * DAY } }, moment: { cycle_phase: 0.75, xray_w: 2e20 } });
  assert.ok(cycleActivity(quiet) < 0.01);
  const o = { width: 64, data: new Uint8ClampedArray(64 * 64 * 4) };
  drawCorona(o, quiet, 0.3);
  const at = (x: number, y: number) => o.data[(y * 64 + x) * 4]!;
  assert.equal(at(32, 32), 0, "the disc is hidden");
  assert.ok(at(32 + 14, 32) > at(32, 32 - 14), "brighter along the equator than over the pole");
});
