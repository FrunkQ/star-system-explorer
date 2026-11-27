// RW16a: nothing below the round size is drawn as a sphere. Every scene body of the Solar System below the size at
// which it pulls itself round gets its own lump from the kernel (`small_shape`, read by small-shape.ts), the same lump
// in a fresh kernel; every body above it stays round.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { readSol, solBatches, solSourceText } from "../src/sol.ts";
import { sceneFromResolved } from "../src/make.ts";
import { readShapes, shapeAsk } from "../src/small-shape.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;

async function solScene() {
  const k = nodeKernel();
  const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
  const sys = "u:kestrel/sys:sol";
  for (const b of solBatches(sheet, sys).batches) {
    const r = await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
    assert.equal(r.status, "applied");
  }
  const r = await s.resolvedSystem(sys);
  assert.ok(r.ok);
  return { k, s, sys, scene: sceneFromResolved(r.result, "The Solar System") };
}

test("every Solar System body below the round size is a lump, the same in a fresh kernel; the round ones stay round", async () => {
  const a = await solScene();
  const b = await solScene();
  try {
    const shapes = await readShapes(a.s, a.sys, a.scene.subjects);
    const asked = a.scene.subjects.filter((x) => shapeAsk(x));
    const small = asked.filter((x) => (shapeAsk(x)!.diameter_km ?? Infinity) < 300);
    console.log(`lumps: ${[...shapes.keys()].map((id) => a.scene.subjects.find((x) => x.address === id)?.label).join(", ")}`);
    assert.ok(small.length >= 2, `small bodies in Sol: ${small.map((x) => x.label).join(", ")}`);
    for (const x of small) {
      const f = shapes.get(x.address);
      assert.ok(f, `${x.label} (${shapeAsk(x)!.diameter_km} km) is drawn as a lump`);
      const [p, , r] = f.axes_m;
      assert.ok(r / p < 0.97 || f.relief > 0.01, `${x.label}: not a sphere (${f.axes_m}, relief ${f.relief})`);
    }
    for (const n of ["Earth", "Moon", "Jupiter", "Pluto"]) {
      const x = a.scene.subjects.find((y) => y.label === n)!;
      assert.equal(shapes.get(x.address), undefined, `${n} stays round`);
    }
    // The same address is the same lump in another kernel (the seed and the address, nothing else).
    const again = await readShapes(b.s, b.sys, b.scene.subjects);
    for (const [id, f] of shapes) assert.deepEqual(again.get(id)?.axes_m, f.axes_m, id);
  } finally {
    await a.k.close();
    await b.k.close();
  }
});
