// TP1's ledger of dates held as doubles, the screens' half (handover/packages/TP1-float-dates.txt).
// The same ledger is checked in full by the Rust test crates/sse-core/tests/float_dates.rs, but CI skips the Rust
// jobs for a change that touches only the screens, so this checks the `packages/` lines here too, with the same
// rule: per file, the clock fields kept in years (`now_yr`, `campaign_now_yr`, `campaign_epoch_yr`, `clock_yr`)
// may only go down. A count above its line fails: hold the date in whole seconds since an anchor instead. A count
// below its line fails too, so the ledger is lowered in the change that lowers it (a line at 0 is deleted).
// Keep the walk and the word rule the same as the Rust test's, so the two always agree.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const root = new URL("../../", import.meta.url).pathname;
const LEDGER = "handover/packages/TP1-float-dates.txt";
const YEAR_CLOCKS = ["now_yr", "campaign_now_yr", "campaign_epoch_yr", "clock_yr"];
const SKIP = new Set(["target", "node_modules", "dist", "generated"]);

function walk(dir: string): string[] {
  let names: string[];
  try {
    names = readdirSync(dir).sort();
  } catch {
    return [];
  }
  return names.flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return SKIP.has(n) || n.startsWith(".") ? [] : walk(p);
    return n.endsWith(".ts") || n.endsWith(".svelte") ? [p] : [];
  });
}

const ident = (c: string | undefined) => c !== undefined && /[A-Za-z0-9_]/.test(c);

/** Whole-word uses of the year clock fields in one line. */
export function yearClocks(line: string): number {
  let n = 0;
  for (const w of YEAR_CLOCKS) {
    let from = 0;
    for (;;) {
      const a = line.indexOf(w, from);
      if (a < 0) break;
      const b = a + w.length;
      if (!ident(line[a - 1]) && !ident(line[b])) n += 1;
      from = b;
    }
  }
  return n;
}

function counts(): Map<string, number> {
  const out = new Map<string, number>();
  for (const f of walk(join(root, "packages"))) {
    const n = readFileSync(f, "utf8").split("\n").reduce((s, l) => s + yearClocks(l), 0);
    if (n > 0) out.set(f.slice(root.length).replaceAll("\\", "/"), n);
  }
  return out;
}

function ledger(): Map<string, number> {
  const out = new Map<string, number>();
  for (const raw of readFileSync(join(root, LEDGER), "utf8").split("\n")) {
    const l = raw.trim();
    if (!l || l.startsWith("#")) continue;
    const i = l.indexOf(" ");
    const path = l.slice(i + 1).trim();
    if (path.startsWith("packages/")) out.set(path, Number(l.slice(0, i)));
  }
  return out;
}

test("the screens' dates held in years only shrink", () => {
  const now = counts();
  const was = ledger();
  const problems: string[] = [];
  for (const [path, n] of now) {
    const w = was.get(path);
    if (w === undefined) problems.push(`${path}: ${n} new year clock use(s); hold the date in whole seconds since an anchor`);
    else if (n > w) problems.push(`${path}: ${n} year clock uses, the ledger allows ${w}; hold the date in whole seconds`);
    else if (n < w) problems.push(`${path}: down to ${n} from ${w}, well done: lower its line in ${LEDGER}`);
  }
  for (const path of was.keys()) if (!now.has(path)) problems.push(`${path}: none left: delete its line in ${LEDGER}`);
  assert.deepEqual(problems, [], "\nThe counts today:\n" + [...now].map(([p, n]) => `${n} ${p}`).join("\n"));
});

test("the counter sees what it should", () => {
  assert.equal(yearClocks("ws.campaign_now_yr = now_yr + 1"), 2);
  assert.equal(yearClocks("snow_yr my_clock_yrs"), 0);
  assert.equal(yearClocks("defaultCalendar(ws.campaign_epoch_yr ?? ws.campaign_now_yr ?? ws.now_yr)"), 3);
});
