// Air in True at system scale: a giant's face is its cloud decks (bands from the cells, colours from the sheet), and a
// rocky world's clouds and haze lie over its ground.
import { test } from "node:test";
import assert from "node:assert/strict";
import { flowFace, nameKey, sheetColours, trueGiant, withAir, type SheetSubstance } from "../src/true-air.ts";
import type { TrueTexture } from "../src/true-look.ts";
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

// ------------------------------------------------------------------ carried by the winds (VP-A1)

const JUP_A = 7.0e7;
/** Cells with a deck everywhere, a wind rule per place, and a dark spot (a thin deck) where `spot` says. */
function windy(u: (lat: number) => number, spot?: { lat: number; lon: number; r_deg: number; v: number }): WeatherTile[] {
  const cells: WeatherCell[] = [];
  for (let lat = -87.5; lat <= 87.5; lat += 5)
    for (let lon = -177.5; lon <= 177.5; lon += 5) {
      let we = u(lat);
      let wn = 0;
      let tau = 6;
      if (spot) {
        const dx = (lon - spot.lon) * Math.cos((lat * Math.PI) / 180);
        const dy = lat - spot.lat;
        const r = Math.hypot(dx, dy) / spot.r_deg;
        if (r < 2) {
          // Anticlockwise, solid in the core.
          const vt = spot.v * r * Math.exp(0.5 * (1 - r * r));
          const d = Math.hypot(dx, dy) || 1;
          we += -vt * (dy / d);
          wn += vt * (dx / d);
        }
        // A dark east half inside the spot, so turning shows.
        if (r < 1 && dx > 0) tau = 0.3;
      }
      const l = { ...layer("ammonia ice", 0.95, tau), wind_m_s: [we, wn] as [number, number] };
      cells.push(cell(lat, lon, [l]));
    }
  return [{ key: { face: 0, level: 0, x: 0, y: 0 }, n: 0, cells }];
}

test("each band slides east at its own jet's speed: a fast equator laps a still band", () => {
  // A stripe of dark cloud along one meridian, at every latitude; the equator's wind is 100 m/s, the rest still.
  const look = trueGiant(windy((lat) => (Math.abs(lat) < 10 ? 100 : 0)), sheetColours(SHEET), 72, 36)!;
  assert.ok(look.flow, "a giant whose cells carry wind has a flow");
  const f = look.flow!;
  const eq = Math.round(f.height / 2);
  assert.ok(f.rowEast[eq]! > 80 && Math.abs(f.rowEast[rowAtLat(f.height, 45)]!) < 5, `${f.rowEast[eq]} ${f.rowEast[rowAtLat(f.height, 45)]}`);
  // Mark a meridian: paint column 10 dark in the face, then let it run for one day.
  const face = { ...look.tex, data: Uint8Array.from(look.tex.data) };
  for (let j = 0; j < face.height; j++) face.data.set([0, 0, 0, 255], (j * face.width + 10) * 4);
  const day = 86400;
  const later = flowFace(face, f, { radius_m: JUP_A, dt_s: day });
  const darkest = (j: number) => {
    let best = 0;
    let lo = 999;
    for (let i = 0; i < face.width; i++) if (later.data[(j * face.width + i) * 4 + 1]! < lo) (lo = later.data[(j * face.width + i) * 4 + 1]!), (best = i);
    return best;
  };
  // 100 m/s for a day round a 70 000 km equator: 100 × 86400 / 7e7 rad, about 7.1°, about 1.4 columns of 5°.
  const moved = ((darkest(eq) - 10) * 360) / face.width;
  const expect = (f.rowEast[eq]! * day) / JUP_A / (Math.PI / 180) / Math.cos(((eq + 0.5) / f.height) * Math.PI - Math.PI / 2);
  assert.ok(Math.abs(moved - expect) <= 360 / face.width, `moved ${moved}°, expected ${expect}°`);
  assert.equal(darkest(rowAtLat(f.height, 45)), 10, "the still band stays");
  // No time, no change.
  assert.deepEqual(flowFace(face, f, { radius_m: JUP_A, dt_s: 0 }).data, face.data);
});

test("a storm moves as one piece and turns the way its winds go round it", () => {
  const spot = { lat: -20, lon: 0, r_deg: 10, v: 120 };
  const look = trueGiant(windy(() => 0, spot), sheetColours(SHEET), 144, 72)!;
  const f = look.flow!;
  const storm = { lat_deg: spot.lat, lon_deg: spot.lon, size_km: 2 * spot.r_deg * (Math.PI / 180) * JUP_A * Math.cos((spot.lat * Math.PI) / 180) / 1000 };
  // The winds go round at v / R at the storm's edge; the drawing turns in cycles of a quarter turn there, and half
  // way through a cycle only that cycle shows.
  const R = storm.size_km * 500;
  const half = (0.5 * (Math.PI / 2) * R) / spot.v;
  const grey = (t: TrueTexture, lat: number, lon: number) => {
    const i = Math.floor(((lon + 180) / 360) * t.width);
    const j = Math.floor(((lat + 90) / 180) * t.height);
    return t.data[(j * t.width + i) * 4 + 1]!;
  };
  const off = 5;
  const east0 = grey(look.tex, spot.lat, spot.lon + off);
  const north0 = grey(look.tex, spot.lat + off, spot.lon);
  assert.ok(east0 < north0 - 10, `dark east half at the start: ${east0} vs ${north0}`);
  // Anticlockwise: the dark east half has gone round toward the north-east.
  const later = flowFace(look.tex, f, { radius_m: JUP_A, dt_s: half, storms: [storm] });
  // At 120° round from east (west of north, bright at the start) it is now dark; at -60° (south of east, dark at
  // the start) it is now bright: the dark half went round anticlockwise, by about 65° this far out.
  const at = (t: TrueTexture, deg: number) => grey(t, spot.lat + off * Math.sin((deg * Math.PI) / 180), spot.lon + (off * Math.cos((deg * Math.PI) / 180)) / Math.cos((spot.lat * Math.PI) / 180));
  assert.ok(at(later, 120) < at(look.tex, 120) - 10, `went dark: ${at(look.tex, 120)} to ${at(later, 120)}`);
  assert.ok(at(later, -60) > at(look.tex, -60) + 10, `went bright: ${at(look.tex, -60)} to ${at(later, -60)}`);
  // Away from the storm nothing changes.
  assert.equal(grey(later, 40, 120), grey(look.tex, 40, 120));
  // In a 60 m/s wind a still storm, dark half and all, is carried east by u Δt / (a cos φ) and keeps its shape.
  const still = { ...spot, v: 0 };
  const carried = trueGiant(windy(() => 60, still), sheetColours(SHEET), 144, 72)!;
  // Four pixels' worth (10°), so the picture is compared pixel for pixel.
  const dlon = 10;
  const dt = (((dlon * Math.PI) / 180) * JUP_A * Math.cos((spot.lat * Math.PI) / 180)) / 60;
  const moved = flowFace(carried.tex, carried.flow!, { radius_m: JUP_A, dt_s: dt, storms: [storm] });
  for (const [dy, dx] of [[0, 5], [6, -4], [-6, 4]] as const)
    assert.ok(Math.abs(grey(moved, spot.lat + dy, spot.lon + dx + dlon) - grey(carried.tex, spot.lat + dy, spot.lon + dx)) <= 6, `carried ${dlon.toFixed(1)}° east at ${dy}, ${dx}: ${grey(moved, spot.lat + dy, spot.lon + dx + dlon)} vs ${grey(carried.tex, spot.lat + dy, spot.lon + dx)}`);
});

const rowAtLat = (h: number, lat: number) => Math.min(h - 1, Math.floor(((lat + 90) / 180) * h));

test("the one True door carries a giant's face by its winds, and the size comparison's watch is the body's own time", async () => {
  const { trueLook } = await import("../src/true-body.ts");
  const { bodySeconds, SECONDS_PER_TURN } = await import("../src/lineup-3d.ts");
  const look = trueGiant(windy((lat) => (Math.abs(lat) < 10 ? 100 : 0)), sheetColours(SHEET), 72, 36)!;
  // A dark meridian, so the slide shows.
  const tex = { ...look.tex, data: Uint8Array.from(look.tex.data) };
  for (let j = 0; j < tex.height; j++) tex.data.set([0, 0, 0, 255], (j * tex.width + 10) * 4);
  const base = { key: "j", star: false, colour: "#888", radius_m: JUP_A, surface: tex, pole: [0, 0, 1] as [number, number, number] };
  assert.equal(trueLook(base).surface!.texture, tex, "no flow: the face as read");
  const moved = trueLook({ ...base, flow: { flow: look.flow!, dt_s: 86400 } }).surface!;
  assert.equal(moved.version, 86400);
  assert.notDeepEqual(moved.texture.data, tex.data);
  // One turn watched is one of the body's days at "same" pace; at "true" pace the quickest body's day.
  assert.equal(bodySeconds(9.9, SECONDS_PER_TURN, "same", 9.9), 9.9 * 3600);
  assert.equal(bodySeconds(24, SECONDS_PER_TURN, "true", 9.9), 9.9 * 3600);
  assert.equal(bodySeconds(undefined, 10, "true", 9.9), 0);
});
