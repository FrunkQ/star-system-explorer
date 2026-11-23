// V3 gap 2: Add here offers what an orbit would make, sampled from formation runs, or anything, marked impossible.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { formsHere, anythingClip, clipAt, freeOrbit, placeClip } from "../src/index.ts";
import { AU_M } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";

const base = JSON.parse(readFileSync(new URL("../../../packs/base-physics/pack.json", import.meta.url), "utf8"));

test("what forms near 1 AU round a Sun-like star: real worlds from formation runs, the same each time", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [base]);
    const h0 = await s.stateHash();
    const t0 = performance.now();
    const a = await formsHere(s, { system: "u:kestrel/sys:x", star_mass_sun: 1, a_au: 1 });
    const ms = performance.now() - t0;
    assert.ok(a.ok, JSON.stringify(a));
    console.log(`formsHere: ${a.found.length} of ${a.runs} runs made a world near 1 AU, in ${ms.toFixed(0)} ms`, a.found.map((f) => `${f.words} ${f.formed_au.toFixed(2)} AU ${f.mass_earth.toFixed(2)} ME`));
    for (const f of a.found) {
      assert.ok(f.formed_au > 1 / 1.6 && f.formed_au < 1.6);
      assert.equal(f.clip.nodes.length, 1, "the world alone");
      assert.ok(f.clip.events.every((e) => !e.impossible), "what the physics made is not impossible");
    }
    assert.equal(await s.stateHash(), h0, "nothing is written");
    const b = await formsHere(s, { system: "u:kestrel/sys:x", star_mass_sun: 1, a_au: 1 });
    assert.deepEqual(b, a);
  } finally {
    await k.close();
  }
});

test("anything goes in marked impossible, at the orbit asked for", () => {
  const c = clipAt(anythingClip({ name: "Odd", kind: "planet", mass_earth: 2, radius_earth: 0.5 }), 1.3);
  const p = placeClip(c, { system: "u:x/sys:y", host: "u:x/sys:y/star:0", hostKind: "star", taken: [] });
  const mass = p.edits.find((e) => e.field === "body.mass")!;
  assert.ok(mass.impossible);
  assert.ok(!p.edits.find((e) => e.field === "name")!.impossible);
  assert.equal(p.edits.find((e) => e.field === "orbit.semi_major_axis")!.value, 1.3 * AU_M);
});

test("a free orbit sits in the widest gap, or past the last world", () => {
  assert.equal(freeOrbit([]), 0.4);
  assert.equal(freeOrbit([0.4, 0.7, 1, 1.5, 5.2]), 2.8);
  assert.equal(freeOrbit([1, 1.2]), 2);
});
