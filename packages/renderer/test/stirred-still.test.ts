// The stirred still's composing (stirred-still.ts): where each pixel's cloud is read, and through which level.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { generations, type CarryTable } from "../src/stirred-carry.ts";
import { composeStill, placesFor, stretchAt } from "../src/stirred-still.ts";
import type { TrueTexture } from "../src/true-look.ts";

const fx = JSON.parse(readFileSync(new URL("../../../tests/fixtures/weather/carry.json", import.meta.url), "utf8")) as { neptune: { table: CarryTable } };
const t = fx.neptune.table;
const W = 64;
const H = 32;

/** A face from a colour rule by pixel. */
const faceOf = (f: (i: number, j: number) => [number, number, number]): TrueTexture => {
  const data = new Uint8Array(W * H * 4);
  for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) data.set([...f(i, j), 255], (j * W + i) * 4);
  return { width: W, height: H, data, mean: [10, 20, 30] };
};
/** Read places: both generations read where `f` says, from each pixel's own place. */
const readOf = (f: (lat: number, lon: number, gen: 0 | 1) => [number, number]): Float32Array => {
  const p = placesFor(W, H);
  const out = new Float32Array(W * H * 4);
  for (let k = 0; k < W * H; k++) out.set([...f(p[k * 2]!, p[k * 2 + 1]!, 0), ...f(p[k * 2]!, p[k * 2 + 1]!, 1)], k * 4);
  return out;
};
const stripes = faceOf((i, j) => [i * 4, j * 8, (i + j) % 2 ? 200 : 40]);

test("with no wind, the still is the face itself", () => {
  const s = composeStill(stripes, t, readOf((la, lo) => [la, lo]));
  assert.deepEqual(Array.from(s.data), Array.from(stripes.data));
  assert.deepEqual(s.mean, stripes.mean, "stirring makes no colour: the disc's mean is the face's");
});

test("cloud carried a pixel east is read a pixel west, both generations weighed as the kernel weighs them", () => {
  const step = 360 / W;
  // The younger generation came from a pixel west, the older from two: each pixel mixes those two columns.
  const s = composeStill(stripes, t, readOf((la, lo, g) => [la, lo - step * (g + 1)]));
  for (const [i, j] of [[5, 3], [0, 10], [40, 20]] as const) {
    const lat = ((j + 0.5) / H) * 180 - 90;
    const [[, w1], [, w2]] = generations(t, lat);
    const at = (x: number) => stripes.data[(j * W + ((x + W) % W)) * 4]!;
    const want = w1 * at(i - 1) + w2 * at(i - 2);
    assert.ok(Math.abs(s.data[(j * W + i) * 4]! - want) <= 0.5 + 1e-9, `pixel ${i}, ${j}: ${s.data[(j * W + i) * 4]} against ${want}`);
  }
});

test("cloud drawn finer than a pixel is read through a coarser level: a checkerboard stretched eightfold is its mean", () => {
  const checks = faceOf((i, j) => ((i + j) % 2 ? [255, 255, 255] : [0, 0, 0]));
  // Each picture pixel reads eight of the face's pixels apart east-west, on a pixel's centre: the map stretches cloud
  // eightfold, and read at level 0 every pixel would land on one colour of the board.
  const read = readOf((la, lo) => [la, lo * 8 + 180 / W]);
  assert.ok(Math.abs(stretchAt(read, W, H, 10, 10, 0) - 8) < 1e-3);
  const s = composeStill(checks, t, read);
  for (const [i, j] of [[10, 10], [33, 20]] as const) {
    const v = s.data[(j * W + i) * 4]!;
    assert.ok(Math.abs(v - 127.5) < 2, `pixel ${i}, ${j}: ${v}, the mean is 127.5`);
  }
  // Unstretched, the same checkerboard keeps its contrast.
  const sharp = composeStill(checks, t, readOf((la, lo) => [la, lo]));
  assert.equal(sharp.data[(10 * W + 10) * 4], 0);
});

test("cloud sheared along its band is averaged along the band alone: the bands keep their colours", () => {
  // Bands two pixels tall, read from places dragged east by forty degrees per degree of latitude (a jet's shear):
  // neighbours north and south read forty of the face's pixels apart east-west and none north-south.
  const bands = faceOf((i, j) => (j % 4 < 2 ? [255, 255, 255] : [0, 0, 0]));
  const s = composeStill(bands, t, readOf((la, lo) => [la, lo + 40 * la]));
  for (const [i, j] of [[10, 10], [33, 13], [50, 20], [3, 25]] as const) {
    const want = bands.data[(j * W + i) * 4]!;
    const v = s.data[(j * W + i) * 4]!;
    assert.ok(Math.abs(v - want) < 2, `pixel ${i}, ${j}: ${v}, its band is ${want}`);
  }
});
