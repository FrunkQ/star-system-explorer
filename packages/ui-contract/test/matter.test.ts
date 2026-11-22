// The Reference screen's reading of the data sheet (MP10 part 1): formatting, formulas, and the page read through
// a real kernel with base-reference loaded.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { sourceName, dimensionless, evalFormula, formatUnit, formatValue, readConstantsPage, rowNote, siUnits, unitFactor } from "../src/matter.ts";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { nodeKernel } from "./node-kernel.ts";

test("values print in threes, with powers of ten for the large and the small", () => {
  assert.equal(formatValue(299792458), "299 792 458");
  assert.equal(formatValue(6.6743e-11), "6.674 3 × 10⁻¹¹");
  assert.equal(formatValue(6.6743e-11, false), "6.674 × 10⁻¹¹");
  assert.equal(formatValue(9460730472580800), "9.460 730 472 580 8 × 10¹⁵");
  assert.equal(formatValue(-1), "−1");
  assert.equal(formatValue(0.3111), "0.311 1");
  assert.equal(formatValue(273.15), "273.15");
});

test("source ids read as names", () => {
  assert.equal(sourceName("clowe2006"), "Clowe 2006");
  assert.equal(sourceName("codata2018"), "CODATA 2018");
  assert.equal(sourceName("iau2012b2"), "IAU 2012 B2");
  assert.equal(sourceName("fudge-register"), "fudge-register");
});

test("units take superscript powers; a pure number has none", () => {
  assert.equal(formatUnit("m3 kg-1 s-2"), "m³ kg⁻¹ s⁻²");
  assert.equal(formatUnit("W m-2 K-4"), "W m⁻² K⁻⁴");
  assert.equal(formatUnit("1"), "");
  assert.equal(formatUnit("J s"), "J s");
});

test("sheet formulas evaluate with the sheet's values, and an unknown id gives null", () => {
  const v = new Map([["c", 3], ["yr_julian", 2], ["au", 10], ["a", 0.5]]);
  assert.equal(evalFormula("c yr_julian", v), 6);
  assert.equal(evalFormula("au 648000 / pi", v), (10 * 648000) / Math.PI);
  assert.equal(evalFormula("2 c^2 / (a yr_julian)", v), 18);
  assert.equal(evalFormula("1 - a - a", v), 0);
  assert.equal(evalFormula("1e6 yr_julian", v), 2e6);
  assert.equal(evalFormula("nope c", v), null);
  assert.equal(evalFormula("c +", v), null);
});

test("the Constants and Units pages read the kernel's sheet: groups, theoretical marks, factors", async () => {
  const k = nodeKernel();
  try {
    const pack = (n: string) => JSON.parse(readFileSync(new URL(`../../../packs/${n}/pack.json`, import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "ref", seed: "ref" }, [pack("base-physics"), pack("base-reference")]);
    const r = await readConstantsPage(s);
    assert.ok(r.ok, JSON.stringify(r));
    const { groups, rows, units } = r.page;
    assert.deepEqual(groups.slice(0, 4).map((g) => g.id), ["nature", "convention", "factor", "cosmos"]);
    assert.ok(groups.every((g) => g.about), "every group says what it is");
    assert.ok((groups.find((g) => g.id === "cosmos")!.notes ?? []).length >= 4, "cosmos carries its notes");
    const marked = rows.filter((x) => x.mark);
    assert.ok(marked.length >= 4 && marked.every((x) => x.mark === "theoretical" && (x.mark_note ?? "").length > 20), marked.map((x) => x.id).join());
    assert.ok(marked.some((x) => x.id === "Omega_c") && marked.some((x) => x.id === "w0"));
    assert.equal(rowNote(rows.find((x) => x.id === "c")!), "exact");
    assert.equal(rowNote(rows.find((x) => x.id === "m_p")!), "CODATA 2018");
    assert.equal(rowNote(rows.find((x) => x.id === "ly")!), "worked out");
    const values = new Map(rows.map((x) => [x.id, x.value.v]));
    const pc = units.find((u) => u.id === "pc")!;
    const f = unitFactor(pc, values)!;
    assert.ok(Math.abs(f / 3.0857e16 - 1) < 1e-4, `parsec ${f}`);
    assert.equal(siUnits(units).get("length")!.id, "m");
    const [alpha, ag] = dimensionless(values);
    assert.match(alpha!.shown, /^1 \/ 137\.036$/);
    assert.ok(ag && ag.v > 5e-39 && ag.v < 6e-39, String(ag?.v));
  } finally {
    await k.close();
  }
});

test("the Elements page: 118 elements in table places, shares in the legend's colours", async () => {
  const { readElementsPage, tablePosition, originShares, originGradient, shareWords } = await import("../src/matter.ts");
  const k = nodeKernel();
  try {
    const pack = (n: string) => JSON.parse(readFileSync(new URL(`../../../packs/${n}/pack.json`, import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "el", seed: "el" }, [pack("base-physics"), pack("base-reference")]);
    const r = await readElementsPage(s);
    assert.ok(r.ok, JSON.stringify(r));
    const { elements, legend } = r.page;
    assert.equal(elements.length, 118);
    assert.deepEqual(legend.map((l) => l.id), ["big_bang", "cosmic_rays", "low_mass_stars", "massive_stars", "white_dwarfs", "neutron_star_mergers", "human_made"]);
    const places = new Set(elements.map((e) => String(tablePosition(e.z))));
    assert.equal(places.size, 118, "no two elements share a cell");
    assert.deepEqual(tablePosition(79), [6, 11]);
    assert.deepEqual(tablePosition(57), [9, 3]);
    const au = elements.find((e) => e.id === "au")!;
    const parts = originShares(au, legend);
    assert.equal(parts[0]!.kind.id, "neutron_star_mergers");
    assert.equal(shareWords(parts[0]!, true), "Merging neutron stars, about 94 parts in 100");
    assert.match(originGradient(parts), /--sse-origin-neutron-star-mergers.*--sse-origin-low-mass-stars/);
    for (const e of elements) assert.ok(e.origin_story?.plain, `${e.id} has a plain origin story`);
  } finally {
    await k.close();
  }
});
