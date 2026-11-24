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
