import { test } from "node:test";
import assert from "node:assert/strict";
import { formatYears, fraction, keyStep, labelRows, niceStep, ticks } from "../src/ruler.ts";
import { distanceAt, fitScale, polar, radius } from "../src/scale.ts";

test("years read the way the chart writes them", () => {
  assert.equal(formatYears(4_610_000_000), "4.61 Gyr");
  assert.equal(formatYears(10_000_000_000), "10 Gyr");
  assert.equal(formatYears(820_000_000), "820 Myr");
  assert.equal(formatYears(12_500), "12.5 kyr");
  assert.equal(formatYears(300), "300 yr");
  assert.equal(formatYears(0), "0 yr");
});

test("ruler ticks are nice, ordered and labelled on majors", () => {
  assert.equal(niceStep(10e9, 10), 1e9);
  const t = ticks(0, 10e9, 10);
  assert.equal(t.length, 11);
  assert.equal(t[0]!.label, "0 yr");
  assert.equal(t[10]!.label, "10 Gyr");
  assert.ok(t.every((x, i) => i === 0 || x.at > t[i - 1]!.at));
});

test("keyboard steps stay in range and land on whole years", () => {
  assert.equal(keyStep("ArrowRight", 4.61e9, 0, 10e9), 4.71e9);
  assert.equal(keyStep("PageDown", 0.5e9, 0, 10e9), 0);
  assert.equal(keyStep("End", 1, 0, 10e9), 10e9);
  assert.equal(keyStep("x", 1, 0, 10), null);
  assert.equal(fraction(5, 0, 10), 0.5);
});

test("log-radial scale is monotone and fits its orbits", () => {
  const s = fitScale([0.2, 0.63, 1.4, 5, 30], 20, 400);
  const rs = [0.2, 0.63, 1.4, 5, 30].map((a) => radius(a, s));
  assert.ok(rs.every((r, i) => i === 0 || r > rs[i - 1]!));
  assert.ok(rs[0]! > 20 && rs[4]! < 400);
  for (const a of [0.2, 1.4, 30]) assert.ok(Math.abs(distanceAt(radius(a, s), s) - a) < 1e-9 * a);
  const [x, y] = polar(0, 0, 10, 90);
  assert.ok(Math.abs(x) < 1e-9 && Math.abs(y + 10) < 1e-9);
});

test("ruler labels never overlap: a third crowded label keeps its marker and moves to the tooltip", () => {
  const ev = (at_yr: number, label: string) => ({ at_yr, label });
  // 1000 px over 0 to 10: one year is 100 px.
  assert.deepEqual(labelRows([ev(1, "Red giant"), ev(5, "Nebula")], 0, 10, 1000), [0, 0], "far apart: both on the first row");
  assert.deepEqual(labelRows([ev(1, "Planetary nebula"), ev(1.1, "Engulfed: Tarn"), ev(1.2, "White dwarf cools")], 0, 10, 1000), [0, 1, -1]);
  // A label that ends before the next one starts shares the row.
  assert.deepEqual(labelRows([ev(1, "ab"), ev(1.5, "cd")], 0, 10, 1000), [0, 0]);
});
