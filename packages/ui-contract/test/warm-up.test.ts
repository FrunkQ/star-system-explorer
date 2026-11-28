// Warming up (FL7, SSE4-visual-flourish-design.md 6A and 8A): the queue of builds known to be coming, the time told,
// the hold-backs and the memory cache. Pure; nothing here may touch storage.
import { test } from "node:test";
import assert from "node:assert/strict";
import { BuildTimes, WarmCache, levelPicture, readyLine, warmOrder, warmPlan, type WarmItem } from "../src/settle.ts";

const level = (i: number, needed: number, on_screen = false): WarmItem => ({ key: `journey.l${i}`, kind: `level${i}`, needed_in_s: needed, on_screen, distance: i });

test("builds are asked for in the order they are needed, and what is on screen is never queued behind them (absolute)", () => {
  const items = [level(3, 30), level(1, 5), level(2, 12), { key: "screen", kind: "giant.cloud", needed_in_s: 0, on_screen: true, distance: 0 }, level(4, 45)];
  const order = warmOrder(items).map((i) => i.key);
  assert.equal(order[0], "screen");
  assert.deepEqual(order, ["screen", "journey.l1", "journey.l2", "journey.l3", "journey.l4"]);
  // Even when something ahead is needed sooner than the screen's own picture claims.
  const odd = warmOrder([{ ...level(1, -5) }, { key: "screen", kind: "x", needed_in_s: 99, on_screen: true, distance: 0 }]);
  assert.equal(odd[0]!.key, "screen");
});

test("a hot device warms one level only, and only when the viewer asked; data saver and memory hold back too", () => {
  const items = [level(1, 5), level(2, 12), level(3, 30), { key: "screen", kind: "x", needed_in_s: 0, on_screen: true, distance: 0 }];
  const calm = { hot: false, data_saver: false, memory_over: false, asked: false };
  assert.equal(warmPlan(items, calm).length, 4);
  assert.deepEqual(warmPlan(items, { ...calm, hot: true, asked: true }).map((i) => i.key), ["screen", "journey.l1"]);
  assert.deepEqual(warmPlan(items, { ...calm, hot: true }).map((i) => i.key), ["screen"]);
  assert.deepEqual(warmPlan(items, { ...calm, data_saver: true }).map((i) => i.key), ["screen"]);
  assert.deepEqual(warmPlan(items, { ...calm, memory_over: true, asked: true }).map((i) => i.key), ["screen", "journey.l1"]);
});

test("the time told: the design's estimate the first time, said as about; after one run, within a factor of two", () => {
  const t = new BuildTimes();
  assert.deepEqual(t.estimate("journey.nearby", 8700), { ms: 8700, about: true });
  // The first run took 37 s (the phone stand-in, GZ0); the second takes 33 s.
  t.record("journey.nearby", 37_000);
  const told = t.estimate("journey.nearby", 8700);
  assert.equal(told.about, false);
  const second = 33_000;
  assert.ok(told.ms / second <= 2 && second / told.ms <= 2, `${told.ms}`);
  t.record("journey.nearby", Number.NaN);
  assert.equal(t.estimate("journey.nearby", 0).ms, 37_000);
  assert.equal(readyLine("the way down", 3, 7, 20_400, true), "Getting the way down ready: about 20 seconds, 3 of 7 levels.");
  assert.equal(readyLine("the way down", 7, 7, 0, false), "The way down is ready.");
  assert.equal(readyLine("the way down", 0, 1, 200_000, false), "Getting the way down ready: 3 minutes, 0 of 1 level.");
});

test("under memory pressure the warm cache is emptied before anything on screen, whole levels, furthest first", () => {
  const c = new WarmCache();
  c.keep({ key: "screen", bytes: 100, warm: false, level: 0 });
  for (const l of [1, 2, 3]) for (const p of ["a", "b"]) c.keep({ key: `l${l}${p}`, bytes: 10, warm: true, level: l });
  // Need 15 bytes: level 3 goes whole (20 bytes), nothing else.
  assert.deepEqual(c.shed(15, 0), ["l3a", "l3b"]);
  assert.ok(c.has("l2a") && c.has("screen"));
  // Need a lot: every warm level, furthest first, then the screen's.
  assert.deepEqual(c.shed(130, 0), ["l2a", "l2b", "l1a", "l1b", "screen"]);
  // A picture on screen that is not a level, beside one warm level: the warm level goes first, however near.
  const e = new WarmCache();
  e.keep({ key: "giant.cloud", bytes: 50, warm: false, level: null });
  e.keep({ key: "l1", bytes: 50, warm: true, level: 1 });
  assert.deepEqual(e.shed(10, 1), ["l1"]);
  // A viewer at level 3 gives up level 0's warm picture before level 2's.
  const d = new WarmCache();
  d.keep({ key: "l0", bytes: 5, warm: true, level: 0 });
  d.keep({ key: "l2", bytes: 5, warm: true, level: 2 });
  assert.deepEqual(d.shed(1, 3), ["l0"]);
});

test("a journey whose levels are all warmed never shows a stand-in; one not ready shows its coarse picture or holds", () => {
  const c = new WarmCache();
  const keys = [1, 2, 3, 4].map((i) => `journey.l${i}`);
  for (const k of keys) c.keep({ key: k, bytes: 1, warm: true, level: 0 });
  assert.ok(keys.every((k) => levelPicture(k, c, false) === "proper"));
  assert.equal(levelPicture("journey.l9", c, true), "coarse");
  assert.equal(levelPicture("journey.l9", c, false), "hold");
});

test("nothing warmed is written to storage", () => {
  const g = globalThis as unknown as Record<string, unknown>;
  const was = { localStorage: g.localStorage, sessionStorage: g.sessionStorage, indexedDB: g.indexedDB };
  const touched: string[] = [];
  const trap = (name: string) => new Proxy({}, { get: (_t, p) => (touched.push(`${name}.${String(p)}`), () => undefined) });
  g.localStorage = trap("localStorage");
  g.sessionStorage = trap("sessionStorage");
  g.indexedDB = trap("indexedDB");
  try {
    const c = new WarmCache();
    const t = new BuildTimes();
    for (let i = 0; i < 20; i++) {
      c.keep({ key: `k${i}`, bytes: 1e6, warm: i > 2, level: i });
      t.record(`kind${i % 3}`, i * 10);
    }
    c.shed(5e6, 0);
    warmPlan([level(1, 1), level(2, 2)], { hot: true, data_saver: false, memory_over: false, asked: true });
    t.estimate("kind1", 5);
  } finally {
    Object.assign(g, was);
  }
  assert.deepEqual(touched, []);
});
