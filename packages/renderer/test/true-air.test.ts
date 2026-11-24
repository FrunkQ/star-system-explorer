// Air in True at system scale: a giant's face is its cloud decks (bands from the cells, colours from the sheet), and a
// rocky world's clouds and haze lie over its ground.
import { test } from "node:test";
import assert from "node:assert/strict";
import { nameKey, sheetColours, trueGiant, withAir, type SheetSubstance } from "../src/true-air.ts";
import type { WeatherCell, WeatherTile } from "../src/weather.ts";

const layer = (condensate: string, cover: number, tau: number) => ({ cloud_cover: cover, cloud_type: "deck", condensate, optical_depth: tau, wind_m_s: [0, 0] as [number, number], turbulence: 0 });
const cell = (lat: number, lon: number, layers: ReturnType<typeof layer>[], aerosols: WeatherCell["aerosols"] = []): WeatherCell => ({ lat_deg: lat, lon_deg: lon, layers, precip_kind: "none", precip_mm_day: 0, temp_offset_k: 0, lightning_km2_yr: 0, aerosols });

/** A grid of cells every 10°, from a rule per latitude. */
function tiles(rule: (lat: number) => WeatherCell["layers"], aerosols?: (lat: number) => WeatherCell["aerosols"]): WeatherTile[] {
  const cells: WeatherCell[] = [];
  for (let lat = -85; lat <= 85; lat += 10) for (let lon = -175; lon <= 175; lon += 10) cells.push(cell(lat, lon, rule(lat), aerosols?.(lat) ?? []));
  return [{ key: { face: 0, level: 0, x: 0, y: 0 }, n: 0, cells }];
}

const SHEET: SheetSubstance[] = [
  { id: "nh3", names: { physics: "ammonia", plain: "ammonia" }, optical: { condensed_colour_rgb: { v: [0.98, 0.96, 0.9] } } },
  { id: "h2s", names: { plain: "hydrogen sulfide" } },
];

test("condensate names match the sheet's names, whatever the phase word or spelling", () => {
  assert.equal(nameKey("Ammonia ice"), "ammonia");
  assert.equal(nameKey("hydrogen sulphide ice"), "hydrogen sulfide");
  const c = sheetColours(SHEET);
  assert.deepEqual(c("ammonia ice"), [0.98, 0.96, 0.9]);
  // On the sheet but with no colour: none, not invented.
  assert.equal(c("hydrogen sulphide ice"), null);
  assert.equal(c("unobtainium"), null);
});

test("a giant's zones (thick high deck) are brighter than its belts, and its bands follow latitude", () => {
  // Zones between ±20° and poleward of 40°, a belt between: as the weather op's cells give them.
  const zone = (lat: number) => Math.abs(lat) < 20 || Math.abs(lat) > 40;
  const look = trueGiant(
    tiles((lat) => [layer("water", 0.6, 10), layer("ammonium hydrosulphide", 0.8, 4), zone(lat) ? layer("ammonia ice", 0.95, 6) : layer("ammonia ice", 0.55, 1.5)]),
    sheetColours(SHEET),
    64,
    32,
  )!;
  const { width, data } = look.tex;
  // Row j's latitude is 90 - 180(j + ½)/32, north at the last row.
  const lum = (j: number) => {
    let s = 0;
    for (let i = 0; i < width; i++) s += data[(j * width + i) * 4 + 1]!;
    return s / width;
  };
  const rowAt = (lat: number) => Math.round(((lat + 90) / 180) * 32 - 0.5);
  assert.ok(lum(rowAt(0)) > lum(rowAt(30)) + 5, `zone ${lum(rowAt(0))} belt ${lum(rowAt(30))}`);
  assert.ok(Math.abs(lum(rowAt(30)) - lum(rowAt(-30))) < 2, "the bands mirror about the equator");
  // Ammonia's sheet colour is warm: red at least blue on the zone.
  const o = (rowAt(0) * width + 10) * 4;
  assert.ok(data[o]! >= data[o + 2]!);
  // NH4SH has no colour on the sheet: said, and drawn white.
  assert.ok(look.said.some((s) => s.includes("ammonium hydrosulphide")), look.said.join(" "));
});

test("a haze is laid over the ground in its own colour, and clear air leaves the ground as it was", () => {
  const ground = { width: 16, height: 8, data: new Uint8Array(16 * 8 * 4).fill(100), mean: [100, 100, 100] as [number, number, number] };
  const clear = withAir(ground, tiles(() => [layer("water", 0, 0)]), sheetColours(SHEET));
  assert.equal(clear.tex.data[0], 100);
  const hazy = withAir(ground, tiles(() => [layer("water", 0, 0)], () => [{ kind: "organic haze", optical_depth: 5, colour: [0.85, 0.55, 0.25] }]), sheetColours(SHEET));
  const [r, g, b] = [hazy.tex.data[0]!, hazy.tex.data[1]!, hazy.tex.data[2]!];
  assert.ok(r > g && g > b, `orange haze: ${r} ${g} ${b}`);
  assert.equal(hazy.said.length, 0);
});

test("a world with no weather cells keeps its ground; a giant with none has no face", () => {
  const ground = { width: 4, height: 2, data: new Uint8Array(32).fill(7), mean: [7, 7, 7] as [number, number, number] };
  assert.equal(withAir(ground, [], sheetColours([])).tex, ground);
  assert.equal(trueGiant([], sheetColours([])), null);
});
