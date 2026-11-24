// The editing grammar's check on figures (ED1a, editing design section 3C): a screen says a number through the units
// engine (`say`, `sayFact` in ui-contract/src/figures.ts), never by formatting it itself. Today's own formatting
// (toFixed, toPrecision, toLocaleString) is counted per file in figures-ledger.json; a file may go down, never up,
// and a new file starts at none. When a file goes down, lower its row (or run with WRITE_LEDGER=1 to rewrite the
// ledger from the code), so the count only ever falls, to zero, as the typed-copy ledger did.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = new URL("../../", import.meta.url).pathname;
const LEDGER = new URL("./figures-ledger.json", import.meta.url);
const SCOPE = ["packages/views", "packages/shell/src", "packages/chart-grammar/src", "packages/ui-contract/src", "packages/renderer/src"];
/** The engine itself writes numbers: that is its job. */
const OWN = new Set(["packages/ui-contract/src/figures.ts"]);

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return n === "node_modules" || n === "test" || n === "dist" ? [] : files(p);
    return /\.(svelte|ts)$/.test(n) && !/\.test\.ts$/.test(n) ? [p] : [];
  });
}

/** How many times a source formats a number itself. */
export function ownFormatting(src: string): number {
  return [...src.replace(/\/\/.*$/gm, "").matchAll(/\.(toFixed|toPrecision|toLocaleString)\(/g)].length;
}

function counts(): Record<string, number> {
  const out: Record<string, number> = {};
  for (const d of SCOPE) {
    for (const f of files(join(root, d)).sort()) {
      const rel = f.slice(root.length);
      if (OWN.has(rel)) continue;
      const n = ownFormatting(readFileSync(f, "utf8"));
      if (n) out[rel] = n;
    }
  }
  return out;
}

test("the checker counts a screen's own number formatting", () => {
  assert.equal(ownFormatting(`const a = x.toFixed(2) + y.toLocaleString("en-GB"); // z.toPrecision(3)`), 2);
  assert.equal(ownFormatting(`say(x, "length").text`), 0);
});

test("no screen formats more of its own numbers than the ledger allows", () => {
  const now = counts();
  if (process.env.WRITE_LEDGER) {
    writeFileSync(LEDGER, JSON.stringify({ about: "Own number formatting per file (tests/ui/figures-ledger.test.ts). Goes down, never up.", total: Object.values(now).reduce((a, b) => a + b, 0), files: now }, null, 1) + "\n");
    return;
  }
  const ledger = JSON.parse(readFileSync(LEDGER, "utf8")) as { files: Record<string, number> };
  const up: string[] = [];
  const down: string[] = [];
  for (const [f, n] of Object.entries(now)) {
    const was = ledger.files[f] ?? 0;
    if (n > was) up.push(`${f}: ${n} (ledger ${was}): say it with the units engine (say, sayFact) instead`);
    else if (n < was) down.push(`${f}: ${n} (ledger ${was})`);
  }
  for (const [f, was] of Object.entries(ledger.files)) if (!(f in now)) down.push(`${f}: 0 (ledger ${was})`);
  assert.deepEqual(up, [], "more own number formatting than the ledger allows");
  assert.deepEqual(down, [], "fewer than the ledger: lower these rows (WRITE_LEDGER=1 rewrites it)");
});
