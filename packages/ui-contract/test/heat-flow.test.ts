import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { booksClose, ladderLengths, lifeLadder, lifePace, lifeWhy, heatBooks, heatCaveats, heatColumn, heatMarksBy, heatWhen, heatWhy, sunBooks, sunColumn, sunWhy, surfaceArea, HEAT_NODES, SUN_NODES, type HeatEnvelope, type HeatHistory, type HeatStep, type Sunlight } from "../src/heat-flow.ts";
import { layoutFlow } from "../src/flow.ts";
import type { LifeEnergy, LifeEnergyLines } from "../src/life.ts";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { sceneFromResolved } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");

const R = 6.371e6;
const env = (core: HeatHistory | null, over: Partial<HeatEnvelope> = {}): HeatEnvelope => ({ label: "World", kind: "solid", regime_now: "mobile-lid", regime_label: "Moving plates", reference_radius_m: R, tidal_w_m2: 0, core, ...over });
const today = (t: number, lost: number, core: number): HeatStep => ({ t_gyr: t, t_mantle_k: 1700, t_cmb_k: 4000, q_core_tw: core, q_surface_tw: lost, inner_core_km: 0 });

// Section 7's books at one step, W/m², closing exactly: lost = made + mantle + lid + core.
const closed = (o: Partial<HeatStep> = {}): HeatStep => ({
  ...today(4.5, 0, 0),
  heat_radiogenic: 0.04,
  heat_tidal: 0.001,
  heat_made: 0.041,
  mantle_cooling: 0.02,
  lid_cooling: 0,
  core_heat: 0.015,
  core_freezing: 0.003,
  core_cooling: 0.012,
  heat_lost: 0.076,
  heat_conducted: 0.07,
  heat_erupted: 0.006,
  ...o,
});

test("today's core history draws only what it publishes and says what is missing", () => {
  const h: HeatHistory = { version: "x", core_radius_km: 3480, steps: [today(0.1, 200, 30), today(4.5, 35, 9)], events: [], dynamo_now: true, why_now: "A dynamo runs." };
  const b = heatBooks(env(h), h.steps[1]!);
  assert.equal(b.closes, false);
  assert.deepEqual(b.rows.map((r) => [r.from, r.to, r.amount]), [["Core", "World", 9e12], ["World", HEAT_NODES.surface, 35e12]]);
  assert.match(b.missing, /HF4b\.1/);
  const why = heatWhy(env(h), h, 1);
  assert.match(why[0]!, /loses about 35 TW|loses 35 TW/);
  assert.ok(why.some((l) => /running down as it ages/.test(l)), why.join(" | "));
  assert.equal(heatWhy(env(h), h, 1, "curious").length, 2);
  assert.equal(heatWhen(h, 1).ago, "now");
  assert.match(heatWhen(h, 0).ago, /ago$/);
});

test("section 7's books close: every band is a published figure times the area, and the world balances", () => {
  const s = closed();
  assert.ok(booksClose(s));
  const e = env(null);
  const b = heatBooks(e, s);
  const a = surfaceArea(e);
  assert.equal(b.closes, true);
  assert.equal(b.lost_w, 0.076 * a);
  const into = b.rows.filter((r) => r.to === "World").reduce((x, r) => x + r.amount, 0);
  const out = b.rows.filter((r) => r.from === "World").reduce((x, r) => x + r.amount, 0);
  // Absolute, not a ratio: the published books close to rounding in watts.
  assert.ok(Math.abs(into - out) < 1, `${into} in, ${out} out`);
  assert.ok(b.rows.some((r) => r.from === HEAT_NODES.freezing && r.to === HEAT_NODES.core));
  // A store that warms takes heat in, drawn as where some heat goes.
  const warming = heatBooks(e, closed({ mantle_cooling: -0.01, heat_lost: 0.046, heat_conducted: 0.04 }));
  assert.ok(warming.rows.some((r) => r.from === "World" && r.to === HEAT_NODES.warmsMantle && Math.abs(r.amount - 0.01 * a) < 1));
  assert.ok(!warming.rows.some((r) => r.from === HEAT_NODES.mantle));
  // It lays out in four columns with nothing below zero.
  const L = layoutFlow(b.rows, { width: 640, height: 320, column: heatColumn(e) });
  assert.equal(L.columns.length, 4);
  assert.ok(L.bands.every((x) => x.width > 0));
  // A missing field means the books do not close.
  const { heat_erupted: _, ...partial } = s;
  assert.equal(booksClose(partial), false);
});

// Earth-like sunlight books, W/m² (the climate's HV2a figures): arriving = reflected + absorbed, emitted = absorbed + internal.
const sunny = (o: Partial<Sunlight> = {}): Sunlight => ({ arriving_w_m2: 340, absorbed_w_m2: 238, reflected_w_m2: 102, bond_albedo: 0.3, internal_w_m2: 0.087, emitted_w_m2: 238.087, ...o });

test("the outside half draws the climate's sunlight books, closing, at their own scale", () => {
  const e = env(null, { label: "Earth" });
  const b = sunBooks(e, sunny());
  const a = surfaceArea(e);
  assert.equal(b.arriving_w, 340 * a);
  const into = b.rows.filter((r) => r.to === SUN_NODES.skin).reduce((x, r) => x + r.amount, 0);
  const out = b.rows.filter((r) => r.from === SUN_NODES.skin).reduce((x, r) => x + r.amount, 0);
  // Absolute, not a ratio: what reaches the surface and air leaves it, to a kilowatt in 10^17 W (float rounding).
  assert.ok(Math.abs(into - out) < 1e3, `${into} in, ${out} out`);
  assert.deepEqual(b.rows.map((r) => sunColumn(r.from)), [0, 0, 1, 1]);
  const L = layoutFlow(b.rows, { width: 640, height: 240, column: sunColumn });
  assert.equal(L.columns.length, 3);
  const why = sunWhy(e, sunny());
  assert.match(why[0]!, /reflects 30% straight back/);
  assert.match(why[1]!, /sunlight, not its inside, sets how warm/);
  assert.equal(sunWhy(e, sunny(), "curious").length, 2);
  // A climate that leaves the inside heat out still compares with the core history's heat lost, and says so.
  const left = sunWhy(e, sunny({ internal_w_m2: 0, emitted_w_m2: 238 }), "science", 35e12);
  assert.match(left[1]!, /times the heat coming up from inside \((about )?35 TW\)/);
  assert.ok(left.some((l) => /climate leaves out the heat from inside/.test(l)), left.join(" | "));
  // A world warmed from inside more than by its sun says so.
  assert.match(sunWhy(e, sunny({ absorbed_w_m2: 0.01, reflected_w_m2: 0.01, arriving_w_m2: 0.02, internal_w_m2: 2, emitted_w_m2: 2.01 }))[1]!, /its own heat, not its sun/);
  // No sunlight at all: one honest line.
  assert.match(sunWhy(e, sunny({ arriving_w_m2: 0, absorbed_w_m2: 0, reflected_w_m2: 0 }))[0]!, /No sunlight reaches/);
});

const lines = (o: Partial<LifeEnergyLines> = {}): LifeEnergyLines => ({ arriving_w: 4e15, offered_w: 1e15, captured_w: 1e14, keep_alive_w: 2e13, surplus_w: 8e13, brought_in_w: 0, ...o });
const ledger = (o: Partial<LifeEnergyLines> = {}, barely = false): LifeEnergy => ({ habitats: [{ habitat: "h", kind: "ocean-sunlit", ...lines(o), light_w: 4e15, chemical_w: 0, exergy_factor: 0.93, barely, closed: null }], world: lines(o), unknown: [], absorbed_sunlight_w: null, notes: [] });

test("life's share: the kernel's ledger as a ladder under the sunlight, and the pace of evolution from its clock", () => {
  const rungs = lifeLadder(ledger(), 1.2e17);
  assert.deepEqual(rungs.map((r) => r.id), ["absorbed", "arriving", "offered", "captured", "keep_alive", "surplus"]);
  const L = ladderLengths(rungs);
  assert.equal(L[0], 1);
  assert.ok(L.every((x, i) => i === 0 || x < L[i - 1]! || rungs[i]!.w >= rungs[i - 1]!.w), L.join(","));
  const why = lifeWhy(ledger(), 1.2e17, 1);
  assert.match(why[0]!, /Life catches .*1 part in 1,200 of the sunlight/);
  assert.match(why[1]!, /costs .*20 TW; where life pays its keep it has .*80 TW spare for growing and spreading/);
  assert.match(why[2]!, /about Earth's pace/);
  assert.match(lifeWhy(ledger({ captured_w: 1e6, keep_alive_w: 2e8, surplus_w: -2e8 }, true), 1e15, null)[1]!, /alive, barely/);
  assert.match(lifeWhy(ledger({ captured_w: 1e6, keep_alive_w: 2e8, surplus_w: -2e8 }), 1e15, null)[1]!, /life here is doubtful/);
  assert.match(lifeWhy(ledger({ captured_w: 0, surplus_w: 0 }), 1e15, null)[0]!, /No life catches any/);
  assert.equal(lifeWhy(ledger(), 1e17, 3, "curious").length, 2);
  // The pace is Earth-Myr of opportunity gathered per Myr, over the last two points.
  const pt = (at: number, opp: number) => ({ at_myr: at, alive: true, capabilities: [], diversity: 1, opportunity: opp, production_gc_yr: 1 });
  assert.equal(lifePace({ timeline: [pt(0, 0), pt(100, 30), pt(200, 330)] }), 3);
  assert.equal(lifePace({ timeline: [pt(0, 0), { ...pt(10, 5), alive: false }] }), null);
});

test("caveats follow the world's conditions, never its name", () => {
  const lid = heatCaveats(env(null, { regime_now: "stagnant-lid" }), today(4.5, 1, 0.1)).map((c) => c.id);
  assert.ok(lid.includes("lid-hot"));
  const plates = heatCaveats(env(null, { label: "Mars" }), today(4.5, 1, 0.1)).map((c) => c.id);
  assert.ok(!plates.includes("lid-hot"));
  assert.ok(heatCaveats(env(null, { tidal_w_m2: 2 }), today(1, 1, 1)).some((c) => c.id === "tides-one-response"));
});

test("the GM reads a world's heat history from the kernel; Earth runs down as it ages; a player cannot", async () => {
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const a = await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") });
    assert.ok(a.ok);
    const sys = a.result.systems[0]!;
    const r = await s.resolvedSystem(sys);
    assert.ok(r.ok);
    const subjects = sceneFromResolved(r.result).subjects;
    const earth = await s.surveyHeat(sys, subjects.find((b) => b.label === "Earth")!.address);
    assert.ok(earth.ok, JSON.stringify(earth));
    const h = earth.envelope.core!;
    assert.ok(h && h.steps.length > 10, "Earth has a core history");
    const first = heatBooks(earth.envelope, h.steps[0]!);
    const now = heatBooks(earth.envelope, h.steps[h.steps.length - 1]!);
    assert.ok(now.lost_w > 1e12 && now.lost_w < 1e14, `Earth loses ${now.lost_w} W now`);
    assert.ok(first.lost_w > now.lost_w, "young Earth lost more heat than today");
    assert.ok(now.rows.length >= 2);
    assert.ok(heatWhy(earth.envelope, h, h.steps.length - 1).length >= 3);
    assert.ok(heatMarksBy(h, h.steps.length - 1).length >= heatMarksBy(h, 0).length);
    // The outside half comes from the same survey: Earth soaks up thousands of times its inside heat.
    assert.ok(earth.sunlight, "Earth's survey carries the climate's sunlight books");
    const sb = sunBooks(earth.envelope, earth.sunlight);
    assert.ok(sb.albedo > 0.2 && sb.albedo < 0.4, `Earth's albedo ${sb.albedo}`);
    assert.ok(sb.absorbed_w > 1e3 * now.lost_w, `${sb.absorbed_w} W absorbed against ${now.lost_w} W from inside`);
    // Life's share from the kernel's ledger (LE2): Earth's life catches energy and pays its keep.
    const life = await s.lifeView({ body: subjects.find((b) => b.label === "Earth")!.address });
    assert.ok(life.ok && life.view.energy, JSON.stringify(life).slice(0, 300));
    const cap = life.view.energy.world.captured_w ?? 0;
    assert.ok(cap > 0 && cap < sb.absorbed_w, `Earth's life catches ${cap} W`);
    const moon = await s.surveyHeat(sys, subjects.find((b) => b.label === "Moon")!.address);
    assert.ok(moon.ok);
    const p = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs).catch(() => null);
    if (p) assert.equal((await p.surveyHeat(sys, subjects[1]!.address)).ok, false);
  } finally {
    await k.close();
  }
});
