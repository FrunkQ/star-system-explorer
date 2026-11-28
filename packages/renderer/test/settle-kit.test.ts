// The settle kit's processor twins (FL2, settle-kit-cpu.ts) and the wind table (wind-table.ts): the design's 8A tests
// that do not need a GPU. The GPU half is held to these by tests/ui/settle-kit.e2e.mjs.
import { test } from "node:test";
import assert from "node:assert/strict";
import { carryCpu, exposureCpu, fadeCpu, loopPhases, meanShare, rowMeans, sampleAt } from "../src/settle-kit-cpu.ts";
import { LOOP_MAX_S, loopLength, rowLat, windAt, windTable } from "../src/wind-table.ts";

const W = 64;
const H = 32;
const DAY = 86_400;
/** A picture from a rule by texel. */
const picture = (f: (i: number, j: number) => [number, number, number]): Float32Array => {
  const out = new Float32Array(W * H * 4);
  for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) out.set([...f(i, j), 255], (j * W + i) * 4);
  return out;
};
// Smooth in longitude, so a slide of a fraction of a pixel is a fraction of a pixel's change.
const bands = picture((i, j) => [128 + 100 * Math.cos((2 * Math.PI * i) / W), 128 + 100 * Math.sin((2 * Math.PI * i) / W), 4 * j]);
const noise = picture((i, j) => [((i * 73 + j * 151) * 2654435761) % 251, ((i * 31 + j * 17) * 40503) % 241, (i ^ j) * 3]);
// Jets as Jupiter's are ordered: a few bands each way, about 1e-5 rad/s (150 m/s at Jupiter's radius).
const jet = (lat: number) => 2e-5 * Math.cos(6 * lat);
const eddies = (lat: number, lon: number): [number, number] => [3e-6 * Math.sin(4 * lon) * Math.cos(3 * lat), 2e-6 * Math.cos(5 * lon) * Math.cos(lat)];

test("a carry of zero seconds returns the picture untouched, to the bit", () => {
  const t = windTable(32, 16, jet, eddies);
  for (const src of [bands, noise]) assert.deepEqual(Array.from(carryCpu(src, W, H, t, 0)), Array.from(src));
});

test("a pure jet carried for ten days is the rows slid exactly, however many turns", () => {
  const t = windTable(32, 16, jet);
  const dt = 10 * DAY;
  const got = carryCpu(noise, W, H, t, dt);
  const acc = new Float64Array(4);
  const wv = new Float64Array(4);
  let worst = 0;
  for (let j = 0; j < H; j++)
    for (let i = 0; i < W; i++) {
      // The row's own slide, worked out on its own: east by its turn rate times the time, in texels.
      windAt(t, i, j, W, H, wv);
      acc.fill(0);
      sampleAt(noise, W, H, i - (wv[0]! * dt * W) / (2 * Math.PI), j, acc, 0, 1);
      for (let c = 0; c < 4; c++) worst = Math.max(worst, Math.abs(got[(j * W + i) * 4 + c]! - acc[c]!));
    }
  assert.ok(worst < 1e-3, `${worst}`);
  // And the slide is the jet's: ten days at 2e-5 rad/s is 17.3 rad, 2.75 turns, at the equator.
  assert.ok(Math.abs((jet(0) * dt) / (2 * Math.PI) - 2.75) < 0.01);
});

test("the two loops always weigh one between them, and each is weightless where it starts again", () => {
  const L = 16 * 3600;
  for (let dt = -3 * L; dt <= 3 * L; dt += L / 37) {
    const [[s0, w0], [s1, w1]] = loopPhases(dt, L);
    assert.ok(Math.abs(w0 + w1 - 1) < 1e-12, `${dt}`);
    for (const [s, w] of [[s0, w0], [s1, w1]]) {
      assert.ok(s >= -L / 2 - 1e-9 && s <= L / 2 + 1e-9);
      // Weight falls to nothing as the offset reaches either end, so the jump back is never seen.
      assert.ok(Math.abs(w - (1 - Math.abs(s) / (L / 2))) < 1e-9);
    }
  }
});

test("carried on, the picture changes smoothly: no jump from one moment to the next as the loops turn over", () => {
  const t = windTable(32, 16, jet, eddies, 16 * 3600);
  const step = t.loop_s / 64;
  let worst = 0;
  let prev = carryCpu(bands, W, H, t, 0);
  for (let k = 1; k <= 160; k++) {
    const next = carryCpu(bands, W, H, t, k * step);
    for (let i = 0; i < next.length; i++) worst = Math.max(worst, Math.abs(next[i]! - prev[i]!));
    prev = next;
  }
  // The steepest the test picture changes is about 10 a pixel; a 64th of a loop moves cloud well under a pixel.
  assert.ok(worst < 10, `${worst}`);
});

test("the long exposure at its limit is each row's mean, and below half a turn it is untouched by the mean", () => {
  const t = windTable(32, 16, (lat) => 1e-4 * (0.5 + Math.cos(lat)));
  const means = rowMeans(noise, W, H);
  // A whole turn of the slowest row in one picture: every row is at the limit.
  const span = (2 * Math.PI) / (1e-4 * 0.5) + 1;
  const got = exposureCpu(noise, W, H, t, span, means);
  for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) for (let c = 0; c < 4; c++) assert.equal(got[(j * W + i) * 4 + c], means[j * 4 + c]);
  assert.equal(meanShare(1e-4, 0), 0);
  assert.equal(meanShare(1e-4, Math.PI / 1e-4), 0, "half a turn: not yet");
  assert.equal(meanShare(-1e-4, (2 * Math.PI) / 1e-4), 1, "either way round");
  // A short exposure of a still picture (no wind) is the picture itself.
  const calm = exposureCpu(noise, W, H, windTable(8, 4, () => 0), DAY, means);
  for (let i = 0; i < calm.length; i++) assert.ok(Math.abs(calm[i]! - noise[i]!) < 1e-3);
});

test("the fade: nought is the first picture, one the second, exactly", () => {
  assert.deepEqual(Array.from(fadeCpu(bands, noise, 0)), Array.from(bands));
  assert.deepEqual(Array.from(fadeCpu(bands, noise, 1)), Array.from(noise));
  assert.deepEqual(Array.from(fadeCpu(bands, noise, 7)), Array.from(noise));
  const half = fadeCpu(bands, noise, 0.5);
  assert.ok(Math.abs(half[0]! - (bands[0]! + noise[0]!) / 2) < 1e-4);
});

test("the loop is set from the table's own numbers: the jets' shear, or the fastest eddy's turn", () => {
  // Jupiter's figures (design section 5): bands of about 7.5° and jets of 150 m/s make a shear time of hours.
  const a = 69_911e3;
  const u = 150;
  const band = (7.5 * Math.PI) / 180;
  const shear = windTable(256, 128, (lat) => (u / a) * Math.sin((Math.PI * lat) / band));
  const hours = shear.loop_s / 3600;
  // Turn rates in a sine by latitude, steepest at the equator: rows draw apart at u · π / (band · a), a loop of about
  // 4.4 hours. (The design's 16 h is a band's mean shear; the steepest is what stretches the picture first.)
  const want = (band * a) / (u * Math.PI) / 3600;
  assert.ok(Math.abs(hours - want) / want < 0.05, `${hours} h against ${want} h`);
  assert.equal(windTable(64, 32, () => 0).loop_s, LOOP_MAX_S, "no shear and no turning: the longest loop");
  // The whole air turning as one body, a radian in 0.9 days, about an axis through the equator: v = Ω x̂ × p, whose
  // vorticity is at most 2Ω, so its loop is 0.9 days.
  const om = 1 / (0.9 * DAY);
  const solid = windTable(512, 256, () => 0, (lat, lon) => [(om * -Math.sin(lat) * Math.cos(lon)) / Math.max(Math.cos(lat), 0.02), om * Math.sin(lon)]);
  assert.ok(Math.abs(solid.loop_s / DAY - 0.9) < 0.05, `${solid.loop_s / DAY} days`);
  assert.equal(loopLength(solid), solid.loop_s);
});
