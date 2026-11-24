import { test } from "node:test";
import assert from "node:assert/strict";
import { childWordsFor, elsewhereRows, kindChips, matchesQuery, pickerRows, subjectItems, type ElsewhereItem, type PickItem } from "../src/picker.ts";

const sol: PickItem[] = [
  { id: "sun", label: "Sun", kinds: ["Stars"] },
  { id: "jupiter", label: "Jupiter", parent: "sun", kinds: ["Planets"], order: 5.2 },
  { id: "earth", label: "Earth", parent: "sun", kinds: ["Planets"], order: 1 },
  { id: "moon", label: "Moon", parent: "earth", kinds: ["Moons"], order: 1.0026 },
  { id: "iss", label: "ISS", parent: "earth", kinds: ["Ships and stations"], order: 1.00003 },
  { id: "io", label: "Io", parent: "jupiter", kinds: ["Moons"], order: 5.203 },
  { id: "belt", label: "Main asteroid belt", parent: "sun", kinds: ["Belts and rings"], order: 2.7 },
];

test("browsing: the thing, what is in it in the place's own order, and one level further folded behind a count", () => {
  const { rows, mode } = pickerRows(sol, { childWords: childWordsFor });
  assert.equal(mode, "browse");
  assert.deepEqual(rows.map((r) => `${"  ".repeat(r.depth)}${r.label}${r.folded ? ` [${r.folded.count} ${r.folded.words}]` : ""}`), ["Sun", "  Earth [2 inside]", "  Main asteroid belt", "  Jupiter [1 moon]"]);
});

test("a folded item opens in place", () => {
  const { rows } = pickerRows(sol, { open: ["earth"], childWords: childWordsFor });
  assert.deepEqual(rows.map((r) => r.label), ["Sun", "Earth", "ISS", "Moon", "Main asteroid belt", "Jupiter"]);
  assert.equal(rows.find((r) => r.id === "earth")?.opened, true);
});

test("a kind filter keeps the path to a match, quiet and not pickable", () => {
  const { rows } = pickerRows(sol, { kinds: ["Ships and stations"] });
  assert.deepEqual(rows.map((r) => [r.label, !!r.context]), [["Sun", true], ["Earth", true], ["ISS", false]]);
});

test("chips add together; none pressed shows everything; counts never move", () => {
  assert.equal(pickerRows(sol, { kinds: ["Moons", "Belts and rings"] }).rows.filter((r) => !r.context).length, 3);
  assert.deepEqual(kindChips(sol, ["Stars", "Planets", "Moons"]), [
    { kind: "Stars", count: 1 },
    { kind: "Planets", count: 2 },
    { kind: "Moons", count: 2 },
    { kind: "Belts and rings", count: 1 },
    { kind: "Ships and stations", count: 1 },
  ]);
});

test("searching is a flat list that says where each match is", () => {
  const { rows, mode } = pickerRows(sol, { query: "o" });
  assert.equal(mode, "search");
  assert.deepEqual(rows.map((r) => [r.label, r.caption ?? ""]), [["Moon", "in Earth"], ["Io", "in Jupiter"], ["Main asteroid belt", ""]]);
  assert.deepEqual(pickerRows(sol, { query: "o", kinds: ["Planets"] }).rows, []);
});

test("a loop in the data still lists its items", () => {
  const loop: PickItem[] = [{ id: "a", label: "A", parent: "b", kinds: ["Other"] }, { id: "b", label: "B", parent: "a", kinds: ["Other"] }];
  assert.deepEqual(pickerRows(loop).rows.map((r) => r.label), ["A", "B"]);
});

test("a system's subjects: the star on top, a moon after its planet, distances as captions", () => {
  const items = subjectItems([
    { address: "s", kind: "star", label: "Sun", facts: [], stamps: [] },
    { address: "m", kind: "planet", label: "Mars", parent: "s", orbit: { a_au: 1.52, phase_deg: 0 }, facts: [], stamps: [] },
    { address: "e", kind: "planet", label: "Earth", parent: "s", orbit: { a_au: 1, phase_deg: 0 }, facts: [], stamps: [] },
    { address: "l", kind: "moon", label: "Moon", parent: "e", orbit: { a_au: 0.0026, phase_deg: 0 }, facts: [], stamps: [] },
    { address: "b", kind: "belt", label: "Main asteroid belt", parent: "s", band_au: [2.1, 3.3], facts: [], stamps: [] },
    { address: "j", kind: "planet", label: "Jupiter", parent: "s", orbit: { a_au: 5.2, phase_deg: 0 }, facts: [], stamps: [] },
  ] as never);
  const { rows } = pickerRows(items, { open: ["e"] });
  // A belt sorts by the middle of its band and says its span.
  assert.deepEqual(rows.map((r) => [r.label, r.caption ?? ""]), [["Sun", ""], ["Earth", "1 AU"], ["Moon", ""], ["Mars", "1.5 AU"], ["Main asteroid belt", "2.1 to 3.3 AU"], ["Jupiter", "5.2 AU"]]);
});

test("the fold's words follow the children: one ring, two moons, a mix is inside", () => {
  const it = (label: string, kinds: string[]): PickItem => ({ id: label, label, kinds });
  assert.equal(childWordsFor([it("Main ring", ["Belts and rings"])]), "ring");
  assert.equal(childWordsFor([it("Phobos", ["Moons"]), it("Deimos", ["Moons"])]), "moons");
  assert.equal(childWordsFor([it("Phobos", ["Moons"]), it("Main ring", ["Belts and rings"])]), "inside");
});

test("a search starting with # finds a tag, from its start, ignoring case", () => {
  const tagged: PickItem[] = [...sol.map((i) => (i.id === "moon" ? { ...i, tags: ["Ruins", "#precursor"] } : i))];
  assert.deepEqual(pickerRows(tagged, { query: "#ruin" }).rows.map((r) => r.label), ["Moon"]);
  assert.deepEqual(pickerRows(tagged, { query: "#PRECURSOR" }).rows.map((r) => r.label), ["Moon"]);
  assert.deepEqual(pickerRows(tagged, { query: "#uins" }).rows, []);
  // A tag with parts ("class:planet/ocean") is found by any part's start, or the whole.
  assert.equal(matchesQuery({ label: "Tarn", tags: ["class:planet/ocean"] }, "#ocean"), true);
  assert.equal(matchesQuery({ label: "Tarn", tags: ["class:planet/ocean"] }, "#class:planet"), true);
  assert.equal(matchesQuery({ label: "Tarn", tags: ["class:planet/ocean"] }, "#cean"), false);
  assert.equal(matchesQuery({ label: "Ruins Hill" }, "#ruins"), false, "a # search does not match names");
  assert.equal(matchesQuery({ label: "Ruins Hill" }, "#"), false);
});

test("searching also finds places elsewhere in the campaign, after this level's, saying where they are", () => {
  const away: ElsewhereItem[] = [
    { id: "kestrel", label: "Kestrel", kinds: ["Systems"], where: "a system in this campaign" },
    { id: "k/tarn", label: "Tarn", kinds: ["Planets"], where: "in Kestrel", tags: ["ruins"] },
    { id: "earth", label: "Earth", kinds: ["Planets"], where: "in Sol" },
  ];
  assert.deepEqual(elsewhereRows(away, "", sol).rows, [], "nothing while browsing");
  assert.deepEqual(elsewhereRows(away, "ar", sol).rows.map((r) => [r.label, r.caption]), [["Tarn", "in Kestrel"]], "this level's Earth is not listed twice");
  assert.deepEqual(elsewhereRows(away, "#ruins", sol).rows.map((r) => r.id), ["k/tarn"]);
  const many: ElsewhereItem[] = Array.from({ length: 60 }, (_, i) => ({ id: `x${i}`, label: `Rock ${i}`, kinds: [], where: "in Sol" }));
  assert.deepEqual([elsewhereRows(many, "rock").rows.length, elsewhereRows(many, "rock").more], [50, 10]);
});

test("a system's subjects carry their tags into the picker", () => {
  const items = subjectItems([
    { address: "s", kind: "star", label: "Sol", facts: [], stamps: [], tags: [] },
    { address: "s/e", kind: "planet", label: "Earth", parent: "s", orbit: { a_au: 1, phase_deg: 0 }, facts: [], stamps: [], tags: ["homeworld"] },
  ]);
  assert.deepEqual(items.find((i) => i.id === "s/e")?.tags, ["homeworld"]);
  assert.equal(items.find((i) => i.id === "s")?.tags, undefined);
});
