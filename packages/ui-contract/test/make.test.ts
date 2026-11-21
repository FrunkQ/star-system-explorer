// V13 in Node: the kernel's importer and formation run, through the session,
// become scenes the chart can draw. Both ops are pure (the state hash does
// not move); a save that is not V3 is refused with the kernel's code; values
// are the save's own; honesty marks stay; a generated system is the same for
// the same seed.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { beltBand, importedSystems, noWorldsWhy, reportSummary, sceneFromFormation, sceneFromImport } from "../src/make.ts";
import { describeLight, litFromSystem } from "../src/light.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const base = JSON.parse(text("packs/base-physics/pack.json"));
const save = text("tests/fixtures/v3/synthetic-sol.json");

test("a V3 save imports as a preview scene; the open universe does not move", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [base]);
    const h0 = await s.stateHash();
    const r = await s.importV3(save);
    assert.ok(r.ok, JSON.stringify(r));
    assert.equal(await s.stateHash(), h0, "import writes nothing");
    const systems = importedSystems(r.result);
    assert.deepEqual(systems.map((x) => x.name), ["Sol"]);
    const scene = sceneFromImport(r.result, systems[0]!.address);
    assert.equal(scene.title, "Sol");
    assert.deepEqual(scene.subjects.map((x) => [x.label, x.kind]), [["Sun", "star"], ["Earth", "planet"], ["Moon", "moon"], ["Station", "station"]]);
    const earth = scene.subjects.find((x) => x.label === "Earth")!;
    assert.equal(earth.orbit!.a_au, 1);
    assert.equal(earth.facts.find((x) => x.field === "mass_earth")!.value, 1);
    assert.ok(Math.abs(scene.now_yr - 4.57e9) < 1e3);
    // The Moon's density was set below physics in V3: it carries the honesty mark.
    assert.ok(scene.subjects.find((x) => x.label === "Moon")!.stamps.includes("impossible"));
    const sum = reportSummary(r.result.report);
    assert.equal(sum.outside.length, 1);
    assert.match(sum.line, /1 system, 3 bodies, 1 constructs, 85 facts/);
    // The Sun's temperature from the save gives it light.
    assert.ok(!describeLight(litFromSystem(scene)).missing.includes("Sun"));
    const bad = await s.importV3("not json");
    assert.ok(!bad.ok && bad.code === "import_failed", JSON.stringify(bad));
    const player = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, [base]);
    assert.ok(!(await player.importV3(save)).ok, "players do not import");
  } finally {
    await k.close();
  }
});

test("a generated system is a scene; the same seed gives the same system", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [base]);
    const h0 = await s.stateHash();
    const a = await s.runFormation({ seed: "kernel-form-6", star_pins: { primary_mass: 1 } });
    assert.ok(a.ok, JSON.stringify(a));
    assert.equal(await s.stateHash(), h0, "a formation run writes nothing");
    const b = await s.runFormation({ seed: "kernel-form-6", star_pins: { primary_mass: 1 } });
    assert.ok(b.ok);
    assert.deepEqual(sceneFromFormation(a.result, "kernel-form-6"), sceneFromFormation(b.result, "kernel-form-6"));
    const scene = sceneFromFormation(a.result, "kernel-form-6");
    assert.equal(scene.subjects[0]!.kind, "star");
    assert.equal(scene.subjects.length, a.result.run.worlds.length + 1);
    if (a.result.run.worlds.length) {
      const w = scene.subjects[1]!;
      assert.ok(w.orbit && w.orbit.a_au > 0);
      assert.ok(w.facts.some((x) => x.field === "surface_k"));
    } else assert.match(noWorldsWhy(a.result)!, /No worlds/);
    const bad = await s.runFormation({ seed: "x", block: "sideways" as never });
    assert.ok(!bad.ok);
  } finally {
    await k.close();
  }
});

test("a belt's band: its edges at orbital scale, else ±10 % about its orbit", () => {
  const AU = 149_597_870_700;
  assert.deepEqual(beltBand(undefined, 2 * AU, 3.3 * AU), [2, 3.3]);
  const b = beltBand(2.7 * AU, undefined, undefined)!;
  assert.ok(Math.abs(b[0] - 2.43) < 1e-9 && Math.abs(b[1] - 2.97) < 1e-9);
  // A ring's radii (thousands of km) are not an orbital band: fall back to the orbit.
  assert.deepEqual(beltBand(undefined, 7e7, 1.4e8), undefined);
});
