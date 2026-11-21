// The relief passes' shared numbers: shading normals tilt with the real
// slope of the elevation layer and liquid is flat; the narration (text tier)
// describes the face in view from the kernel's tiles for adopted Earth.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { sceneFromResolved } from "../../ui-contract/src/make.ts";
import type { Globe, SurfaceTile } from "../../ui-contract/src/globe.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { fastFace, lightDir, lit, lookup, lookupFast, narrate, prepare, viewRotation } from "../src/surface.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");

/** One synthetic face-0 tile: a ramp rising along i (east), with a sea on the low third. */
function rampTile(): SurfaceTile {
  const s = 32;
  const n = s * s;
  const elevation_dm: number[] = [];
  const cover: number[] = [];
  for (let j = 0; j < s; j++)
    for (let i = 0; i < s; i++) {
      elevation_dm.push((i - 10) * 2000);
      cover.push(i < 10 ? 1 : 0);
    }
  return { address: "t", key: { face: 0, level: 0, x: 0, y: 0 }, samples: s, spacing_m: 4_000, epoch_myr: 0, elevation_dm, material: Array(n).fill(0), cover, albedo: Array(n).fill(300), slope_ddeg: Array(n).fill(0), surface_age_myr: Array(n).fill(0), pending: [], hash: "" };
}

test("shading normals: unit length, tilted against the slope, flat on liquid", () => {
  const p = prepare([rampTile()], "survey", 10);
  const f = p.faces[0]!;
  for (let k = 0; k < f.s * f.s; k++) assert.ok(Math.abs(Math.hypot(f.normal[k * 3]!, f.normal[k * 3 + 1]!, f.normal[k * 3 + 2]!) - 1) < 1e-5);
  // Land at the tile's middle row: ground rises along +u (face 0's right is +y), so the normal leans to -y.
  const k = 16 * 32 + 20;
  const radialY = Math.sin(Math.atan(Math.tan((((20.5 / 32) * 2 - 1) * Math.PI) / 4)));
  assert.ok(f.normal[k * 3 + 1]! < radialY - 0.3, `leans away from the rise: ${f.normal[k * 3 + 1]} against radial ${radialY}`);
  // Sea: radial.
  const w = 16 * 32 + 3;
  const r = [f.normal[w * 3]!, f.normal[w * 3 + 1]!, f.normal[w * 3 + 2]!];
  assert.ok(r[0]! > 0.7 && Math.abs(r[2]!) < 0.1, "liquid keeps the radial normal");
  // More exaggeration, more tilt.
  assert.ok(prepare([rampTile()], "survey", 30).faces[0]!.normal[k * 3 + 1]! < f.normal[k * 3 + 1]!);
});

test("a slope facing the light is brighter than one facing away", () => {
  const p = prepare([rampTile()], "survey", 10);
  const rot = viewRotation(0, 0); // face 0 (+x) towards the viewer; +y is to the right on screen
  const k = 16 * 32 + 20;
  const f = p.faces[0]!;
  const nv = rot.toView(f.normal[k * 3]!, f.normal[k * 3 + 1]!, f.normal[k * 3 + 2]!);
  const fromLeft = lightDir(270, 40);
  const fromRight = lightDir(90, 40);
  const d = (L: number[]) => nv[0] * L[0]! + nv[1] * L[1]! + nv[2] * L[2]!;
  assert.ok(lit(d(fromLeft), 0.8, 0.16) > lit(d(fromRight), 0.8, 0.16) + 0.1, "the slope faces left (it rises to the right)");
  assert.equal(lit(-1, 0.8, 0.16), 1 - 0.8 + 0.8 * 0.16, "the night side keeps the ambient share");
});

test("the view rotation inverts cleanly", () => {
  const r = viewRotation(73, -28);
  const w = r.toWorld(0.3, -0.2, Math.sqrt(1 - 0.13));
  const back = r.toView(...w);
  assert.ok(Math.abs(back[0] - 0.3) < 1e-12 && Math.abs(back[1] + 0.2) < 1e-12);
});

test("Earth from the kernel, narrated: water, highs and lows in view, and where they lie", async () => {
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const a = await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") });
    assert.ok(a.ok);
    const sys = a.result.systems[0]!;
    const r = await s.resolvedSystem(sys);
    assert.ok(r.ok);
    const earth = sceneFromResolved(r.result).subjects.find((b) => b.label === "Earth")!;
    const t = await s.surfaceTiles(sys, earth.address);
    assert.ok(t.ok);
    const globe: Globe = [0, 1, 2, 3, 4, 5].map((f) => t.tiles.find((x) => x.key.face === f));
    const p = prepare(globe, "survey", 12);
    assert.equal(p.samples, globe.reduce((n, x) => n + x!.samples * x!.samples, 0));
    assert.ok(p.highest_m > 0);
    const full = narrate(p, "Earth", 0, 20, "full", lightDir(315, 40));
    // The Orbit globe carries cover (S16a part 2), so polar and sea ice may be named.
    assert.match(full, /^Earth, the face centred on 20° N, 0° E: \d+% under water, (\d+% ice, )?the rest bare ground\. The highest ground in view, [\d.]+ km, lies (to the|near)/);
    assert.match(full, /Light falls from the upper left\.$/);
    assert.equal(narrate(p, "Earth", 0, 20, "brief").split(". ").length, 1);
    // Turning changes what is in view.
    assert.notEqual(narrate(p, "Earth", 180, -20, "full"), full);
    // Every direction finds a sample.
    for (const d of [[1, 0, 0], [0, 0, -1], [0.577, -0.577, 0.577]] as const) assert.ok(lookup(p, d));
  } finally {
    k.close();
  }
});

test("the per-pixel lookup finds the same sample as the plain one", () => {
  const p = prepare([0, 1, 2, 3, 4, 5].map((f) => ({ ...rampTile(), key: { face: f, level: 0, x: 0, y: 0 } })), "survey", 10);
  let seed = 5;
  const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648) * 2 - 1;
  for (let n = 0; n < 5000; n++) {
    const v = [rnd(), rnd(), rnd()];
    const l = Math.hypot(...v);
    const d = [v[0]! / l, v[1]! / l, v[2]! / l] as const;
    const a = lookup(p, d)!;
    const k = lookupFast(p, ...d);
    assert.equal(k, a.k);
    assert.equal(p.faces[fastFace], a.f);
  }
});
