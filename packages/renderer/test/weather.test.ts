// Weather on the Body globe (R2b part 1): the reflectance sums, reading the
// kernel's tiles, laying clouds over the ground, and the words.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { cubeDirection } from "../../ui-contract/src/index.ts";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { addLayers, cloudReflectance, narrateWeather, readWeather, slabReflectance, stormAt, weatherAt, withWeather, type WeatherCell, type WeatherSystem, type WeatherTile } from "../src/weather.ts";

const layer = (cover: number, tau: number, type = "stratus") => ({ cloud_cover: cover, cloud_type: type, condensate: cover > 0 ? "water" : "", optical_depth: tau, wind_m_s: [0, 0] as [number, number], turbulence: 0 });
const cell = (lat: number, lon: number, cover: number, tau: number): WeatherCell => ({ lat_deg: lat, lon_deg: lon, layers: [layer(cover, tau), layer(0, 0, "none"), layer(0, 0, "none")], precip_kind: cover > 0.5 ? "rain" : "none", precip_mm_day: cover > 0.5 ? 4 : 0, temp_offset_k: 0, lightning_km2_yr: 0, aerosols: [] });

test("a scattering slab: none at zero depth, rising toward one, and layers add like Bohren's two-stream", () => {
  assert.equal(slabReflectance(0, 0.85), 0);
  const thick = slabReflectance(100, 0.85);
  assert.ok(thick > 0.85 && thick < 1, `${thick}`);
  // A typical water cloud of depth 10 reflects about 43% (Bohren 1987's formula with g = 0.85).
  assert.ok(Math.abs(slabReflectance(10, 0.85) - 1.5 / 3.5) < 1e-12);
  // Nothing over the ground leaves the ground; a mirror under anything stays a mirror (no absorption).
  assert.equal(addLayers(0, 0.3), 0.3);
  assert.ok(Math.abs(addLayers(0.4, 1) - 1) < 1e-12);
  // Cloud over dark sea brightens it; cover scales the cloud.
  assert.ok(addLayers(0.43, 0.06) > 0.43);
  assert.ok(cloudReflectance(cell(0, 0, 0.5, 10)) < cloudReflectance(cell(0, 0, 1, 10)));
  assert.equal(cloudReflectance(cell(0, 0, 0, 10)), 0);
});

test("clouds whiten the prepared ground where the kernel puts them, and nowhere else", () => {
  // One face of 4 × 4 samples, dark ground; a cloud cell over the face centre, clear cells elsewhere.
  const s = 4;
  const face = { s, rgb: new Uint8ClampedArray(s * s * 3).fill(30), normal: new Float32Array(s * s * 3), albedo: new Float32Array(s * s).fill(0.06) };
  const p = { faces: [face, null, null, null, null, null] };
  const c0 = cubeDirection(0, 0, 0);
  const lat = (Math.asin(c0[2]) * 180) / Math.PI;
  const lon = (Math.atan2(c0[1], c0[0]) * 180) / Math.PI;
  const cells = [cell(lat, lon, 1, 20)];
  for (let la = -80; la <= 80; la += 20) for (let lo = -180; lo < 180; lo += 20) if (Math.abs(la - lat) + Math.abs(lo - lon) > 60) cells.push(cell(la, lo, 0, 0));
  const w = readWeather([{ key: { face: 0, level: 0, x: 0, y: 0 }, n: 1, cells } as WeatherTile], [], { day: 3 });
  const out = withWeather(p, w, cubeDirection);
  const f = out.faces[0]!;
  const mid = 1 * s + 1;
  assert.ok(f.albedo[mid]! > 0.5, `cloud over the centre: ${f.albedo[mid]}`);
  assert.ok(f.rgb[mid * 3]! > 150, "and it is drawn white");
  assert.ok(Math.abs(f.albedo[0]! - 0.06) < 1e-6, "the clear corner keeps its ground");
  assert.ok(Math.abs(face.albedo[mid]! - 0.06) < 1e-6, "the prepared samples are not changed");
  assert.match(narrateWeather(w, "Test"), /^Test, weather on day 3: cloud over about \d+% of the globe \(mostly stratus, made of water\); [\d.]+ mm of rain or snow a day on average, a wet day \(1 mm or more\) over \d+%\.$/);
});

test("a storm is found where its track has it on the day", () => {
  const s: WeatherSystem = { id: "c1", kind: "cyclone", lat_deg: 40, lon_deg: 0, track: [[1, 40, 0], [2, 42, 10], [3, 44, 20]], size_km: 2000, strength: 20, born: 1, ends: 3, why: "a storm track" };
  assert.deepEqual(stormAt(s, 2.5), { lat_deg: 42, lon_deg: 10 });
  const w = readWeather([{ key: {}, n: 1, cells: [cell(42, 10, 1, 10), cell(-42, -170, 0, 0)] }], [s], { day: 2.5 });
  const b = (42 * Math.PI) / 180;
  const a = (10 * Math.PI) / 180;
  const here = weatherAt(w, [Math.cos(b) * Math.cos(a), Math.cos(b) * Math.sin(a), Math.sin(b)], 6371);
  assert.equal(here?.storm?.id, "c1");
  assert.equal(weatherAt(w, [0, 0, -1], 6371)?.storm, undefined);
  assert.match(narrateWeather(w, ""), /Storms: a cyclone 2,000 km across at 42°N 10°E\./);
});

test("the kernel's Earth weather: cloudy but not overcast, storms named, the same each time", async () => {
  const k = nodeKernel();
  try {
    const base = JSON.parse(readFileSync(new URL("../../../packs/base-physics/pack.json", import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "wx", seed: "wx" }, [base]);
    const tiles: WeatherTile[] = [];
    for (const face of [0, 1, 2, 3, 4, 5]) {
      const r = await s.weather({ want: "field", world: "Earth", day: 100, tile: [face, 0, 0, 0], n: 8 });
      assert.ok(r.ok, JSON.stringify(r));
      tiles.push(r.data as unknown as WeatherTile);
    }
    const sy = await s.weather({ want: "systems", world: "Earth", day: 100 });
    assert.ok(sy.ok);
    const w = readWeather(tiles, sy.data as unknown as WeatherSystem[], { day: 100 });
    assert.equal(w.cells.length, 6 * 64);
    const mean = w.cloudR.reduce((a, b) => a + b, 0) / w.cells.length;
    // Earth's clouds reflect roughly a fifth to a third of sunlight (cloud albedo forcing; global albedo about 0.3).
    assert.ok(mean > 0.1 && mean < 0.5, `mean cloud reflectance ${mean}`);
    const words = narrateWeather(w, "Earth");
    assert.match(words, /cloud over about \d+% of the globe/);
    assert.match(words, /Storms: /);
    const again = await s.weather({ want: "field", world: "Earth", day: 100, tile: [0, 0, 0, 0], n: 8 });
    assert.ok(again.ok);
    assert.deepEqual(again.data, tiles[0], "the same weather each time");
  } finally {
    await k.close();
  }
});
