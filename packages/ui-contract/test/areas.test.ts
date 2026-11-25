// OV4, the drawing side of territory: holdings to areas in a view's frame by the kernel's reach rule, lettering and
// the key's shares. Pure: no kernel (the kernel's own rule is checked in sse-record's holdings tests).
import { test } from "node:test";
import assert from "node:assert/strict";
import { areaShares, areasFromHoldings, insideRings, itemArea, letterArea, nearestWithin, parsecs, traceRings } from "../src/areas.ts";
import type { Holding } from "../src/overlay.ts";

const hold = (place: string, key: string, extra: Partial<Holding> = {}): Holding => ({ place, layer: "powers", key, status: "holds", via: "held", why: "", ...extra });

test("traced rings close around what the test says yes to, holes included", () => {
  const at = (i: number, j: number): [number, number] => [i, j];
  const square = traceRings(10, 10, (i, j) => i >= 2 && i <= 6 && j >= 2 && j <= 6, at);
  assert.equal(square.length, 1);
  assert.ok(insideRings(square, [4, 4]));
  assert.ok(!insideRings(square, [8, 8]));
  assert.ok(Math.abs(Math.abs(itemArea(square)) - 5 * 5) < 3, `about 25 (${itemArea(square)})`);
  const donut = traceRings(12, 12, (i, j) => Math.max(Math.abs(i - 6), Math.abs(j - 6)) <= 4 && Math.max(Math.abs(i - 6), Math.abs(j - 6)) >= 2, at);
  assert.equal(donut.length, 2, "an outside and a hole");
  assert.ok(insideRings(donut, [6, 3]));
  assert.ok(!insideRings(donut, [6, 6]), "the hole is bare");
  assert.ok(itemArea(donut) > 0 && itemArea(donut) < Math.abs(itemArea([donut[0]!])) + Math.abs(itemArea([donut[1]!])));
});

test("drawn borders agree with the reach rule at 1,000 sample points", () => {
  // A small sector: twelve held systems of three powers, a claim and an influence.
  let seed = 11;
  const rnd = () => (seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648;
  const places = Array.from({ length: 12 }, (_, i) => ({ place: `u:k/sys:f${i}`, at: [rnd() * 30, rnd() * 20] as [number, number], key: ["crown", "league", "vesh"][i % 3]! }));
  const pos = new Map(places.map((p) => [p.place, p.at]));
  const holdings = [...places.map((p) => hold(p.place, p.key)), hold("u:k/sys:f0", "league", { status: "claims" }), hold("u:k/sys:f5", "guild", { status: "influences", strength: 0.5, extent: { reach: { value: 6, unit: "pc" } } })];
  const ask = { frame: "galaxy-plane-pc" as const, bounds: [0, 0, 30, 20] as [number, number, number, number], positionOf: (p: string) => pos.get(p) ?? null, inkOf: (k: string) => ({ slot: (["crown", "league", "vesh", "guild"].indexOf(k) + 1) as 1 }), nameOf: (k: string) => k.toUpperCase(), cells: 150 };
  const items = areasFromHoldings(holdings, ask);
  const holds = items.filter((i) => i.status === "holds");
  assert.deepEqual(holds.map((i) => i.key), ["crown", "league", "vesh"]);
  assert.ok(items.some((i) => i.status === "claims" && i.key === "league"));
  const inf = items.find((i) => i.status === "influences")!;
  assert.equal(inf.strength, 0.5);
  const seeds = places.map((p) => ({ ...p, reach: parsecs({ value: 4, unit: "pc" })! }));
  const step = 30 / 150;
  let checked = 0;
  for (let n = 0; n < 1000; n++) {
    const p: [number, number] = [rnd() * 30, rnd() * 20];
    const want = nearestWithin(seeds, p);
    // A point within a cell of a border may fall either side of the traced line; it is not a disagreement.
    const near = [[step, 0], [-step, 0], [0, step], [0, -step], [step, step], [-step, -step], [step, -step], [-step, step]].some(([dx, dy]) => nearestWithin(seeds, [p[0] + dx!, p[1] + dy!]) !== want);
    if (near) continue;
    checked++;
    const drawn = holds.filter((i) => insideRings(i.rings, p)).map((i) => i.key);
    assert.deepEqual(drawn, want ? [want] : [], `at ${p.map((x) => x.toFixed(2))}`);
  }
  assert.ok(checked > 850, `most samples are away from a border (${checked})`);
  // A held place sits inside its own power's area.
  for (const pl of places) assert.ok(insideRings(holds.find((i) => i.key === pl.key)!.rings, pl.at), pl.place);
  // Shares add to one; lettering lies along the long axis inside the bounds.
  const shares = areaShares(items);
  assert.ok(Math.abs(shares.reduce((s, x) => s + x.share, 0) - 1) < 1e-9);
  for (const h of holds) {
    const l = letterArea(h.rings)!;
    assert.ok(l.length > 0 && l.turn >= -90 && l.turn <= 90);
  }
});

test("a stretched area letters along its length; ended holdings, unplaced places and pencil", () => {
  const l = letterArea([[[0, 0], [10, 0], [10, 2], [0, 2]]])!;
  assert.ok(Math.abs(l.turn) < 1e-6 && Math.abs(l.length - 10) < 1e-9 && Math.abs(l.width - 2) < 1e-9);
  const tall = letterArea([[[0, 0], [2, 0], [2, 10], [0, 10]]])!;
  assert.ok(Math.abs(Math.abs(tall.turn) - 90) < 1e-6);
  const pos = new Map<string, [number, number]>([["a", [5, 5]]]);
  const ask = { frame: "galaxy-plane-pc" as const, bounds: [0, 0, 10, 10] as [number, number, number, number], positionOf: (p: string) => pos.get(p) ?? null, inkOf: () => "ink" as const, cells: 40 };
  assert.equal(areasFromHoldings([hold("a", "crown", { until_s: 5 }), hold("b", "crown")], ask).length, 0, "ended or not placed: nothing drawn");
  const secret = areasFromHoldings([hold("a", "crown")], { ...ask, pencil: () => true });
  assert.equal(secret[0]!.pencil, true);
  const drawn = areasFromHoldings([hold("x", "crown", { extent: { outline: [[1, 1], [3, 1], [3, 3]], frame: "galaxy-plane-pc" } })], ask);
  assert.deepEqual(drawn[0]!.rings, [[[1, 1], [3, 1], [3, 3]]], "an outline is drawn as stored");
  assert.equal(parsecs({ value: 3.26156, unit: "ly" })!.toFixed(5), "1.00000");
});
