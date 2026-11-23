import { test } from "node:test";
import assert from "node:assert/strict";
import { gridOverlay, hexCentre, hexCorners, hexLabel, overlaysFor, overlaySpecsOf, placeBy, type OverlayGrid, type OverlaySpec } from "../src/overlay.ts";

const GRID: OverlayGrid = { block: { columns: 8, rows: 10 }, blocks: { across: 4, down: 4 }, label: "{c2}{r2}", block_names: "ABCDEFGHIJKLMNOP".split(""), shifted: "even", hex_distance: { value: 1, unit: "pc" } };
const pack = (id: string, other: Record<string, unknown>) => ({ id, provides: { other } });
const near = (a: readonly number[], b: readonly number[]) => assert.ok(Math.hypot(a[0]! - b[0]!, a[1]! - b[1]!) < 1e-9, `${a} vs ${b}`);

test("a pack's overlays come from the stack, the higher pack's winning, and a grid can be the hex map's", () => {
  const packs = [
    pack("low", { overlays: [{ id: "g", label: "Low grid", frame: "galaxy-plane-pc", grid: GRID }] }),
    pack("high", { overlays: [{ id: "g", label: "Sector grid", frame: "galaxy-plane-pc", grid: "sse.view.hex-map" }, { id: "bad", label: "x", frame: "nowhere" }], views: { "sse.view.hex-map": { grid: GRID } } }),
  ];
  const specs = overlaySpecsOf(packs);
  assert.deepEqual(specs.map((s) => [s.id, s.label, s.pack]), [["g", "Sector grid", "high"]]);
  const o = overlaysFor(packs, ["galaxy-plane-pc"], {}, [0, -10, 10, 0], 40);
  assert.equal(o.length, 1);
  assert.match(o[0]!.why!, /Place Sector grid on the map first/);
  assert.equal(overlaysFor(packs, ["system-au"], {}, [0, 0, 1, 1], 1).length, 0, "only the view's own frames");
});

test("hexes: centres one hex distance apart, even columns half a hex lower, rows down the sheet; labels by the form", () => {
  near(hexCentre(GRID, 1, 1), [0, 0]);
  near(hexCentre(GRID, 1, 2), [0, -1]);
  const c21 = hexCentre(GRID, 2, 1);
  assert.ok(Math.abs(Math.hypot(c21[0], c21[1]) - 1) < 1e-12, "a neighbour across is one hex distance away");
  assert.ok(c21[1] < 0, "even columns hang lower");
  assert.equal(hexLabel(GRID, 3, 7), "0307");
  const k = hexCorners([0, 0], 1, "flat");
  near(k[0]!, [1, 0]);
});

test("placed by the anchor: turned about hex 0101, then moved; only hexes in view, labels when they fit", () => {
  const a = { origin: [100, 50] as [number, number], rotation_deg: 90 };
  near(placeBy(a, [1, 0]), [100, 51]);
  const spec: OverlaySpec = { id: "g", label: "Sector grid", frame: "galaxy-plane-pc", pack: "p" };
  // Far out: the sheet and its blocks, block names, no hexes.
  const far = gridOverlay(spec, GRID, { origin: [0, 0], rotation_deg: 0 }, [-5, -45, 50, 5], 4);
  assert.deepEqual(far.layers.map((l) => l.kind), ["lines", "lines", "labels"]);
  assert.equal(far.layers[1]!.items.length, 6, "three block bounds across, three down");
  // Close in: hexes in view only, labelled.
  const close = gridOverlay(spec, GRID, { origin: [0, 0], rotation_deg: 0 }, [-0.5, -3, 3, 0.5], 40);
  const hexes = close.layers.find((l) => l.kind === "hexes");
  assert.ok(hexes && hexes.items.length > 4 && hexes.items.length < 40, `${hexes?.items.length} hexes`);
  assert.equal((hexes!.items[0] as { label?: string }).label, "0101");
});

// OV2: map layers of areas (territory overlays design 4 and 10).
import { areaProblems, areaRole, areaSpecsOf, assignSlots, mapSlotToken, MAP_PATTERNS, MAP_SLOTS, OVERLAY_FRAMES } from "../src/overlay.ts";

const POWERS = {
  id: "so.powers",
  label: "Powers of the Reach",
  areas: {
    by: { relation: "holds" },
    answers: { from: "actors", under: "so" },
    level: "auto",
    fill: { between_stars: { reach: { value: 4, unit: "pc" } }, in_system: "hill", on_surface: "regions" },
    weigh_by: "population",
    look: "sse.look.atlas",
    tier: "obvious",
  },
};

test("OV2: a map layer needs no frame, round-trips through the stack unchanged, and OV1's grids are untouched", () => {
  const packs = [pack("so", { overlays: [POWERS, { id: "noframe", label: "x" }, { id: "g", label: "Sector grid", frame: "galaxy-plane-pc", grid: GRID }] })];
  const specs = overlaySpecsOf(packs);
  assert.deepEqual(specs.map((s) => s.id), ["so.powers", "g"], "a frameless overlay without areas is dropped");
  const back = JSON.parse(JSON.stringify(areaSpecsOf(packs)));
  assert.deepEqual(back, [{ ...POWERS, pack: "so" }]);
  // Areas are drawn from holdings, so the fixed-data path skips them; the grid still comes through.
  assert.deepEqual(overlaysFor(packs, OVERLAY_FRAMES, {}, [0, 0, 1, 1], 1).map((o) => o.id), ["g"]);
  assert.ok(OVERLAY_FRAMES.includes("body-latlon") && OVERLAY_FRAMES.includes("web-mpc"));
});

test("OV2: every kind of source reads; mistakes are said in words", () => {
  for (const by of [{ relation: "holds" }, { tag_group: "tech era" }, { choice: "travel zone" }, { fact: "biome" }, { fact: "population", bands: [1e3, 1e6, 1e9] }, { knowledge: "charted" }, { reach: { from: "u:x/sys:1", radius: { value: 0, unit: "ly" }, rate: { value: 1, unit: "c" } } }]) assert.deepEqual(areaProblems({ by }), [], JSON.stringify(by));
  assert.match(areaProblems({ by: {} })[0]!, /one thing to read/);
  assert.match(areaProblems({ by: { relation: "holds", tag_group: "x" } })[0]!, /one thing/);
  assert.match(areaProblems({ by: { fact: "population", bands: [5, 1] } })[0]!, /larger than the last/);
  assert.match(areaProblems({ by: { relation: "holds" }, fill: { between_stars: { reach: { value: -1, unit: "pc" } } } })[0]!, /between_stars/);
  assert.match(areaProblems({ by: { relation: "holds" }, level: -2 })[0]!, /level/);
  const bad = [pack("p", { overlays: [{ id: "bad", label: "Bad", areas: { by: {} } }] })];
  assert.deepEqual(areaSpecsOf(bad), [], "a layer that does not read is left out");
});

test("OV2: palette slots: neighbours never share, preferences kept when free, reused by answers that do not touch, same on every device", () => {
  assert.equal(MAP_SLOTS.length, 8);
  assert.equal(MAP_PATTERNS.length, 8);
  assert.equal(mapSlotToken(3), "--sse-map-3");
  const answers = ["crown", "league", "vesh", "free", "a", "b", "c", "d", "e", "f"].map((key) => ({ key }));
  // A ring of ten answers, each touching the next.
  const touching = answers.map((a, i) => [a.key, answers[(i + 1) % answers.length]!.key] as const);
  const s = assignSlots(answers, touching);
  for (const [a, b] of touching) assert.notEqual(s.get(a), s.get(b), `${a} and ${b} touch`);
  assert.ok(new Set(s.values()).size <= 3, "a ring needs few slots: they are reused");
  assert.deepEqual([...assignSlots([...answers].reverse(), touching).entries()].sort(), [...s.entries()].sort(), "order of input does not matter");
  const pref = assignSlots([{ key: "crown", prefer: 5 }, { key: "league", prefer: 5 }], [["crown", "league"]]);
  assert.equal(pref.get("crown"), 5);
  assert.notEqual(pref.get("league"), 5, "the second wanting the same slot as its neighbour gets another");
  // Nine answers all touching one another: the ninth shares the slot fewest neighbours use.
  const nine = Array.from({ length: 9 }, (_, i) => ({ key: `p${i}` }));
  const all = nine.flatMap((a, i) => nine.slice(i + 1).map((b) => [a.key, b.key] as const));
  assert.equal(new Set(assignSlots(nine, all).values()).size, 8);
});

test("OV2: stacked area layers fill, then edge, then mark", () => {
  assert.deepEqual([0, 1, 2, 3].map(areaRole), ["fill", "edges", "marks", "marks"]);
});
