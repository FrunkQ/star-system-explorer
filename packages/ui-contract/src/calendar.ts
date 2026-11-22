/**
 * Calendars (handoff 6, "universe time ... displayed in whatever calendar a
 * pack defines"; 7A part 8). A calendar is pack data over the campaign clock:
 * how long a day is, how many days a year has (and which years are long), its
 * months, weeks and named days outside the weeks, and how a date is written.
 * The campaign says which universe second a given year begins at (the
 * anchor); nothing else ties a calendar to the universe, so one universe can
 * show several calendars side by side.
 *
 * The shape is the kernel's (`provides.other.calendar`, op `calendar`) with
 * optional extras: long years, named days and weeks.
 *
 * Pure: no clock, no locale. The same definition gives the same date on
 * every device.
 */
import { constant } from "./sheet.ts";

export interface Calendar {
  name: string;
  /** Universe second at the start of day 1 of `epoch_year`. */
  epoch_s: number;
  /** The year that begins at epoch_s. 1 when left out. */
  epoch_year?: number;
  /** Seconds in one day. */
  day_s: number;
  /** Days in an ordinary year. */
  year_days: number;
  /** Long years: every `every` years one more day, except every `except_every`, but again every `but_every`. */
  leap?: { every: number; except_every?: number; but_every?: number };
  /** Months in order. Their days add up to year_days; the leap day goes to month `leap_month` (0-based). */
  months?: { name: string; days: number }[];
  leap_month?: number;
  /** Days of the year (1-based) that stand outside the weeks, with their names. */
  named_days?: { day: number; name: string; outside_weeks?: boolean }[];
  /**
   * Weeks: their names, and which day (1-based) is the first day of a week.
   * By default weeks start again each year and `first_day` is a day of the
   * year; with `continuous` they run on across years and `first_day` is a
   * day of the anchor's year.
   */
  week?: { names: string[]; first_day?: number; continuous?: boolean };
  /**
   * How a date is written. Tokens: {year}, {day} (day of the year),
   * {month} (name), {mday} (day of the month), {weekday}, {name} (the named
   * day, if any); a number token padded with zeros as {day:03}.
   */
  format: string;
}

/** Universe second `at_s` is the first moment of day 1 of `year`. A campaign may give its own; otherwise the calendar's epoch. */
export interface Anchor {
  at_s: number;
  year: number;
}

export interface CalendarDate {
  year: number;
  /** Day of the year, 1-based. */
  day: number;
  month?: string;
  /** Day of the month, 1-based. */
  mday?: number;
  weekday?: string;
  name?: string;
  /** Seconds into the day. */
  second: number;
}

const DAY = constant("day");

export function isLong(cal: Calendar, year: number): boolean {
  const l = cal.leap;
  if (!l) return false;
  const div = (n: number | undefined) => n !== undefined && n > 0 && mod(year, n) === 0;
  if (!div(l.every)) return false;
  if (div(l.except_every) && !div(l.but_every)) return false;
  return true;
}

export function yearLength(cal: Calendar, year: number): number {
  return cal.year_days + (isLong(cal, year) ? 1 : 0);
}

/** Problems with a calendar definition, as plain sentences. Empty when it is sound. */
export function checkCalendar(c: unknown): string[] {
  const out: string[] = [];
  const cal = c as Partial<Calendar> | null;
  if (!cal || typeof cal !== "object") return ["a calendar is an object"];
  if (typeof cal.name !== "string" || !cal.name) out.push("name is missing");
  if (typeof cal.epoch_s !== "number" || !Number.isFinite(cal.epoch_s)) out.push("epoch_s is missing");
  if (cal.epoch_year !== undefined && !Number.isInteger(cal.epoch_year)) out.push("epoch_year must be a whole number");
  if (typeof cal.format !== "string") out.push("format is missing");
  if (!Number.isInteger(cal.year_days) || (cal.year_days ?? 0) < 1) out.push("year_days must be a whole number of at least 1");
  if (!(typeof cal.day_s === "number" && cal.day_s > 0)) out.push("day_s must be positive");
  if (cal.months) {
    const sum = cal.months.reduce((s, m) => s + m.days, 0);
    if (sum !== cal.year_days) out.push(`months add up to ${sum} days, not ${cal.year_days}`);
    if (cal.leap && (cal.leap_month === undefined || !cal.months[cal.leap_month])) out.push("a calendar with months and long years needs leap_month");
  }
  if (cal.week && !(cal.week.names?.length > 0)) out.push("week needs names");
  if (cal.week?.continuous && cal.named_days?.some((n) => n.outside_weeks)) out.push("days outside the weeks need weeks that start again each year");
  for (const n of cal.named_days ?? []) if (!(n.day >= 1 && n.day <= (cal.year_days ?? 0) + 1)) out.push(`named day ${n.name} is outside the year`);
  return out;
}

/**
 * The calendar a stack provides (`provides.other.calendar`, the kernel's key;
 * the top pack that has one wins), with any problems.
 */
export function calendarOf(packs: readonly unknown[]): { calendar?: Calendar; problems: string[] } {
  for (let i = packs.length - 1; i >= 0; i--) {
    const raw = (packs[i] as { provides?: { other?: { calendar?: unknown } } } | null)?.provides?.other?.calendar;
    if (raw === undefined) continue;
    const e = checkCalendar(raw);
    return e.length ? { problems: e.map((x) => `calendar: ${x}`) } : { calendar: raw as Calendar, problems: [] };
  }
  return { problems: [] };
}

export function epochOf(cal: Calendar): Anchor {
  return { at_s: cal.epoch_s, year: cal.epoch_year ?? 1 };
}

/** Years after which the pattern of long years repeats. */
function period(cal: Calendar): number {
  const l = cal.leap;
  if (!l) return 1;
  return Math.max(l.every, l.except_every ?? 0, l.but_every ?? 0);
}

function mod(a: number, n: number): number {
  return ((a % n) + n) % n;
}

/** The date at universe second `t_s`. */
export function dateOf(cal: Calendar, t_s: number, anchor: Anchor = epochOf(cal)): CalendarDate {
  const day_s = cal.day_s;
  const since = t_s - anchor.at_s;
  let days = Math.floor(since / day_s);
  const total = days;
  const second = since - days * day_s;
  let year = anchor.year;
  // Whole periods first, so deep time stays cheap.
  const p = period(cal);
  let pDays = 0;
  for (let i = 0; i < p; i++) pDays += yearLength(cal, year + i);
  const whole = Math.floor(days / pDays);
  year += whole * p;
  days -= whole * pDays;
  while (days >= yearLength(cal, year)) days -= yearLength(cal, year++);
  const day = days + 1;
  const out: CalendarDate = { year, day, second };
  const named = cal.named_days?.find((n) => n.day === day);
  if (named) out.name = named.name;
  if (cal.months) {
    let left = days;
    for (let i = 0; i < cal.months.length; i++) {
      const m = cal.months[i]!;
      const len = m.days + (i === cal.leap_month && isLong(cal, year) ? 1 : 0);
      if (left < len) {
        out.month = m.name;
        out.mday = left + 1;
        break;
      }
      left -= len;
    }
  }
  if (cal.week?.continuous) {
    out.weekday = cal.week.names[mod(total - ((cal.week.first_day ?? 1) - 1), cal.week.names.length)];
  } else if (cal.week && !named?.outside_weeks) {
    const first = cal.week.first_day ?? 1;
    // Days outside the weeks before this one, this year.
    const skipped = (cal.named_days ?? []).filter((n) => n.outside_weeks && n.day < day && n.day >= first).length;
    out.weekday = cal.week.names[mod(day - first - skipped, cal.week.names.length)];
  }
  return out;
}

const pad = (n: number, w: number) => (n < 0 ? "-" : "") + String(Math.abs(n)).padStart(w, "0");

/** A date written the calendar's way. */
export function formatDate(cal: Calendar, t_s: number, anchor: Anchor = epochOf(cal)): string {
  const d = dateOf(cal, t_s, anchor);
  return cal.format.replace(/\{(\w+)(?::0(\d+))?\}/g, (all, k: string, w?: string) => {
    const num = (n: number | undefined) => (n === undefined ? "" : w ? pad(n, Number(w)) : String(n));
    switch (k) {
      case "year":
        return num(d.year);
      case "day":
        return num(d.day);
      case "month":
        return d.month ?? "";
      case "mday":
        return num(d.mday);
      case "weekday":
        return d.weekday ?? "";
      case "name":
        return d.name ?? "";
      default:
        return all;
    }
  });
}

/** Universe second at the start of a day of the year. */
export function startOf(cal: Calendar, year: number, day: number, anchor: Anchor = epochOf(cal)): number {
  const day_s = cal.day_s;
  let days = day - 1;
  const p = period(cal);
  let pDays = 0;
  for (let i = 0; i < p; i++) pDays += yearLength(cal, anchor.year + i);
  const whole = Math.floor((year - anchor.year) / p);
  days += whole * pDays;
  for (let y = anchor.year + whole * p; y < year; y++) days += yearLength(cal, y);
  return anchor.at_s + days * day_s;
}

/**
 * Read a date written the calendar's way, back to the universe second at the
 * start of that day. Undefined when the text does not fit the format.
 */
export function parseDate(cal: Calendar, text: string, anchor: Anchor = epochOf(cal)): number | undefined {
  const keys: string[] = [];
  const parts = cal.format.split(/\{(\w+)(?::0\d+)?\}/);
  let re = "";
  parts.forEach((part, i) => {
    if (i % 2 === 0) return void (re += part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
    keys.push(part);
    re += part === "year" ? "(-?\\d+)" : part === "day" || part === "mday" ? "(\\d+)" : "(.*?)";
  });
  const m = new RegExp(`^\\s*${re}\\s*$`).exec(text);
  if (!m) return undefined;
  const got: Record<string, string> = {};
  keys.forEach((k, i) => (got[k] = m[i + 1] ?? ""));
  const year = Number(got.year);
  if (!Number.isInteger(year)) return undefined;
  let day: number | undefined;
  if (got.day !== undefined) day = Number(got.day);
  else if (cal.months && got.mday !== undefined && got.month !== undefined) {
    const i = cal.months.findIndex((x) => x.name.toLowerCase() === got.month!.toLowerCase());
    if (i < 0) return undefined;
    day = Number(got.mday);
    const len = cal.months[i]!.days + (i === cal.leap_month && isLong(cal, year) ? 1 : 0);
    if (!(day >= 1 && day <= len)) return undefined;
    for (let j = 0; j < i; j++) day += cal.months[j]!.days + (j === cal.leap_month && isLong(cal, year) ? 1 : 0);
  }
  if (day === undefined || !(day >= 1 && day <= yearLength(cal, year))) return undefined;
  return startOf(cal, year, day, anchor);
}

/** The everyday calendar, for packs and campaigns that set none. */
export const GREGORIAN: Calendar = {
  name: "Gregorian",
  // Midnight starting 1 January 2000 at universe second 0; campaigns give their own anchor.
  epoch_s: 0,
  epoch_year: 2000,
  day_s: DAY,
  year_days: 365,
  leap: { every: 4, except_every: 100, but_every: 400 },
  months: [
    { name: "January", days: 31 },
    { name: "February", days: 28 },
    { name: "March", days: 31 },
    { name: "April", days: 30 },
    { name: "May", days: 31 },
    { name: "June", days: 30 },
    { name: "July", days: 31 },
    { name: "August", days: 31 },
    { name: "September", days: 30 },
    { name: "October", days: 31 },
    { name: "November", days: 30 },
    { name: "December", days: 31 },
  ],
  leap_month: 1,
  // Set first_day to the first Monday of the anchor's year: 3 for an anchor in 2000 (1 January 2000 was a Saturday).
  week: { names: ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"], continuous: true, first_day: 3 },
  format: "{mday} {month} {year}",
};
