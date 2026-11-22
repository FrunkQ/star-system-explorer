// The start sheet's opening line (top-level design 16). It changes by itself:
// the first three visits show the three expected lines in order, then a
// weighted pick that never repeats the last line and prefers lines about
// things this person has not opened yet. A line whose `needs` are not all
// built is held back, so every line is true of the build in front of the
// user. The lines are data (shell document start.lines.json); the choice is
// here so it can be tested without a browser.

/** What Show me does: a screen at system scale, a world screen, a desk, a dialog, a door of the sheet, a control, or a link. */
export type StartOpens = { screen: string } | { world: string } | { desk: "explore" | "build" | "run" } | { dialog: string } | { door: string } | { control: string } | { link: string };

export interface StartLine {
  id: string;
  headline: string;
  sentence: string;
  opens: StartOpens;
  weight: "first" | "common" | "medium" | "seldom";
  /** Features the line names that must be built before it is shown. */
  needs?: string[];
  audience: string;
}

export interface StartLinesDoc {
  schema: "sse.start-lines/0";
  note?: string;
  lines: StartLine[];
}

/** A key for what a line opens, so "opened already" can be remembered per thing, not per line. */
export function opensKey(o: StartOpens): string {
  const [k, v] = Object.entries(o)[0]!;
  return `${k}:${v}`;
}

/** Lines whose needs are all built. */
export function shownLines(lines: readonly StartLine[], built: ReadonlySet<string>): StartLine[] {
  return lines.filter((l) => (l.needs ?? []).every((n) => built.has(n)));
}

const WEIGHT = { first: 3, common: 3, medium: 2, seldom: 1 } as const;

/**
 * The next line. `visits` counts earlier openings of the sheet; `last` is the
 * line shown just before (never repeated while another can be shown);
 * `opened` lists what this person has opened (keys from `opensKey`), whose
 * lines are picked half as often; `rand` is a number in [0, 1).
 */
export function pickLine(lines: readonly StartLine[], built: ReadonlySet<string>, s: { visits: number; last?: string | null; opened?: readonly string[] }, rand: number): StartLine | null {
  const shown = shownLines(lines, built);
  if (!shown.length) return null;
  const first = shown.filter((l) => l.weight === "first");
  if (s.visits < first.length) {
    const l = first[s.visits]!;
    if (l.id !== s.last) return l;
  }
  const pool = shown.length > 1 ? shown.filter((l) => l.id !== s.last) : shown;
  const opened = new Set(s.opened ?? []);
  const w = pool.map((l) => WEIGHT[l.weight] * (opened.has(opensKey(l.opens)) ? 1 : 2));
  const total = w.reduce((a, b) => a + b, 0);
  let x = Math.min(Math.max(rand, 0), 0.999999) * total;
  for (let i = 0; i < pool.length; i++) {
    x -= w[i]!;
    if (x < 0) return pool[i]!;
  }
  return pool[pool.length - 1]!;
}

/** Check a lines document: ids unique, every field present, weights known. Returns problems in words. */
export function checkStartLines(doc: StartLinesDoc): string[] {
  const out: string[] = [];
  if (doc.schema !== "sse.start-lines/0") out.push("schema must be sse.start-lines/0");
  const ids = new Set<string>();
  for (const l of doc.lines ?? []) {
    if (ids.has(l.id)) out.push(`${l.id}: id used twice`);
    ids.add(l.id);
    if (!l.headline || !l.sentence) out.push(`${l.id}: needs a headline and a sentence`);
    if (l.headline.split(/\s+/).length > 12) out.push(`${l.id}: headline longer than twelve words`);
    if (!(l.weight in WEIGHT)) out.push(`${l.id}: unknown weight ${l.weight}`);
    if (!l.opens || Object.keys(l.opens).length !== 1) out.push(`${l.id}: opens must name one thing`);
  }
  return out;
}
