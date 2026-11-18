import { test } from "node:test";
import assert from "node:assert/strict";
import { encodeRequest } from "../src/json.ts";
import { checkScreen, type ScreenDoc } from "../src/screen.ts";
import { resolveParams, type ViewModule } from "../src/view.ts";
import { subjectOf } from "../src/mark.ts";
import { markerFor, type SymbologyDoc } from "../src/symbology.ts";
import { secondsToYears, yearsToSeconds } from "../src/intent.ts";
import { within } from "../src/seat.ts";

test("bigint times are written as exact digits", () => {
  assert.equal(encodeRequest({ at_s: 145_477_536_000_000_000n, x: 1 }), '{"at_s":145477536000000000,"x":1}');
  assert.equal(encodeRequest({ at_s: 9_223_372_036_854_775_807n }), '{"at_s":9223372036854775807}');
  assert.throws(() => encodeRequest({ v: NaN }));
});

test("years and kernel seconds round-trip, also through a lossy JSON number", () => {
  for (const yr of [0, 1, 4_610_000_000, 13_800_000_000]) {
    const s = yearsToSeconds(yr);
    assert.equal(secondsToYears(s), yr);
    assert.equal(secondsToYears(Number(s)), yr);
  }
});

test("address containment is by segment", () => {
  assert.ok(within("u:k/sys:1/body:3", "u:k/sys:1"));
  assert.ok(!within("u:k/sys:10", "u:k/sys:1"));
});

const view = (id: string): ViewModule => ({
  id,
  version: "0.1.0",
  tiers: ["canvas"],
  inputs: [],
  params: [{ name: "zoom", type: "number", default: 1, range: [0.25, 8], meaning: "chart zoom" }],
  mount: () => ({ update() {}, dispose() {} }),
});

test("parameters: defaults fill in, unknown and out-of-range values are rejected", () => {
  assert.deepEqual(resolveParams(view("a").params, {}), { params: { zoom: 1 }, errors: [] });
  assert.equal(resolveParams(view("a").params, { zoom: 20 }).errors.length, 1);
  assert.equal(resolveParams(view("a").params, { colour: "red" }).errors.length, 1);
});

test("screens mount only when every item is valid", () => {
  const doc: ScreenDoc = {
    schema: "sse.screen/1", id: "t", label: "t", audience: "gm", layout: "chart-room",
    theme: "chart-room", symbology: "s", vocab: "physics",
    items: [{ view: "a", area: "chart" }, { view: "b", area: "card" }],
  };
  assert.equal(checkScreen(doc, [view("a"), view("b")]).ok, true);
  const bad = checkScreen({ ...doc, items: [...doc.items, { view: "c", area: "nowhere" }] }, [view("a"), view("b")]);
  assert.equal(bad.ok, false);
  const twice = checkScreen({ ...doc, items: [{ view: "a", area: "chart" }, { view: "b", area: "chart" }] }, [view("a"), view("b")]);
  assert.equal(twice.ok, false);
});

test("marks come from the symbology document, not the view", () => {
  const sym: SymbologyDoc = {
    schema: "sse.symbology/1", id: "s", name: "s",
    bindings: [
      { applies_to: "planet", channel: "fill_pattern", from: "class", scale: { table: { "planet/ocean": "band" } }, fallback: "solid" },
      { applies_to: "star", channel: "shape", from: "kind", scale: { table: { star: "star" } }, fallback: "disc" },
    ],
  };
  const m = markerFor("s", [sym], { supports: { colours: "full", patterns: true, shapes: true } });
  const p = { address: "u:x/b:1", kind: "planet", label: "P", facts: [], stamps: [], tags: ["class:planet/ocean"] };
  assert.equal(m.mark(subjectOf(p)).fill_pattern, "band");
  assert.equal(m.mark(subjectOf({ ...p, tags: ["class:planet/barren"] })).fill_pattern, "solid");
  assert.equal(m.mark(subjectOf({ ...p, kind: "star" })).shape, "star");
});
