import { test } from "node:test";
import assert from "node:assert/strict";
import { composeFaces, directionFace, faceDirection, faceSize, faceWords, levelFor, reliefNormal, sampleFaces, texelAngle, tilesFacing, TILE_SAMPLES } from "../src/texture-ladder.ts";
import { faceOf, type SurfaceTile } from "../../ui-contract/src/index.ts";

/** A tile filled from a height function of direction (metres), sea below `sea`. */
function tile(face: number, level: number, x: number, y: number, h: (d: [number, number, number]) => number, s = 8, sea = -Infinity): SurfaceTile {
  const n = s * (1 << level);
  const elevation_dm: number[] = [];
  const cover: number[] = [];
  for (let j = 0; j < s; j++) {
    for (let i = 0; i < s; i++) {
      const d = faceDirection(face, -1 + (2 * (x * s + i + 0.5)) / n, -1 + (2 * (y * s + j + 0.5)) / n);
      const e = h(d);
      elevation_dm.push(Math.round(e * 10));
      cover.push(e < sea ? 1 : 0);
    }
  }
  return { address: "t", key: { face, level, x, y }, samples: s, spacing_m: 1e5, epoch_myr: 0, elevation_dm, material: new Array(s * s).fill(0), cover, albedo: new Array(s * s).fill(300), slope_ddeg: [], surface_age_myr: [], pending: [], hash: `${face}${level}${x}${y}` };
}
const globe = (h: (d: [number, number, number]) => number, s = 8) => [0, 1, 2, 3, 4, 5].map((f) => tile(f, 0, 0, 0, h, s));

test("face size: one texel to a screen pixel, a power of two inside the tier's range", () => {
  assert.equal(faceSize(10, "gpu-basic"), 64);
  assert.equal(faceSize(400, "gpu-basic"), 512);
  assert.equal(faceSize(5000, "gpu-basic"), 512);
  assert.equal(faceSize(5000, "gpu-high"), 2048);
  assert.equal(faceSize(5000, "still"), 4096);
  assert.equal(faceSize(500, "canvas"), 0);
  assert.equal(levelFor(32), 0);
  assert.equal(levelFor(64), 1);
  assert.equal(levelFor(512), 4);
  assert.equal(levelFor(500, TILE_SAMPLES), 4);
});

test("the cube map is the kernel's: faceDirection and directionFace invert each other and agree with ui-contract", () => {
  for (let f = 0; f < 6; f++) {
    for (const [u, v] of [[0, 0], [0.5, -0.3], [-0.9, 0.9]] as const) {
      const d = faceDirection(f, u, v);
      const back = directionFace(d);
      assert.equal(back.face, f);
      assert.ok(Math.abs(back.u - u) < 1e-12 && Math.abs(back.v - v) < 1e-12);
      const k = faceOf(d);
      assert.equal(k.face, f);
      assert.ok(Math.abs(k.u - u) < 1e-12 && Math.abs(k.v - v) < 1e-12);
    }
  }
});

test("tiles facing the viewer: nearest the middle first, the far side left out", () => {
  const t = tilesFacing([1, 0, 0], 1, 100);
  assert.deepEqual(t.slice(0, 4).map((k) => k.face), [0, 0, 0, 0]);
  assert.ok(t.every((k) => k.face !== 1));
  assert.equal(tilesFacing([0, 0, 1], 2, 5).length, 5);
});

test("compose: deeper tiles laid over coarser ones, the gaps keep the coarse read", () => {
  const h = (d: number[]) => 1000 * d[0]!;
  const tiles = [...globe(h), tile(0, 1, 1, 1, () => 5000)];
  const c = composeFaces(tiles);
  assert.equal(c.faces[0]!.n, 16);
  assert.equal(c.faces[1]!.n, 8);
  assert.equal(c.faces[0]!.level, 1);
  // The level-1 tile covers u, v > 0 on face 0.
  assert.equal(sampleFaces(c, faceDirection(0, 0.5, 0.5))!.height_m, 5000);
  assert.ok(Math.abs(sampleFaces(c, faceDirection(0, -0.5, -0.5))!.height_m - 1000 * faceDirection(0, -0.5, -0.5)[0]) < 60);
  assert.notEqual(composeFaces(globe(h)).signature, c.signature);
  assert.equal(composeFaces([]).faces.every((f) => f === null), true);
});

test("sampling is continuous across a cube edge and matches the tiles' heights", () => {
  const h = (d: number[]) => 3000 * d[0]! + 2000 * d[2]!;
  const c = composeFaces(globe(h, 32));
  for (let k = 0; k < 200; k++) {
    const a = Math.sin(k * 1.7);
    const b = Math.cos(k * 0.9);
    const d = [a, b, Math.sin(k * 0.3)];
    const l = Math.hypot(d[0]!, d[1]!, d[2]!);
    const u = [d[0]! / l, d[1]! / l, d[2]! / l] as [number, number, number];
    assert.ok(Math.abs(sampleFaces(c, u)!.height_m - h(u)) < 250, `at ${u}`);
  }
});

test("relief normal: flat ground points straight up, a slope tilts it downhill by the true angle", () => {
  const R = 1e6;
  const flat = composeFaces(globe(() => 0, 32));
  const d: [number, number, number] = faceDirection(0, 0.1, 0.2);
  const n0 = reliefNormal(flat, d, R, texelAngle(32));
  assert.ok(Math.abs(n0[0] * d[0] + n0[1] * d[1] + n0[2] * d[2] - 1) < 1e-12);
  // Height rising with z: slope dh/ds = 50000 m over a radian of arc at the equator, so tan(tilt) = 0.05.
  const ramp = composeFaces(globe((p) => 50000 * Math.asin(p[2]), 64));
  const e: [number, number, number] = [1, 0, 0];
  const n = reliefNormal(ramp, e, R, texelAngle(64));
  const tilt = Math.atan2(-n[2], n[0]);
  assert.ok(Math.abs(tilt - Math.atan(0.05)) < 0.002, `tilt ${tilt}`);
  // Seas are flat whatever the sea floor does.
  const sea = composeFaces([0, 1, 2, 3, 4, 5].map((f) => tile(f, 0, 0, 0, (p) => -4000 + 3000 * p[2], 32, 0)));
  const ns = reliefNormal(sea, e, R, texelAngle(32));
  assert.ok(Math.abs(ns[0] - 1) < 1e-12);
});

test("the HUD's words name the finest ground read", () => {
  const c = composeFaces(globe(() => 0, 32));
  assert.equal(faceWords(c, 6.371e6), "Ground read to 313 km.");
  assert.equal(faceWords(composeFaces([]), 1), "");
});
