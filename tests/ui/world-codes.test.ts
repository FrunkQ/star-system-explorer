import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import {
  bodyRows,
  chipGroups,
  classNamesOf,
  disagrees,
  inputFromPacks,
  lineRows,
  segmentRows,
  sheetOf,
  sheetsOf,
  textList,
  textSheet,
  vocabularyOf,
  whySentence,
  type CodesResponse,
  type Reading,
} from "../../packages/views/world-codes/src/sheet.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const json = (p: string) => JSON.parse(readFileSync(root + p, "utf8"));
const traveller = json("packs/traveller/pack.json");
const toy = json("tests/fixtures/packs/codes-toy.json");
// The kernel's own `codes` answer for the toy fixture (contract golden).
const golden: CodesResponse = readFileSync(root + "tests/golden/contract.jsonl", "utf8")
  .split("\n")
  .filter((l) => l.startsWith('{"type":"codes"'))
  .map((l) => JSON.parse(l))
  .find((r) => r.readings.length === 3);

const label = (base: string, fit = 1) => ({ scheme: base.split("/")[0]!, base, fit, modifiers: [], nearest: [], kept_by_stickiness: false });

// An Earth-like reading through the Traveller sheet, built by hand in the
// kernel's shape: size set by hand to 3 against physics 8, a defaulted fact,
// the population multiplier, counted giants, trade codes and a base.
const earth: Reading = {
  sheet: "traveller.world",
  lines: { pbg: "814", uwp: "A367975-9" },
  segments: [
    { id: "size", symbol: "3", value: 3, why: { kind: "authored", field: "traveller.code.size", physics: "8" } },
    { id: "starport", symbol: "A", value: 10, why: { kind: "authored", field: "traveller.code.starport", physics: null } },
    { id: "atmosphere", symbol: "6", value: 6, why: { kind: "classified", label: label("traveller.atmosphere/06"), defaulted: ["atmosphere.pp.so2"] } },
    { id: "hydrographics", symbol: "7", value: 7, why: { kind: "classified", label: { ...label("traveller.hydrographics/07", 0.62), kept_by_stickiness: true } } },
    { id: "population", symbol: "9", value: 9, why: { kind: "classified", label: label("traveller.population/09") } },
    { id: "tech", symbol: "?", value: null, why: { kind: "unclassified", label: { ...label("traveller.tech/09"), base: null, fit: 0, nearest: [{ class: "traveller.tech/09", fit: 0.4, depth: 1, limiting: { axis: "traveller.tech", fit: 0.4, value: 8.5, change_to_core: 0.5 }, axes: [] }] } } },
    { id: "pop_mult", symbol: "8", value: 8, why: { kind: "leading", field: "settlements.population", value: 8.1e9, exponent: 9 } },
    { id: "belts", symbol: "1", value: 1, why: { kind: "counted", tag: "traveller.count.belt", members: ["u:sol/sys:0/body:9"] } },
    { id: "gas_giants", symbol: "4", value: 4, why: { kind: "no-system", tag: "traveller.count.giant" } },
  ],
  groups: [
    { id: "bases", tags: [["N", { tag: "traveller.base.N", rule: "traveller.base.N.v1", evidence: [{ field: "traveller.code.base.N", value: true }] }]] },
    {
      id: "trade",
      tags: [
        ["Hi", { tag: "traveller.trade.Hi", rule: "traveller.trade.Hi.v1", evidence: [{ field: "traveller.uwp.population", value: 9 }] }],
        ["Pr", { tag: "traveller.trade.Pr", rule: "traveller.trade.Pr.v1", evidence: [] }],
      ],
    },
    { id: "zone", tags: [] },
    { id: "zone_hint", tags: [["amber", { tag: "traveller.zonehint.amber", rule: "traveller.zonehint.amber.v1", evidence: [] }]] },
  ],
  prepared: [],
};

test("the toy golden reads through the toy pack's own sheet", () => {
  assert.ok(golden, "contract golden has the three-body codes case");
  const input = inputFromPacks([toy], golden, { names: { "u:toy/sys:0/body:1": "Alpha" } })!;
  assert.equal(input.sheet.id, "toy.world");
  const [a, b] = golden.readings;
  assert.deepEqual(lineRows(a!.reading, input.sheet), [{ id: "code", label: "CODE", text: "X800000-2" }]);

  const rowsA = segmentRows(a!.reading, input.sheet, input.classes, input.names);
  assert.deepEqual(rowsA.map((r) => [r.label, r.symbol, r.kind]), [["Size", "8", "classified"], ["Large bodies in the system", "2", "counted"]]);
  assert.equal(rowsA[0]!.why, "Classified as large, full fit.");
  assert.equal(rowsA[1]!.why, "Counted: 2 members of this system carry toy.large: Alpha, u:toy/sys:0/body:3.");

  // Body 2's size is set by hand to L although its radius says small.
  const size = b!.reading.segments[0]!;
  assert.equal(disagrees(size), true);
  const rowB = segmentRows(b!.reading, input.sheet, input.classes)[0]!;
  assert.equal(rowB.disagrees, true);
  assert.equal(rowB.physics, "1");
  assert.equal(rowB.why, "Set by hand (toy.code.size); physics says 1, small. The hand-set code and the physics disagree.");

  const g = chipGroups(a!.reading, input.sheet, input.vocabulary);
  assert.deepEqual(g.map((x) => [x.label, x.chips.map((c) => c.code)]), [["Notes", ["big"]]]);
  assert.equal(g[0]!.chips[0]!.words, null);
  assert.equal(g[0]!.chips[0]!.title, "big. Rule toy.note.big.v1, read toy.code.size = 2.");

  const rows = bodyRows(input);
  assert.deepEqual(rows.map((r) => [r.name, r.lines[0]!.text, r.groups[0]!.codes, r.disagreements]), [
    ["Alpha", "X800000-2", ["big"], 0],
    ["u:toy/sys:0/body:2", "X800000-2", ["big"], 1],
    ["u:toy/sys:0/body:3", "X800000-2", ["big"], 0],
  ]);
  assert.equal(textList(input)[1], "u:toy/sys:0/body:2: CODE X800000-2; Notes big; 1 hand-set digit differs from physics.");
});

test("an unknown sheet gives no input, and no id picks the first sheet", () => {
  assert.equal(inputFromPacks([toy], { ...golden, sheet: "nope" }), null);
  assert.equal(sheetOf([toy, traveller])!.id, "toy.world");
  assert.equal(sheetOf([toy, traveller], "traveller.world")!.id, "traveller.world");
});

test("a Traveller sheet: lines in layout order, every digit with its why", () => {
  const sheet = sheetOf([traveller], "traveller.world")!;
  const classes = classNamesOf([traveller]);
  assert.deepEqual(lineRows(earth, sheet).map((l) => `${l.label} ${l.text}`), ["UWP A367975-9", "PBG 814"]);

  const rows = segmentRows(earth, sheet, classes, { "u:sol/sys:0/body:9": "Main belt" });
  // Sheet order, not reading order.
  assert.deepEqual(rows.map((r) => r.id), ["starport", "size", "atmosphere", "hydrographics", "population", "tech", "pop_mult", "belts", "gas_giants"]);
  const by = Object.fromEntries(rows.map((r) => [r.id, r]));
  assert.equal(by.size!.label, "Size");
  assert.equal(by.size!.disagrees, true);
  assert.equal(by.size!.why, "Set by hand (traveller.code.size); physics says 8, size 8 (about 12,800 km across). The hand-set code and the physics disagree.");
  assert.equal(by.starport!.disagrees, false);
  assert.equal(by.starport!.why, "Set by hand (traveller.code.starport); the physics gives no reading here.");
  assert.match(by.atmosphere!.why, /^Classified as atmosphere 6 \(.+\), full fit\. Defaulted \(absent, read at the axis's stated value\): atmosphere\.pp\.so2\.$/);
  assert.deepEqual(by.atmosphere!.defaulted, ["atmosphere.pp.so2"]);
  assert.match(by.hydrographics!.why, /, fit 0\.62; kept from before although another class now fits better\.$/);
  assert.equal(by.tech!.why, "Unclassified (?): no class fits. Nearest is tech level 9, fit 0.40; traveller.tech is 8.5 and needs +0.5 to reach it.");
  assert.equal(by.pop_mult!.why, "Leading digit of settlements.population, 8.1 × 10^9, at the power of ten the sheet settled.");
  assert.equal(by.belts!.why, "Counted: 1 member of this system carries traveller.count.belt: Main belt.");
  assert.equal(by.gas_giants!.why, "No system: traveller.count.giant is counted over the whole system, and this body was read without it.");
});

test("authored and physics agreeing is not a disagreement", () => {
  const seg = { id: "size", symbol: "8", value: 8, why: { kind: "authored" as const, field: "traveller.code.size", physics: "8" } };
  assert.equal(disagrees(seg), false);
  assert.equal(whySentence(seg), "Set by hand (traveller.code.size); physics says 8 too.");
});

test("chips: sheet group order, the pack's words, empty groups left out", () => {
  const sheet = sheetOf([traveller], "traveller.world")!;
  const vocab = vocabularyOf([traveller]);
  const g = chipGroups(earth, sheet, vocab);
  assert.deepEqual(g.map((x) => x.id), ["trade", "zone_hint", "bases"]);
  assert.deepEqual(g[0]!.chips.map((c) => [c.code, c.words]), [["Hi", "high population"], ["Pr", "pre-rich"]]);
  assert.equal(g[0]!.label, "Trade codes");
  assert.equal(g[0]!.chips[0]!.title, "Hi: high population. Rule traveller.trade.Hi.v1, read traveller.uwp.population = 9.");
  // Bases: prefix traveller.base. but the words live under traveller.bases.
  assert.equal(g[2]!.chips[0]!.words, "naval base");
  assert.equal(g[1]!.chips[0]!.words, null);
});

test("a later pack's words win; the text tier says it all in sentences", () => {
  const sheet = sheetOf([traveller], "traveller.world")!;
  const house = { provides: { other: { vocabulary: { "traveller.trade": { Hi: "teeming" } } } } };
  const vocab = vocabularyOf([traveller, house]);
  assert.equal(vocab["traveller.trade"]!.Hi, "teeming");
  assert.equal(vocab["traveller.trade"]!.Pr, "pre-rich");

  const response: CodesResponse = { type: "codes", sheet: "traveller.world", readings: [{ address: "u:sol/sys:0/body:3", reading: earth }] };
  const input = inputFromPacks([traveller, house], response, { names: { "u:sol/sys:0/body:3": "Earth" } })!;
  const t = textSheet(input, "u:sol/sys:0/body:3");
  assert.equal(t[0], "Earth: Traveller world profile.");
  assert.equal(t[1], "UWP A367975-9.");
  assert.ok(t.some((l) => l.startsWith("Size 3 (differs from physics): Set by hand")));
  assert.ok(t.includes("Trade codes: Hi (teeming), Pr (pre-rich)."));
  assert.deepEqual(textSheet(input, "u:nowhere"), ["No reading for u:nowhere."]);
  assert.equal(textList(input)[0], "Earth: UWP A367975-9, PBG 814; Trade codes Hi Pr; Travel zone hints amber; Bases N; 1 hand-set digit differs from physics.");
});

test("the Traveller pack gives two sheets, and a value segment says it was read off the physics", () => {
  const all = sheetsOf([traveller]);
  assert.deepEqual(all.map((s) => s.id), ["traveller.world", "traveller.extended"]);
  assert.equal(sheetOf([traveller])?.id, "traveller.world");
  const ext = sheetOf([traveller], "traveller.extended")!;
  const g = { id: "gravity", symbol: "1.0", value: 1, why: { kind: "value" as const, field: "body.surface_gravity", value: 9.80665, per: "g0" } };
  assert.match(whySentence(g, ext), /body\.surface_gravity is 9\.807, over g0 from the data sheet\. Never rolled\./);
  const d = { id: "diameter_km", symbol: "12740", value: 12742, why: { kind: "value" as const, field: "body.radius", value: 6371000 } };
  assert.match(whySentence(d, ext), /times 0\.002/);
  const none = { id: "oxygen", symbol: "?", value: null, why: { kind: "value" as const, field: "atmosphere.pp.o2" } };
  assert.match(whySentence(none, ext), /does not have yet/);
});
