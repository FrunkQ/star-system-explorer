// The Air box's mix (ED8): shares that always sum to one; one gas traded against the rest, which keep their proportions.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { airMix, formulaOf, mixEdits, retrade, substanceNames } from "../src/air.ts";

const earth = { "atmosphere.composition.n2": 0.7808, "atmosphere.composition.o2": 0.2095, "atmosphere.composition.ar": 0.00934, "atmosphere.composition.co2": 0.00042, "atmosphere.surface_pressure": 101400 };

test("the mix reads the composition facts, largest first", () => {
  assert.deepEqual(airMix(earth).map((g) => g.gas), ["n2", "o2", "ar", "co2"]);
  assert.deepEqual(airMix({ "body.mass": 1 }), []);
});

test("setting one gas trades it against the rest, which keep their proportions; the sum stays one", () => {
  const m = airMix(earth);
  const r = retrade(m, "co2", 0.01);
  const sum = r.reduce((t, g) => t + g.share, 0);
  assert.ok(Math.abs(sum - 1) < 1e-12, `sums to one: ${sum}`);
  const get = (g: string) => r.find((x) => x.gas === g)!.share;
  assert.equal(get("co2"), 0.01);
  assert.ok(Math.abs(get("n2") / get("o2") - 0.7808 / 0.2095) < 1e-9, "nitrogen to oxygen as before");
  // A new gas joins the same way; one edit per field that moved.
  const ch4 = retrade(m, "ch4", 0.05);
  assert.equal(ch4.find((g) => g.gas === "ch4")!.share, 0.05);
  const e = mixEdits(m, ch4);
  assert.equal(e.length, 5);
  assert.equal(e.find((x) => x.field === "atmosphere.composition.ch4")!.before, null);
  // All of one gas: the rest go to zero and stay listed, so Undo can put them back.
  const all = retrade(m, "co2", 1);
  assert.equal(all[0]!.gas, "co2");
  assert.ok(all.slice(1).every((g) => g.share === 0));
});

test("names from the data sheet, formulas written out", () => {
  const ref = JSON.parse(readFileSync(new URL("../../../packs/base-reference/pack.json", import.meta.url), "utf8"));
  const n = substanceNames([ref]);
  assert.equal(n.get("co2"), "carbon dioxide");
  assert.equal(formulaOf("co2"), "CO₂");
  assert.equal(formulaOf("so2"), "SO₂");
  assert.equal(formulaOf("ar"), "Ar");
  assert.equal(formulaOf("ch4"), "CH₄");
});

test("Kinds for air come from the catalogued Solar System: Venus, Earth, Mars, Pluto, thickest first", async () => {
  const { readSol } = await import("../src/sol.ts");
  const { airKinds, airKindEdits, gasesToAdd } = await import("../src/air.ts");
  const sol = readSol(JSON.parse(readFileSync(new URL("../../../packs/base-sol/pack.json", import.meta.url), "utf8")));
  const k = airKinds(sol);
  assert.deepEqual(k.map((x) => x.id), ["venus", "earth", "mars", "pluto"]);
  assert.equal(k[1]!.label, "Like Earth");
  assert.equal(k[1]!.pressure_pa, 101400);
  // Mars's air on Earth: the pressure, Mars's gases, and Earth's methane and hydrogen (not in Mars's list) to nothing.
  const e = airKindEdits({ ...earth, "atmosphere.composition.ch4": 1.94e-6 }, k[2]!);
  assert.equal(e[0]!.field, "atmosphere.surface_pressure");
  assert.equal(e.find((x) => x.field === "atmosphere.composition.ch4")!.value, 0);
  assert.ok(e.find((x) => x.field === "atmosphere.composition.xe"));
  const names = new Map([["co2", "carbon dioxide"], ["nh3", "ammonia"]]);
  assert.deepEqual(gasesToAdd(names, airMix(earth)).map((g) => g.gas), ["nh3"]);
});

test("Add a gas offers what can be in an air, not ions, rock or rocket fuel", async () => {
  const { gasNames } = await import("../src/air.ts");
  const ref = JSON.parse(readFileSync(new URL("../../../packs/base-reference/pack.json", import.meta.url), "utf8"));
  const g = gasNames([ref], ["so2", "ne"]);
  for (const x of ["co2", "ch4", "nh3", "n2", "o2", "ar", "he", "xe", "so2", "ne"]) assert.ok(g.has(x), x);
  for (const x of ["h+", "sio2", "glucose", "mmh", "n2h4", "h2o2"]) assert.ok(!g.has(x), x);
});
