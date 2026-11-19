import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { TRAVELLER_GRID, blockOf, blockHexes, distance, extent, label, parse, within, type GridConfig } from "../../packages/views/hex-map/src/grid.ts";
import { marksFor, type HexMapConfig, type MapWorld } from "../../packages/views/hex-map/src/glyphs.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const pack = JSON.parse(readFileSync(root + "packs/traveller/pack.json", "utf8"));
const cfg: HexMapConfig = pack.provides.other.views["sse.view.hex-map"];

test("the Traveller pack's grid is the shipped default grid", () => {
  assert.deepEqual(cfg.grid, TRAVELLER_GRID);
  assert.deepEqual(extent(cfg.grid), { columns: 32, rows: 40 });
});

test("labels round trip and subsectors fall where the labels say", () => {
  const g = TRAVELLER_GRID;
  assert.equal(label(g, { col: 1, row: 1 }), "0101");
  assert.equal(label(g, { col: 19, row: 10 }), "1910");
  assert.deepEqual(parse(g, "3240"), { col: 32, row: 40 });
  assert.equal(parse(g, "32400"), null);
  for (const [h, s] of [["0101", "A"], ["0810", "A"], ["0901", "B"], ["0111", "E"], ["3240", "P"]] as const)
    assert.equal(blockOf(g, parse(g, h)!).name, s, h);
  assert.equal(blockHexes(g, 0).length, 80);
});

test("distances follow the map: even columns sit half a hex lower", () => {
  const g = TRAVELLER_GRID;
  const d = (a: string, b: string) => distance(g, parse(g, a)!, parse(g, b)!);
  assert.equal(d("0101", "0201"), 1);
  assert.equal(d("0101", "0102"), 1);
  assert.equal(d("0101", "0202"), 2);
  // A lower column's neighbours to the right are on its own row and the next.
  assert.equal(d("0202", "0302"), 1);
  assert.equal(d("0202", "0303"), 1);
  assert.equal(d("0202", "0301"), 2);
  assert.equal(d("0101", "0501"), 4);
  assert.equal(d("0101", "0105"), 4);
  // Jump-2 from 0505: the hex itself plus 6 plus 12.
  assert.equal(within(g, parse(g, "0505")!, 2).length, 19);
  assert.equal(within(g, parse(g, "0101")!, 1).length, 3, "corner hex: itself, right, below");
  // Symmetric.
  for (const a of ["0101", "0410", "1723", "3240"])
    for (const b of ["0202", "0909", "2020"]) assert.equal(d(a, b), d(b, a));
});

test("glyph rules draw the starport, giants, bases and zones", () => {
  const w: MapWorld = { hex: "0101", name: "Kessa", fields: { starport: "A", size: 8, hydrographics: 7, gas_giants: 2, bases: ["N", "S"], zone: "A" } };
  const ms = marksFor(w, cfg.glyphs);
  const by = Object.fromEntries(ms.map((m) => [m.at === "ring" ? "ring" : m.at, m]));
  assert.equal(by.top.text, "A");
  assert.equal(by.centre.mark, "world-wet");
  assert.equal(by["top-right"].mark, "gas-giant");
  assert.equal(by["top-left"].mark, "star");
  assert.equal(by["bottom-left"].mark, "triangle");
  assert.equal(by.ring.tone, "caution");
  const belt = marksFor({ hex: "0202", name: "Rubble", fields: { starport: "E", size: 0, hydrographics: 0 } }, cfg.glyphs);
  assert.equal(belt.find((m) => m.at === "centre")!.mark, "belt");
});

test("the same view works with a non-Traveller grid", () => {
  const g: GridConfig = { block: { columns: 6, rows: 6 }, blocks: { across: 2, down: 1 }, label: "{c}.{r}", shifted: "odd", hex_distance: { value: 10, unit: "ly" } };
  assert.equal(label(g, { col: 7, row: 3 }), "7.3");
  assert.deepEqual(parse(g, "12.6"), { col: 12, row: 6 });
  assert.equal(blockOf(g, { col: 7, row: 1 }).name, "2");
  // Odd columns lower here: 1.2 touches 2.2 and 2.3.
  assert.equal(distance(g, { col: 1, row: 2 }, { col: 2, row: 2 }), 1);
  assert.equal(distance(g, { col: 1, row: 2 }, { col: 2, row: 3 }), 1);
  assert.equal(distance(g, { col: 1, row: 2 }, { col: 2, row: 1 }), 2);
  assert.equal(within(g, { col: 4, row: 3 }, 1).length, 7);
});

test("sifters: mains, choke points and the pack's own filters", async () => {
  const { mains, chokePoints, sift } = await import("../../packages/views/hex-map/src/sift.ts");
  const w = (hex: string, fields: Record<string, string | number | boolean | string[]> = {}) => ({ hex, name: hex, fields });
  // A chain 0101-0102-0103-0104-0204 (column 2 sits half a hex lower, so 0204 touches 0104), and a lone pair far away.
  const worlds = [w("0101"), w("0102"), w("0103"), w("0104"), w("0204"), w("1010"), w("1011"), w("2020")];
  const m = mains(TRAVELLER_GRID, worlds, 1, 3);
  assert.deepEqual(m.map((x) => x.hexes), [["0101", "0102", "0103", "0104", "0204"]]);
  assert.deepEqual(mains(TRAVELLER_GRID, worlds, 1, 2).map((x) => x.hexes.length), [5, 2]);
  assert.deepEqual(chokePoints(TRAVELLER_GRID, worlds, 1), ["0102", "0103", "0104"]);
  // Jump-2 joins more.
  assert.ok(chokePoints(TRAVELLER_GRID, worlds, 2).length < 2);
  // Pack sifters.
  const pack = JSON.parse(readFileSync(fileURLToPath(new URL("../../packs/traveller/pack.json", import.meta.url)), "utf8"));
  const cfg = pack.provides.other.views["sse.view.hex-map"];
  const by = (id: string) => cfg.sifters.find((s: { id: string }) => s.id === id);
  const map = [
    w("0101", { gas_giants: 2, hydrographics: 0, starport: "C", zone: "none", atmosphere: 6, law: 3 }),
    w("0102", { gas_giants: 0, hydrographics: 0, starport: "A", zone: "none", atmosphere: 11, law: 2 }),
    w("0103", { gas_giants: 0, hydrographics: 5, starport: "X", zone: "R", atmosphere: 12, law: 15 }),
  ];
  assert.deepEqual(sift(by("refuel"), map), ["0101", "0103"]);
  assert.deepEqual(sift(by("refined"), map), ["0102"]);
  assert.deepEqual(sift(by("red-candidates"), map), ["0102"], "0103 is already red");
});
