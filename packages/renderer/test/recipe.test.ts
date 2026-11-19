// R1 acceptance, node side: every shipped recipe is valid and runs on every
// tier it declares; invalid recipes are rejected with a reason; knobs and
// degrade steps change the plan as documented; estimates follow the cost model.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { checkRecipe, estimateMs, maxLevel, pickRecipe, plan, stepsFrom, type RecipeDoc } from "../src/recipe.ts";
import { LIBRARY } from "../src/passes.ts";

const dir = new URL("../src/recipes/", import.meta.url);
const SHIPPED: RecipeDoc[] = readdirSync(dir).filter((f) => f.endsWith(".json")).map((f) => JSON.parse(readFileSync(new URL(f, dir), "utf8")));
const relief = () => structuredClone(SHIPPED.find((r) => r.id === "survey-relief")!);

test("every shipped recipe checks clean and plans on every tier it declares, at every degrade level", () => {
  assert.ok(SHIPPED.length >= 2);
  for (const r of SHIPPED) {
    assert.deepEqual(checkRecipe(r, LIBRARY).errors, [], r.id);
    for (const t of r.tiers) {
      for (let lv = 0; lv <= maxLevel(r, t); lv++) {
        const p = plan(r, LIBRARY, t, lv);
        const need = p.tier === "text" ? "text" : "image";
        assert.ok(p.passes.some((x) => x.kind === need), `${r.id} on ${t} level ${lv} shows something`);
        assert.equal(p.reductions.length, lv);
      }
    }
  }
});

test("every pass parameter is described for Orac: meaning, group, range on numbers, options on enums", () => {
  for (const p of LIBRARY) {
    assert.ok(p.meaning && p.tiers.length);
    for (const t of p.tiers) assert.ok(p.cost[t], `${p.id} costs something on ${t}`);
    for (const s of p.params) {
      assert.ok(s.meaning.length > 10 && s.group, `${p.id}.${s.name}`);
      if (s.type === "number" || s.type === "integer") assert.ok(s.range, `${p.id}.${s.name} has a range`);
      if (s.type === "enum") assert.ok(s.options?.includes(String(s.default)));
    }
    for (const ref of p.references ?? []) assert.ok(ref.doi || ref.url);
  }
});

test("invalid recipes are rejected, each with its reason", () => {
  const cases: [string, (r: RecipeDoc) => void, RegExp][] = [
    ["wrong schema", (r) => ((r as { schema: string }).schema = "sse.render-recipe/9"), /schema/],
    ["unknown pass", (r) => (r.passes[1]!.pass = "lens-flare"), /no pass lens-flare/],
    ["parameter out of range", (r) => (r.passes[0]!.params = { exaggeration: 900 }), /exaggeration: must be within 1 to 60/],
    ["unknown parameter", (r) => (r.passes[0]!.params = { sparkle: 1 }), /unknown parameter sparkle/],
    ["a tier with no picture", (r) => (r.passes = r.passes.filter((p) => p.id !== "relief-still")), /tier still: no image pass/],
    ["pass on an undeclared tier", (r) => (r.passes[1]!.tiers = ["gpu-high"]), /does not declare/],
    ["knob leaves the range", (r) => (r.knobs![0]!.maps[0]!.at1 = 400), /would leave its range/],
    ["knob on text", (r) => (r.knobs![0]!.maps[0]!.param = "colour"), /is not a number/],
    ["render scale goes up", (r) => r.degrade.push({ render_scale: 0.9, says: "x" }), /goes up/],
    ["step says nothing", (r) => (r.degrade[0]!.says = ""), /says nothing/],
    ["dropping the only picture", (r) => r.degrade.push({ drop: "relief", says: "Globe off." }), /nothing to show/],
    ["tier step that goes up", (r) => r.degrade.push({ tier: "gpu-basic", tiers: ["canvas"], says: "x" }), /not a step down/],
    ["code-bearing", (r) => (r.contains_code = true), /data only/],
    ["bad budget", (r) => (r.budget_ms = { canvas: -3 }), /positive/],
  ];
  for (const [name, mutate, why] of cases) {
    const r = relief();
    mutate(r);
    const c = checkRecipe(r, LIBRARY);
    assert.equal(c.ok, false, name);
    assert.ok(c.errors.some((e) => why.test(e)), `${name}: ${c.errors.join("; ")}`);
  }
});

test("knobs move the deep parameters they map, and integers stay whole", () => {
  const r = relief();
  const lo = plan(r, LIBRARY, "gpu-basic", 0, { relief: 0, detail: 0 }).passes.find((p) => p.id === "relief")!.params;
  const hi = plan(r, LIBRARY, "gpu-basic", 0, { relief: 1, detail: 1 }).passes.find((p) => p.id === "relief")!.params;
  assert.deepEqual([lo.exaggeration, lo.shade, lo.grid], [2, 0.4, 48]);
  assert.deepEqual([hi.exaggeration, hi.shade, hi.grid], [30, 1, 208]);
  assert.equal(plan(r, LIBRARY, "gpu-basic", 0, { detail: 0.33 }).passes[0]!.params.grid, 101);
});

test("the degrade order is followed per tier, through a tier fall", () => {
  const r = relief();
  assert.deepEqual(stepsFrom(r, "canvas").map((s) => s.says), ["Drawn at three-quarter resolution.", "Grid lines off.", "Drawn at half resolution.", "Drawn at a third of full resolution.", "Drawn at a quarter of full resolution."]);
  const g = stepsFrom(r, "gpu-basic").map((s) => s.says);
  assert.equal(g.length, 7, "gpu-basic walks its own steps, falls to canvas, then takes canvas's last steps");
  const last = plan(r, LIBRARY, "gpu-basic", 7);
  assert.equal(last.tier, "canvas");
  assert.equal(last.render_scale, 0.25);
  assert.ok(!last.passes.some((p) => p.id === "grid"));
  assert.equal(plan(r, LIBRARY, "gpu-basic", 2).passes[0]!.params.grid, 64);
});

test("cost estimates follow the model: pixels, render scale and scaled parameters", () => {
  const r = relief();
  const full = estimateMs(plan(r, LIBRARY, "canvas", 0), LIBRARY, 1e6);
  const half = estimateMs(plan(r, LIBRARY, "canvas", 3), LIBRARY, 1e6);
  assert.ok(half < full * 0.3, `${half} vs ${full}`);
  const fine = estimateMs(plan(r, LIBRARY, "gpu-basic", 0, { detail: 1 }), LIBRARY, 1e6);
  const coarse = estimateMs(plan(r, LIBRARY, "gpu-basic", 0, { detail: 0 }), LIBRARY, 1e6);
  assert.ok(fine > coarse);
  assert.equal(estimateMs(plan(r, LIBRARY, "canvas", 0), LIBRARY, 1e6, { canvas: 3 }), full * 3);
});

test("a recipe that cannot run on a tier hands over to one of its family that can", () => {
  const got = pickRecipe(SHIPPED, "survey-contours", "gpu-basic");
  assert.equal(got?.doc.id, "survey-relief");
  assert.match(got!.stood_in!, /Contour survey has no gpu-basic form, so Survey relief stands in/);
  assert.equal(pickRecipe(SHIPPED, "survey-contours", "canvas")?.stood_in, undefined);
  assert.equal(pickRecipe(SHIPPED, "survey-contours", "gpu-high")?.doc.id, "true-colour");
  assert.equal(pickRecipe(SHIPPED, "survey-contours", "still")?.doc.id, "survey-relief");
  assert.equal(pickRecipe(SHIPPED, "no-such-recipe", "canvas"), null);
});
