// Records and "On this day" (track SH5, design `handover/design/SSE4-spaceflight-history-design.md` sections 4.3 and
// 4.6): questions asked of the roll call's own stays and count, never typed facts. Each record carries how it was
// worked out, so a clever user can test it; if the logs improve, the records move by themselves. Nothing here is
// pinned to a figure from a book, and nothing feeds physics or the state hash.
//
// The design calls these saved sifters (the kernel's `sift`, behind Find and Stories). The "Sol today" pack is read
// as a folder, not loaded into the kernel, so for now each record is a pure function over the same `RollIndex` the
// roster uses; they become sifter documents when a pack's people and logs are campaign entities (SH9, package log).
import { dateOf, epochOf, parseDate, type Calendar } from "./calendar.ts";
import { constant } from "./sheet.ts";
import { againstEdgeCounts, countAt, shownName, type RollIndex, type RollStay } from "./rollcall.ts";

export interface RollRecord {
  key: string;
  /** The question, "Most people in space at once". */
  label: string;
  /** The number the record is about (people, or seconds for a span), to be said by the view. */
  n: number;
  /** "people" or "seconds". */
  unit: "people" | "seconds";
  /** The day to go to. */
  at_s: number;
  /** When the record's span ends, if it has one. */
  to_s?: number;
  /** Who, when it is about a person. */
  person?: { id: string; name: string };
  /** How it was worked out, in plain words. */
  how: string;
}

/** A person's time off the ground in one go: legs that follow on with no gap (a Soyuz up, the station, a Soyuz down). */
export interface Stay {
  person: string;
  from_s: number;
  to_s: number | null;
}

/** Counted flight legs joined into whole stays, person by person, in time order. */
export function wholeStays(ix: RollIndex): Stay[] {
  const by = new Map<string, RollStay[]>();
  for (const s of ix.stays) {
    if (!s.flight || !againstEdgeCounts(s, ix)) continue;
    const l = by.get(s.person);
    if (l) l.push(s);
    else by.set(s.person, [s]);
  }
  const out: Stay[] = [];
  for (const [person, legs] of by) {
    legs.sort((a, b) => a.from_s - b.from_s);
    let cur: Stay | null = null;
    for (const s of legs) {
      if (cur && cur.to_s !== null && s.from_s <= cur.to_s) {
        cur.to_s = s.to_s === null ? null : Math.max(cur.to_s, s.to_s);
        continue;
      }
      if (cur) out.push(cur);
      cur = { person, from_s: s.from_s, to_s: s.to_s };
    }
    if (cur) out.push(cur);
  }
  return out.sort((a, b) => a.from_s - b.from_s || (a.person < b.person ? -1 : 1));
}

/**
 * The records over a roll: most at once and the last day with nobody up (from the count steps), the longest single
 * stay and the most time in all (from whole stays). `day_s` is the calendar's day, for the last empty day. A stay still going is left out of the two time records, and the
 * record says so. Empty when there is nothing to ask.
 */
export function rollRecords(ix: RollIndex, steps: readonly (readonly [number, number])[], day_s: number = constant("day")): RollRecord[] {
  const out: RollRecord[] = [];
  const nameOf = (id: string) => ({ id, name: shownName(ix.people.get(id)?.name ?? id) });
  if (steps.length) {
    let best = 0;
    let at = 0;
    let to: number | undefined;
    steps.forEach(([t, n], i) => {
      if (n > best) {
        best = n;
        at = t;
        to = steps[i + 1]?.[0];
      }
    });
    if (best > 0)
      out.push({ key: "most", label: "Most people in space at once", n: best, unit: "people", at_s: at, ...(to !== undefined ? { to_s: to } : {}), how: "The highest step in the count of people in space, first reached that day." });
    // The last stretch with nobody up, after the first person went up.
    const first = steps.findIndex(([, n]) => n > 0);
    for (let i = steps.length - 1; i > first && first >= 0; i--) {
      if (steps[i]![1] !== 0) continue;
      const next = steps[i + 1]?.[0];
      // A whole day before the stretch ended, so the day shown is empty from end to end when the stretch allows it.
      const at = next === undefined ? steps[i]![0] : Math.max(steps[i]![0], next - day_s);
      out.push({ key: "empty", label: "Last day with nobody in space", n: 0, unit: "people", at_s: at, ...(next !== undefined ? { to_s: next } : {}), how: next !== undefined ? "The last zero in the count; someone has been up every day since it ended." : "The last zero in the count, which is where the record ends." });
      break;
    }
  }
  const stays = wholeStays(ix);
  const done = stays.filter((s): s is Stay & { to_s: number } => s.to_s !== null);
  const going = stays.length - done.length;
  const still = going ? " Stays still going are left out until they end." : "";
  if (done.length) {
    let l = done[0]!;
    for (const s of done) if (s.to_s - s.from_s > l.to_s - l.from_s) l = s;
    out.push({ key: "longest", label: "Longest single stay", n: l.to_s - l.from_s, unit: "seconds", at_s: l.from_s, to_s: l.to_s, person: nameOf(l.person), how: `The longest time one person spent off the ground in one go, from launch to landing, joining the craft they moved between.${still}` });
    const total = new Map<string, number>();
    for (const s of done) total.set(s.person, (total.get(s.person) ?? 0) + (s.to_s - s.from_s));
    let who = "";
    let most = -1;
    for (const [p, v] of [...total].sort((a, b) => (a[0] < b[0] ? -1 : 1))) if (v > most) [who, most] = [p, v];
    const last = done.filter((s) => s.person === who).reduce((a, b) => (b.to_s > a.to_s ? b : a));
    out.push({ key: "total", label: "Most time in space in all", n: most, unit: "seconds", at_s: last.from_s, to_s: last.to_s, person: nameOf(who), how: `Every stay one person has made, added up; it goes to the start of their last.${still}` });
  }
  return out;
}

export interface ThisDayYear {
  year: number;
  /** Noon of that day. */
  at_s: number;
  /** People in space that day, from the count. */
  up: number;
  /** People who went up and came down that day. */
  went: number;
  came: number;
}

/**
 * "On this day": the calendar day of `t` in every year from the first step to the last, with who was up and who went
 * up or came down. Years where nothing happened and nobody was up are left out; a day the year lacks (29 February) is
 * skipped.
 */
export function onThisDay(ix: RollIndex, steps: readonly (readonly [number, number])[], cal: Calendar, t: number, span: { from_s: number; to_s: number }): ThisDayYear[] {
  const anchor = epochOf(cal);
  const d = dateOf(cal, t, anchor);
  if (d.mday === undefined || !d.month) return [];
  const [y0, y1] = [dateOf(cal, span.from_s, anchor).year, dateOf(cal, span.to_s, anchor).year];
  const legs = ix.stays.filter((s) => s.flight && againstEdgeCounts(s, ix));
  const out: ThisDayYear[] = [];
  for (let year = y0; year <= y1; year++) {
    const text = cal.format.replace(/\{(\w+)(?::0\d+)?\}/g, (_, k: string) => (k === "year" ? String(year) : k === "mday" ? String(d.mday) : k === "month" ? d.month! : k === "day" ? String(d.day) : ""));
    const s0 = parseDate(cal, text, anchor);
    if (s0 === undefined) continue;
    const back = dateOf(cal, s0, anchor);
    if (back.mday !== d.mday || back.month !== d.month) continue;
    const s1 = s0 + cal.day_s;
    // Whole-stay ends only: moving between craft is not going up or coming down.
    const went = new Set<string>();
    const came = new Set<string>();
    for (const s of legs) {
      if (s.from_s >= s0 && s.from_s < s1 && !legs.some((o) => o !== s && o.person === s.person && o.to_s === s.from_s)) went.add(s.person);
      if (s.to_s !== null && s.to_s >= s0 && s.to_s < s1 && !legs.some((o) => o !== s && o.person === s.person && o.from_s === s.to_s)) came.add(s.person);
    }
    // The most up at any moment of the day: the count at its start, and every step within it.
    let up = countAt(steps, s0);
    for (const [st, n] of steps) if (st >= s0 && st < s1) up = Math.max(up, n);
    if (!up && !went.size && !came.size) continue;
    out.push({ year, at_s: s0 + cal.day_s / 2, up, went: went.size, came: came.size });
  }
  return out;
}
