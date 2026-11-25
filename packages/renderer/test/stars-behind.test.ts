// The sky's stars behind True (VP2): fluxes against the solar-constant exposure, and the long exposure's gain.
import { test } from "node:test";
import assert from "node:assert/strict";
import { longGain, starsBehind, starTint } from "../src/stars-behind.ts";
import { constant } from "../../ui-contract/src/index.ts";

const square = { to_galactic: [[1, 0, 0], [0, 1, 0], [0, 0, 1]] as [[number, number, number], [number, number, number], [number, number, number]] };

test("a sun 1 au away has a share of one; at 1 pc, (au/pc)²", () => {
  const au = constant("au") / constant("pc");
  const { share } = starsBehind([{ dir: [0, 0, 1], luminosity_lsun: 1, distance_pc: au, teff_k: 5800 }, { dir: [1, 0, 0], luminosity_lsun: 1, distance_pc: 1, teff_k: 5800 }], square);
  const mean = (i: number) => (share[i]! + share[i + 1]! + share[i + 2]!) / 3;
  assert.ok(Math.abs(mean(0) - 1) < 1e-6);
  assert.ok(Math.abs(mean(3) / (au * au) - 1) < 1e-6);
});

test("a star's tint keeps its light: the channels average to one", () => {
  for (const t of [3000, 5800, 20000]) {
    const c = starTint(t);
    assert.ok(Math.abs((c[0] + c[1] + c[2]) / 3 - 1) < 1e-9);
  }
});

test("directions are turned from galactic into the system's frame", () => {
  const turned = { to_galactic: [[0, 1, 0], [-1, 0, 0], [0, 0, 1]] as typeof square.to_galactic };
  const { dirs } = starsBehind([{ dir: [0, 1, 0], luminosity_lsun: 1, distance_pc: 1, teff_k: 5800 }], turned);
  // The system frame is the transpose: galactic +y is this frame's -x.
  assert.deepEqual([...dirs].map((x) => Math.round(x) + 0), [-1, 0, 0]);
});

test("the long exposure fills the nth brightest star's pixel to white, and never dims", () => {
  const share = new Float32Array([1e-12, 1e-12, 1e-12, 1e-9, 1e-9, 1e-9]);
  const px = 1e-3;
  const g = longGain(share, px, 2);
  assert.ok(Math.abs(((Math.PI * 1e-12) / (px * px)) * g - 1) < 1e-3);
  assert.equal(longGain(new Float32Array([1, 1, 1]), px, 1), 1);
});
