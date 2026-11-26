import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { booksClose, heatBooks, heatCaveats, heatColumn, heatMarksBy, heatWhen, heatWhy, surfaceArea, HEAT_NODES, type HeatEnvelope, type HeatHistory, type HeatStep } from "../src/heat-flow.ts";
import { layoutFlow } from "../src/flow.ts";
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
    const moon = await s.surveyHeat(sys, subjects.find((b) => b.label === "Moon")!.address);
    assert.ok(moon.ok);
    const p = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs).catch(() => null);
    if (p) assert.equal((await p.surveyHeat(sys, subjects[1]!.address)).ok, false);
  } finally {
    await k.close();
  }
});
