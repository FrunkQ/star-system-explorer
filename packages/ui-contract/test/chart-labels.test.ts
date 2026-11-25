import { test } from "node:test";
import assert from "node:assert/strict";
import { labelAt, placeLabels, textWidth, type LabelBody } from "../src/chart-labels.ts";

const bounds = { x: 0, y: 0, w: 800, h: 600 };
const body = (id: string, x: number, y: number, extra: Partial<LabelBody> = {}): LabelBody => ({ id, x, y, r: 5, w: textWidth(id, 16), h: 16, ...extra });

test("a lone name sits up and to the right, as before", () => {
  const [p] = placeLabels([body("Earth", 400, 300)], [], bounds);
  assert.equal(p!.side, "ne");
  assert.equal(p!.anchor, "start");
  assert.ok(!p!.crowded);
});

test("two bodies side by side: the second name moves rather than cover the first", () => {
  const out = placeLabels([body("Mercury", 400, 300), body("Venus", 420, 296)], [], bounds);
  const [a, b] = out;
  const hit = !(a!.box.x + a!.box.w <= b!.box.x || b!.box.x + b!.box.w <= a!.box.x || a!.box.y + a!.box.h <= b!.box.y || b!.box.y + b!.box.h <= a!.box.y);
  assert.ok(!hit, `boxes overlap: ${JSON.stringify(out.map((p) => p.box))}`);
  assert.ok(!a!.crowded && !b!.crowded);
});

test("a name near the right edge turns to the left; a box it must keep clear is kept clear", () => {
  const [p] = placeLabels([body("Neptune", 790, 300)], [], bounds);
  assert.equal(p!.anchor, "end");
  const clear = [labelAt(body("Saturn", 400, 300), "ne").box];
  const [q] = placeLabels([body("Saturn", 400, 300)], clear, bounds);
  assert.notEqual(q!.side, "ne");
});

test("the selected body is placed first and keeps the usual place", () => {
  const out = placeLabels([body("Ceres", 400, 300), body("Mars", 395, 302, { first: true })], [], bounds);
  assert.equal(out[1]!.side, "ne");
  assert.equal(out[0]!.id, "Ceres");
});

test("the same chart places the same way every time", () => {
  const bs = [body("A", 100, 100), body("B", 108, 104), body("C", 104, 96), body("D", 112, 100)];
  assert.deepEqual(placeLabels(bs, [], bounds), placeLabels(bs, [], bounds));
});
