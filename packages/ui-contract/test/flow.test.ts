// RK13: the Flow view's layout, and a ship's power books drawn as the Power card.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { layoutFlow, flowRowColumns, powerFlowRows, powerColumn, powerRank, powerReadings, radiatorArea, radiatorNode, heatTemperatures, NOWHERE, JET, FUEL_NODE, type FlowRow, type PowerBooks, type RadiatorRule } from "../src/index.ts";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

const rows: FlowRow[] = [
  { from: "Sun", to: "Air", amount: 100, kind: "light" },
  { from: "Air", to: "Space", amount: 70, kind: "heat" },
  { from: "Air", to: "Ground", amount: 30, kind: "heat" },
  { from: "Ground", to: "Space", amount: 30, kind: "heat" },
  { from: "Core", to: "Ground", amount: 0.05, kind: "heat" },
];

test("columns go by longest path from the sources unless given", () => {
  const c = flowRowColumns(rows);
  assert.deepEqual([c.get("Sun"), c.get("Core"), c.get("Air"), c.get("Ground"), c.get("Space")], [0, 0, 1, 2, 3]);
  const given = flowRowColumns(rows, (id) => (id === "Space" ? 2 : undefined));
  assert.equal(given.get("Space"), 2);
});

test("bands are as wide as their amounts, tiny ones are hairlines, and nodes balance", () => {
  const l = layoutFlow(rows, { width: 800, height: 320 });
  const band = (f: string, t: string) => l.bands.find((b) => b.row.from === f && b.row.to === t)!;
  assert.ok(Math.abs(band("Air", "Space").width / band("Air", "Ground").width - 70 / 30) < 1e-9);
  assert.equal(band("Core", "Ground").hairline, true);
  assert.equal(band("Core", "Ground").width, 1);
  const air = l.nodes.find((n) => n.id === "Air")!;
  assert.equal(air.amount, 100);
  assert.ok(l.nodes.every((n) => n.y >= 0 && n.y + n.h <= 320 + 1e-9), "everything fits");
  // On a phone it runs top to bottom: columns step down the page.
  const v = layoutFlow(rows, { width: 360, height: 600, vertical: true });
  const sun = v.nodes.find((n) => n.id === "Sun")!;
  const space = v.nodes.find((n) => n.id === "Space")!;
  assert.ok(space.y > sun.y && sun.h === 10);
  // Nothing at all lays out as nothing.
  assert.deepEqual(layoutFlow([], { width: 100, height: 100 }).nodes, []);
});

const rule: RadiatorRule = JSON.parse(readFileSync(new URL("../../../packs/base-ships/pack.json", import.meta.url), "utf8")).provides.other.ship_design.radiator;

test("radiator area is the waste heat over both faces at the panel's emissivity", () => {
  // 1 MW at 600 K, both faces, emissivity 0.9: 1e6 / (2 × 0.9 × 5.670e-8 × 600⁴) ≈ 75.6 m².
  assert.ok(Math.abs(radiatorArea(1e6, 600, rule) - 75.6) < 0.1, String(radiatorArea(1e6, 600, rule)));
});

test("heat goes to the temperature its part states, else to the one the books leave over", () => {
  const books = { ledger: { heat_by_k: { "300": 5, "900": 7 } } } as unknown as PowerBooks;
  const t = heatTemperatures(books, { power: [{ id: "plant", reject_k: 900 }], loads: [{ name: "Quarters" }] });
  assert.equal(t("plant"), 900);
  assert.equal(t("Quarters"), 300);
  const lone = { ledger: { heat_by_k: { "450": 1 } } } as unknown as PowerBooks;
  assert.equal(heatTemperatures(lone, null)("anything"), 450);
  assert.equal(heatTemperatures(books, null)("anything"), undefined);
});

// Against the real kernel (RK9 `ship_power`).
const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
const hall = {
  name: "Hall tug", accel_g: 0.05, sustained: true, exhaust_km_s: 20, wet_t: 400, dry_t: 200,
  power: [{ id: "plant", kind: "fission-reactor", electric_w: 1.2e5, thermal_w: 3e5, reject_k: 900 }],
  loads: [{ name: "Quarters", electric_w: 2e4 }],
  engines: [{ engine: { name: "Hall thruster", thrust_n: 530, exhaust_km_s: 20, energy_from: "electric", electric_w: 1e5, reject_k: 600 } }],
};
const sunfarer = {
  name: "Sunfarer", accel_g: 1e-5, sustained: true, exhaust_km_s: 30.4, wet_t: 1.2, dry_t: 0.75,
  power: [{ id: "Solar arrays", kind: "solar-array", electric_w: 1e4, law: { kind: "sunlight", ref_flux_w_m2: 1361 } }],
  loads: [{ name: "Radio", electric_w: 1500 }],
  engines: [{ engine: { name: "NSTAR", thrust_n: 0.092, exhaust_km_s: 30.4, energy_from: "electric", electric_w: 2600, efficiency: 0.54, reject_k: 400 } }],
};

test("the Power card: three columns, broken books from Nowhere on the GM's seat only, and the readings", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "flow", seed: "power card" }, [base], k.scratch);
    const r = await s.shipPower({ ship: hall });
    assert.ok(r.ok, JSON.stringify(r));
    const p: PowerBooks = r.power;
    // The kernel names each heat row's temperature.
    assert.ok(p.ledger.flows.filter((f) => f.kind === "heat").every((f) => typeof f.reject_k === "number"));
    const gmRows = powerFlowRows(p, { gm: true, ship: hall });
    const col = powerColumn(gmRows);
    assert.equal(col("plant"), 0);
    assert.equal(col(NOWHERE), 0);
    assert.equal(col("Hall thruster"), 1);
    assert.equal(col("Quarters"), 1);
    assert.equal(col("Lost in plant"), 1);
    assert.equal(col(JET), 2);
    assert.equal(col(radiatorNode(900)), 2);
    assert.equal(col(radiatorNode(300)), 2);
    const dashed = gmRows.filter((x) => x.dashed);
    assert.equal(dashed.length, 1);
    assert.ok(Math.abs(dashed[0]!.amount - 5.2e6) < 2e4, String(dashed[0]!.amount));
    // No row passes through the bus: every draw comes straight from a source.
    assert.ok(gmRows.every((x) => x.from !== "electric" && x.to !== "electric"));
    // A plant's own heat is a part that uses power, on its way to its radiators.
    assert.ok(gmRows.some((x) => x.from === "plant" && x.to === "Lost in plant" && Math.abs(x.amount - 3e5) < 1));
    // Players see their own ship's books without the honesty mark.
    assert.ok(powerFlowRows(p, { gm: false }).every((x) => x.from !== NOWHERE));
    // Nowhere on top, the jet first among the ends, hottest radiators before cooler.
    const rank = powerRank(p);
    assert.ok(rank(NOWHERE) < rank("plant"));
    assert.ok(rank(JET) < rank(radiatorNode(900)) && rank(radiatorNode(900)) < rank(radiatorNode(300)));
    const read = powerReadings(p, rule);
    assert.deepEqual(read.map((x) => x.label), ["Thrust", "Power to spare", "Hot radiators", "Cool radiators", "How visible"]);
    assert.match(read[0]!.value, /530\.0 N/);
    const lay = layoutFlow(gmRows, { width: 800, height: 320, column: col, rank });
    assert.equal(lay.columns.length, 3);

    // A solar craft: the fuel's own energy is not in these books, and the flow thins further out.
    const near = await s.shipPower({ ship: sunfarer, flux_w_m2: 1361 });
    const far = await s.shipPower({ ship: sunfarer, flux_w_m2: 1361 / 9 });
    assert.ok(near.ok && far.ok);
    const made = (b: PowerBooks) => powerFlowRows(b, { gm: true }).filter((x) => x.from === "Solar arrays").reduce((a, x) => a + x.amount, 0);
    assert.ok(made(far.power) < made(near.power) / 2);
    assert.ok(powerFlowRows(near.power, { gm: true }).every((x) => x.from !== FUEL_NODE && !x.dashed));
    // Switching the radio off gives the engine more.
    const quiet = await s.shipPower({ ship: { ...sunfarer, loads: [{ name: "Radio", electric_w: 1500, off: true }] }, flux_w_m2: 1361 / 4 });
    const loud = await s.shipPower({ ship: sunfarer, flux_w_m2: 1361 / 4 });
    assert.ok(quiet.ok && loud.ok && quiet.power.ledger.thrust_n > loud.power.ledger.thrust_n);
  } finally {
    await k.close();
  }
});

test("the card's radiator sum is the named Equation sse.ship.radiator_area (RK3), checked on its worked example", () => {
  const f = JSON.parse(readFileSync(new URL("packs/base-physics/formulas/ship.json", root), "utf8")) as { atoms: { id: string; equation: { examples: Record<string, number>[] } }[] };
  const ex = f.atoms.find((a) => a.id === "sse.ship.radiator_area")!.equation.examples[0]!;
  const a = radiatorArea(ex.Q!, ex.T!, { emissivity: ex.e!, two_sided: ex.n === 2 });
  assert.ok(Math.abs(a / ex.A! - 1) < 1e-9, `${a} against ${ex.A}`);
});
