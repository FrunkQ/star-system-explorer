// The declared list of stand-ins against the fudge register (FL4, SSE4-visual-flourish-design.md rule 7 and 8A), and
// the note's clause: said only while a stand-in shows.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import list from "../src/stand-ins.json" with { type: "json" };
import { STAND_IN_KINDS, SettleBoard, drawFor, settleLineWords, standInClause, type DearLayer, type Movement } from "../src/settle.ts";
import { motionWords, withRate, stoppedAt } from "../src/picture-clock.ts";

const ROOT = new URL("../../../", import.meta.url).pathname;
type Ref = { file: string; row: string };
type Use = { id: string; kinds: string[]; moves_with: string[]; register: Ref; said: string | null; moves_onto_schedule: string; driven_by?: string };
const uses = (list as unknown as { uses: Use[] }).uses;
const notStandIns = (list as unknown as { not_stand_ins: { id: string; register: Ref }[] }).not_stand_ins;

/** Every register file, recursively. */
function registerFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? registerFiles(p) : n.endsWith(".md") ? [p] : [];
  });
}
const rowsOf = (file: string) => readFileSync(join(ROOT, file), "utf8").split("\n").filter((l) => l.startsWith("| "));

test("every declared stand-in has its row in the fudge register", () => {
  assert.ok(uses.length >= 8);
  for (const u of [...uses, ...notStandIns]) {
    const rows = rowsOf(u.register.file);
    assert.ok(rows.some((r) => r.startsWith(`| ${u.register.row}`)), `${u.id}: no row starting "${u.register.row}" in ${u.register.file}`);
  }
});

test("every register row that names a declared use is in the list", () => {
  const ids = new Set(uses.map((u) => u.id));
  let named = 0;
  for (const f of registerFiles(join(ROOT, "docs/fudges")))
    for (const line of readFileSync(f, "utf8").split("\n")) {
      for (const m of line.matchAll(/declared use `([a-z0-9.-]+)`/g)) {
        named++;
        assert.ok(ids.has(m[1]!), `${f}: names declared use ${m[1]} that stand-ins.json does not list`);
      }
    }
  assert.ok(named >= 3, `${named}`);
});

test("each use is well formed: honest kinds, what moves it, who moves it onto the schedule", () => {
  const moves: Movement[] = ["clock", "camera", "place"];
  const seen = new Set<string>();
  for (const u of uses) {
    assert.ok(!seen.has(u.id), `${u.id} twice`);
    seen.add(u.id);
    assert.ok(u.kinds.length > 0 && u.kinds.every((k) => (STAND_IN_KINDS as readonly string[]).includes(k)), u.id);
    // No use may draw a Hint without Alex's yes for that case (design question 4).
    assert.ok(!u.kinds.includes("hint"), `${u.id}: a Hint needs Alex's yes`);
    assert.ok(u.moves_with.every((m) => (moves as string[]).includes(m)), u.id);
    assert.ok(u.moves_with.length > 0 || !!u.driven_by, `${u.id}: moved by nothing and says nothing of why`);
    assert.match(u.moves_onto_schedule, /^FL[0-9]$/);
  }
});

const giant: DearLayer = { id: "giant.cloud", moves_with: ["clock"], own_time_s: 3.2 * 86_400, cycle_s: 35_748, stand_in: ["carried", "exposure"] };
const jupiter = { name: "Jupiter's clouds", plural: true };

test("the clause is said only while a stand-in shows", () => {
  const host = { reduced: false, gpu: true };
  assert.equal(standInClause(jupiter, drawFor(giant, "rest", host), "clock"), "");
  assert.equal(standInClause(jupiter, drawFor(giant, "live", host), "clock"), "Jupiter's clouds are carried along by their winds between pictures. The full picture settles when the clock stops.");
  assert.equal(standInClause(jupiter, drawFor(giant, "too-fast", host), "clock"), "Jupiter's clouds are blurred as a long exposure: they change faster than the screen can show. The full picture settles when the clock stops.");
  assert.equal(standInClause({ name: "the galaxies nearby", plural: true }, "coarse", "place"), "The galaxies nearby are drawn coarsely on the way. The full picture settles when the zoom stops.");
  const running = withRate(stoppedAt(0), 86_400, 0);
  const clause = standInClause(jupiter, "carried", "clock");
  assert.ok(motionWords(running, [], [clause]).endsWith(clause));
  assert.ok(!motionWords(running, [], [standInClause(jupiter, "proper", "clock")]).includes("carried"));
  assert.equal(motionWords(running, []), motionWords(running, [], [""]), "no stand-in, the note as before");
  // Stopped but not yet settled: the clause still shows; settled, it goes.
  assert.ok(motionWords(stoppedAt(0), [], [clause]).includes("settles when the clock stops"));
  assert.equal(motionWords(stoppedAt(0), []), "The clock is stopped, so the picture is still; play moves everything at its own speed.");
});

test("the readout's board: one line a layer, biggest first, and a report that changes nothing is not news", () => {
  const b = new SettleBoard();
  b.report({ id: "giant.cloud", state: "live", build_ms: 96, size: 512 });
  b.report({ id: "journey.nearby", state: "carried-on", build_ms: null, size: 1024 });
  const v = b.version;
  b.report({ id: "giant.cloud", state: "live", build_ms: 96, size: 512 });
  assert.equal(b.version, v);
  assert.deepEqual(b.lines().map((l) => l.id), ["journey.nearby", "giant.cloud"]);
  const ms = (x: number) => `${Math.round(x)} ms`;
  assert.equal(settleLineWords(b.lines()[1]!, ms), "giant.cloud: live, rebuilt as it changes; last built in 96 ms at 512 wide");
  assert.equal(settleLineWords(b.lines()[0]!, ms), "journey.nearby: carried on, rebuilt when it can be; not built yet at 1024 wide");
  b.drop("journey.nearby");
  assert.equal(b.lines().length, 1);
});
