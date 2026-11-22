import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkStartLines, opensKey, pickLine, shownLines, type StartLinesDoc } from "../src/start.ts";

const doc = JSON.parse(readFileSync(new URL("../../shell/src/documents/start.lines.json", import.meta.url), "utf8")) as StartLinesDoc & { atlas: { verb: string; opens: object; needs?: string[] }[] };
const none = new Set<string>();

test("the shipped lines are well formed: 42 lines, three first, ten verbs", () => {
  assert.deepEqual(checkStartLines(doc), []);
  assert.equal(doc.lines.length, 42);
  assert.deepEqual(doc.lines.filter((l) => l.weight === "first").map((l) => l.id), ["l1", "l2", "l3"]);
  assert.equal(doc.atlas.length, 10);
});

test("a line naming a feature that is not built is held back", () => {
  const shown = shownLines(doc.lines, none);
  assert.ok(!shown.some((l) => l.id === "l2"), "the Solar System line waits for the Solar System");
  assert.ok(!shown.some((l) => l.id === "l29" || l.id === "l30"), "the lines marked as waiting in the design");
  assert.ok(shownLines(doc.lines, new Set(["sol"])).some((l) => l.id === "l2"));
});

test("the first visits show the expected lines in order, skipping any held back", () => {
  assert.equal(pickLine(doc.lines, none, { visits: 0 }, 0.5)!.id, "l1");
  assert.equal(pickLine(doc.lines, none, { visits: 1 }, 0.5)!.id, "l3");
  assert.equal(pickLine(doc.lines, new Set(["sol"]), { visits: 1 }, 0.5)!.id, "l2");
});

test("later picks never repeat the last line, and cover the shown lines", () => {
  const seen = new Set<string>();
  let last: string | null = null;
  for (let i = 0; i < 400; i++) {
    const l = pickLine(doc.lines, none, { visits: 10, last }, (i * 0.6180339887) % 1)!;
    assert.notEqual(l.id, last);
    seen.add(l.id);
    last = l.id;
  }
  assert.equal(seen.size, shownLines(doc.lines, none).length);
});

test("lines about something not opened yet are picked more often", () => {
  const count = (opened: string[]) => {
    let n = 0;
    for (let i = 0; i < 1000; i++) if (pickLine(doc.lines, none, { visits: 10, opened }, i / 1000)!.opens && opensKey(pickLine(doc.lines, none, { visits: 10, opened }, i / 1000)!.opens) === "screen:route") n++;
    return n;
  };
  assert.ok(count([]) > count(["screen:route"]), "route lines thin out once Route has been opened");
});
