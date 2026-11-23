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
