import { test } from "node:test";
import assert from "node:assert/strict";
import { armField, galaxyFace, sheetWeights, surfaceAt } from "../src/galaxy-true.ts";
import type { GalaxyChart, GalaxyRing } from "../../ui-contract/src/index.ts";

// Test data, not physics: an exponential disc of stars (scale 3 kpc) with gas twice as extended, two log-spiral arms.
const ring = (inner: number, outer: number): GalaxyRing => {
  const mid = (inner + outer) / 2;
  const area = Math.PI * (outer * outer - inner * inner);
  return { inner_kpc: inner, outer_kpc: outer, radius_kpc: mid, stars: 1e9 * Math.exp(-mid / 3) * area, gas: 1e8 * Math.exp(-mid / 6) * area, abundances: {} as never, deaths: {} as never, cc_per_century: 0, ia_per_century: 0 };
};
const edges = [0.2, 0.5, 1, 1.6, 2.4, 3.4, 4.6, 6, 7.6, 9.4, 11.4, 13.6, 16];
const rings = edges.slice(0, -1).map((e, i) => ring(e, edges[i + 1]!));
const spiral = (off: number) => Array.from({ length: 80 }, (_, i) => [1.5 * Math.exp(0.035 * i), off + 0.12 * i] as [number, number]);
const chart = {
  key: "test",
  halo: "test",
  rings,
  arms: [spiral(0), spiral(Math.PI)],
  arm_contrast: 0.6,
  disc_kpc: 3,
  barred: false,
  at_gyr: 13.8,
  now_gyr: 13.8,
  sun_radius_kpc: 8,
  now: { bulge: 5e9, z_gas: 0.014 },
  assembly: null,
} as unknown as GalaxyChart;

const lum = (d: Uint8ClampedArray, w: number, x: number, y: number) => {
  const k = (y * w + x) * 4;
  return d[k]! + d[k + 1]! + d[k + 2]!;
};

test("surface densities: a ring's own value at its middle, falling past the last ring", () => {
  const g = rings[5]!;
  const s = surfaceAt(rings, (g.inner_kpc + g.outer_kpc) / 2, 3);
  assert.ok(Math.abs(s.stars / (g.stars / (Math.PI * (g.outer_kpc ** 2 - g.inner_kpc ** 2))) - 1) < 1e-9);
  assert.ok(surfaceAt(rings, 20, 3).stars < surfaceAt(rings, 15, 3).stars);
});

test("arm field: 1 on a crest, falling across it", () => {
  const f = armField([[[5, 0], [5, 0.01]]], 64, 10, 0.5);
  const at = (x: number, y: number) => f[Math.floor((10 - y) / (20 / 64)) * 64 + Math.floor((x + 10) / (20 / 64))]!;
  assert.ok(at(5, 0) > 0.9);
  assert.ok(at(6.5, 0) < 0.1);
});

test("visible light: brightest near the centre, faint at the edge, the same bytes every time", () => {
  const p = galaxyFace(chart, 13.8, "visible", 96);
  const c = p.width / 2;
  assert.ok(lum(p.data, p.width, c, c) > lum(p.data, p.width, c + 40, c));
  assert.deepEqual(galaxyFace(chart, 13.8, "visible", 96).data, p.data);
  assert.ok(p.bulge.level > 0 && p.bulge.scale_kpc > 0);
  assert.match(p.words, /true colour/);
});

test("each band reads its own part: radio the gas reaching past the stars, ultraviolet only where stars form", () => {
  const w = 96;
  const radio = galaxyFace(chart, 13.8, "radio", w);
  const ir = galaxyFace(chart, 13.8, "infrared", w);
  const c = w / 2;
  // Out near the last ring the gas (scale 6 kpc) still shows in radio more than the stars (3 kpc) do in infrared,
  // each against its own centre.
  const far = c + 36;
  const share = (p: typeof radio) => lum(p.data, w, far, c) / Math.max(1, lum(p.data, w, c + 4, c));
  assert.ok(share(radio) > share(ir), `radio ${share(radio)} vs infrared ${share(ir)}`);
  const noGas = { ...chart, rings: rings.map((g) => ({ ...g, gas: 0 })) } as GalaxyChart;
  const uv = galaxyFace(noGas, 13.8, "ultraviolet", w);
  assert.equal(uv.data.reduce((a, v, i) => a + (i % 4 === 3 ? 0 : v), 0), 0, "no gas, no young stars, no ultraviolet");
  for (const b of ["infrared", "ultraviolet", "xray", "radio"] as const) assert.match(galaxyFace(chart, 13.8, b, 32).words, /mapped grey/);
});

test("the disc's thickness: layers symmetric about the plane, their light adding to one", () => {
  const l = sheetWeights(7);
  assert.ok(Math.abs(l.reduce((a, x) => a + x.weight, 0) - 1) < 1e-12);
  assert.ok(Math.abs(l[0]!.z_kpc + l[6]!.z_kpc) < 1e-12);
  assert.ok(l[3]!.weight > l[0]!.weight);
});
