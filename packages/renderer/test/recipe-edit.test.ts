// R4: recipe edits are checked, costed against the budget, and never change shipped recipes.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { applyEdits, editableSettings, LIBRARY, type RecipeDoc } from "../src/index.ts";

const dir = new URL("../src/recipes/", import.meta.url);
const SHIPPED_RECIPES: RecipeDoc[] = readdirSync(dir).filter((f) => f.endsWith(".json")).map((f) => JSON.parse(readFileSync(new URL(f, dir), "utf8")));

const tc = () => SHIPPED_RECIPES.find((r) => r.id === "true-colour")!;
const ctx = { library: LIBRARY };

test("a setting change makes a new recipe, says what changed, and costs it", () => {
  const before = JSON.stringify(tc());
  const r = applyEdits(tc(), [{ op: "set", use: "air", param: "exposure", value: 1.6 }, { op: "name", name: "Bright true colour" }], ctx);
  assert.deepEqual(r.errors, []);
  assert.ok(r.ok);
  assert.equal(r.doc.id, "true-colour-edited");
  assert.equal(r.doc.derived_from, "true-colour");
  assert.equal(r.doc.name, "Bright true colour");
  assert.match(r.changes[0]!, /Atmosphere \(air\): exposure 1 to 1\.6/);
  assert.deepEqual(r.doc.history, r.changes);
  assert.equal(JSON.stringify(tc()), before, "the shipped recipe is untouched");
  const high = r.cost.find((c) => c.tier === "gpu-high")!;
  assert.ok(high.before_ms! > 0 && high.after_ms === high.before_ms, "exposure costs nothing");
});

test("an edit that blows the budget is reported, not hidden", () => {
  const r = applyEdits(tc(), [{ op: "set", use: "air", param: "steps", value: 64 }, { op: "knob_default", knob: "detail", value: 1 }, { op: "set", use: "air", param: "light_steps", value: 16 }], { ...ctx, budget_ms: 16.7 });
  assert.ok(r.ok, r.errors.join("; "));
  const high = r.cost.find((c) => c.tier === "gpu-high")!;
  assert.ok(high.after_ms! > high.before_ms! * 2, `${high.before_ms} to ${high.after_ms}`);
  assert.ok(r.newly_over.includes("gpu-high"), JSON.stringify(r.cost));
  assert.match(r.changes[0]!, /the Detail knob also moves this/);
});

test("bad edits are refused with reasons", () => {
  const cases: [Parameters<typeof applyEdits>[1], RegExp][] = [
    [[{ op: "set", use: "nope", param: "x", value: 1 }], /no pass nope/],
    [[{ op: "set", use: "air", param: "colour", value: 1 }], /has no setting colour/],
    [[{ op: "set", use: "air", param: "steps", value: 900 }], /steps/],
    [[{ op: "knob_default", knob: "detail", value: 2 }], /takes 0 to 1/],
    [[{ op: "add", use: { id: "x", pass: "shader-toy" } }], /no pass shader-toy in the library/],
    [[{ op: "add", use: { id: "air", pass: "atmosphere" } }], /already has a pass air/],
    [[{ op: "remove", use: "relief" }, { op: "remove", use: "relief-still" }, { op: "remove", use: "relief-basic" }], /show nothing|no pass relief-basic/],
    [[{ op: "budget", tier: "canvas", ms: -1 }], /positive/],
    [[{ op: "warp" } as never], /unknown edit "warp"/],
    [[], /no edits/],
  ];
  for (const [eds, want] of cases) {
    const r = applyEdits(tc(), eds, ctx);
    assert.equal(r.ok, false, JSON.stringify(eds));
    assert.match(r.errors.join("; "), want, JSON.stringify(eds));
  }
});

test("removing a pass takes its reduction steps and knob links with it, and says so", () => {
  const sr = SHIPPED_RECIPES.find((r) => r.id === "survey-relief")!;
  const grid = sr.passes.find((p) => p.pass === "graticule")!.id;
  const r = applyEdits(sr, [{ op: "remove", use: grid }], ctx);
  assert.ok(r.ok, r.errors.join("; "));
  assert.ok(!r.doc.degrade.some((s) => "drop" in s && s.drop === grid));
  assert.match(r.changes[0]!, /Removed Graticule/);
});

test("an edited recipe edits again under its own id, history growing", () => {
  const a = applyEdits(tc(), [{ op: "knob_default", knob: "detail", value: 0.8 }], ctx);
  const b = applyEdits(a.doc, [{ op: "unset", use: "air", param: "exposure" }], ctx);
  assert.ok(b.ok, b.errors.join("; "));
  assert.equal(b.doc.id, a.doc.id);
  assert.equal(b.doc.history!.length, 2);
});

test("the settings menu lists every pass with values and knob links", () => {
  const m = editableSettings(tc(), LIBRARY);
  const air = m.find((x) => x.use === "air")!;
  assert.equal(air.name, "Atmosphere");
  assert.equal(air.settings.find((s) => s.name === "steps")!.knob, "Detail");
  assert.equal(air.settings.find((s) => s.name === "exposure")!.value, 1);
});
