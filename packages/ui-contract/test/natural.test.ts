import { test } from "node:test";
import assert from "node:assert/strict";
import { cubeDirection, edgeNoise, globeCellAngle, naturalDirection, naturalEdge, sampleAt, sampleColour, sampleNatural, softenGround, type SurfaceTile } from "../src/index.ts";

const tile = (face: number, s: number): SurfaceTile => ({ address: "t", key: { face, level: 0, x: 0, y: 0 }, samples: s, spacing_m: 1, epoch_myr: 0, elevation_dm: new Array(s * s).fill(0), material: new Array(s * s).fill(0), cover: Array.from({ length: s * s }, (_, k) => k % 3 === 0 ? 1 : 0), albedo: [], slope_ddeg: [], surface_age_myr: [], pending: [], hash: "h" });

test("natural outlines never move a cell's centre out of its own cell: every kernel fact stays where it was computed", () => {
  for (const s of [16, 128, 256]) {
    const globe = [0, 1, 2, 3, 4, 5].map((f) => tile(f, s));
    for (let f = 0; f < 6; f++) {
      for (let j = 0; j < s; j++) {
        for (let i = 0; i < s; i++) {
          const d = cubeDirection(f, -1 + (2 * (i + 0.5)) / s, -1 + (2 * (j + 0.5)) / s);
          const got = sampleNatural(globe, d)!;
          assert.equal(got.tile.key.face, f, `face at ${s} ${f} ${i} ${j}`);
          assert.equal(got.i, j * s + i, `sample at ${s} ${f} ${i} ${j}`);
        }
      }
    }
  }
});

test("natural outlines change the shapes between centres, the same way every time (no shimmer)", () => {
  const s = 32;
  const globe = [0, 1, 2, 3, 4, 5].map((f) => tile(f, s));
  let moved = 0;
  for (let k = 0; k < 2000; k++) {
    const d = cubeDirection(k % 6, Math.sin(k * 1.3) * 0.99, Math.cos(k * 0.7) * 0.99);
    const a = sampleAt(globe, d)!;
    const b = sampleNatural(globe, d)!;
    if (a.i !== b.i || a.tile !== b.tile) moved++;
    assert.deepEqual(sampleNatural(globe, d), sampleNatural(globe, d));
  }
  assert.ok(moved > 50 && moved < 1500, `moved ${moved}`);
});

test("the edge noise is about -1 to 1, varies within a cell, and a threshold through it is deterministic", () => {
  let lo = Infinity;
  let hi = -Infinity;
  for (let k = 0; k < 5000; k++) {
    const d = cubeDirection(k % 6, Math.sin(k), Math.cos(k * 1.1));
    const n = edgeNoise(d, 0.01);
    lo = Math.min(lo, n);
    hi = Math.max(hi, n);
    assert.equal(naturalEdge(d, 0.5, 0.5, 0.3, 0.01), naturalEdge(d, 0.5, 0.5, 0.3, 0.01));
  }
  assert.ok(lo >= -1 && hi <= 1 && hi - lo > 1, `${lo} ${hi}`);
  assert.notEqual(edgeNoise([1, 0, 0], 0.01, 1), edgeNoise([1, 0, 0], 0.01, 2), "a seed changes it");
});

test("bare ground's colour is softened over bare neighbours only: patches become gradients, seas and covers untouched", () => {
  const s = 16;
  const t: SurfaceTile = {
    ...tile(0, s),
    // Blocks of 4 × 4 of two materials, and a sea column at the right.
    material: Array.from({ length: s * s }, (_, k) => ((Math.floor((k % s) / 4) + Math.floor(Math.floor(k / s) / 4)) % 2 === 0 ? 0 : 1)),
    cover: Array.from({ length: s * s }, (_, k) => (k % s === s - 1 ? 1 : 0)),
    albedo: new Array(s * s).fill(300),
  };
  const soft = softenGround(t);
  assert.deepEqual(soft.cover, t.cover);
  const before = new Set(Array.from({ length: s * s }, (_, k) => sampleColour(t, k)[0]));
  const after = new Set(Array.from({ length: s * s }, (_, k) => sampleColour(soft, k)[0]));
  assert.ok(after.size > before.size + 4, `${before.size} → ${after.size} distinct reds`);
  for (let j = 0; j < s; j++) assert.deepEqual(sampleColour(soft, j * s + s - 1), sampleColour(t, j * s + s - 1), "the sea keeps its colour");
});
