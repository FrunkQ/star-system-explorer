import { test } from "node:test";
import assert from "node:assert/strict";
import { profileShare, checkLayer, detailLine, discScale, LADDER_LAYERS, ladderAt, ladderLayer, ladderSeed, MAX_RUNGS, MAX_STRETCH, resolveLayer, rungOrigin, rungs, sizeWords, TIER_RUNGS, type LadderLayer, type ResolvedLayer } from "../src/ladder.ts";
import { cell, curl, NUM, WRAP, simplex, type V3 } from "../src/ladder-noise.ts";

// A Jupiter-like giant's curl figures (round numbers, for the mechanism only).
const JUPITER = { rhines_scale_m: 1.0e7, deformation_radius_m: 2.5e6, jet_shear_m_s: 60, jet_speed_m_s: 120, eddy_speed_m_s: 30 };
const R_JUP = 6.9911e7;
const curls = () => resolveLayer(ladderLayer("giant.curls")!, JUPITER) as ResolvedLayer;

let seedR = 7;
const rnd = () => ((seedR = (seedR * 1103515245 + 12345) % 2147483648), seedR / 2147483648);

test("every starter declaration is sound and binds its sizes, amplitude and speed to kernel values", () => {
  assert.ok(LADDER_LAYERS.length >= 6);
  for (const l of LADDER_LAYERS) assert.deepEqual(checkLayer(l), [], l.id);
  assert.deepEqual(checkLayer({ ...LADDER_LAYERS[0]!, amplitude: 3 as unknown as LadderLayer["amplitude"] }), ["amplitude must be a kernel value"]);
});

test("a missing kernel value means nothing is drawn, and the HUD says which value", () => {
  const r = resolveLayer(ladderLayer("giant.curls")!, { rhines_scale_m: 1e7, jet_speed_m_s: 120, eddy_speed_m_s: 30 });
  assert.equal(r.drawn, false);
  if (!r.drawn) assert.match(r.why, /no jet_shear_m_s/);
  // A ratio whose divisor is missing names the divisor.
  const s = resolveLayer(ladderLayer("giant.curls")!, { ...JUPITER, deformation_radius_m: undefined });
  assert.equal(s.drawn, false);
  if (!s.drawn) assert.match(s.why, /deformation_radius_m/);
  assert.match(detailLine([{ layer: r }]), /not drawn/);
});

test("bindings read the facts: ratios, factors and the stretch cap", () => {
  const r = curls();
  assert.equal(r.largest_m, 1e7);
  assert.equal(r.amplitude, 0.5);
  assert.equal(r.stretch, 4);
  assert.equal(r.speed_m_s, 30);
  const ring = resolveLayer(ladderLayer("ring.fine")!, { profile_spacing_m: 100, ring_radius_m: 1e8, opacity_contrast: 0.2, kepler_shear_m_s: 1 }) as ResolvedLayer;
  assert.equal(ring.stretch, MAX_STRETCH);
});

test("a distant giant draws no rungs; rungs arrive as it grows on screen, finest last", () => {
  const r = curls();
  const seed = ladderSeed("sol/jupiter");
  // 10 px across: a 10,000 km curl is under 1 px.
  assert.equal(rungs(r, discScale(R_JUP, 10, 1000), "gpu-high", 0, seed).length, 0);
  // 2,000 px across: 35 km a pixel, so curls down to 70 km show; the tier allows four.
  const near = rungs(r, discScale(R_JUP, 2000, 1000), "gpu-high", 0, seed);
  assert.ok(near.length >= 4);
  const total = near.reduce((s, x) => s + x.weight, 0);
  assert.ok(total <= TIER_RUNGS["gpu-high"] + 1e-9);
  for (let i = 1; i < near.length; i++) assert.ok(near[i]!.size_m < near[i - 1]!.size_m);
  for (const x of near) assert.ok(x.size_m / discScale(R_JUP, 2000, 1000).metres_per_px >= 2);
});

test("the tier caps the drawn rungs: basic one, high four, ultra six, at rest all the shader holds", () => {
  const r = curls();
  const view = { metres_per_px: 10, span_m: 1e9 };
  const sum = (t: Parameters<typeof rungs>[2]) => rungs(r, view, t, 0, 1).reduce((s, x) => s + x.weight, 0);
  assert.equal(sum("gpu-basic"), 1);
  assert.equal(sum("gpu-high"), 4);
  assert.equal(sum("gpu-ultra"), 6);
  assert.equal(sum("still"), MAX_RUNGS);
  assert.equal(sum("text"), 0);
});

test("weights change smoothly as the view closes in: no rung pops in or out", () => {
  const r = curls();
  let last: number[] | null = null;
  for (let k = 0; k <= 4000; k++) {
    const px = 50 * Math.pow(10, k / 1000); // 50 px to 500,000 px across
    const rs = rungs(r, discScale(R_JUP, px, 1200), "gpu-high", 0, 3);
    const w = new Array(30).fill(0);
    for (const x of rs) w[x.index] = x.weight;
    if (last) for (let i = 0; i < 30; i++) assert.ok(Math.abs(w[i]! - last[i]!) < 0.02, `rung ${i} jumped at ${px.toFixed(0)} px`);
    last = w;
  }
});

test("coarse rungs fade out once larger than the view, so the cost per pixel stays about the same", () => {
  const r = curls();
  const rs = rungs(r, { metres_per_px: 1, span_m: 2000 }, "gpu-ultra", 0, 3);
  for (const x of rs) assert.ok(x.size_m < 4000);
  assert.ok(rs.length > 0);
});

test("seed and clock: the same body, time and view always give the same rungs; another body differs", () => {
  const r = curls();
  const v = discScale(R_JUP, 3000, 1000);
  const a = rungs(r, v, "gpu-high", 1e9, ladderSeed("sol/jupiter"));
  const b = rungs(r, v, "gpu-high", 1e9, ladderSeed("sol/jupiter"));
  const c = rungs(r, v, "gpu-high", 1e9, ladderSeed("sol/saturn"));
  assert.deepEqual(a, b);
  assert.notDeepEqual(a.map((x) => x.origin), c.map((x) => x.origin));
  // Time turns the gradients; finer rungs turn faster (Kolmogorov: turnover ∝ size^(2/3)).
  const t0 = rungs(r, v, "gpu-high", 0, 1);
  const t1 = rungs(r, v, "gpu-high", 3600, 1);
  const turn = (i: number) => ((t1[i]!.alpha - t0[i]!.alpha + 2 * Math.PI) % (2 * Math.PI));
  assert.ok(turn(3) > turn(0));
  // Still layers never move.
  const ground = resolveLayer(ladderLayer("ground.roughness")!, { fact_floor_m: 76, grain_m: 0.01, roughness_m: 2 }) as ResolvedLayer;
  assert.ok(rungs(ground, { metres_per_px: 0.05, span_m: 100 }, "gpu-high", 1e12, 1).every((x) => x.alpha === 0 && x.phase === 0));
});

test("ground roughness starts at the fact floor and stops at the grain", () => {
  const ground = resolveLayer(ladderLayer("ground.roughness")!, { fact_floor_m: 76, grain_m: 1, roughness_m: 2 }) as ResolvedLayer;
  const rs = rungs(ground, { metres_per_px: 0.001, span_m: 1e4 }, "still", 0, 1);
  assert.equal(rs[0]!.size_m, 76);
  assert.ok(rs.every((x) => x.size_m >= 1));
  // Amplitude falls by 2^−H a halving.
  assert.ok(Math.abs(rs[1]!.amplitude / rs[0]!.amplitude - Math.pow(2, -0.8)) < 1e-12);
});

test("anchors: moving the anchor and the local point together leaves the detail where it was on the ground", () => {
  const ground = resolveLayer(ladderLayer("ground.roughness")!, { fact_floor_m: 76, grain_m: 0.01, roughness_m: 2 }) as ResolvedLayer;
  const view = { metres_per_px: 0.02, span_m: 50 };
  const far: [number, number, number] = [6.371e6, 1234.5, -987.25];
  for (let k = 0; k < 20; k++) {
    const p: V3<number> = [rnd() * 40 - 20, rnd() * 40 - 20, rnd() * 4];
    const a = ladderAt("ridged", rungs(ground, view, "gpu-high", 0, 9, far), p) as number;
    const shifted: [number, number, number] = [far[0] + 13, far[1] - 7, far[2] + 2];
    const b = ladderAt("ridged", rungs(ground, view, "gpu-high", 0, 9, shifted), [p[0] - 13, p[1] + 7, p[2] - 2]) as number;
    assert.ok(Math.abs(a - b) < 1e-6, `${a} vs ${b}`);
  }
  // The origin is kept inside the noise's period, so the shader only ever sees small numbers.
  for (const c of rungOrigin(1, "x", 0, 0.01, far)) assert.ok(c >= 0 && c < WRAP);
});

test("simplex: periodic over 867 cells along each axis, its gradient is the true derivative, and turning it changes the pattern", () => {
  for (let k = 0; k < 50; k++) {
    const q: V3<number> = [rnd() * 50, rnd() * 50, rnd() * 50];
    const a = rnd() * 6;
    assert.ok(Math.abs(simplex(NUM, q, a).value - simplex(NUM, [q[0] + WRAP, q[1] - WRAP, q[2] + 2 * WRAP], a).value) < 1e-9);
    const s = simplex(NUM, q, a);
    const e = 1e-6;
    for (let d = 0; d < 3; d++) {
      const p = [...q] as V3<number>;
      p[d] = p[d]! + e;
      assert.ok(Math.abs((simplex(NUM, p, a).value - s.value) / e - s.grad[d]!) < 1e-3);
    }
  }
  const q: V3<number> = [10.3, 4.4, 7.1];
  assert.notEqual(simplex(NUM, q, 0).value, simplex(NUM, q, 1).value);
  let mx = 0;
  for (let k = 0; k < 20000; k++) mx = Math.max(mx, Math.abs(simplex(NUM, [rnd() * 289, rnd() * 289, rnd() * 289], rnd() * 6).value));
  assert.ok(mx <= 1 && mx > 0.6, `max ${mx}`);
});

test("curl lies along the surface and has no divergence along it", () => {
  // On a plane with normal z, the curl is (∂ψ/∂y, −∂ψ/∂x, 0): divergence ∂²ψ/∂x∂y − ∂²ψ/∂y∂x = 0.
  const n: V3<number> = [0, 0, 1];
  for (let k = 0; k < 30; k++) {
    const q: V3<number> = [rnd() * 20, rnd() * 20, rnd() * 20];
    const v = curl(NUM, q, 0.4, n);
    assert.ok(Math.abs(v[2]) < 1e-12);
    const h = 1e-4;
    const dx = (curl(NUM, [q[0] + h, q[1], q[2]], 0.4, n)[0] - curl(NUM, [q[0] - h, q[1], q[2]], 0.4, n)[0]) / (2 * h);
    const dy = (curl(NUM, [q[0], q[1] + h, q[2]], 0.4, n)[1] - curl(NUM, [q[0], q[1] - h, q[2]], 0.4, n)[1]) / (2 * h);
    assert.ok(Math.abs(dx + dy) < 1e-3, `div ${dx + dy}`);
  }
});

test("cells: F1 is under F2, both continuous, and the mean spacing is one cell", () => {
  let mean = 0;
  for (let k = 0; k < 3000; k++) {
    const q: V3<number> = [rnd() * 30, rnd() * 30, rnd() * 30];
    const c = cell(NUM, q);
    assert.ok(c.f1 <= c.f2);
    const d = cell(NUM, [q[0] + 1e-6, q[1], q[2]]);
    assert.ok(Math.abs(d.f1 - c.f1) < 1e-4);
    mean += c.f1;
  }
  mean /= 3000;
  assert.ok(mean > 0.4 && mean < 0.65, `mean ${mean}`);
});

test("stretch: features run across the axis", () => {
  const r = curls();
  const rs = rungs(r, { metres_per_px: 2e4, span_m: 1e8 }, "gpu-high", 0, 1).map((x) => ({ ...x, amplitude: 1 }));
  // Correlation length along x (across the axis z) against along z, with a stretch of 4.
  const corr = (dir: V3<number>) => {
    let s = 0;
    for (let k = 0; k < 400; k++) {
      const p: V3<number> = [rnd() * 1e8, rnd() * 1e8, rnd() * 1e8];
      const step = 3e5;
      const a = ladderAt("simplex", rs, p, { axis: [0, 0, 1], stretch: 4 }) as number;
      const b = ladderAt("simplex", rs, [p[0] + dir[0] * step, p[1] + dir[1] * step, p[2] + dir[2] * step], { axis: [0, 0, 1], stretch: 4 }) as number;
      s += (a - b) * (a - b);
    }
    return s;
  };
  assert.ok(corr([1, 0, 0]) < corr([0, 0, 1]) * 0.6);
});

test("the HUD's detail line names where facts end in the body's own units", () => {
  const r = curls();
  const v = discScale(R_JUP, 3000, 1000);
  assert.equal(detailLine([{ layer: r, shown: rungs(r, v, "gpu-high", 0, 1) }]), "Curls at band edges: finer than 10,000 km is drawn from the wind's figures, not computed.");
  assert.equal(detailLine([{ layer: r, shown: [] }]), "");
  assert.equal(sizeWords(76), "76 m");
  assert.equal(sizeWords(0.05), "5 cm");
  assert.equal(sizeWords(2500), "2.5 km");
});

test("per-latitude profiles (the weather op's jets): resolved at their peak, shares per band for the caller", () => {
  const jets = { rhines_scale_m: [4e6, 1e7, 6e6], deformation_radius_m: [2e6, 2.5e6, 3e6], jet_speed_m_s: [-40, 150, 60], jet_shear_m_s: [20, 75, null], eddy_speed_m_s: [10, 35, 20] };
  const r = resolveLayer(ladderLayer("giant.curls")!, jets) as ResolvedLayer;
  assert.equal(r.drawn, true);
  assert.equal(r.largest_m, 1e7);
  assert.equal(r.speed_m_s, 35);
  // Shear over speed, element by element: 0.5, 0.5, missing.
  assert.equal(r.amplitude, 0.5);
  assert.equal(r.stretch, 4);
  assert.deepEqual(profileShare(ladderLayer("giant.curls")!.amplitude, jets), [1, 1, 0]);
  assert.deepEqual(profileShare({ fact: "jet_speed_m_s" }, jets), [40 / 150, 1, 60 / 150]);
  assert.deepEqual(profileShare({ fact: "x" }, { x: 3 }), [1]);
  // A profile with nothing in it cuts the layer.
  assert.equal(resolveLayer(ladderLayer("giant.curls")!, { ...jets, jet_shear_m_s: [null, null, null] }).drawn, false);
});
