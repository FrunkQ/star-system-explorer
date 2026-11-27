// What is due (three desks design 7.3; time scales design 8): the Run desk's clock shows the next few dated things
// as ticks on a short line, and "Jump to a date…" says what a jump will pass over. Both read one reply: the kernel's
// `happenings` op (package TP6), the same for every scale. This file is that reply's shape and the pure reads of it,
// so the clock is built against the op before the op exists. Until TP6 lands the shell has no source, and the clock
// says "Nothing due" honestly; it never keeps a second list of its own.
//
// Times are whole seconds (`bigint`), never years in a double (time scales design 5).

/** One mark: something that happens at a date (time scales design 8). */
export interface TimeMark {
  /** When it happens, in the kernel's seconds. */
  at_s: bigint;
  /** How long it lasts, in seconds; absent for an instant. */
  lasts_s?: bigint;
  /** What kind of thing it is, as a word the reply names it by ("arrival", "eclipse", "ruling"). */
  kind: string;
  /** The addresses it is about. */
  subjects: string[];
  /** The sifter's significance score: higher is more interesting. */
  rank: number;
  /** One sentence on what happens, for the tick's title and the jump dialog. */
  why: string;
  /** The process or record that gave it. */
  source: string;
}

/** A crowded kind, counted rather than listed (a red dwarf's flares over a million years). */
export interface Summarised {
  kind: string;
  count: number;
  biggest?: TimeMark;
}

/** The `happenings` reply: `happenings { scope, from_s, to_s, at_least?, limit? }`. */
export interface Happenings {
  marks: TimeMark[];
  summarised: Summarised[];
}

export const NO_HAPPENINGS: Happenings = { marks: [], summarised: [] };

/** One tick on the clock's due line. */
export interface DueTick {
  mark: TimeMark;
  /** Seconds from now to the mark. */
  ahead_s: bigint;
  /** Where on the line it sits, 0 (now) to 1 (the end of the window). */
  x: number;
}

/** A fixed order for marks: by date, then the more interesting first, then by kind and source, so every device agrees. */
export function byDate(a: TimeMark, b: TimeMark): number {
  if (a.at_s !== b.at_s) return a.at_s < b.at_s ? -1 : 1;
  if (a.rank !== b.rank) return b.rank - a.rank;
  return a.kind < b.kind ? -1 : a.kind > b.kind ? 1 : a.source < b.source ? -1 : a.source > b.source ? 1 : 0;
}

/**
 * The due line: the marks after now and within the window, earliest first, at most `limit` of them. A mark at now
 * itself is not due (it is happening); one at the window's end is.
 */
export function dueAhead(h: Happenings, now_s: bigint, window_s: bigint, limit = 5): DueTick[] {
  if (window_s <= 0n) return [];
  const end = now_s + window_s;
  return h.marks
    .filter((m) => m.at_s > now_s && m.at_s <= end)
    .sort(byDate)
    .slice(0, Math.max(0, limit))
    .map((mark) => {
      const ahead_s = mark.at_s - now_s;
      // A ratio of two whole numbers of seconds, each well under 2^53 over a few days, so Number is exact enough here.
      return { mark, ahead_s, x: Number(ahead_s) / Number(window_s) };
    });
}

const plural = (n: number, kind: string) => (n === 1 ? `1 ${kind}` : `${n} ${/(s|x|ch|sh)$/.test(kind) ? `${kind}es` : /[^aeiou]y$/.test(kind) ? `${kind.slice(0, -1)}ies` : `${kind}s`}`);

/**
 * What a stretch holds, in a few words: "3 eclipses, 1 arrival". Listed marks and summarised kinds are counted
 * together; the commonest kind first (ties by name). Empty when nothing is marked.
 */
export function passedWords(h: Happenings, most = 4): string {
  const n = new Map<string, number>();
  for (const m of h.marks) n.set(m.kind, (n.get(m.kind) ?? 0) + 1);
  for (const s of h.summarised) n.set(s.kind, (n.get(s.kind) ?? 0) + s.count);
  const kinds = [...n].filter(([, c]) => c > 0).sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1));
  const shown = kinds.slice(0, most).map(([k, c]) => plural(c, k));
  const rest = kinds.slice(most).reduce((t, [, c]) => t + c, 0);
  return [...shown, ...(rest ? [`${rest} more`] : [])].join(", ");
}

/** A happenings reply, as data from the kernel or a test: problems as sentences, empty when usable. */
export function checkHappenings(x: unknown): string[] {
  const h = x as Partial<Happenings> | null;
  if (!h || typeof h !== "object") return ["not a happenings reply"];
  const out: string[] = [];
  if (!Array.isArray(h.marks)) out.push("marks must be a list");
  else
    h.marks.forEach((m, i) => {
      if (typeof m?.at_s !== "bigint") out.push(`mark ${i}: at_s must be whole seconds`);
      if (typeof m?.kind !== "string" || !m.kind) out.push(`mark ${i}: kind must be a word`);
      if (typeof m?.why !== "string") out.push(`mark ${i}: why must be a sentence`);
      if (!Number.isFinite(m?.rank)) out.push(`mark ${i}: rank must be a number`);
    });
  if (!Array.isArray(h.summarised)) out.push("summarised must be a list");
  return out;
}
