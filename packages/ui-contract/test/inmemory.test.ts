// In memory (SH6): the page's parts, names and first line from a pack's in-memory.json. A made-up record checks each
// rule; the real "Sol today" record (RS9 part 2) is checked for its own rules (two sources, every person known), never
// against a figure from a book.
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { correctionUrl, inMemory, inMemoryLine, lossesOn, recordDate } from "../src/inmemory.ts";
import { GREGORIAN } from "../src/calendar.ts";

const src = [{ text: "Record one", url: "https://example.org/1" }, { text: "Record two" }];
const crews = { people: [{ id: "p:ada", name: "Quill, Ada *Jo" }, { id: "p:bo", name: "Fenn, Bo" }, { id: "p:dot", name: "Dot", being: "dog" }] };
const doc = {
  losses: [
    { id: "b", at_s: 200, date: "2001-02-03", where: "Pad 2", during: "ground_work", sentence: "A tank burst during fuelling.", on: null, people: [], names: [{ name: "Cy Marr", sources: src }], unconfirmed: "and others whose names these sources do not give", toll: [5, 9], toll_why: "the two records differ", sources: src },
    { id: "a", at_s: 100, date: "2000-01-02", where: "above the sea", during: "flight", sentence: "The Heron broke up during entry.", on: "heron", people: ["p:ada", "p:bo", "p:dot"], unconfirmed: null, toll: 2, sources: src },
    { id: "c", at_s: 150, date: "2000-06-01", where: "the test stand", during: "training", sentence: "A fire broke out in a chamber test.", on: null, people: ["p:bo"], unconfirmed: null, toll: 1, sources: src },
  ],
  earth_memorials: [{ name: "The Wall", where: "Port", sources: src }],
};

test("three parts in order, each in date order, and animals apart at the foot", () => {
  const m = inMemory(doc, crews);
  assert.deepEqual(m.parts.map((p) => p.heading), ["On a flight", "Preparing to fly", "Working to send others", "Animals that died on flights"]);
  assert.deepEqual(m.parts[0]!.losses[0]!.names.map((n) => n.name), ["Quill, Ada Jo", "Fenn, Bo"]);
  assert.deepEqual(m.parts[3]!.losses[0]!.names.map((n) => n.name), ["Dot"]);
  assert.equal(m.parts[3]!.losses[0]!.unconfirmed, null);
  assert.equal(m.parts[2]!.losses[0]!.unconfirmed, "and others whose names these sources do not give");
  assert.equal(m.earth[0]!.name, "The Wall");
});

test("the first line is computed, and a disputed toll gives a range", () => {
  const m = inMemory(doc, crews);
  assert.deepEqual(m.people, [8, 12]);
  assert.equal(inMemoryLine(m), "Between 8 and 12 people have died flying to space, preparing to, or working to send others. These are their names.");
  const one = inMemory({ losses: [doc.losses[1]] }, crews);
  assert.match(inMemoryLine(one), /^2 people have died/);
});

test("a record's date is written with the calendar's months; the day of a loss finds it once", () => {
  assert.equal(recordDate("1986-01-28", GREGORIAN.months), "28 January 1986");
  assert.equal(recordDate("about 1960", GREGORIAN.months), "about 1960");
  const m = inMemory(doc, crews);
  assert.deepEqual(lossesOn(m, 100, 101).map((l) => l.id), ["a"]);
});

test("a correction is a prepared report that sends nothing by itself", () => {
  const u = new URL(correctionUrl(inMemory(doc, crews).parts[0]!.losses[0]!));
  assert.equal(u.hostname, "github.com");
  assert.match(u.searchParams.get("body") ?? "", /Two independent sources/);
});

const PACK = join(import.meta.dirname, "../../../store-packs/sol-today");
const has = existsSync(join(PACK, "in-memory.json")) && existsSync(join(PACK, "crews.json"));
test("Sol today: every loss has two sources and a sentence; every named person is in the crews", { skip: !has && "in-memory.json is not on this branch yet" }, () => {
  const raw = JSON.parse(readFileSync(join(PACK, "in-memory.json"), "utf8"));
  const c = JSON.parse(readFileSync(join(PACK, "crews.json"), "utf8"));
  const ids = new Set(c.people.map((p: { id: string }) => p.id));
  for (const l of raw.losses) {
    assert.ok(l.sources.length >= 2, `${l.id}: two sources`);
    assert.ok(l.sentence.trim(), `${l.id}: a sentence`);
    for (const p of l.people) assert.ok(ids.has(p), `${l.id}: ${p} is in the crews`);
    for (const n of l.names ?? []) assert.ok((n.sources ?? []).length >= 2, `${l.id}: ${n.name} has two sources`);
  }
  const m = inMemory(raw, c);
  assert.ok(m.parts.length >= 1);
  assert.ok(m.people[0] > 0 && m.people[0] <= m.people[1]);
});
