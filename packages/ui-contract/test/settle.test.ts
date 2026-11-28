// The settle schedule (FL1, SSE4-visual-flourish-design.md 4.3 and 8A): when a dear picture is rebuilt, when a
// stand-in shows, and that at rest it is always the proper picture.
import { test } from "node:test";
import assert from "node:assert/strict";
import { Settle, SETTLE_REST_MS, REBUILD_CAP_MS, bySize, dragPicture, drawFor, latticeDate, latticeStep, outrunsPictures, properKey, settleWeight, sizeFor, type DearLayer, type SettleMoment, type SettleState } from "../src/settle.ts";

const HOUR = 3600;
const DAY = 86_400;
// Jupiter, from the figures in the repo (design section 5): renewal 32 days, so its own time is 3.2 days; a day of
// 9.93 hours; a 512-wide still in about 0.1 s on AC-ProArt and 1 to 2 s on a phone (WS5's estimate).
const jupiter: DearLayer = { id: "giant.cloud", moves_with: ["clock"], own_time_s: 3.2 * DAY, cycle_s: 9.93 * HOUR, stand_in: ["carried", "exposure"] };
const nearby: DearLayer = { id: "journey.nearby", moves_with: ["place"], own_time_s: null, cycle_s: null, stand_in: ["coarse", "held"] };
const at = (now_ms: number, rate: number, build_ms: number): SettleMoment => ({ now_ms, rate, pictures_per_s: 30, build_ms });
const ask = (s: Settle, now_ms: number, rate: number, at_s: number, o: { build_ms?: number; view?: string; px?: number; spare?: boolean } = {}) =>
  s.ask({ ...at(now_ms, rate, o.build_ms ?? 100), at_s, view: o.view ?? "v", px: o.px ?? 500, spare: o.spare ?? true });

test("the hand check: Jupiter's states from an hour a second to a week a second, desktop and phone", () => {
  const s = new Settle(jupiter);
  const rows: [number, SettleState, SettleState][] = [
    [HOUR, "live", "live"],
    [8 * HOUR, "live", "live"],
    [DAY, "live", "live"],
    [3 * DAY, "live", "carried-on"],
    [7 * DAY, "too-fast", "too-fast"],
  ];
  for (const [rate, desk, phone] of rows) {
    assert.equal(s.state(at(0, rate, 100)), desk, `desktop at ${rate / HOUR} h/s`);
    assert.equal(s.state(at(0, rate, 2000)), phone, `phone at ${rate / HOUR} h/s`);
  }
  // Past about 6 days a second the planet turns in under two pictures.
  assert.equal(outrunsPictures(jupiter, 5 * DAY, 30), false);
  assert.equal(outrunsPictures(jupiter, 7 * DAY, 30), true);
});

test("at rest the state is Rest, whatever came before (absolute)", () => {
  for (const before of [HOUR, 3 * DAY, 30 * DAY]) {
    const s = new Settle(jupiter);
    // Playing at any rate, dragged at the end, then the clock stopped.
    assert.notEqual(s.state(at(1000, before, 5000)), "rest");
    s.moved("clock", 2000);
    assert.notEqual(s.state(at(2000 + SETTLE_REST_MS - 1, 0, 5000)), "rest", "half a second still has not passed");
    assert.equal(s.state(at(2000 + SETTLE_REST_MS, 0, 5000)), "rest");
    assert.equal(s.state(at(2000 + SETTLE_REST_MS, 0, 1e9)), "rest", "however dear the picture");
  }
  // A movement the layer does not move with leaves it at rest.
  const z = new Settle(nearby);
  z.moved("clock", 0);
  assert.equal(z.state(at(1, 0, 100)), "rest");
  z.moved("place", 10);
  assert.equal(z.state(at(11, 0, 100)), "carried-on");
  assert.equal(z.state(at(10 + SETTLE_REST_MS, 0, 100)), "rest");
});

test("the same date reached by play, by a drag and by a jump asks for the same proper picture", () => {
  const T = 123 * DAY + 4567.25;
  const keys: string[] = [];
  // By play: running at a day a second up to T, then stopped.
  const play = new Settle(jupiter);
  for (let k = 0; k < 10; k++) {
    const a = ask(play, k * 1000, DAY, T - (10 - k) * DAY);
    if (a) {
      play.started(a, k * 1000);
      play.built(a);
    }
  }
  play.moved("clock", 10_000);
  keys.push(properKey(ask(play, 10_000 + SETTLE_REST_MS, 0, T)!));
  // By a drag: many dates, released at T.
  const drag = new Settle(jupiter);
  for (let k = 0; k < 20; k++) {
    drag.moved("clock", k * 16);
    assert.equal(ask(drag, k * 16, 0, T + (k - 20) * 7 * DAY), null, "nothing is rebuilt mid-drag");
  }
  keys.push(properKey(ask(drag, 20 * 16 + SETTLE_REST_MS, 0, T)!));
  // By a jump (Back to now, a bookmark).
  const jump = new Settle(jupiter);
  jump.moved("clock", 0);
  keys.push(properKey(ask(jump, SETTLE_REST_MS, 0, T)!));
  assert.equal(new Set(keys).size, 1, keys.join(" / "));
  assert.equal(keys[0], `giant.cloud|${T}|v|512`);
});

test("with a build time of zero there is never a stand-in for want of the device (rule 5)", () => {
  const s = new Settle(jupiter);
  for (let rate = 1; rate < 1e9; rate *= 1.7) {
    const st = s.state(at(0, rate, 0));
    // Only the sampling limit (a picture outrun by the clock, question 5's long exposure) can give anything but Live.
    if (outrunsPictures(jupiter, rate, 30)) assert.equal(st, "too-fast");
    else assert.equal(st, "live", `at ${rate} s/s`);
  }
  const z = new Settle(nearby);
  z.moved("place", 0);
  assert.equal(z.state(at(1, 0, 0)), "live");
});

test("no more than one rebuild a second is asked for while anything moves", () => {
  const s = new Settle(jupiter);
  const asked: number[] = [];
  // Ten seconds at a day a second, sixty asks a second, every build instant: the date moves past a lattice step often.
  for (let ms = 0; ms <= 10_000; ms += 1000 / 60) {
    const a = ask(s, ms, DAY, (ms / 1000) * DAY * 2, { build_ms: 0 });
    if (!a) continue;
    s.started(a, ms);
    s.built(a);
    asked.push(ms);
  }
  assert.ok(asked.length >= 5, `${asked.length}`);
  for (let k = 1; k < asked.length; k++) assert.ok(asked[k]! - asked[k - 1]! >= REBUILD_CAP_MS, `${asked[k - 1]} then ${asked[k]}`);
  // A zoom's level, its place moving every frame and every build free: still one a second (no lattice spaces these).
  const z = new Settle(nearby);
  const zoomed: number[] = [];
  for (let ms = 0; ms <= 5000; ms += 1000 / 60) {
    z.moved("place", ms);
    const a = ask(z, ms, 0, 0, { build_ms: 0, view: `pose ${ms}` });
    if (!a) continue;
    z.started(a, ms);
    z.built(a);
    zoomed.push(ms);
  }
  assert.ok(zoomed.length >= 5 && zoomed.length <= 6, `${zoomed.length} asks in five seconds`);
  // And never while a build is under way, nor in a frame without spare time.
  const t = new Settle(jupiter);
  const first = ask(t, 0, DAY, 0)!;
  t.started(first, 0);
  assert.equal(ask(t, 5000, DAY, 10 * DAY), null);
  t.built(first);
  assert.equal(ask(t, 5000, DAY, 10 * DAY, { spare: false }), null);
  assert.ok(ask(t, 5000, DAY, 10 * DAY));
});

test("a stopped clock asks for exactly one build, and then none", () => {
  const s = new Settle(jupiter);
  s.moved("clock", 0);
  const asks: unknown[] = [];
  for (let ms = 0; ms <= 20_000; ms += 1000 / 60) {
    const a = ask(s, ms, 0, 42 * DAY);
    if (!a) continue;
    asks.push(a);
    s.started(a, ms);
    s.built(a);
  }
  assert.equal(asks.length, 1);
  assert.deepEqual(asks[0], { id: "giant.cloud", date_s: 42 * DAY, view: "v", size: 512, exact: true });
  // The camera turned (a new view): one more, at its own key.
  const b = ask(s, 21_000, 0, 42 * DAY, { view: "w" });
  assert.equal(b?.view, "w");
});

test("too fast asks for nothing until the rate drops; a build given up is asked for again", () => {
  const s = new Settle(jupiter);
  assert.equal(ask(s, 0, 30 * DAY, 0), null);
  const a = ask(s, 0, DAY, 0)!;
  s.started(a, 0);
  s.built(a, false);
  assert.equal(s.have, null);
  assert.deepEqual(ask(s, 2000, DAY, 0), a);
});

test("built at the size shown, from the ladder", () => {
  assert.equal(sizeFor(20), 64);
  assert.equal(sizeFor(64), 64);
  assert.equal(sizeFor(65), 128);
  assert.equal(sizeFor(700), 1024);
  assert.equal(sizeFor(5000), 1024);
  const asks = [
    { id: "a", date_s: 0, view: "", size: 64, exact: true },
    { id: "b", date_s: 0, view: "", size: 1024, exact: true },
    { id: "c", date_s: 0, view: "", size: 256, exact: true },
  ];
  assert.deepEqual(bySize(asks).map((x) => x.id), ["b", "c", "a"], "the biggest on screen first");
});

test("rebuild dates sit on a lattice anchored at the epoch, coarser on a slow device", () => {
  const own = 3.2 * DAY;
  assert.equal(latticeStep(own, DAY, 100), own);
  // A phone at 3 days a second: a build covers 6 days of clock, so the step doubles once to 6.4 days.
  assert.equal(latticeStep(own, 3 * DAY, 2000), 2 * own);
  assert.equal(latticeStep(null, DAY, 100), null);
  assert.equal(latticeDate(7 * DAY, own), 2 * own);
  assert.equal(latticeDate(-1, own), -own);
  // Every date between two lattice points asks for the same picture.
  assert.equal(latticeDate(own + 1, own), latticeDate(2 * own - 1, own));
});

test("a drag carries a kept picture within a step, holds the nearest further, else shows the too fast picture", () => {
  const step = 3.2 * DAY;
  assert.deepEqual(dragPicture(10 * DAY, [0, 9 * DAY, 30 * DAY], step), { kind: "carried", from_s: 9 * DAY });
  assert.deepEqual(dragPicture(10 * DAY, [12 * DAY], step), { kind: "carried", from_s: 12 * DAY }, "the carry runs backwards too");
  assert.deepEqual(dragPicture(20 * DAY, [0, 9 * DAY], step), { kind: "held", from_s: 9 * DAY });
  assert.deepEqual(dragPicture(20 * DAY, [], step), { kind: "too-fast" });
});

test("what is drawn: the proper picture at rest, honest kinds first, held without a GPU or under reduced motion", () => {
  const gpu = { reduced: false, gpu: true };
  assert.equal(drawFor(jupiter, "rest", gpu), "proper");
  assert.equal(drawFor(jupiter, "live", gpu), "carried");
  assert.equal(drawFor(jupiter, "carried-on", gpu), "carried");
  assert.equal(drawFor(jupiter, "too-fast", gpu), "exposure");
  assert.equal(drawFor(jupiter, "carried-on", { reduced: true, gpu: true }), "held");
  assert.equal(drawFor(jupiter, "too-fast", { reduced: false, gpu: false }), "held");
  assert.equal(drawFor(jupiter, "rest", { reduced: true, gpu: false }), "proper");
  assert.equal(drawFor(nearby, "carried-on", gpu), "coarse");
  assert.equal(drawFor(nearby, "too-fast", gpu), "held");
});

test("the settle is one fade of the interface's 240 ms", () => {
  assert.equal(settleWeight(0), 0);
  assert.equal(settleWeight(120), 0.5);
  assert.equal(settleWeight(240), 1);
  assert.equal(settleWeight(10_000), 1);
});
