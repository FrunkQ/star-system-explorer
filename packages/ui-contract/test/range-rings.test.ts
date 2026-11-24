// RK12: effect at range as rings round the one who holds the weapon.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { rangeRings, rangeText, ringPoints, RANGE_RING_LAYER, HUD_OVERLAY_LAYERS, type RangeEffect } from "../src/index.ts";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

test("ring points circle the centre at the radius, in the orbital plane", () => {
  const pts = ringPoints([5, -3, 2], 10, 8);
  assert.equal(pts.length, 8);
  for (const [x, y, z] of pts) {
    assert.ok(Math.abs(Math.hypot(x - 5, y + 3) - 10) < 1e-9);
    assert.equal(z, 2);
  }
});

test("distances read in a reader's unit", () => {
  assert.equal(rangeText(850), "850 m");
  assert.equal(rangeText(12_300), "12.3 km");
  assert.equal(rangeText(4.2e9), "4.20 million km");
});

test("the range layer is a layer name of its own", () => {
  assert.equal(RANGE_RING_LAYER, "range");
  assert.ok(Array.isArray(HUD_OVERLAY_LAYERS));
});

const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));

test("a 20 MW laser against a typed armour figure: in range, marginal and even odds, nested", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "range", seed: "range rings" }, [base], k.scratch);
    const r = await s.effectAtRange([
      {
        weapon: { kind: "beam", beam_power_w: 2e7, wavelength_m: 1e-6, aperture_m: 1, efficiency: 0.3, reject_k: 600 },
        target: { skin: { takes_j_m2: 1e7, thickness_m: 0 }, radius_m: 10, accel_m_s2: 10 },
        distance_m: 1e5,
      },
    ]);
    assert.ok(r.ok, JSON.stringify(r));
    const e: RangeEffect = r.effects[0]!;
    assert.equal(e.reading, "in range");
    const rings = rangeRings(e, "Laser");
    const kinds = rings.map((g) => g.kind);
    assert.ok(kinds.includes("in range") && kinds.includes("marginal"), kinds.join());
    const inR = rings.find((g) => g.kind === "in range")!;
    const marg = rings.find((g) => g.kind === "marginal")!;
    assert.ok(inR.radius_m < marg.radius_m, "the marginal ring sits outside the in-range ring");
    assert.ok(inR.radius_m < e.rings.enough_m! && e.rings.enough_m! < marg.radius_m);
    assert.match(inR.words, /^Laser in range out to /);
    // A slug's energy does not fall with distance: only the odds draw its ring.
    const slug = await s.effectAtRange([{ weapon: { kind: "projectile", mass_kg: 1, muzzle_m_s: 3000, face_m2: 0.01 }, target: { skin: { takes_j_m2: 1e6, thickness_m: 0 }, radius_m: 10, accel_m_s2: 10 }, distance_m: 1e5 }]);
    assert.ok(slug.ok);
    assert.deepEqual(rangeRings(slug.effects[0]!).map((g) => g.kind), ["even odds"]);
  } finally {
    await k.close();
  }
});

test("the fixture of effects (for views to draw from until a pack carries weapons) is what the kernel says", async () => {
  const fx = JSON.parse(readFileSync(new URL("fixtures/range-effects.json", import.meta.url), "utf8")) as { asks: Parameters<Session["effectAtRange"]>[0]; effects: RangeEffect[] };
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "range-fx", seed: "range rings" }, [base], k.scratch);
    const r = await s.effectAtRange(fx.asks);
    assert.ok(r.ok);
    assert.deepEqual(r.effects.map((e) => e.rings), fx.effects.map((e) => e.rings));
    const rings = rangeRings(fx.effects[0]!);
    assert.ok(rings.some((g) => g.kind === "in range") && rings.some((g) => g.kind === "marginal"));
    assert.ok(rings.every((g, n) => n === 0 || g.radius_m >= rings[n - 1]!.radius_m), "innermost first");
  } finally {
    await k.close();
  }
});
