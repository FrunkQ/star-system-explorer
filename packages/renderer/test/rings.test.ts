// V10f: rings and belts in True.
import { test } from "node:test";
import assert from "node:assert/strict";
import { beltHaze, poleOf, RING_FALLBACK, ringProfile } from "../src/rings.ts";

test("a ring's cover follows its optical depth; gaps are clear; the narrowest ring speaks inside a broader one", () => {
  const p = ringProfile(
    [
      { name: "thick", inner_m: 100, outer_m: 200, tau: 2, albedo: 0.5 },
      { name: "gap", inner_m: 140, outer_m: 150, tau: 0.1, albedo: 0.5 },
      { name: "thin", inner_m: 300, outer_m: 400, tau: 0.05, albedo: 0.5 },
    ],
    [1, 1, 1],
    300,
  )!;
  assert.equal(p.inner_m, 100);
  assert.equal(p.outer_m, 400);
  assert.equal(p.assumed, false);
  const alpha = (r: number) => p.data[Math.floor(((r - 100) / 300) * 300) * 4 + 3]!;
  assert.equal(alpha(120), Math.round(255 * (1 - Math.exp(-2))));
  assert.equal(alpha(145), Math.round(255 * (1 - Math.exp(-0.1))), "the gap inside the broad ring");
  assert.equal(alpha(250), 0, "clear between rings");
  assert.ok(alpha(350) > 0 && alpha(350) < alpha(120));
});

test("a ring with nothing on record is drawn at the fallbacks and says so; no edges, no picture", () => {
  const p = ringProfile([{ name: "?", inner_m: 1, outer_m: 2, tau: null, albedo: null }], [0.5, 0.5, 0.5], 10)!;
  assert.equal(p.assumed, true);
  assert.equal(p.data[3], Math.round(255 * (1 - Math.exp(-RING_FALLBACK.tau))));
  assert.equal(ringProfile([{ name: "bad", inner_m: 2, outer_m: 1, tau: 1, albedo: 1 }], [1, 1, 1]), null);
});

test("the pole leans by the tilt; a belt's haze stays between its edges, thin, and the same every time", () => {
  const z = poleOf(0);
  assert.deepEqual(z, [0, 0, 1]);
  const t = poleOf(Math.PI / 2);
  assert.ok(Math.abs(t[0] - 1) < 1e-12 && Math.abs(t[2]) < 1e-12);
  const h = beltHaze(2, 3, 0.05, 500);
  assert.equal(h.length, 500);
  for (const [x, y, zz] of h) {
    const r = Math.hypot(x, y);
    assert.ok(r >= 2 - 1e-9 && r <= 3 + 1e-9);
    assert.ok(Math.abs(zz) < 0.5 * r, "a disc, not a shell");
  }
  assert.deepEqual(beltHaze(2, 3, 0.05, 500), h);
});

test("belt members scatter without a lattice and go round at the Kepler rate, the inner edge ahead", async () => {
  const { beltParticles, beltAt } = await import("../src/rings.ts");
  const AU = 1.495978707e11;
  const GM = 1.32712440018e20;
  const ps = beltParticles(2.1 * AU, 3.3 * AU, 0.08, 2000);
  assert.equal(ps.length, 2000);
  assert.deepEqual(beltParticles(2.1 * AU, 3.3 * AU, 0.08, 2000), ps, "the same members every time");
  for (const p of ps) assert.ok(p.a_m >= 2.1 * AU && p.a_m <= 3.3 * AU);
  // No lattice: neighbours in draw order are not neighbours in angle (a spiral would put them close).
  let close = 0;
  for (let i = 1; i < ps.length; i++) if (Math.abs(ps[i]!.phase - ps[i - 1]!.phase) < 0.05) close++;
  assert.ok(close < 60, `consecutive members spread at random (${close} close pairs)`);
  // One year on: a member at 2.1 AU has gone round 1/2.1^1.5 of a turn, one at 3.3 AU less.
  const year = 3.15576e7;
  const lin = (a: number) => a / AU;
  const inner = { a_m: 2.1 * AU, phase: 0, incl: 0, node: 0 };
  const outer = { a_m: 3.3 * AU, phase: 0, incl: 0, node: 0 };
  const [pi, po] = beltAt([inner, outer], year, GM, lin, 100);
  const turn = (p: [number, number, number]) => ((Math.atan2(p[1], p[0]) / (2 * Math.PI)) + 1) % 1;
  assert.ok(Math.abs(turn(pi!) - (1 / Math.pow(2.1, 1.5)) % 1) < 0.01, `inner ${turn(pi!)}`);
  assert.ok(Math.abs(turn(po!) - (1 / Math.pow(3.3, 1.5)) % 1) < 0.01, `outer ${turn(po!)}`);
});

test("a ring is lit as a layer of particles: brighter than a flat surface when the light is low, dim from its unlit face", async () => {
  const { ringLight } = await import("../src/rings.ts");
  const n = [0, 0, 1];
  const low = [1, 0, 0.2];
  const lit = ringLight(n, low, [0, 1, 1]);
  const mu0 = 0.2 / Math.hypot(1, 0.2);
  assert.ok(lit > mu0, `brighter than Lambert at low sun (${lit} > ${mu0})`);
  const behind = ringLight(n, low, [0, 1, -1]);
  assert.ok(behind < lit && behind > 0, "the unlit face is dimmer but not black");
  assert.ok(ringLight(n, [1, 0, 0], [0, 1, 1]) < 0.01, "edge-on to the light, dark");
});

// ── VP-R2: the ring's light per pixel (ringLook) ──
import { ringLook, RING_LOOK_SCALE, type RingLookData } from "../src/rings.ts";

const PHASE = { back_g: -0.3, forward_g: 0.7, surge_b0: 0.33, surge_width_deg: 0.1 };
/** Pixels of the given depths, all grey ice of reflectance 0.5, with the given dust share. */
function strip(taus: number[], dust = 0.01): RingLookData {
  return { tau: Float32Array.from(taus), rgb: Float32Array.from(taus.flatMap(() => [0.5, 0.5, 0.5])), dust: Float32Array.from(taus.map(() => dust)), phase: PHASE };
}
/** Linear brightness of pixel i as shown (premultiplied colour, times the scale the light puts back). */
function shown(px: Uint8Array | Uint8ClampedArray, i: number): number {
  const lin = (b: number) => { const v = b / 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); };
  return lin(px[i * 4]!) * RING_LOOK_SCALE;
}
const up: [number, number, number] = [0, 0, 1];
const dir = (elev_deg: number, az_deg: number): [number, number, number] => {
  const e = (elev_deg * Math.PI) / 180, a = (az_deg * Math.PI) / 180;
  return [Math.cos(e) * Math.cos(a), Math.cos(e) * Math.sin(a), Math.sin(e)];
};

test("from the lit side the densest band is brightest; from below it goes dark and a middling one glows", () => {
  const d = strip([0.05, 0.4, 2.5]);
  const lit = ringLook(d, up, dir(20, 0), dir(30, 180));
  assert.ok(shown(lit, 2) > shown(lit, 1) && shown(lit, 1) > shown(lit, 0));
  const below = ringLook(d, up, dir(20, 0), dir(-30, 180));
  // Sun at 20 degrees, eye at 30 below: transmitted light peaks near tau 0.41 (visual polish design touch 6).
  assert.ok(shown(below, 1) > shown(below, 2) * 2, `${shown(below, 1)} vs ${shown(below, 2)}`);
  assert.ok(shown(below, 1) > shown(below, 0));
});

test("the opposition surge brightens the ring by about a third at zero phase", () => {
  const d = strip([1.5]);
  const at0 = shown(ringLook(d, up, dir(20, 0), dir(20, 0)), 0);
  const at2 = shown(ringLook(d, up, dir(20, 0), dir(20, 4)), 0);
  assert.ok(at0 / at2 > 1.2 && at0 / at2 < 1.5, `${at0 / at2}`);
});

test("dusty rings flare when backlit and nearly vanish front-lit", () => {
  const dust = strip([1e-4], 1);
  const back = shown(ringLook(dust, up, dir(10, 0), dir(-10, 180 - 2)), 0); // looking toward the sun through the ring
  const front = shown(ringLook(dust, up, dir(10, 0), dir(10, 5)), 0);
  assert.ok(back > 10 * front, `${back} vs ${front}`);
});
