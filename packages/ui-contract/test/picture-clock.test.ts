import { test } from "node:test";
import assert from "node:assert/strict";
import { clockAt, followTime, motionWords, phaseAt, pictureSpan, rateWords, stoppedAt, tooFast, cycleAt, withRate, YEAR_SECONDS } from "../src/picture-clock.ts";

const EPOCH = 4.6e9;
// How finely a year near a star's age says a time: about 30 s a step, so a few steps of slack.
const GRAIN = EPOCH * Number.EPSILON * 4 * YEAR_SECONDS;
const DAY = 86_400;

test("a stopped clock stands still and sits at the time in view", () => {
  const c = stoppedAt(YEAR_SECONDS, 1000);
  assert.equal(c.rate, 0);
  assert.equal(clockAt(c, 1000), clockAt(c, 61_000));
  // Seconds since the epoch, not years: a year after it is a year of seconds.
  assert.equal(clockAt(c, 0), YEAR_SECONDS);
});

test("running, the clock moves at its rate and resolves well under a second near a star's age", () => {
  const c = withRate(stoppedAt(0), 1, 0);
  // Real time: half a second later is half a second later, however old the star (years there resolve about 30 s).
  assert.equal(clockAt(c, 500), 0.5);
  const d = withRate(c, DAY, 1000);
  // Changing the rate never jumps the picture.
  assert.equal(clockAt(d, 1000), clockAt(c, 1000));
  assert.equal(clockAt(d, 2000) - clockAt(d, 1000), DAY);
  assert.equal(withRate(d, 0, 3000).rate, 0);
  assert.equal(clockAt(withRate(d, 0, 3000), 9e9), clockAt(d, 3000));
});

test("ticks of play keep the anchor; a jump moves it", () => {
  const c = withRate(stoppedAt(0), DAY, 0);
  // The ruler says a day has passed after a second (to its rounding, here a year's float at a star's age): the anchor stays.
  const tick = followTime(c, DAY + 0.9 * GRAIN, GRAIN, 1000);
  assert.ok(GRAIN > 30 && GRAIN < 200, `${GRAIN}`);
  assert.equal(tick, c);
  // A drag to next year: re-anchored there, still running.
  const jump = followTime(c, YEAR_SECONDS, GRAIN, 1000);
  assert.notEqual(jump, c);
  assert.equal(jump.rate, DAY);
  assert.equal(clockAt(jump, 1000), YEAR_SECONDS);
  // Held in whole seconds (grain 0), a tick a few seconds out at a day a second is still within the half-second of play.
  assert.equal(followTime(c, DAY + 3, 0, 1000), c);
  // Stopped, every move is taken.
  const s = stoppedAt(0);
  assert.equal(clockAt(followTime(s, DAY, GRAIN, 0), 0), DAY);
});

test("an orbit runs on at its mean motion from the kernel's phase", () => {
  const P = 365.25 * DAY;
  assert.equal(phaseAt(10, 0, P, 0), 10);
  assert.ok(Math.abs(phaseAt(10, 0, P, P / 4) - 100) < 1e-9);
  assert.ok(Math.abs(phaseAt(350, 0, P, P / 12) - 20) < 1e-9);
  // Backwards in time, and with no period, it stays sensible.
  assert.ok(Math.abs(phaseAt(10, 0, P, -P / 36) - 0) < 1e-9);
  assert.equal(phaseAt(42, 0, undefined, 1e9), 42);
  assert.ok(Math.abs(cycleAt(10, 25) - 0.5) < 1e-12);
  assert.ok(Math.abs(cycleAt(10, -2.5) - 0.75) < 1e-12);
});

test("a motion faster than the pictures is named, never slowed", () => {
  const c = withRate(stoppedAt(0), 30 * DAY, 0);
  const span = pictureSpan(c, 30);
  assert.equal(span, DAY);
  // Mercury (88 days) is followable at thirty days a second; a two-hour moon is not.
  assert.equal(tooFast(88 * DAY, span), false);
  assert.equal(tooFast(2 * 3600, span), true);
  assert.equal(tooFast(2 * 3600, 0), false, "stopped: nothing moves too fast");
  assert.match(motionWords(c, ["Phobos"]), /30 days a second\. Phobos goes round faster than the screen can show/);
  assert.match(motionWords(stoppedAt(0), []), /stopped, so the picture is still/);
  assert.deepEqual([1, 60, 3600, DAY, 7 * DAY, 0].map(rateWords), ["real time", "a minute a second", "an hour a second", "a day a second", "a week a second", "stopped"]);
});
