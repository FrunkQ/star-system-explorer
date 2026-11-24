// SL1 in Node: the sample Sol's Earth and Moon, adopted, give landmarks from their named landscape, a descent
// planned to any site (a sea site lands with its danger shown, a capsule on the airless Moon is shown failing, not
// refused), and the ground round a site at the kernel's fact floor (finer is refused, and the refusal names the
// floor). Players get none of it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { sceneFromResolved } from "../src/make.ts";
import { landingSeaOf, landingShowOf, landmarkOrder, roundPose, suggestLander } from "../src/landing.ts";
import { nodeKernel } from "./node-kernel.ts";
import { floorFromRefusal, levelFor, patchKeys, readPatch, groundHeightAt } from "../../renderer/src/index.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];

test("the lander a world's air suggests", () => {
  assert.equal(suggestLander(0, 250), "moon-lander");
  assert.equal(suggestLander(0.006, 210), "mars-lander");
  assert.equal(suggestLander(1.5, 94), "titan-probe");
  assert.equal(suggestLander(1.013, 288), "capsule");
});

test("landmarks, descents and the ground round a site on the sample Earth and Moon", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const a = await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") });
    assert.ok(a.ok, JSON.stringify(a));
    const sys = a.result.systems[0]!;
    const r = await s.resolvedSystem(sys);
    assert.ok(r.ok);
    const subs = sceneFromResolved(r.result).subjects;
    const earth = subs.find((b) => b.label === "Earth")!;
    const moon = subs.find((b) => b.label === "Moon")!;

    const lm = await s.surfaceLandmarks(sys, earth.address);
    assert.ok(lm.ok, JSON.stringify(lm).slice(0, 300));
    assert.ok(lm.landmarks.length >= 3 && lm.total >= lm.landmarks.length);
    const ordered = landmarkOrder(lm.landmarks);
    assert.ok(ordered[0]!.remarkable.length > 0, "the remarkable come first");
    assert.ok(ordered.every((l) => Number.isFinite(l.lat_deg) && Number.isFinite(l.lon_deg) && l.original_name));

    // A sea site: the lander gets down, with the water shown as a danger, never refused.
    const ocean = lm.landmarks.find((l) => l.kind === "ocean")!;
    const sea = await s.planDescent({ system: sys, body: earth.address, lat_deg: ocean.lat_deg, lon_deg: ocean.lon_deg, lander: "capsule" });
    assert.ok(sea.ok, JSON.stringify(sea).slice(0, 300));
    assert.equal(sea.plan.verdict, "landed-on-a-bad-site");
    assert.ok(sea.plan.hazards.some((h) => h.kind === "liquid" && h.level === "danger"));
    assert.ok(sea.plan.descent.telemetry.length > 10 && sea.plan.descent.events.some((e) => e.kind === "touchdown"));

    // A capsule on the airless Moon cannot stop: shown, with why.
    const mc = await s.planDescent({ system: sys, body: moon.address, lat_deg: 20, lon_deg: 30, lander: "capsule" });
    assert.ok(mc.ok);
    assert.equal(mc.plan.verdict, "still-descending");
    assert.ok(mc.plan.descent.outcome.needs_engines);

    // The ground round a site at the fact floor: Earth's level 9 at 76 m, finer refused naming the floor.
    const t0 = await s.surfaceTiles(sys, earth.address);
    assert.ok(t0.ok);
    const L = levelFor(t0.tiles[0]!.spacing_m, 100);
    assert.equal(L, 9);
    const keys = patchKeys(20, 30, 6000, L, 6.371e6);
    assert.ok(keys.length >= 1 && keys.length <= 4, `${keys.length} tiles`);
    const got = await s.surfaceTilesAt(sys, earth.address, keys);
    assert.ok(got.ok && got.tiles.length === keys.length, JSON.stringify(got).slice(0, 300));
    near(got.tiles[0]!.spacing_m, 76.27, 0.05);
    const p = readPatch(got.tiles, 20, 30, 6.371e6, 6000, 158);
    assert.equal(p.missing, 0);
    const sp = await s.planDescent({ system: sys, body: earth.address, lat_deg: 20, lon_deg: 30, lander: "capsule" });
    assert.ok(sp.ok);
    // The patch's floor under the site and the kernel's judged site height agree to within the site's relief.
    const floorAt = p.floor_m[Math.floor(p.n / 2) * p.n + Math.floor(p.n / 2)]!;
    assert.ok(Math.abs(floorAt - sp.plan.site.height_m) < Math.max(200, sp.plan.site.relief_m), `${floorAt} vs ${sp.plan.site.height_m}`);
    assert.ok(Number.isFinite(groundHeightAt(p, 123, -456)));
    const finer = await s.surfaceTilesAt(sys, earth.address, [{ ...keys[0]!, level: L + 2, x: keys[0]!.x * 4, y: keys[0]!.y * 4 }]);
    assert.ok(finer.ok && finer.tiles.length === 0);
    assert.equal(floorFromRefusal(finer.refused[0]!.why), L);

    const player = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs);
    assert.equal((await player.planDescent({ system: sys, body: earth.address, lat_deg: 0, lon_deg: 0, lander: "capsule" })).ok, false);
    assert.equal((await player.surfaceLandmarks(sys, earth.address)).ok, false);
    assert.equal((await player.surfaceTilesAt(sys, earth.address, keys)).ok, false);
    // While the table watches a landing on Earth, the seat may ask its own kernel for that world's ground, and no
    // other (this test's seat has its own universe with nothing adopted, so the kernel, not the gate, answers).
    const asked = (r: { ok: boolean; code?: string }) => (r.ok ? "ok" : r.code);
    player.grantLanding({ system: sys, body: earth.address });
    assert.notEqual(asked(await player.surfaceTilesAt(sys, earth.address, keys)), "not_gm", "the watched world's ground");
    assert.notEqual(asked(await player.planDescent({ system: sys, body: earth.address, lat_deg: 0, lon_deg: 0, lander: "capsule" })), "not_gm", "its descent");
    assert.equal(asked(await player.surfaceLandmarks(sys, earth.address)), "not_gm", "never its landmarks");
    assert.equal(asked(await player.surfaceTiles(sys, moon.address)), "not_gm", "never another world");
    player.grantLanding(null);
    assert.equal(asked(await player.surfaceTilesAt(sys, earth.address, keys)), "not_gm", "the grant ends with the landing");
  } finally {
    await k.close();
  }
});

function near(a: number, b: number, tol: number) {
  assert.ok(Math.abs(a - b) <= tol, `${a} vs ${b}`);
}

test("a landing sent to the table is read back only when it is one", () => {
  const world = { radius_m: 6.371e6, pressure_bar: 1, t_k: 288, air: null, star_k: 5772, sun_radius: 0.0047, tilt_deg: 23.4 };
  const show = { system: "u/sys:sol", body: "u/sys:sol/planet:3", label: "Earth", site: { lat_deg: 10.5, lon_deg: 75.97, label: "" }, lander: "capsule", phase: "flying", flight_s: 24, fly_u: 0.4, hour_deg: -30, world };
  assert.deepEqual(landingShowOf(JSON.parse(JSON.stringify(show))), show);
  assert.ok(landingShowOf({ ...show, lander: { ship: { id: "x" } } }), "a ship as the lander");
  assert.equal(landingShowOf(null), null);
  assert.equal(landingShowOf({ ...show, phase: "walking" }), null, "an unknown moment");
  assert.equal(landingShowOf({ ...show, site: { lat_deg: "10" } }), null, "a site without numbers");
  assert.equal(landingShowOf({ ...show, world: { ...world, radius_m: 0 } }), null, "a world without a size");
});

test("the GM's pose is rounded for the wire", () => {
  assert.deepEqual(roundPose({ x_m: 1.23456, y_m: -0.004, yaw: 1.5707963, pitch: -0.05, drone: true }), { x_m: 1.23, y_m: -0, yaw: 1.571, pitch: -0.05, drone: true });
});

test("the sea at a site from the weather report, with the tide now", () => {
  const data = { sea: { liquid: "water", wave_m: 1.8, period_s: 7.5, wind_m_s: -6, fetch_km: 900, line: "Waves about 1.8 m", tides: { by: [{ raiser: "moon 1", range_m: 0.54, period_h: 12.42 }, { raiser: "star", range_m: 0.25, period_h: 12 }] } } };
  const s = landingSeaOf(data, 0)!;
  assert.equal(s.tide_m, 0.4, "both high together at time 0");
  assert.equal(s.wind_m_s, -6);
  assert.equal(landingSeaOf(data, 6.21 * 3600)!.tide_m < 0, true, "half a lunar tide on: lower");
  assert.equal(landingSeaOf({ sea: null }, 0), null);
  assert.equal(landingSeaOf({}, 0), null);
});
