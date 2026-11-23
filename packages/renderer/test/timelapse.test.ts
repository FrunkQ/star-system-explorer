// Time-lapse layers: a small synthetic surface_frames reply (an 8-per-face cube
// sphere, three keyframes, two plates, an ice age and an eruption) read,
// bracketed, blended and coloured; the globe's samples recoloured; words.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { cubeDirection } from "../../ui-contract/src/index.ts";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { bracket, cellOf, eventsBetween, layerColour, layersFor, narrateFrame, onPlateEdge, pickAt, readFrames, recolour, seaLevelAt, valueAt, type RawFrames } from "../src/timelapse.ts";

const N = 8;
function fixture(b64 = false): RawFrames {
  const cells = 6 * N * N;
  const dirs: number[] = [];
  for (let f = 0; f < 6; f++)
    for (let j = 0; j < N; j++)
      for (let i = 0; i < N; i++) dirs.push(...cubeDirection(f, -1 + ((i + 0.5) * 2) / N, -1 + ((j + 0.5) * 2) / N));
  const frame = (t: number, ice: number, vent: number) => {
    const plate: number[] = [];
    const elev: number[] = [];
    const ice_m: number[] = [];
    const temp: number[] = [];
    const v: number[] = [];
    for (let c = 0; c < cells; c++) {
      const [x, y, z] = [dirs[c * 3]!, dirs[c * 3 + 1]!, dirs[c * 3 + 2]!];
      // Two plates split by a plane that turns with time (plates move).
      const ang = t / 1000;
      plate.push(x * Math.cos(ang) + y * Math.sin(ang) > 0 ? 0 : 1);
      elev.push(x > 0.3 ? 1500 : -3000);
      ice_m.push(Math.abs(z) > 0.8 ? ice : 0);
      temp.push(288 - 40 * z * z - (ice ? 10 : 0));
      v.push(x > 0.9 ? vent : 0);
    }
    const f: Record<string, unknown> = { t_myr: t, sea_level_m: ice ? -80 : 0, plate, continental: elev.map((e) => e > 0), elevation_m: elev, ice_m, temp_k: temp, vent: v };
    if (b64) {
      f.temp_k_b64 = Buffer.from(new Float32Array(temp).buffer).toString("base64");
      delete f.temp_k;
    }
    return f as RawFrames["frames"][number];
  };
  return {
    grid: { n: N, cells },
    dirs,
    frames: [frame(4500, 0, 0), frame(3000, 0, 1), frame(4000, 2000, 0)],
    events: [
      { t_myr: 3000, kind: "eruption", text: "Flood basalts", dir: [1, 0, 0], radius_km: 800, why: ["A plume head reached the base of the plate."] },
      { t_myr: 4000, kind: "ice-age", text: "Ice age begins" },
    ],
    pending: ["rain_m_yr", "runoff_m_yr", "ice-age onsets (dated only inside the orbit-history window)"],
  };
}

test("reads frames in time order, base64 or JSON alike, and finds cells by direction", () => {
  const tl = readFrames(fixture());
  assert.deepEqual(tl.frames.map((f) => f.t_myr), [3000, 4000, 4500]);
  for (const c of [0, 77, 200, 383]) assert.equal(cellOf(tl, tl.dirs[c * 3]!, tl.dirs[c * 3 + 1]!, tl.dirs[c * 3 + 2]!), c);
  const b = readFrames(fixture(true));
  assert.equal(b.frames[0]!.fields.temp_k![5], tl.frames[0]!.fields.temp_k![5]);
  assert.throws(() => readFrames({ ...fixture(), grid: { n: N, cells: 10 } }), /should have/);
});

test("smooth fields blend between keyframes; plates and heights take the nearer frame", () => {
  const tl = readFrames(fixture());
  const pole = cellOf(tl, 0, 0, 1);
  assert.equal(valueAt(tl, "ice_m", pole, 3500), 1000, "ice halfway between none and 2 km");
  assert.equal(valueAt(tl, "ice_m", pole, 5000), 0, "after the last frame: the last frame");
  const { w } = bracket(tl, 3250);
  assert.equal(w, 0.25);
  // Plates: no blend; the nearer frame's map.
  const c = cellOf(tl, -0.2, 0.97, 0);
  const at3000 = valueAt(tl, "plate", c, 3000);
  assert.equal(valueAt(tl, "plate", c, 3400), at3000);
  assert.equal(valueAt(tl, "plate", c, 3600), valueAt(tl, "plate", c, 4000));
  assert.equal(seaLevelAt(tl, 3750), -60);
  assert.equal(valueAt(tl, "rain_m_yr", c, 3500), null, "a field no frame carries is null, not invented");
});

test("layers: colours follow the fields, pending fields are named, plate edges found", () => {
  const tl = readFrames(fixture());
  const ls = layersFor(tl);
  assert.equal(ls.find((l) => l.spec.id === "rain")!.available, false);
  assert.deepEqual(ls.find((l) => l.spec.id === "rain")!.pending, ["rain_m_yr"]);
  assert.deepEqual(ls.find((l) => l.spec.id === "ice")!.pending, ["ice-age onsets (dated only inside the orbit-history window)"], "a pending note in words reaches its layer");
  assert.equal(ls.find((l) => l.spec.id === "ice")!.available, true);
  const pole = cellOf(tl, 0, 0, 1);
  const icy = layerColour(tl, "ice", pole, 4000);
  const bare = layerColour(tl, "ice", pole, 3000);
  assert.ok(icy[0] + icy[1] + icy[2] > bare[0] + bare[1] + bare[2] + 200, `ice is white: ${icy} vs ${bare}`);
  const vent = cellOf(tl, 1, 0, 0);
  const hot = layerColour(tl, "volcanism", vent, 3000);
  const calm = layerColour(tl, "volcanism", vent, 4500);
  assert.ok(hot[0] > calm[0] + 60 && hot[0] > hot[2] + 100, `an eruption glows: ${hot} vs ${calm}`);
  const warm = layerColour(tl, "temperature", cellOf(tl, 0, 1, 0), 4500);
  const cold = layerColour(tl, "temperature", pole, 4500);
  assert.ok(warm[0] > cold[0] && cold[2] > warm[2], `${warm} warmer than ${cold}`);
  // The plate boundary at t=3000 lies on the plane x = 0.
  assert.equal(onPlateEdge(tl, [0.02, 0.9998, 0], 3000, 0.3), true);
  assert.equal(onPlateEdge(tl, [0.9, 0.3, 0.3], 3000, 0.1), false);
});

test("pick and events: what is under the cursor, with the kernel's why", () => {
  const tl = readFrames(fixture());
  const p = pickAt(tl, [1, 0, 0], 3000);
  assert.equal(p.values.vent, 1);
  assert.equal(p.event?.kind, "eruption");
  assert.deepEqual(p.why, ["A plume head reached the base of the plate."]);
  assert.equal(pickAt(tl, [-1, 0, 0], 3000).event, null, "nothing happened on the far side");
  assert.deepEqual(eventsBetween(tl, 3500, 4200).map((e) => e.kind), ["ice-age"]);
});

test("recolour: every sample takes the layer's colour; past frames are flat-lit; words", () => {
  const tl = readFrames(fixture());
  const s = 4;
  const face = () => ({ s, rgb: new Uint8ClampedArray(s * s * 3), normal: new Float32Array(s * s * 3).fill(0.5), cover: new Uint8Array(s * s) });
  const p = { faces: [face(), face(), face(), face(), face(), face()] };
  const past = recolour(p, tl, "ice", 4000, cubeDirection);
  const top = past.faces[4]!;
  const mid = ((s / 2) * s + s / 2) * 3;
  assert.ok(top.rgb[mid]! > 200, `the pole is icy white: ${top.rgb.slice(mid, mid + 3)}`);
  assert.ok(past.faces[0]!.rgb[mid]! < 200, "the equator is not");
  const n0 = [top.normal[0]!, top.normal[1]!, top.normal[2]!];
  assert.ok(Math.abs(Math.hypot(...n0) - 1) < 1e-5, "past frames use the sphere's own normal");
  const now = recolour(p, tl, "land", 4500, cubeDirection);
  assert.equal(now.faces[0]!.normal, p.faces[0]!.normal, "today keeps the hill shading");
  const words = narrateFrame(tl, "ice", 4000, "Fixture");
  assert.match(words, /500 million years ago, ice and glaciers: land \d+% of the surface/);
  assert.doesNotMatch(words, /\.\.|\.,/);
  assert.match(words, /Ice age begins/);
});

test("the kernel's Earth: surface_frames read whole, every layer drawable, events placed, the same each time", async () => {
  const k = nodeKernel();
  try {
    const base = JSON.parse(readFileSync(new URL("../../../packs/base-physics/pack.json", import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "frames", seed: "frames" }, [base]);
    const r = await s.surfaceFrames({ fixture: "Earth", frames_max: 12 });
    assert.ok(r.ok, JSON.stringify(r));
    const tl = readFrames(r.frames as unknown as RawFrames);
    assert.ok(tl.frames.length >= 6 && tl.frames.length <= 12, `${tl.frames.length} frames`);
    assert.equal(tl.frames[tl.frames.length - 1]!.t_myr, 4500, "the last frame is today");
    for (const l of layersFor(tl)) assert.ok(l.available, `${l.spec.id} has its fields`);
    assert.ok(tl.events.some((e) => e.kind === "collision"), "plates collide in Earth's history");
    // Today: about 30% land, ice at the poles, the poles colder than the equator.
    const now = 4500;
    let land = 0;
    for (let c = 0; c < tl.cells; c++) if ((valueAt(tl, "elevation_m", c, now) ?? 0) >= seaLevelAt(tl, now)) land++;
    assert.ok(land / tl.cells > 0.15 && land / tl.cells < 0.5, `land share ${land / tl.cells}`);
    const pole = cellOf(tl, 0, 0, -1);
    const equator = cellOf(tl, 1, 0, 0);
    assert.ok((valueAt(tl, "temp_k", pole, now) ?? 0) < (valueAt(tl, "temp_k", equator, now) ?? 0) - 30);
    // Ice sheets at high latitude: the plates decide whether land sits on the pole itself, so look over both caps
    // (above 60 degrees) rather than at the one cell under the pole.
    let capIce = 0;
    for (let lat = 60; lat <= 90; lat += 3)
      for (let lon = 0; lon < 360; lon += 6)
        for (const sgn of [-1, 1]) {
          const la = (sgn * lat * Math.PI) / 180, lo = (lon * Math.PI) / 180;
          const c = cellOf(tl, Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la));
          capIce = Math.max(capIce, valueAt(tl, "ice_m", c, now) ?? 0);
        }
    assert.ok(capIce > 100, `Antarctic-like ice today (thickest cap ice ${capIce} m)`);
    const again = await s.surfaceFrames({ fixture: "Earth", frames_max: 12 });
    assert.ok(again.ok);
    assert.deepEqual(readFrames(again.frames as unknown as RawFrames).frames[3]!.fields.plate, tl.frames[3]!.fields.plate, "the same world each time");
  } finally {
    await k.close();
  }
});
