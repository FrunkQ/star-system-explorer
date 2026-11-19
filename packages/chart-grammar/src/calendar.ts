// Human time on the ruler (U11): a campaign calendar, dates in words, ticks
// that switch from billions of years to hours as the ruler zooms, and the
// zoom window itself. Pure functions, no DOM. Times are years since the
// universe's origin (the workspace's unit); a calendar counts days and years
// from its own epoch, so "Year 1, day 1" is where the campaign's calendar
// starts, not the Big Bang. Presentation only: the kernel's time is seconds,
// and positions at a time come from the kernel.
import { formatYears, niceStep, type Tick } from "./ruler.ts";

/** Seconds in the workspace's year (the Julian year, as the kernel's YEAR_S). */
export const YEAR_S = 31_557_600;
export const DAY_S = 86_400;

export interface Calendar {
  id: string;
  label: string;
  /** Where day 1 of year 1 falls, in years since the universe's origin. */
  epoch_yr: number;
  /** Length of a day and of a year (in days), in this calendar. */
  day_s: number;
  year_days: number;
  /** Optional months, in order; their days should sum to year_days (the remainder runs into the last). */
  months?: { name: string; days: number }[];
  /**
   * Optional date format. Placeholders: {year}, {day} (day of the year, from
   * 1), {day:03} (zero-padded to 3), {month}, {mday} (day of the month),
   * {hh}, {mm}. Default "Year {year}, day {day}" or "{mday} {month}, year {year}".
   */
  format?: string;
}

/** The calendar used when no pack gives one: Earth days and Julian years, counted from the campaign's start. */
export function defaultCalendar(epoch_yr: number): Calendar {
  return { id: "sse.calendar.standard", label: "Standard (days and years from the campaign's start)", epoch_yr, day_s: DAY_S, year_days: 365.25 };
}

/** Problems with a calendar document, as sentences; empty when usable. */
export function checkCalendar(c: unknown): string[] {
  const d = c as Partial<Calendar> | null;
  if (!d || typeof d !== "object") return ["not a calendar"];
  const out: string[] = [];
  if (!Number.isFinite(d.epoch_yr)) out.push("epoch_yr must be a number of years");
  if (!(Number(d.day_s) > 0)) out.push("day_s must be positive");
  if (!(Number(d.year_days) >= 1)) out.push("year_days must be at least 1");
  if (d.months !== undefined) {
    if (!Array.isArray(d.months) || !d.months.length || d.months.some((m) => !m || typeof m.name !== "string" || !(m.days > 0))) out.push("months must each have a name and a positive number of days");
  }
  return out;
}

const dayYr = (c: Calendar) => c.day_s / YEAR_S;
const yearYr = (c: Calendar) => (c.day_s * c.year_days) / YEAR_S;

export interface DateParts {
  /** Calendar year: 1 is the epoch's year; years before it are 0, -1, ... */
  year: number;
  /** Day of the year, from 1. */
  day: number;
  month?: string;
  mday?: number;
  hh: number;
  mm: number;
}

/** A time as calendar parts. Exact enough for hours at any universe age (f64 years carry about 30 s at 14 Gyr). */
export function dateParts(yr: number, c: Calendar): DateParts {
  // Whole minutes since the epoch first: f64 years carry about 30 s at 14 Gyr, so rounding to the minute is honest and stops 11:59 for noon.
  const totalMin = Math.round(((yr - c.epoch_yr) * YEAR_S) / 60);
  const perDay = c.day_s / 60;
  const dayPos = totalMin / perDay;
  const years = Math.floor(dayPos / c.year_days + 1e-12);
  const intoYear = dayPos - years * c.year_days;
  const dayIndex = Math.min(Math.floor(intoYear + 1e-9), Math.ceil(c.year_days) - 1);
  const minutes = Math.min(Math.max(0, Math.round((intoYear - dayIndex) * perDay)), Math.floor(perDay) - 1);
  const p: DateParts = { year: years + 1, day: dayIndex + 1, hh: Math.floor(minutes / 60), mm: minutes % 60 };
  if (c.months?.length) {
    let left = dayIndex;
    for (let i = 0; i < c.months.length; i++) {
      const m = c.months[i]!;
      if (left < m.days || i === c.months.length - 1) {
        p.month = m.name;
        p.mday = left + 1;
        break;
      }
      left -= m.days;
    }
  }
  return p;
}

/** A date in words, in the calendar's format. With `time`, the hour too. */
export function dateWords(yr: number, c: Calendar, time = false): string {
  const p = dateParts(yr, c);
  const fmt = c.format ?? (p.month ? "{mday} {month}, year {year}" : "Year {year}, day {day}");
  const pad = (n: number, w: number) => String(n).padStart(w, "0");
  const s = fmt.replace(/\{(\w+)(?::0(\d))?\}/g, (_, k: string, w?: string) => {
    const v = k === "year" ? p.year : k === "day" ? p.day : k === "mday" ? (p.mday ?? p.day) : k === "month" ? (p.month ?? "") : k === "hh" ? p.hh : k === "mm" ? p.mm : `{${k}}`;
    return typeof v === "number" && w ? pad(v, Number(w)) : String(v);
  });
  return time && !/\{hh\}/.test(fmt) ? `${s}, ${pad(p.hh, 2)}:${pad(p.mm, 2)}` : s;
}

/**
 * The ruler's readout for a time, choosing the right register for the span
 * on screen: deep time ("4.61 Gyr") when the window spans more than 10,000
 * years, a date otherwise, with the hour when the window is under 30 days.
 */
export function timeWords(yr: number, c: Calendar, span_yr: number): string {
  if (span_yr > 10_000) return formatYears(yr);
  return dateWords(yr, c, span_yr < 30 * dayYr(c));
}

/** A length of time in plain words: "3 days", "6 hours", "2.5 years", "40 Myr". */
export function spanWords(yr: number, c: Calendar = defaultCalendar(0)): string {
  const a = Math.abs(yr);
  const n = (v: number, one: string, many: string) => {
    const r = v >= 10 ? Math.round(v) : Math.round(v * 10) / 10;
    return `${r.toLocaleString("en-GB")} ${r === 1 ? one : many}`;
  };
  if (a >= 1e4) return formatYears(yr);
  if (a >= yearYr(c) * 0.999) return n(a / yearYr(c), "year", "years");
  if (a >= dayYr(c) * 0.999) return n(a / dayYr(c), "day", "days");
  if (a * YEAR_S >= 3600 * 0.999) return n((a * YEAR_S) / 3600, "hour", "hours");
  return n((a * YEAR_S) / 60, "minute", "minutes");
}

/** Steps for stepping and playing, from an hour to a million years, in years of the workspace. */
export function steps(c: Calendar): { id: string; label: string; yr: number }[] {
  const d = dayYr(c);
  const y = yearYr(c);
  const month = c.months?.length ? y / c.months.length : y / 12;
  return [
    { id: "hour", label: "hour", yr: (3600 / c.day_s) * d },
    { id: "day", label: "day", yr: d },
    { id: "week", label: "week", yr: 7 * d },
    { id: "month", label: "month", yr: month },
    { id: "year", label: "year", yr: y },
    { id: "century", label: "century", yr: 100 * y },
    { id: "kyr", label: "1,000 years", yr: 1e3 },
    { id: "myr", label: "million years", yr: 1e6 },
  ];
}

/**
 * Ticks for a window of the ruler. Over 10,000 years: the deep-time ticks.
 * Under that, calendar units: years, months (or a twelfth of the year),
 * weeks, days, hours, labelled as dates.
 */
export function calendarTicks(from: number, to: number, c: Calendar, count = 10): Tick[] {
  const span = to - from;
  if (span > 10_000 || span <= 0) {
    const step = niceStep(span || 1, count);
    const out: Tick[] = [];
    for (let i = Math.ceil(from / step); i * step <= to + step * 1e-9; i++) {
      const at = i * step;
      out.push(i % 2 === 0 ? { at, major: true, label: formatYears(at) } : { at, major: false });
    }
    return out;
  }
  const d = dayYr(c);
  const y = yearYr(c);
  const units = [
    { yr: (3600 / c.day_s) * d, label: (t: number) => { const p = dateParts(t, c); return `${String(p.hh).padStart(2, "0")}:${String(p.mm).padStart(2, "0")}`; } },
    { yr: (6 * 3600 / c.day_s) * d, label: (t: number) => { const p = dateParts(t, c); return `${String(p.hh).padStart(2, "0")}:00`; } },
    { yr: d, label: (t: number) => dateWords(t, c) },
    { yr: 7 * d, label: (t: number) => dateWords(t, c) },
    { yr: y / 12, label: (t: number) => dateWords(t, c) },
    { yr: y, label: (t: number) => `Year ${dateParts(t, c).year}` },
    ...[10, 100, 1000].map((k) => ({ yr: k * y, label: (t: number) => `Year ${dateParts(t, c).year.toLocaleString("en-GB")}` })),
  ];
  const unit = units.find((u) => span / u.yr <= count) ?? units[units.length - 1]!;
  // Ticks sit on the calendar's grid (counted from its epoch), so "day 3" is a real day boundary.
  const first = Math.ceil((from - c.epoch_yr) / unit.yr - 1e-9);
  const out: Tick[] = [];
  for (let i = first; c.epoch_yr + i * unit.yr <= to + unit.yr * 1e-9 && out.length < 400; i++) {
    const at = c.epoch_yr + i * unit.yr;
    out.push(i % 2 === 0 ? { at, major: true, label: unit.label(at) } : { at, major: false });
  }
  return out;
}

export interface RulerWindow {
  from: number;
  to: number;
}

/**
 * Zoom the window by `factor` (under 1 zooms in) about `about`, kept inside
 * `bounds` and no narrower than `min_span` (an hour by default; f64 years at
 * billions carry about 30 s, so an hour is the finest honest window).
 */
export function zoom(w: RulerWindow, factor: number, about: number, bounds: RulerWindow, min_span = 1 / (24 * 365.25)): RulerWindow {
  const full = bounds.to - bounds.from;
  const span = Math.min(full, Math.max(min_span, (w.to - w.from) * factor));
  const f = (about - w.from) / Math.max(1e-300, w.to - w.from);
  let from = about - f * span;
  from = Math.min(Math.max(bounds.from, from), bounds.to - span);
  return { from, to: from + span };
}

/** A window of `span` centred on `at`, inside `bounds`. */
export function around(at: number, span: number, bounds: RulerWindow): RulerWindow {
  return zoom({ from: at - span / 2, to: at + span / 2 }, 1, at, bounds, span);
}

/**
 * A window part way (k from 0 to 1) between two windows, for the ruler's
 * glide. The span changes on a log scale (a glide from 14 Gyr to a day reads
 * as a steady zoom, not a jump at the end), about the point that sits at the
 * same place in both windows; a pan with no zoom moves linearly.
 */
export function between(a: RulerWindow, b: RulerWindow, k: number): RulerWindow {
  const sa = a.to - a.from;
  const sb = b.to - b.from;
  if (k <= 0) return a;
  if (k >= 1) return b;
  if (Math.abs(sa - sb) <= Math.max(sa, sb) * 1e-9) {
    const from = a.from + (b.from - a.from) * k;
    return { from, to: from + sa };
  }
  const span = Math.exp(Math.log(sa) + (Math.log(sb) - Math.log(sa)) * k);
  const f = (b.from - a.from) / (sa - sb);
  const p = a.from + f * sa;
  const from = p - f * span;
  return { from, to: from + span };
}
