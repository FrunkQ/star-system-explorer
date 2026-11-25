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
import { bracket, cellOf, eventsBetween, iceAgeAt, iceAges, iceAgeWords, layerColour, layersFor, narrateFrame, onPlateEdge, pickAt, readFrames, recolour, seaLevelAt, underSea, valueAt, type RawFrames } from "../src/timelapse.ts";

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

test("a dry world has no sea, and ice ages get a window of their own with the kernel's cells", () => {
  // Dry: the kernel sends sea_level_m null; low ground stays land, never ocean.
  const dry = fixture();
  for (const f of dry.frames) f.sea_level_m = null;
  const td = readFrames(dry);
  assert.ok(td.frames.every((f) => f.dry));
  const low = cellOf(td, -1, 0, 0);
  assert.ok((valueAt(td, "elevation_m", low, 4500) ?? 0) < 0, "the fixture's west is low ground");
  assert.equal(underSea(td, low, 4500), false);
  const c = layerColour(td, "land", low, 4500);
  assert.ok(c[2] < c[1] + 20, `low ground on a dry world is not sea blue: ${c}`);
  assert.equal(pickAt(td, [-1, 0, 0], 4500).sea, false);
  assert.match(narrateFrame(td, "land", 4500, "Dry"), /no sea/);
  // Ice ages: three onsets in the last 100 thousand years, the latest with the cells where ice starts.
  const wet = fixture();
  const cold = [cellOf(td, 0.6, 0, 0.8), cellOf(td, 0.6, 0.1, 0.79)];
  wet.events = [
    ...wet.events!,
    { t_myr: 4500 - 0.09, kind: "ice-age", text: "An ice age begins: ice starts to grow on 2.0% of the land.", why: ["The orbit's swing."] },
    { t_myr: 4500 - 0.05, kind: "ice-age", text: "An ice age begins: ice starts to grow on 1.0% of the land." },
    { t_myr: 4500 - 0.02, kind: "ice-age", text: "An ice age begins: ice starts to grow on 3.0% of the land.", cells: cold },
  ];
  const tl = readFrames(wet);
  const ia = iceAges(tl)!;
  assert.equal(ia.events.length, 4, "the fixture's older onset counts too");
  // The fixture's 4000 Myr onset stretches the window; without it the window is a tidy 100 thousand years.
  const recent = readFrames({ ...wet, events: wet.events.filter((e) => e.t_myr > 4400) });
  const r = iceAges(recent)!;
  assert.equal(r.to_myr, 4500);
  assert.ok(Math.abs(r.to_myr - r.from_myr - 0.1) < 1e-9, `window ${r.to_myr - r.from_myr} Myr`);
  assert.equal(iceAgeAt(recent, 4500 - 0.03)?.text, recent.events[1]!.text);
  assert.equal(iceAgeAt(recent, 4500 - 0.5), null, "before the window");
  assert.match(iceAgeWords(recent, recent.events[2]!), /^Ice age 3 of 3 in the last 100 thousand years, begun 20 thousand years ago: ice starts to grow on 3\.0% of the land\.$/);
  assert.match(narrateFrame(recent, "ice", 4500 - 0.01, "Fixture"), /10 thousand years ago, .*Ice age 3 of 3/);
  // Where the kernel names the cells, the ice layer shows ice starting there during that ice age, not before.
  const during = layerColour(recent, "ice", cold[0]!, 4500 - 0.01);
  const before = layerColour(recent, "ice", cold[0]!, 4500 - 0.03);
  assert.ok(during[2] > before[2] + 20, `growing ice is paler: ${during} vs ${before}`);
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

test("the Solar System's Earth: the kernel's ice ages read into their own window, and each frame's climate reads its own land", async () => {
  const { readSol, solBatches, solSourceText } = await import("../../ui-contract/src/sol.ts");
  const text = (p: string) => readFileSync(new URL(`../../../${p}`, import.meta.url), "utf8");
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
    const s = await Session.open(k.client, gmSeat(), { id: "lapse", seed: "lapse" }, packs);
    const sys = "u:lapse/sys:sol";
    const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;
    const { batches, addresses } = solBatches(sheet, sys);
    for (const b of batches) {
      const r = await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
      assert.equal(r.status, "applied", `${b.source}: ${JSON.stringify(r)}`);
    }
    const r = await s.surfaceFrames({ system: sys, body: addresses.earth!, detail: "orbit" });
    assert.ok(r.ok, JSON.stringify(r));
    const tl = readFrames(r.frames as unknown as RawFrames);
    const ia = iceAges(tl);
    assert.ok(ia, `ice ages dated (pending: ${tl.pending.join("; ")})`);
    assert.ok(ia.to_myr - ia.from_myr < 10, `the ice-age window is recent: ${ia.to_myr - ia.from_myr} Myr`);
    assert.ok(ia.events.every((e) => e.t_myr >= ia.from_myr && e.t_myr <= ia.to_myr));
    const last = ia.events[ia.events.length - 1]!;
    // The newest ice age may have begun "today" (the frame at the epoch) or some thousand years ago.
    const told = narrateFrame(tl, "ice", last.t_myr, "Earth");
    assert.match(told, /Ice age \d+ of \d+/);
    assert.match(told, /thousand years ago|begun today/);
    assert.ok(tl.frames.every((f) => !f.dry && f.why.length), "Earth has a sea in every frame, and a line for each");
    // Each frame's climate reads that frame's own land (PR 394): the first frame is an ocean world, and its climate
    // differs from today's. The catalogued Earth's measured temperature is today's, and its history follows the Sun's
    // light (SF1 part 3): the faint young Sun makes the first frame colder, and the temperature layer is not pending.
    const landShare = (f: (typeof tl.frames)[number]) => f.fields.elevation_m!.filter((z) => z >= f.sea_level_m).length / tl.cells;
    const mean = (f: (typeof tl.frames)[number]) => f.fields.temp_k!.reduce((a, b) => a + b, 0) / tl.cells;
    const first = tl.frames[0]!;
    const now = tl.frames[tl.frames.length - 1]!;
    assert.ok(landShare(first) < 0.05 && landShare(now) > 0.15, `land ${landShare(first)} then, ${landShare(now)} now`);
    assert.ok(Math.abs(mean(first) - mean(now)) > 1, `an ocean world's climate is its own: ${mean(first)} K then, ${mean(now)} K now`);
    const temp = layersFor(tl).find((l) => l.spec.id === "temperature")!;
    assert.ok(!temp.pending.some((n) => /fixed value/.test(n)), temp.pending.join("; "));
    assert.ok(mean(first) < mean(now) - 5, `the faint young Sun: ${mean(first)} K then, ${mean(now)} K now`);
  } finally {
    await k.close();
  }
});
