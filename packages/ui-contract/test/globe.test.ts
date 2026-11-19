// V6 part 4 in Node: the globe's cube-sphere mapping matches the kernel's
// (direction and face_of round trip; the sample a direction falls in is the
// sample whose centre the kernel built), and the sample Sol's Earth, adopted
// into the campaign, comes back as six level-0 tiles that say it is mostly
// under water. A fixture-only body (Kestrel's Tarn) has no tiles.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { direction, faceOf, globeStats, sampleAt, sampleColour, viewToWorld, type Globe } from "../src/globe.ts";
import { sceneFromResolved } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];

test("direction and faceOf round trip on every face", () => {
  for (let f = 0; f < 6; f++)
    for (const u of [-0.9, -0.3, 0, 0.45, 0.8])
      for (const v of [-0.7, 0, 0.6]) {
        const d = direction(f, u, v);
        assert.ok(Math.abs(Math.hypot(...d) - 1) < 1e-12);
        const back = faceOf(d);
        assert.equal(back.face, f);
        assert.ok(Math.abs(back.u - u) < 1e-9 && Math.abs(back.v - v) < 1e-9, `face ${f} (${u}, ${v}) -> (${back.u}, ${back.v})`);
      }
});

test("the view's centre faces the turned longitude; the edge of the disk is off the globe", () => {
  const c = viewToWorld(0, 0, 0, 0)!;
  assert.ok(Math.abs(c[0] - 1) < 1e-12 && Math.abs(c[2]) < 1e-12);
  const q = viewToWorld(0, 0, 90, 0)!;
  assert.ok(Math.abs(q[1] - 1) < 1e-12);
  const pole = viewToWorld(0, 0, 0, 90)!;
  assert.ok(Math.abs(pole[2] - 1) < 1e-12, "tipped 90° the pole faces the viewer");
  assert.equal(viewToWorld(0.8, 0.8, 0, 0), null);
});

test("adopted Earth reads back as six tiles, mostly under water; Tarn has none; players get none", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const a = await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") });
    assert.ok(a.ok, JSON.stringify(a));
    const sys = a.result.systems[0]!;
    const r = await s.resolvedSystem(sys);
    assert.ok(r.ok);
    const earth = sceneFromResolved(r.result).subjects.find((b) => b.label === "Earth");
    assert.ok(earth, "Earth in the resolved system");
    const t = await s.surfaceTiles(sys, earth.address);
    assert.ok(t.ok, JSON.stringify(t).slice(0, 300));
    assert.equal(t.tiles.length, 6);
    assert.ok(t.regime, "the surface regime is named");
    const globe: Globe = [0, 1, 2, 3, 4, 5].map((f) => t.tiles.find((x) => x.key.face === f));
    assert.ok(globe.every((x) => x && x.elevation_dm.length === x.samples * x.samples));
    const st = globeStats(globe);
    assert.ok((st.covered.water ?? 0) > 0.4, `Earth is mostly under water: ${JSON.stringify(st.covered)}`);
    assert.ok(st.highest_m > 0 && st.lowest_m < 0);
    // A sample's centre direction falls back in that sample.
    const tile = globe[2]!;
    const n = tile.samples;
    for (const [i, j] of [[0, 0], [n - 1, 3], [7, n - 1], [n >> 1, n >> 1]] as const) {
      const d = direction(2, ((i + 0.5) / n) * 2 - 1, ((j + 0.5) / n) * 2 - 1);
      const hit = sampleAt(globe, d)!;
      assert.equal(hit.tile, tile);
      assert.equal(hit.i, j * n + i);
      const rgb = sampleColour(hit.tile, hit.i, 0.8);
      assert.ok(rgb.every((c) => c >= 0 && c <= 255));
    }

    const tarn = await s.surfaceTiles("u:kestrel/sys:1", "u:kestrel/sys:1/body:3");
    assert.ok(!tarn.ok || tarn.tiles.length === 0, "a fixture-only body has no surface tiles");

    const p = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs).catch(() => null);
    if (p) assert.equal((await p.surfaceTiles(sys, earth.address)).ok, false);
  } finally {
    k.close();
  }
});
