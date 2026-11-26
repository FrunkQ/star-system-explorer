import { test } from "node:test";
import assert from "node:assert/strict";
import { composeFaces, directionFace, faceDirection, faceSize, faceWords, levelFor, reliefNormal, sampleFaces, texelAngle, tilesFacing, TILE_SAMPLES, withFiner, patchLevel, tilesNear, patchFor, PATCH_TILES } from "../src/texture-ladder.ts";
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
  assert.equal(levelFor(32, 32), 0);
  assert.equal(levelFor(64, 32), 1);
  assert.equal(levelFor(512, 32), 4);
  assert.equal(levelFor(512), 1);
  assert.equal(levelFor(1000, TILE_SAMPLES), 2);
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

test("compose: a coarse tile under finer ones is spread smoothly between its samples, not in blocks", () => {
  // A step in height across face 0's level-0 tile, then a level-1 tile elsewhere so the face is composed at 16.
  const h = (d: number[]) => (d[1]! > 0 ? 2000 : 0);
  const c = composeFaces([...globe(h), tile(0, 1, 1, 1, () => 5000)]);
  const f = c.faces[0]!;
  assert.equal(f.n, 16);
  // Along a row of the coarse lower half, across the step at u = 0: values pass through between (500, 1500) rather
  // than jumping 0 → 2000 between two blocks of two texels.
  const heights = Array.from({ length: 8 }, (_, i) => f.height_m[2 * 16 + 4 + i]!);
  assert.ok(heights.some((x) => x > 100 && x < 1900), `row ${heights}`);
  assert.equal(new Set(heights.map((x) => Math.round(x))).size >= 4, true, `row ${heights}`);
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

test("finer tiles laid over a kept surface: recomposed once, never doubled; a painted face is left alone", () => {
  const base = { width: 1, faces: composeFaces(globe(() => 0)) };
  const fine = [tile(0, 1, 1, 1, () => 5000)];
  const t = withFiner(base, fine);
  assert.equal(t.width, 1);
  assert.equal(t.faces!.faces[0]!.level, 1);
  assert.equal(sampleFaces(t.faces!, faceDirection(0, 0.5, 0.5))!.height_m, 5000);
  assert.equal(withFiner(t, fine), t);
  const painted = { width: 1 };
  assert.equal(withFiner(painted, fine), painted);
});

test("the deep patch: its level from metres a pixel, its tiles round the point, over a cube edge too", () => {
  const R = 6.371e6;
  // Level 0 spacing on Earth is about 39 km (256 samples); at 100 m a pixel a sample every 200 m is level 8 (153 m).
  assert.equal(patchLevel(100, R), 8);
  assert.equal(patchLevel(100, R, TILE_SAMPLES, 7), 7);
  assert.equal(patchLevel(100, R, 32), 11);
  assert.equal(patchLevel(1e6, R), 0);
  assert.equal(patchFor({ toward: [1, 0, 0], m_per_px: 5e4 }, R).keys.length, 0);
  const mid = tilesNear([1, 0.0001, 0.0001], 8);
  assert.equal(mid.length, PATCH_TILES);
  assert.deepEqual(mid[0], { face: 0, level: 8, x: 128, y: 128 });
  assert.ok(mid.every((k) => k.face === 0 && Math.abs(k.x - 128) <= 2 && Math.abs(k.y - 128) <= 2));
  // On the edge between face 0 (+x) and face 2 (+y): both faces' tiles.
  const edge = tilesNear([1, 1, 0.001], 6);
  assert.ok(edge.some((k) => k.face === 0) && edge.some((k) => k.face === 2));
  const p = patchFor({ toward: [0, 0, 1], m_per_px: 100 }, R);
  assert.equal(p.level, 8);
  assert.equal(p.keys[0]!.face, 4);
});

test("the deep patch picture: the tiles' colour and heights about the centre, relief by the true slope, gaps left clear", async () => {
  const { composePatch, samplePatch, patchDirection } = await import("../src/texture-ladder.ts");
  const R = 1e6;
  const L = 5;
  const toward = faceDirection(0, 0.01, -0.02);
  // A ramp rising 0.05 m a metre eastward along the patch frame, on the tiles round the point.
  const keys = tilesNear(toward, L, 9);
  const probe = composePatch([tile(0, L, keys[0]!.x, keys[0]!.y, () => 0, 16)], toward, R, 64)!;
  const east = probe.east;
  const h = (d: number[]) => 0.05 * R * (d[0]! * east[0] + d[1]! * east[1] + d[2]! * east[2]);
  const tiles = keys.map((k) => tile(k.face, k.level, k.x, k.y, h, 16));
  const p = composePatch(tiles, toward, R, 64)!;
  assert.equal(p.level, L);
  const mid = samplePatch(p, toward)!;
  assert.equal(mid.rgba[3], 255);
  // The normal leans west by atan(0.05) against the ramp.
  const tilt = Math.acos(mid.normal[0] * toward[0] + mid.normal[1] * toward[1] + mid.normal[2] * toward[2]);
  assert.ok(Math.abs(tilt - Math.atan(0.05)) < 0.002, `tilt ${tilt}`);
  assert.ok(mid.normal[0] * east[0] + mid.normal[1] * east[1] + mid.normal[2] * east[2] < 0);
  // Outside the patch nothing; with one tile only, the far corners are gaps.
  assert.equal(samplePatch(p, [-toward[0], -toward[1], -toward[2]]), null);
  const one = composePatch(tiles.slice(0, 1), toward, R, 64)!;
  const corner = patchDirection(one, -one.half * 0.95, -one.half * 0.95);
  const far = patchDirection(one, one.half * 0.95, one.half * 0.95);
  assert.ok(samplePatch(one, corner)!.rgba[3] === 0 || samplePatch(one, far)!.rgba[3] === 0);
  assert.equal(composePatch([], toward, R), null);
});
