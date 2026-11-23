// Find in Look up: one box, best hit first.
import { test } from "node:test";
import assert from "node:assert/strict";
import { findInSheet } from "../src/index.ts";

const el = (z: number, symbol: string, name: string) => ({ id: symbol.toLowerCase(), symbol, name, z });
const sheet = {
  elements: [el(1, "H", "hydrogen"), el(2, "He", "helium"), el(26, "Fe", "iron"), el(50, "Sn", "tin"), el(78, "Pt", "platinum")],
  constants: [{ id: "G", name: "Newtonian constant of gravitation", group: "nature", value: { v: 6.674e-11 } }, { id: "h", name: "Planck constant", group: "nature", value: { v: 6.6e-34 } }],
  units: [{ id: "pc", name: "parsec", symbol: "pc", measures: "length", group: "astronomy" }],
} as never;

test("a name, a symbol or a number finds the element first", () => {
  assert.equal(findInSheet("iron", sheet)[0]!.id, "fe");
  assert.equal(findInSheet("Fe", sheet)[0]!.id, "fe");
  assert.equal(findInSheet("He", sheet)[0]!.id, "he", "the symbol's own case wins over helium's h-words");
  assert.equal(findInSheet("26", sheet)[0]!.id, "fe");
  assert.equal(findInSheet("Íron ", sheet)[0]!.id, "fe", "accents and spaces do not matter");
});

test("constants and units are found by any word of their name", () => {
  assert.equal(findInSheet("planck", sheet)[0]!.id, "h");
  assert.equal(findInSheet("gravitation", sheet)[0]!.id, "G");
  assert.equal(findInSheet("parsec", sheet)[0]!.table, "units");
  assert.deepEqual(findInSheet("", sheet), []);
  assert.deepEqual(findInSheet("zz", sheet), []);
});
