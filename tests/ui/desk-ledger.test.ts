// The three desks' check on desk tests (TM1, three desks design section 10): whether a thing is on screen on a desk is
// data (`desks` on rail entries, part controls and the shell's parts) read through `shows()` in ui-contract/src/desk.ts.
// A screen that tests the desk by name instead (`desk === "build"`, `ws.desk !== "run"`) is counted per file in
// desk-ledger.json, the way the figures ledger counts own number formatting: a file may go down, never up, and a new
// file starts at none. ui-contract is where the desks are declared, so it is not counted. When a file goes down, lower
// its row (or run with WRITE_LEDGER=1 to rewrite the ledger from the code). A row may be raised only on purpose, with the
// reason in the pull request.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = new URL("../../", import.meta.url).pathname;
const LEDGER = new URL("./desk-ledger.json", import.meta.url);
const SCOPE = ["packages/views", "packages/shell/src", "packages/chart-grammar/src", "packages/renderer/src", "packages/skins", "packages/agents/src"];

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return n === "node_modules" || n === "test" || n === "dist" ? [] : files(p);
    return /\.(svelte|ts)$/.test(n) && !/\.test\.ts$/.test(n) ? [p] : [];
  });
}

/** How many times a source tests the desk by name: a desk-named value compared with === or !==, either way round. */
export function deskTests(src: string): number {
  const code = src.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  return [...code.matchAll(/\b[\w.]*desk\w*\s*[!=]==|[!=]==\s*[\w.]*desk\b/gi)].length;
}

function counts(): Record<string, number> {
  const out: Record<string, number> = {};
  for (const d of SCOPE) {
    for (const f of files(join(root, d)).sort()) {
      const n = deskTests(readFileSync(f, "utf8"));
      if (n) out[f.slice(root.length)] = n;
    }
  }
  return out;
}

test("the checker counts a screen's own desk tests", () => {
  assert.equal(deskTests(`if (ws.desk !== "build" && desk === "run") x(); const y = "run" === ctx.workspace.desk; // desk === "explore"`), 3);
  assert.equal(deskTests(`shows(entry, desk) && partShows("eyes", ws.desk)`), 0);
  assert.equal(deskTests(`/* desk === "build" */ deskKeeps(desk)`), 0);
});

test("no screen tests the desk by name more than the ledger allows", (t) => {
  const now = counts();
  if (process.env.WRITE_LEDGER) {
    writeFileSync(LEDGER, JSON.stringify({ about: "Desk tested by name per file (tests/ui/desk-ledger.test.ts); say it with desks and shows() instead. Goes down, never up.", total: Object.values(now).reduce((a, b) => a + b, 0), files: now }, null, 1) + "\n");
    return;
  }
  const ledger = JSON.parse(readFileSync(LEDGER, "utf8")) as { files: Record<string, number> };
  const up: string[] = [];
  const down: string[] = [];
  for (const [f, n] of Object.entries(now)) {
    const was = ledger.files[f] ?? 0;
    if (n > was) up.push(`${f}: ${n} (ledger ${was}): give the thing its desks and ask shows() instead`);
    else if (n < was) down.push(`${f}: ${n} (ledger ${was})`);
  }
  for (const [f, was] of Object.entries(ledger.files)) if (!(f in now)) down.push(`${f}: 0 (ledger ${was})`);
  assert.deepEqual(up, [], "more desk tests by name than the ledger allows");
  // As with the figures ledger: a row above the code is reported, not failed, so one builder lowering a row never turns
  // another's pull request red.
  if (down.length) t.diagnostic(`fewer than the ledger, lower these rows: ${down.join("; ")}`);
});
