// The sea as the kernel describes it (W-sea2): the reader keeps only well-formed numbers and names what is missing;
// Fresnel's split and the floor seen through the liquid behave as optics says.
import { test } from "node:test";
import assert from "node:assert/strict";
import { fresnel, readSeaOptics, seaBody, seaWords } from "../src/sea.ts";
import { landingSeaOf } from "../../ui-contract/src/landing.ts";

test("the reader keeps well-formed numbers and names the missing ones by step", () => {
  const none = readSeaOptics(undefined);
  assert.deepEqual(none.optics, {});
  assert.deepEqual(none.missing.colour, ["deep_rgb", "absorb_per_m", "refractive_index"]);
  const r = readSeaOptics({ deep_rgb: [0.002, 0.01, 0.03], absorb_per_m: [0.45, 0.06, 0.02], refractive_index: 1.333, bands_nm: [650, 550, 450], trains: [{ wavelength_m: 40, amplitude_m: 0.5, period_s: 5, dir_deg: 10 }, { wavelength_m: -1, amplitude_m: 1, period_s: 1, dir_deg: 0 }], foam_albedo: "x" });
  assert.deepEqual(r.missing.colour, []);
  assert.equal(r.optics.trains?.length, 1, "a train with no length is dropped");
  assert.deepEqual(r.missing.froth, ["whitecap_fraction", "foam_albedo"], "a foam albedo that is not a number is not read");
  assert.match(seaWords(r.optics, r.missing), /fastest in red .*waiting on the kernel for froth\.$/);
});

test("Fresnel: (n−1)²/(n+1)² straight down, all reflected at grazing, a methane sea reflects less than water", () => {
  for (const n of [1.27, 1.333, 1.5]) assert.ok(Math.abs(fresnel(n, 1) - ((n - 1) / (n + 1)) ** 2) < 1e-12);
  assert.ok(fresnel(1.333, 0) > 0.999);
  assert.ok(fresnel(1.27, 0.5) < fresnel(1.333, 0.5));
  // Brewster's angle: p vanishes, so the mean is half of s.
  const n = 1.333;
  const cb = Math.cos(Math.atan(n));
  const ct = Math.sqrt(1 - (1 - cb * cb) / (n * n));
  const rs = ((cb - n * ct) / (cb + n * ct)) ** 2;
  assert.ok(Math.abs(fresnel(n, cb) - rs / 2) < 1e-12);
});

test("the floor shows through shallow water and the liquid's own colour through deep water", () => {
  const o = readSeaOptics({ deep_rgb: [0.002, 0.01, 0.03], absorb_per_m: [0.45, 0.06, 0.02], refractive_index: 1.333 }).optics;
  const sand: [number, number, number] = [0.35, 0.35, 0.35];
  assert.deepEqual(seaBody(o, 0, sand), sand);
  const shallow = seaBody(o, 2, sand);
  assert.ok(shallow[0] < shallow[2], `red goes first: ${shallow}`);
  const deep = seaBody(o, 500, sand);
  for (let i = 0; i < 3; i++) assert.ok(Math.abs(deep[i]! - o.deep_rgb![i]!) < 1e-3);
  assert.deepEqual(seaBody({}, 50, sand), sand, "without the kernel's numbers, the older look");
});

test("the landing's sea carries the liquid's numbers to the drawing and to watching seats", () => {
  const s = landingSeaOf({ sea: { wave_m: 1.2, period_s: 6, wind_m_s: 5, deep_rgb: [0.01, 0.02, 0.03], refractive_index: 1.27, liquid_note: "Liquid methane", tides: { by: [] } } }, 0)!;
  assert.deepEqual(s.optics, { deep_rgb: [0.01, 0.02, 0.03], refractive_index: 1.27, liquid_note: "Liquid methane" });
  assert.equal(landingSeaOf({ sea: { wave_m: 1, period_s: 5 } }, 0)!.optics, undefined);
});

test("waves: the kernel's own trains when sent (so methane and water differ), the stand-in otherwise, always six", async () => {
  const { trainsToDraw, MAX_TRAINS } = await import("../src/sea.ts");
  const standIn = [{ k: 0.1, a: 0.5, w: 1, dir: 0, phase: 0 }];
  const none = trainsToDraw({}, standIn);
  assert.equal(none.source, "stand-in");
  assert.equal(none.trains.length, MAX_TRAINS);
  assert.deepEqual(none.trains[0], standIn[0]);
  assert.equal(none.trains[5]!.a, 0, "unused trains are still");
  const water = readSeaOptics({ trains: [{ wavelength_m: 40, amplitude_m: 0.6, period_s: 5.06, dir_deg: 90 }] }).optics;
  const methane = readSeaOptics({ trains: [{ wavelength_m: 40, amplitude_m: 0.9, period_s: 12.6, dir_deg: 90 }] }).optics;
  const w = trainsToDraw(water, standIn);
  assert.equal(w.source, "kernel");
  assert.ok(Math.abs(w.trains[0]!.k - (2 * Math.PI) / 40) < 1e-12);
  assert.ok(Math.abs(w.trains[0]!.w - (2 * Math.PI) / 5.06) < 1e-12);
  assert.ok(Math.abs(w.trains[0]!.dir - Math.PI / 2) < 1e-12);
  assert.ok(trainsToDraw(methane, standIn).trains[0]!.w < w.trains[0]!.w, "the same wavelength runs slower under Titan's gravity");
});

test("froth: the kernel's share and reflectance, streaks along the wind unless the kernel says otherwise", async () => {
  const { frothToDraw } = await import("../src/sea.ts");
  assert.equal(frothToDraw({}, 0), null, "no froth drawn until the kernel sends it");
  const f = frothToDraw(readSeaOptics({ whitecap_fraction: 0.03, foam_albedo: 0.55 }).optics, 1.2)!;
  assert.deepEqual(f, { cover: 0.03, albedo: 0.55, dir: 1.2 });
  assert.equal(frothToDraw(readSeaOptics({ whitecap_fraction: 2, foam_albedo: 0.5, streak_dir_deg: 180 }).optics, 0)!.cover, 1);
  assert.ok(Math.abs(frothToDraw(readSeaOptics({ whitecap_fraction: 0.1, foam_albedo: 0.5, streak_dir_deg: 180 }).optics, 0)!.dir - Math.PI) < 1e-12);
});
