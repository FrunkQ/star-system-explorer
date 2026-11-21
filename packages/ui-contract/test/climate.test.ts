import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { climateBelts, isExtrapolation, regimeWords } from "../src/climate.ts";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { sceneFromResolved } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");

test("the climate's bands gather into six 30° belts, north first, weighted by area", () => {
  const lat_deg = [-75, -45, -15, 15, 45, 75];
  const b = climateBelts({ lat_deg, t_k: [250, 270, 300, 300, 280, 240], precip_m_yr: [0.1, 0.9, 2, 2, 0.8, 0.1] });
  assert.deepEqual(b.map((x) => x.label), ["60–90° N", "30–60° N", "Equator to 30° N", "Equator to 30° S", "30–60° S", "60–90° S"]);
  assert.ok(Math.abs(b[0]!.t_k - 240) < 1e-9);
  assert.ok(Math.abs(b[5]!.t_k - 250) < 1e-9);
  // Two bands in one belt: the one nearer the equator covers more ground.
  const two = climateBelts({ lat_deg: [2.5, 27.5], t_k: [300, 280], precip_m_yr: [3, 0] });
  assert.equal(two.length, 1);
  assert.ok(two[0]!.t_k > 290 && two[0]!.t_k < 300, String(two[0]!.t_k));
});

test("regime and extrapolation words", () => {
  assert.equal(regimeWords("earth-like"), "Earth-like");
  assert.equal(regimeWords("airless"), "Airless");
  assert.equal(regimeWords("tidally-locked"), "Tidally locked");
  assert.ok(isExtrapolation("pressure 90 bar is outside the tested range (0.006 to 10 bar): an extrapolation"));
  assert.ok(!isExtrapolation("mean 288 K; equator 300 K"));
});

test("the GM reads a world's climate from a quick survey; a player cannot", async () => {
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
    const earth = await s.surveyClimate(sys, subjects.find((b) => b.label === "Earth")!.address);
    assert.ok(earth.ok, JSON.stringify(earth));
    assert.equal(earth.climate.regime, "earth-like");
    assert.equal(earth.climate.cells, 3);
    assert.equal(earth.climate.lat_deg.length, earth.climate.t_k.length);
    assert.ok(earth.climate.why.length > 0 && earth.climate.assumed.length > 0);
    const belts = climateBelts(earth.climate);
    assert.equal(belts.length, 6);
    assert.ok(belts[2]!.t_k > belts[0]!.t_k + 20, "the tropics are warmer than the pole");
    const moon = await s.surveyClimate(sys, subjects.find((b) => b.label === "Moon")!.address);
    assert.ok(moon.ok && moon.climate.regime === "airless" && moon.climate.precip_m_yr.every((p) => p === 0));
    const craft = subjects.find((b) => b.address.includes("craft"));
    if (craft) assert.equal((await s.surveyClimate(sys, craft.address)).ok, false);
    const p = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs).catch(() => null);
    if (p) assert.equal((await p.surveyClimate(sys, subjects[1]!.address)).ok, false);
  } finally {
    await k.close();
  }
});
