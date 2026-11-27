// One epoch (RW17 part 2): the screen's years count from the viewed system's birth second, a real-sky system's days
// and the date its door opens on come from its anchor, and the device's UTC clock lands on the universe's TT second.
import { test } from "node:test";
import assert from "node:assert/strict";
import { anchorOf, drawnSecond, motionWord, screenYears, skyCalendar, taiMinusUtc, universeSecondOf } from "../src/sky-clock.ts";
import { birthDigits } from "../src/session.ts";
import { fromPackCalendar } from "../../chart-grammar/src/calendar.ts";
import { yearsToSeconds } from "../src/intent.ts";
import type { OrbitPlaced } from "../src/orbits.ts";

const J2000 = 63_115_200_000n;
const TT = (y: number, mo: number, d: number, h: number, mi: number, s: number) => BigInt((Date.UTC(y, mo - 1, d, h, mi, s) - Date.UTC(2000, 0, 1, 12, 0, 0)) / 1000);

test("the device's clock reads onto TT: J2000.0 is 11:58:55.816 UTC, and 2024's eclipse lands on its TD second", () => {
  // TT ran 32.184 + 32 s ahead of UTC at J2000.0.
  assert.equal(universeSecondOf(Date.UTC(2000, 0, 1, 11, 58, 55, 816), J2000), J2000);
  // Greatest eclipse 8 April 2024, 18:18:29 TD; TT minus UTC was 69.184 s (37 leap seconds).
  assert.equal(taiMinusUtc(Date.UTC(2024, 3, 8)), 37);
  assert.equal(universeSecondOf(Date.UTC(2024, 3, 8, 18, 18, 29) - 69_184, J2000), J2000 + TT(2024, 4, 8, 18, 18, 29));
  // Across a leap second the universe clock does not jump: 2016's last UTC second and 2017's first are two seconds apart.
  const before = universeSecondOf(Date.UTC(2016, 11, 31, 23, 59, 59), J2000);
  const after = universeSecondOf(Date.UTC(2017, 0, 1, 0, 0, 0), J2000);
  assert.equal(after - before, 2n);
  // Before J2000 it rounds down, not towards zero.
  assert.equal(universeSecondOf(Date.UTC(2000, 0, 1, 11, 58, 55, 316), J2000), J2000 - 1n);
});

test("the screen's years and the drawn second are one mapping, exact at a star's age", () => {
  const birth = -144_131_731_200_000_000n; // a star born 4.567 Gyr before second 0, as Sol's spec gives it
  assert.equal(drawnSecond(birth, 4.5673e9), birth + yearsToSeconds(4.5673e9));
  // Without a birth second (the campaign's own scene) nothing moves.
  assert.equal(drawnSecond(null, 12.5), yearsToSeconds(12.5));
  // Back again, to the double's resolution at that age (about 30 s in years).
  const at = J2000 + 840_000_000n;
  const yr = screenYears(birth, at);
  assert.ok(Math.abs(Number(drawnSecond(birth, yr) - at)) <= 64, `${drawnSecond(birth, yr) - at}`);
});

test("a real-sky system's calendar starts at midnight of J2000's day, read from its anchor, on the screen's axis", () => {
  const cal = skyCalendar(J2000);
  assert.equal(cal.epoch_s, 63_115_156_800);
  assert.equal(cal.epoch_year, 2000);
  // On a viewed system's axis the epoch is taken from its birth second; without one it is the universe second.
  const birth = -144_000_000_000_000_000n;
  const yr = 31_557_600;
  assert.equal(fromPackCalendar(cal).epoch_yr, 63_115_156_800 / yr);
  assert.equal(fromPackCalendar(cal, birth).epoch_yr, Number(63_115_156_800n - birth) / yr);
});

test("the anchor and the birth second are read exactly", () => {
  assert.equal(anchorOf({ "u:x/sys:sol": { "system.age": 1 }, "u:x/sys:sol/sun": { "time.j2000_s": 63_115_200_000 } }), J2000);
  assert.equal(anchorOf({ "u:x/sys:a": { "system.age": 1 } }), null);
  // Past 2^53 a double cannot hold it: the reply's own digits are read.
  const text = `{"type":"system_resolved","spec":{"birth_year":9200000000},"birth_s":290330000000000123,"surfaces":[]}`;
  assert.equal(birthDigits(text), 290_330_000_000_000_123n);
  assert.equal(birthDigits(`{"facts":{"a":{"system.birth_s":5}},"birth_s":-7}`), -7n);
  assert.equal(birthDigits(`{"type":"system_resolved"}`), undefined);
});

test("the chart says where the bodies came from only when one names a motion model", () => {
  const at = (motion?: "measured" | "plausible") => [{ at_s: 0, r_m: [0, 0, 0] as [number, number, number], phase_deg: 0, ...(motion ? { motion } : {}) }];
  const model = { model: "vsop87a.emb" };
  const b = (o: Partial<OrbitPlaced>): OrbitPlaced => ({ address: "a", placed: true, ...o });
  assert.equal(motionWord([b({ at: at() })]), null);
  assert.equal(motionWord([b({ motion: { ...model, unused: "no anchor" }, at: at() })]), null);
  assert.equal(motionWord([b({ motion: model, at: at("measured") })])?.word, "measured");
  assert.equal(motionWord([b({ motion: model, at: at("plausible") })])?.word, "plausible");
  assert.equal(motionWord([b({ motion: model, at: at("measured") }), b({ motion: model, at: at("plausible") })])?.word, "mixed");
});
