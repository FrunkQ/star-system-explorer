// V10a: True on the system chart. Each world's picture is its own globe laid out for a sphere whose poles are on z,
// so the north row shows the +z face, the middle column of the equator the +x face, and so on; the star's colour is
// its temperature's.
import { test } from "node:test";
import assert from "node:assert/strict";
import { equirect, starCss } from "../src/true-look.ts";
import type { Prepared } from "../src/surface.ts";

// One sample per cube face, each face its own colour (faces: +x, -x, +y, -y, +z, -z).
const COLOURS: [number, number, number][] = [
  [200, 0, 0],
  [0, 200, 0],
  [0, 0, 200],
  [200, 200, 0],
  [255, 255, 255],
  [10, 10, 10],
];
const p = {
  samples: 6,
  faces: COLOURS.map((c) => ({ s: 1, rgb: new Uint8ClampedArray(c) })),
} as unknown as Prepared;

test("the picture wraps the globe the way the sphere is turned", () => {
  const W = 64;
  const H = 32;
  const t = equirect(p, W, H);
  const px = (i: number, j: number) => Array.from(t.data.slice((j * W + i) * 4, (j * W + i) * 4 + 3));
  assert.deepEqual(px(5, H - 1), COLOURS[4], "north row: +z");
  assert.deepEqual(px(5, 0), COLOURS[5], "south row: -z");
  assert.deepEqual(px(W / 2, H / 2), COLOURS[0], "u = ½ on the equator looks along +x");
  assert.deepEqual(px((3 * W) / 4, H / 2), COLOURS[2], "u = ¾ looks along +y");
  assert.deepEqual(px(1, H / 2), COLOURS[1], "u near 0 looks along -x");
  assert.equal(t.data[3], 255);
  // The mean is weighted by area, so the equator's four faces outweigh the poles.
  assert.ok(t.mean.every((v) => v > 0 && v < 255));
});

test("a face with no tile is shown grey, not invented", () => {
  const holed = { samples: 5, faces: [...p.faces.slice(0, 4), null, p.faces[5]] } as unknown as Prepared;
  const t = equirect(holed, 16, 8);
  assert.deepEqual(Array.from(t.data.slice((7 * 16 + 3) * 4, (7 * 16 + 3) * 4 + 3)), [128, 128, 128]);
});

test("a star's colour is its temperature's", () => {
  const rgb = (s: string) => s.match(/\d+/g)!.map(Number);
  const [r1, , b1] = rgb(starCss(3000));
  const [r2, , b2] = rgb(starCss(10000));
  assert.ok(r1! > b1!, "a cool star is red");
  assert.ok(b2! >= r2!, "a hot star is blue-white");
});

test("thermal infrared grey: the star is white, a warm world grey, a cold one darker, nothing dark at zero", async () => {
  const { thermalGrey } = await import("../src/true-look.ts");
  assert.equal(thermalGrey(5772, 5772), 1);
  const earth = thermalGrey(288, 5772);
  const pluto = thermalGrey(44, 5772);
  assert.ok(earth > 0.3 && earth < 0.6, `Earth-like ${earth}`);
  assert.ok(pluto < earth, "colder is darker");
  assert.equal(thermalGrey(0, 5772), 0);
});
