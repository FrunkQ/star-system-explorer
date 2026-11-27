// SL1: the ground round a landing site and the light over it. The local frame is orthonormal and centred on the
// site; the level arithmetic matches the kernel's `level_for`; the curve and the horizon are the true ones; the
// flown track ends on the site; the sky's single scattering behaves as Rayleigh's does (blue overhead, a reddened
// sun low down, nothing with no air).
import { test } from "node:test";
import assert from "node:assert/strict";
import { airMass, curveDrop, descentTrack, floorFromRefusal, groundColours, groundMesh, horizonDistance, levelFor, localToDir, siteFrame, skyIrradiance, skyRadiance, sunDirection, sunTransmittance } from "../src/index.ts";

const near = (a: number, b: number, tol: number, what: string) => assert.ok(Math.abs(a - b) <= tol, `${what}: ${a} vs ${b}`);
const EARTH_R = 6.371e6;

test("the site's frame is orthonormal and a point at the origin is the site's own direction", () => {
  for (const [lat, lon] of [[0, 0], [45, 30], [-70, -150], [89, 10]] as const) {
    const f = siteFrame(lat, lon);
    const dot = (a: number[], b: number[]) => a[0]! * b[0]! + a[1]! * b[1]! + a[2]! * b[2]!;
    near(dot(f.up, f.up), 1, 1e-12, "up");
    near(dot(f.east, f.north), 0, 1e-12, "east·north");
    near(dot(f.up, f.east), 0, 1e-12, "up·east");
    assert.ok(f.north[2] >= 0, "north leans to the pole");
    const d = localToDir(f, 0, 0, EARTH_R);
    near(dot(d, f.up), 1, 1e-12, "origin");
    // 1 km east is 1/R radians away.
    const e = localToDir(f, 1000, 0, EARTH_R);
    near(Math.acos(Math.min(1, dot(e, f.up))) * EARTH_R, 1000, 0.01, "1 km east");
  }
});

test("levels as the kernel counts them; a refusal's floor is read back", () => {
  // Earth's level-0 tiles: 256 samples at 39.1 km; the 100 m floor is level 9 (76 m), as the kernel says.
  assert.equal(levelFor((EARTH_R * Math.PI) / 2 / 256, 100), 9);
  assert.equal(levelFor((1.7374e6 * Math.PI) / 2 / 256, 100), 7);
  assert.equal(floorFromRefusal("BelowFactFloor { level: 12, floor_level: 9 }"), 9);
  assert.equal(floorFromRefusal("Withheld { state: Scanned, max_level: Some(0) }"), 0);
  assert.equal(floorFromRefusal("bad tile key"), null);
});

test("the curve and the horizon are the world's own", () => {
  near(horizonDistance(1.7, EARTH_R), 4654, 2, "an eye at 1.7 m sees 4.65 km on Earth");
  near(horizonDistance(1.7, 1.7374e6), 2430, 2, "and 2.43 km on the Moon");
  near(curveDrop(5000, EARTH_R), (5000 * 5000) / (2 * EARTH_R), 1e-3, "the drop at 5 km");
  const m = groundMesh("t", 5, 10000, 0, EARTH_R, () => ({ z: 0, cover: 0, material: 0, albedo: 0.3, depth: 0 }));
  // The corner of a flat patch sits below the centre by the curve.
  near(m.positions[2]!, -curveDrop(Math.hypot(10000, 10000), EARTH_R), 1e-3, "corner drop");
  near(m.positions[(2 * 5 + 2) * 3 + 2]!, 0, 1e-9, "centre on the datum");
  assert.equal(m.indices.length, 4 * 4 * 6);
  const holed = groundMesh("h", 5, 10000, 5000, EARTH_R, () => ({ z: 0, cover: 0, material: 0, albedo: 0.3, depth: 0 }));
  assert.equal(holed.indices.length, (16 - 4) * 6, "the four inner cells are left to the finer ring");
});

test("a point's reflectance is the kernel's albedo; water reads as water", () => {
  const rock = groundColours({ z: 100, cover: 0, material: 0, albedo: 0.2, depth: 0 });
  const lum = 0.2126 * rock.albedo[0] + 0.7152 * rock.albedo[1] + 0.0722 * rock.albedo[2];
  near(lum, 0.2, 0.01, "luminance is the albedo");
  const sea = groundColours({ z: 0, cover: 1, material: 0, albedo: 0.06, depth: 500 });
  assert.ok(sea.chart[2] > sea.chart[0], "the sea is blue on the chart");
});

test("the flown track ends on the site and starts back along the way in", () => {
  const tel = [0, 1, 2, 3].map((i) => ({ t_s: i * 10, altitude_m: 30000 - i * 10000, speed_m_s: 100, vertical_m_s: -10, downrange_km: i * 50, decel_g: 0, heat_w_cm2: 0, phase: "entry", thrust: 0 }));
  const tr = descentTrack(tel, -200, EARTH_R, 90);
  const end = tr[tr.length - 1]!;
  near(end.x, 0, 1e-6, "x");
  near(end.y, 0, 1e-6, "y");
  near(end.z, -200, 1e-6, "on the ground");
  assert.ok(tr[0]!.x < -149000 && Math.abs(tr[0]!.y) < 1e-6, "from the west");
  near(tr[0]!.to_go_km, 150, 1e-9, "150 km to go");
  // 30 km up and 150 km back: below the level plane by about the curve's drop.
  near(tr[0]!.z, -200 + 30000 - curveDrop(150000, EARTH_R), 40, "height on the curve");
});

test("the sky: blue overhead, a reddened sun low down, black with no air", () => {
  // Earth's Rayleigh optical depths at 680, 550 and 440 nm, and its 8 km scale height.
  const tau = [0.045, 0.097, 0.236];
  const H = 8000;
  near(airMass(1, EARTH_R, H), 1, 1e-3, "air mass overhead");
  near(airMass(0, EARTH_R, H), Math.sqrt((2 * EARTH_R) / H + 1), 1e-9, "air mass at the horizon, the shell's sqrt(2R/H + 1)");
  near(airMass(0, EARTH_R, H), 38.1, 2, "within 5% of Kasten and Young's 38.1 for the real air");
  const zen = tau.map((t) => skyRadiance(t, 1, Math.cos((30 * Math.PI) / 180), Math.cos((30 * Math.PI) / 180), EARTH_R, H));
  assert.ok(zen[2]! > zen[1]! && zen[1]! > zen[0]!, `blue overhead: ${zen}`);
  const high = tau.map((t) => sunTransmittance(t, 0.9, EARTH_R, H));
  const low = tau.map((t) => sunTransmittance(t, 0.02, EARTH_R, H));
  assert.ok(low[0]! / low[2]! > 2 * (high[0]! / high[2]!), "the low sun is reddened");
  assert.equal(skyRadiance(0, 1, 0.5, 0.5, EARTH_R, H), 0);
  assert.equal(skyIrradiance(0, 0.5, EARTH_R, H), 0);
  assert.ok(skyIrradiance(0.236, 0.8, EARTH_R, H) > skyIrradiance(0.045, 0.8, EARTH_R, H), "more blue sky light than red");
});

test("the sun: overhead at noon on the equator at the equinox, low in the south at 60° N, west after noon", () => {
  const s = sunDirection(0, 0, 0);
  near(s[2], 1, 1e-12, "overhead");
  const n = sunDirection(60, 0, 0);
  near(Math.asin(n[2]) * (180 / Math.PI), 30, 1e-9, "30° up");
  assert.ok(n[1] < 0, "in the south");
  assert.ok(sunDirection(20, 45, 0)[0] < 0, "west in the afternoon");
});

test("the flown descent plays at a steady apparent motion of the ground", async () => {
  const { playbackKeys } = await import("../src/index.ts");
  // A fast pass high up then a slow fall low down: the low part gets most of the playback, though it is short.
  const tr = [
    { x: -100000, y: 0, z: 20000 },
    { x: -50000, y: 0, z: 15000 },
    { x: 0, y: 0, z: 1000 },
    { x: 0, y: 0, z: 100 },
    { x: 0, y: 0, z: 0 },
  ];
  const k = playbackKeys(tr, 0);
  assert.equal(k[0], 0);
  assert.equal(k[k.length - 1], 1);
  for (let i = 1; i < k.length; i++) assert.ok(k[i]! > k[i - 1]!, "steadily on");
  assert.ok(k[4]! - k[2]! > 0.1, `the last kilometre gets a fair share: ${k}`);
  assert.ok(k[1]! < 0.1, `50 km high up passes quickly: ${k}`);
});

test("RW16a: on a lump the ground lies on the lump, the same one the globe draws; without one it is the tiles alone", async () => {
  const { patchKeys, readPatch, smallBodyShape } = await import("../src/index.ts");
  const { heightAt } = await import("../src/surface-patch.ts");
  const R = 11_100;
  const S = 16;
  const flat = (key: { face: number; level: number; x: number; y: number }) => ({ address: "phobos", key, samples: S, spacing_m: (R * Math.PI) / 2 / S, epoch_myr: 0, elevation_dm: new Array(S * S).fill(0), material: new Array(S * S).fill(1), cover: new Array(S * S).fill(0), albedo: new Array(S * S).fill(70), slope_ddeg: new Array(S * S).fill(0), surface_age_myr: new Array(S * S).fill(0), pending: [], hash: `${key.face}` });
  const half = 3000;
  const tiles = patchKeys(10, 20, half, 0, R, S).map(flat);
  const round = readPatch(tiles, 10, 20, R, half, 33);
  assert.ok(round.height_m.every((h) => h === 0), "flat tiles, no lump: flat ground");
  // A 27 × 22 × 18 km figure, Phobos-like.
  const figure = { kind: "irregular", axes_m: [13_500, 11_000, 9_000] as [number, number, number], lobes: [], relief: 0.08 };
  const shape = smallBodyShape(figure, "u:sol/sys:0/body:3/moon:0");
  const lumpy = readPatch(tiles, 10, 20, R, half, 33, 0, shape.mean);
  const f = siteFrame(10, 20);
  for (const [x, y] of [[0, 0], [2500, -1500], [-3000, 3000]] as const) {
    const want = (shape.mean(localToDir(f, x, y, R)) - 1) * R;
    near(heightAt(lumpy, x, y), want, 1, `ground at ${x}, ${y} is the lump's radius less the mean`);
  }
  const span = Math.max(...lumpy.height_m) - Math.min(...lumpy.height_m);
  assert.ok(span > 50, `the lump shapes the ground across 6 km: ${span.toFixed(0)} m`);
});
