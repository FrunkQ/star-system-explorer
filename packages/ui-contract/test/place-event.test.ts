// The scale views' place ids (U24 with V9): cells, haloes' cells and slices of the web.
import { test } from "node:test";
import assert from "node:assert/strict";
import { cellId, parseCellId, parseWebId, webId } from "../src/place-event.ts";

test("a cell id round-trips, and a halo's address names its cell", () => {
  assert.equal(cellId([1, -2, 0]), "cell:1.-2.0");
  assert.deepEqual(parseCellId("cell:1.-2.0"), [1, -2, 0]);
  assert.deepEqual(parseCellId("u:t/cell:3.0.-1/halo:7"), [3, 0, -1]);
  assert.equal(parseCellId("cell:1.2"), null);
  assert.equal(parseCellId(42), null);
});
test("a web id carries the slice's side", () => {
  assert.equal(webId(200), "web:200");
  assert.equal(parseWebId("web:400"), "400");
  assert.equal(parseWebId("web:"), null);
  assert.equal(parseWebId(undefined), null);
});
