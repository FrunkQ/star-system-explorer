// U11: the calendar and the zooming ruler's arithmetic.
import { test } from "node:test";
import assert from "node:assert/strict";
import { YEAR_S, around, calendarTicks, checkCalendar, dateParts, dateWords, defaultCalendar, spanWords, steps, timeWords, zoom, type Calendar } from "../src/calendar.ts";

const EPOCH = 4.61e9;
const cal = defaultCalendar(EPOCH);
const day = 86400 / YEAR_S;

test("dates count from the campaign's start, to the hour, even 4.6 billion years in", () => {
  assert.equal(dateWords(EPOCH, cal), "Year 1, day 1");
  assert.equal(dateWords(EPOCH + 2 * day + 0.5 * day, cal, true), "Year 1, day 3, 12:00");
  assert.equal(dateWords(EPOCH + 365.25 * day, cal), "Year 2, day 1");
  assert.equal(dateParts(EPOCH - day, cal).year, 0, "the day before the start is in year 0");
  assert.equal(dateWords(EPOCH + 40 * day + (14 / 24) * day, cal, true), "Year 1, day 41, 14:00");
});

test("a pack calendar with months and a Traveller-style format", () => {
  const imperial: Calendar = { id: "t", label: "Imperial", epoch_yr: EPOCH, day_s: 86400, year_days: 365, format: "{day:03}-{year}" };
  assert.deepEqual(checkCalendar(imperial), []);
  assert.equal(dateWords(EPOCH + 4 * imperial.day_s / YEAR_S, imperial), "005-1");
  const months: Calendar = { id: "m", label: "Tarn", epoch_yr: EPOCH, day_s: 100_000, year_days: 300, months: [{ name: "Thaw", days: 100 }, { name: "Bloom", days: 100 }, { name: "Ice", days: 100 }] };
  assert.equal(dateWords(EPOCH + 150 * months.day_s / YEAR_S, months), "51 Bloom, year 1");
  assert.ok(checkCalendar({ epoch_yr: 0, day_s: 0, year_days: 1 }).length);
});

test("the readout and ticks change register with the window", () => {
  assert.equal(timeWords(EPOCH, cal, 1e9), "4.61 Gyr");
  assert.equal(timeWords(EPOCH + 3 * day, cal, 5 * day), "Year 1, day 4, 00:00");
  assert.equal(timeWords(EPOCH + 3 * day, cal, 400 * day), "Year 1, day 4");
  const deep = calendarTicks(0, 1e10, cal);
  assert.ok(deep.some((t) => t.label === "4 Gyr"));
  const days = calendarTicks(EPOCH, EPOCH + 8 * day, cal);
  assert.ok(days.length >= 8 && days.length <= 10, `${days.length} day ticks`);
  assert.equal(days[0]!.label, "Year 1, day 1");
  const hours = calendarTicks(EPOCH, EPOCH + 0.3 * day, cal);
  assert.ok(hours.some((t) => t.label === "06:00"));
  assert.equal(spanWords(3 * day), "3 days");
  assert.equal(spanWords(6 / 24 * day), "6 hours");
  assert.equal(spanWords(2.5), "2.5 years");
  assert.ok(steps(cal).find((s) => s.id === "day")!.yr === day);
});

test("zoom keeps the point under the pointer, stays inside the span, and stops at an hour", () => {
  const b = { from: 0, to: 1e10 };
  const w = zoom(b, 0.5, 5e9, b);
  assert.deepEqual(w, { from: 2.5e9, to: 7.5e9 });
  const edge = zoom(b, 0.5, 0, b);
  assert.equal(edge.from, 0);
  let z = b;
  for (let i = 0; i < 200; i++) z = zoom(z, 0.1, EPOCH, b);
  assert.ok(Math.abs((z.to - z.from) - 1 / (24 * 365.25)) < 3e-6, "no narrower than an hour");
  assert.ok(z.from <= EPOCH && EPOCH <= z.to);
  const a = around(EPOCH, 10 * day, b);
  assert.ok(Math.abs(a.to - a.from - 10 * day) < 1e-6 && a.from < EPOCH && a.to > EPOCH);
  assert.deepEqual(zoom(b, 10, 5e9, b), b, "never wider than the whole span");
});

test("the ruler's glide: log-scale zoom about the shared point, linear pan", async () => {
  const { between } = await import("../src/calendar.ts");
  const a = { from: 0, to: 1e10 };
  const b = { from: 4.6e9 - 0.5, to: 4.6e9 + 0.5 };
  assert.deepEqual(between(a, b, 0), a);
  assert.deepEqual(between(a, b, 1), b);
  const m = between(a, b, 0.5);
  assert.ok(Math.abs(m.to - m.from - 1e5) / 1e5 < 1e-6, `half way in log span: ${m.to - m.from}`);
  assert.ok(m.from < 4.6e9 && m.to > 4.6e9, "the shared point stays in view");
  const p = between({ from: 0, to: 10 }, { from: 10, to: 20 }, 0.25);
  assert.deepEqual(p, { from: 2.5, to: 12.5 });
});

test("a pack's calendar: Imperial dates count from its epoch year", async () => {
  const { fromPackCalendar } = await import("../src/calendar.ts");
  const c = fromPackCalendar({ name: "Imperial calendar", epoch_s: 0, epoch_year: 1105, day_s: 86400, year_days: 365, format: "{day:03}-{year}" });
  assert.equal(dateWords(0, c), "001-1105");
  assert.equal(dateWords((400 * 86400) / YEAR_S, c), "036-1106");
  const m = fromPackCalendar({ name: "Months", epoch_s: 0, day_s: 86400, year_days: 60, months: [{ name: "Thaw", days: 30 }, { name: "Frost", days: 30 }], format: "{d} {month} {year}" });
  assert.equal(dateWords((45 * 86400) / YEAR_S, m), "16 Frost 1");
});
