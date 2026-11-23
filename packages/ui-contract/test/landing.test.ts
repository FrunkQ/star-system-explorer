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
import { landmarkOrder, suggestLander } from "../src/landing.ts";
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
  } finally {
    await k.close();
  }
});

function near(a: number, b: number, tol: number) {
  assert.ok(Math.abs(a - b) <= tol, `${a} vs ${b}`);
}
