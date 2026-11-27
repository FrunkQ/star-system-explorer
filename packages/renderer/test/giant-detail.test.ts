// A giant's finer face (giant-detail.ts): patches read closer round its poles and storms, storms turned by their own
// winds, curls through the detail ladder.
import { test } from "node:test";
import assert from "node:assert/strict";
import { curlFacts, detailGiant, GIANT_W, MAX_STORM_PATCHES, patchesFor, POLE_N, STORM_N, type FieldPatch, type GiantFacts } from "../src/giant-detail.ts";
import { sheetColours } from "../src/true-air.ts";
import type { TrueTexture } from "../src/true-look.ts";
import type { WeatherCell, WeatherSystem } from "../src/weather.ts";

const A = 6e7;
const colours = sheetColours([{ id: "nh3", names: { physics: "ammonia" }, optical: { condensed_colour_rgb: { v: [1, 1, 1] } } }]);
const sys = (o: Partial<WeatherSystem> & { aspect?: number }): WeatherSystem => ({ id: "s", kind: "oval", lat_deg: 20, lon_deg: 0, size_km: 6000, strength: 100, born: 0, ends: 1e9, why: "", ...o }) as WeatherSystem;
const cell = (cover: number, wind: [number, number] = [0, 0]): WeatherCell => ({
  lat_deg: 0,
  lon_deg: 0,
  layers: [{ cloud_cover: cover, cloud_type: "deck", condensate: "ammonia ice", optical_depth: 8, wind_m_s: wind, turbulence: 0 }],
  precip_kind: "none",
  precip_mm_day: 0,
  temp_offset_k: 0,
  lightning_km2_yr: 0,
  aerosols: [],
});
const grey = (w = 128, h = 64): TrueTexture => ({ width: w, height: h, data: new Uint8Array(w * h * 4).fill(128), mean: [128, 128, 128] });
const at = (t: TrueTexture, lat: number, lon: number) => {
  const i = Math.floor(((lon + 180) / 360) * t.width) % t.width;
  const j = Math.min(t.height - 1, Math.floor(((lat + 90) / 180) * t.height));
  return t.data[(j * t.width + i) * 4]!;
};
const facts = (systems: WeatherSystem[], patches: FieldPatch[]): GiantFacts => ({ address: "u:t/sys:0/body:1", radius_m: A, t_s: 0, systems, patches, jets: null });

test("a pole with anything reaching it gets one patch round it; storms too small for the face get none; the largest come first", () => {
  const systems = [
    sys({ id: "poly", kind: "polygon-jet", lat_deg: 78, lon_deg: 10, size_km: 2 * 12000 }),
    sys({ id: "vortex", kind: "polar-vortex", lat_deg: 90, size_km: 4000 }),
    sys({ id: "big", lat_deg: -22, size_km: 12000, aspect: 1.5 }),
    sys({ id: "small", lat_deg: 30, size_km: 300 }),
  ];
  const asks = patchesFor(systems, A);
  assert.equal(asks.length, 2, JSON.stringify(asks));
  assert.deepEqual([asks[0]!.lat, asks[0]!.n], [90, POLE_N]);
  // The polygon is centred on its pole: its edge (12,000 km) and a margin are inside the patch.
  assert.ok(asks[0]!.half_km > 12000 && asks[0]!.half_km < 20000, `${asks[0]!.half_km}`);
  assert.deepEqual([asks[1]!.lat, asks[1]!.n], [-22, STORM_N]);
  // Three of its radii, stretched by its shear's aspect: 1.5 × 12,000 × √1.5.
  assert.ok(Math.abs(asks[1]!.half_km - 1.5 * 12000 * Math.sqrt(1.5)) < 1e-6);
  // Many storms: only the largest few.
  const many = Array.from({ length: 30 }, (_, k) => sys({ id: `o${k}`, lat_deg: -40 + 2 * k, lon_deg: 10 * k, size_km: 3000 + k }));
  const got = patchesFor(many, A);
  assert.equal(got.length, MAX_STORM_PATCHES);
  assert.equal(got[0]!.lat, -40 + 2 * 29);
});

test("a patch's finer cells are laid over the face: a bright storm shows where it is, and nothing changes elsewhere", () => {
  const n = 16;
  // A bright core inside the patch, dull round it.
  const cells = Array.from({ length: n * n }, (_, k) => {
    const x = (k % n) - (n - 1) / 2;
    const y = Math.floor(k / n) - (n - 1) / 2;
    return cell(Math.hypot(x, y) < 3 ? 1 : 0);
  });
  const p: FieldPatch = { lat_deg: 20, lon_deg: 0, half_km: 9000, n, cells };
  const d = detailGiant(grey(), undefined, facts([], [p]), colours);
  assert.equal(d.tex.width, GIANT_W);
  assert.ok(at(d.tex, 20, 0) > at(d.tex, 20, 90) + 40, `${at(d.tex, 20, 0)} vs ${at(d.tex, 20, 90)}`);
  // Far away, the face is the base's (128 in sRGB, give or take the rounding).
  assert.ok(Math.abs(at(d.tex, -50, 120) - 128) <= 1);
  assert.match(d.said.join(" "), /read closer \(1 patch\)/);
});

test("a storm is wound the way its own winds go round it, and a still one is not wound", () => {
  const n = 24;
  const half = 9000e3;
  const R = 3000e3;
  // Cells: bright east of the centre, dark west, and winds round the centre (anticlockwise seen from above at +1).
  const build = (sense: number) =>
    Array.from({ length: n * n }, (_, k) => {
      const x = (((k % n) + 0.5) / n) * 2 * half - half;
      const y = ((Math.floor(k / n) + 0.5) / n) * 2 * half - half;
      const r = Math.hypot(x, y) / R;
      const v = 100 * r * Math.exp(0.5 * (1 - r * r));
      const s = Math.hypot(x, y) || 1;
      return cell(x > 0 ? 1 : 0, [(-sense * v * y) / s, (sense * v * x) / s]);
    });
  const storm = [sys({ lat_deg: 20, lon_deg: 0, size_km: 2 * R / 1000 })];
  // Bands whose shear (ζ about 5e-5 s⁻¹ at 20°) winds the core by about a radian: one shear time.
  const jets = { lat_deg: [10, 30], jet_speed_m_s: [0, 1182], jet_shear_m_s: [0, 0], eddy_speed_m_s: [0, 0], rhines_scale_m: [0, 0], deformation_radius_m: [1, 1] };
  const look = (sense: number) => detailGiant(grey(), undefined, { ...facts(storm, [{ lat_deg: 20, lon_deg: 0, half_km: half / 1000, n, cells: build(sense) }]), jets }, colours, { deep: [0.05, 0.05, 0.05] }).tex;
  // Just north of the centre, a little west: anticlockwise winds bring the bright east side round to it, clockwise
  // ones the dark west, and still air leaves it dark.
  const deg = Math.PI / 180;
  const north = 20 + (0.6 * R) / A / deg;
  const west = -(0.3 * R) / (A * Math.cos(20 * deg)) / deg;
  const acw = at(look(1), north, west);
  const cw = at(look(-1), north, west);
  const still = at(look(0), north, west);
  assert.ok(acw > still + 20 && cw <= still + 5, `anticlockwise ${acw}, still ${still}, clockwise ${cw}`);
});

test("the curls' share is the eddy speed over the eddy speed and the band's shear", () => {
  const f = curlFacts({ lat_deg: [-10, 10], jet_speed_m_s: [100, 100], jet_shear_m_s: [51, -17], eddy_speed_m_s: [17, 17], rhines_scale_m: [1e7, 1e7], deformation_radius_m: [2e6, 2e6] });
  assert.deepEqual(f.curl_share, [0.25, 0.5]);
});
