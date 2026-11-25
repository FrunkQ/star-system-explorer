// VP5 acceptance: sharpen at rest (SSE4-visual-polish-design.md 8.4). Nothing happens while moving or for half a
// second after; then jittered frames up to the tier's count, or until the time budget; any movement drops back at once.
import { test } from "node:test";
import assert from "node:assert/strict";
import { REST_AFTER_MS, REST_BUDGET_MS, REST_FRAMES, RestClock, restJitter } from "../src/still.ts";

test("frames by tier: more on the richer tiers, one supersampled frame on a canvas, none in words", () => {
  assert.equal(REST_FRAMES.text, 0);
  assert.equal(REST_FRAMES.canvas, 1);
  assert.ok(REST_FRAMES["gpu-ultra"] >= REST_FRAMES["gpu-high"]);
  assert.ok(REST_FRAMES["gpu-basic"] >= 8, "Light gets the most from resting");
});

test("jitter: the first frame unshifted, then a Halton spread within half a pixel, never repeating", () => {
  assert.deepEqual(restJitter(0), [0, 0]);
  const seen = new Set<string>();
  for (let k = 1; k < 32; k++) {
    const [x, y] = restJitter(k);
    assert.ok(x >= -0.5 && x < 0.5 && y >= -0.5 && y < 0.5);
    seen.add(`${x},${y}`);
  }
  assert.equal(seen.size, 31);
  // Spread: the mean offset is near zero.
  const mx = Array.from({ length: 16 }, (_, k) => restJitter(k + 1)[0]).reduce((a, b) => a + b, 0) / 16;
  assert.ok(Math.abs(mx) < 0.05);
});

test("waits half a second, then refines to the tier's count", () => {
  const c = new RestClock({ frames: 4 });
  c.moved(0);
  assert.equal(c.state, "waiting");
  assert.equal(c.next(REST_AFTER_MS - 1), null);
  assert.equal(c.dueIn(100), REST_AFTER_MS - 100);
  let t = REST_AFTER_MS;
  const ks: number[] = [];
  for (;;) {
    const s = c.next(t);
    if (!s) break;
    ks.push(s.k);
    c.done(t + 10, 10);
    t += 16;
  }
  assert.deepEqual(ks, [0, 1, 2, 3]);
  assert.equal(c.state, "sharp");
  assert.equal(c.frames, 4);
  assert.equal(c.dueIn(t), Infinity);
});

test("movement drops back at once and starts the wait again", () => {
  const c = new RestClock({ frames: 8 });
  c.moved(0);
  c.next(600);
  c.done(610, 10);
  assert.equal(c.state, "sharpening");
  c.moved(700);
  assert.equal(c.state, "waiting");
  assert.equal(c.frames, 0);
  assert.equal(c.next(800), null);
  assert.equal(c.next(1200)?.k, 0);
});

test("a slow device stops at the time budget, keeping what it averaged", () => {
  const c = new RestClock({ frames: 32 });
  c.moved(0);
  let t = REST_AFTER_MS;
  let n = 0;
  for (;;) {
    const s = c.next(t);
    if (!s) break;
    t += 700;
    c.done(t, 700);
    n++;
  }
  assert.equal(c.state, "sharp");
  assert.ok(n >= 2 && n < 32);
  assert.ok(t - REST_AFTER_MS <= REST_BUDGET_MS + 700);
});

test("no frames (spinning, or words only): never rests", () => {
  const c = new RestClock({ frames: 0 });
  c.moved(0);
  assert.equal(c.state, "moving");
  assert.equal(c.next(10_000), null);
  c.aim(REST_FRAMES["gpu-high"], 0);
  assert.equal(c.state, "waiting");
});
