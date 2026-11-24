// The shapes of the deliveries (AT15a), without a browser.
import { test } from "node:test";
import assert from "node:assert/strict";
import { easingFor, keyframesFor, staggerOf } from "../src/deliver.ts";

test("arriving and leaving are the same shape run backwards, except Collapse to a dot", () => {
  const fadeIn = keyframesFor({ kind: "fade", direction: "down", exit: false });
  const fadeOut = keyframesFor({ kind: "fade", direction: "down", exit: true });
  assert.deepEqual(fadeIn.map((f) => f.opacity), [0, 1]);
  assert.deepEqual(fadeOut.map((f) => f.opacity), [1, 0]);
  const dot = keyframesFor({ kind: "collapse_dot", direction: "down", exit: true });
  assert.equal(dot[0]!.transform, "scale(1, 1)");
  assert.equal(dot[1]!.transform, "scale(1, 0.006)", "first a bright line");
  assert.equal(dot[2]!.transform, "scale(0.006, 0.006)", "then a dot");
  assert.equal(dot.at(-1)!.opacity, 0, "then nothing");
});

test("a wipe comes in from the side it names; a page turns about its spine", () => {
  assert.equal(keyframesFor({ kind: "wipe", direction: "right", exit: false })[0]!.clipPath, "inset(0 100% 0 0)");
  assert.equal(keyframesFor({ kind: "wipe", direction: "down", exit: false })[0]!.clipPath, "inset(0 0 100% 0)");
  assert.match(String(keyframesFor({ kind: "page_turn", direction: "down", exit: false })[0]!.transform), /rotateY\(-80deg\)/);
  assert.equal(keyframesFor({ kind: "none", direction: "down", exit: false }).length, 0);
});

test("the warm up starts black and steadies at full brightness", () => {
  const w = keyframesFor({ kind: "warm_up", direction: "down", exit: false });
  assert.equal(w[0]!.opacity, 0);
  assert.match(String(w.at(-1)!.filter), /brightness\(1\)/);
});

test("cheap electronics redraw in steps; type on steps a character at a time", () => {
  assert.equal(easingFor({ kind: "stepped_wipe", steps: 6 }), "steps(6, end)");
  assert.equal(easingFor({ kind: "type_on", steps: 6 }, 42), "steps(42, end)");
});

test("parts follow one another inside the plan's time; type on gives long lines more time", () => {
  const s = staggerOf(4, 1000);
  assert.equal(s[0]!.delay, 0);
  assert.ok(Math.abs(s.at(-1)!.delay + s.at(-1)!.ms - 1000) < 1e-9);
  const t = staggerOf(2, 900, [10, 20]);
  assert.equal(t[0]!.ms, 300);
  assert.equal(t[1]!.delay, 300);
  assert.equal(t[1]!.ms, 600);
});
