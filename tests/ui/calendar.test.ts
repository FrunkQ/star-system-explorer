import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { GREGORIAN, calendarOf, checkCalendar, dateOf, formatDate, parseDate, startOf, type Anchor } from "../../packages/ui-contract/src/calendar.ts";

const DAY = 86_400;
const y2000: Anchor = { at_s: 0, year: 2000 };

test("Gregorian dates: months, long years, weekdays, before the anchor", () => {
  assert.deepEqual(checkCalendar(GREGORIAN), []);
  assert.equal(formatDate(GREGORIAN, 0, y2000), "1 January 2000");
  assert.equal(formatDate(GREGORIAN, 31 * DAY, y2000), "1 February 2000");
  assert.equal(formatDate(GREGORIAN, 59 * DAY, y2000), "29 February 2000"); // 2000 is long (divisible by 400)
  assert.equal(formatDate(GREGORIAN, 365 * DAY, y2000), "31 December 2000");
  assert.equal(formatDate(GREGORIAN, 366 * DAY, y2000), "1 January 2001");
  assert.equal(formatDate(GREGORIAN, -1, y2000), "31 December 1999");
  // 1900 was not long, 2024 was.
  const y1900: Anchor = { at_s: 0, year: 1900 };
  assert.equal(formatDate(GREGORIAN, 59 * DAY, y1900), "1 March 1900");
  assert.equal(dateOf(GREGORIAN, startOf(GREGORIAN, 2024, 60, y2000), y2000).mday, 29);
  // Weekdays: 1 January 2000 was a Saturday, 19 November 2026 a Thursday.
  const sat = GREGORIAN;
  assert.equal(dateOf(sat, 0, y2000).weekday, "Saturday");
  const t = parseDate(sat, "19 November 2026", y2000)!;
  assert.equal(dateOf(sat, t, y2000).weekday, "Thursday");
  // Far from the anchor stays exact and cheap: 400-year cycles of 146 097 days.
  assert.equal(formatDate(GREGORIAN, 146_097 * 1000 * DAY, y2000), "1 January 402000");
  assert.equal(formatDate(GREGORIAN, -146_097 * DAY, y2000), "1 January 1600");
});

test("dates read back to the same second, both ways", () => {
  for (const d of [0, 1, 58, 59, 60, 364, 365, 366, 1000, -1, -400, 40_000]) {
    const t = d * DAY;
    const text = formatDate(GREGORIAN, t, y2000);
    assert.equal(parseDate(GREGORIAN, text, y2000), t, `${d}: ${text}`);
  }
  assert.equal(parseDate(GREGORIAN, "30 February 2001", y2000), undefined);
  assert.equal(parseDate(GREGORIAN, "Smarch 3 2001", y2000), undefined);
});

test("the Traveller pack's Imperial calendar: day-year, Holiday outside the weeks", () => {
  const pack = JSON.parse(readFileSync(fileURLToPath(new URL("../../packs/traveller/pack.json", import.meta.url)), "utf8"));
  const { calendar, problems } = calendarOf([{}, pack]);
  assert.deepEqual(problems, []);
  const imp = calendar!;
  assert.equal(formatDate(imp, 0), "001-1105", "the calendar's own epoch");
  const a: Anchor = { at_s: 1e17, year: 1105 };
  assert.equal(formatDate(imp, a.at_s, a), "001-1105");
  assert.equal(dateOf(imp, a.at_s, a).name, "Holiday");
  assert.equal(dateOf(imp, a.at_s, a).weekday, undefined);
  assert.equal(dateOf(imp, a.at_s + DAY, a).weekday, imp.week!.names[0]);
  assert.equal(dateOf(imp, a.at_s + 8 * DAY, a).weekday, imp.week!.names[0]);
  assert.equal(formatDate(imp, a.at_s + 364 * DAY, a), "365-1105");
  assert.equal(formatDate(imp, a.at_s + 365 * DAY, a), "001-1106");
  assert.equal(parseDate(imp, "187-1105", a), a.at_s + 186 * DAY);
  assert.equal(parseDate(imp, "366-1105", a), undefined, "no long years");
});

test("bad calendars say what is wrong", () => {
  const ok = { name: "X", epoch_s: 0, day_s: 86400, format: "{year}" };
  assert.match(checkCalendar({ ...ok, year_days: 10, months: [{ name: "a", days: 9 }] }).join(), /add up to 9/);
  assert.match(checkCalendar({ ...ok, year_days: 0 }).join(), /year_days/);
  assert.match(checkCalendar({ ...ok, year_days: 10, day_s: 0 }).join(), /day_s/);
  assert.match(calendarOf([{ provides: { other: { calendar: { name: "Y" } } } }]).problems.join(), /epoch_s/);
  assert.equal(calendarOf([{}]).calendar, undefined);
});

test("a pack's words replace general terms; anything else keeps its own", async () => {
  const { termsOf, word } = await import("../../packages/ui-contract/src/words.ts");
  const pack = JSON.parse(readFileSync(fileURLToPath(new URL("../../packs/traveller/pack.json", import.meta.url)), "utf8"));
  const t = termsOf([{ provides: { other: { vocabulary: { terms: { planet: "orb" } } } } }, pack]);
  assert.equal(word(t, "planet"), "world");
  assert.equal(word(t, "Planet"), "World");
  assert.equal(word(t, "Spacecraft"), "Ship");
  assert.equal(word(t, "nebula"), "nebula");
  assert.equal(word(termsOf([]), "planet"), "planet");
});
