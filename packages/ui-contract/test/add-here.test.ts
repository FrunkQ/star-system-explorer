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

test("the three moon recipes (ED7): formation's leftovers rules for one planet, at the middle of their ranges", async () => {
  const { moonRecipes } = await import("../src/index.ts");
  const earth = moonRecipes({ name: "Tarn", mass_earth: 1, radius_earth: 1, a_au: 1, star_mass_sun: 1 });
  assert.deepEqual(earth.map((r) => r.id), ["impact", "captured"], "a rocky world: an impact moon and a captured body, no disc");
  const impact = earth[0]!;
  assert.ok(Math.abs(impact.mass_earth / 0.0123 - 1) < 0.05, "about the Moon's share of the Earth");
  assert.ok(impact.a_m > 2.5e7 && impact.a_m < 3.2e7, "born about 4.5 Earth radii out");
  assert.ok(impact.radius_earth > 0.25 && impact.radius_earth < 0.3, "rock: about the Moon's size");
  assert.ok(impact.clip.events.every((e) => !e.impossible), "the physics makes moons this way: not impossible");
  const jupiter = moonRecipes({ name: "Jove", mass_earth: 317.8, radius_earth: 11.2, a_au: 5.2, star_mass_sun: 1 });
  assert.deepEqual(jupiter.map((r) => r.id), ["disc", "captured"]);
  assert.ok(Math.abs(jupiter[0]!.mass_earth / (317.8e-4 / 3) - 1) < 1e-9, "a third of the 1e-4 disc");
  const cap = jupiter[1]!;
  assert.ok(cap.a_au > 0.09 && cap.a_au < 0.12, "a third of Jupiter's Hill radius");
  const inc = cap.clip.events.find((e) => e.field === "orbit.inclination")!.value as number;
  assert.ok(inc > Math.PI / 2, "backwards");
  // Clear of the moons already there.
  const crowded = moonRecipes({ name: "Jove", mass_earth: 317.8, radius_earth: 11.2, a_au: 5.2, star_mass_sun: 1 }, [jupiter[0]!.a_au]);
  assert.ok(crowded[0]!.a_au > jupiter[0]!.a_au * 1.3);
  // No star figures: no Hill radius, so no captured body.
  assert.deepEqual(moonRecipes({ name: "Drift", mass_earth: 2, radius_earth: 1.2, a_au: null, star_mass_sun: null }).map((r) => r.id), ["impact"]);
});
