// The star chooser (ED6 part 2): the mass strip, its letters and walls, and Kinds found on the kernel's life line at
// the system's present age; choosing writes the star's mass, radius, light and temperature as one change.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { readSol, solBatches, solSourceText } from "../src/sol.ts";
import { resultsOf } from "../src/handles.ts";
import { commitFields } from "../src/edit.ts";
import { STAR_KINDS, birthMass, kindMass, letterOf, readMassStrip, starEdits, systemAgeYr, type Describe } from "../src/star-chooser.ts";
import { plainWorkspace } from "./workspace.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const kind = (id: string) => STAR_KINDS.find((k) => k.id === id)!;

test("spectral letters are the kernel's class; none for a remnant", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" });
    const sun = await s.describeStar(1, 4.5673e9);
    assert.ok(sun.ok);
    assert.match(sun.star.now.class.code, /^G\d V$/, "a sun-like star at the Sun's age is a G dwarf");
    assert.equal(sun.star.now.class.family, "main-sequence");
    assert.equal(letterOf(sun.star), "G");
    const red = await s.describeStar(0.3, 4.5673e9);
    assert.ok(red.ok && letterOf(red.star) === "M");
    const wd = await s.describeStar(1, 1.3e10);
    assert.ok(wd.ok && wd.star.now.phase === "white-dwarf");
    assert.equal(letterOf(wd.star), null);
    assert.equal(wd.star.now.class.family, "white-dwarf");
  } finally {
    await k.close();
  }
});

test("the strip and the kinds at the Sun's age, and a young system's", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" });
    const describe: Describe = (m, a, f) => s.describeStar(m, a, f);
    const age = 4.5673e9;
    const strip = (await readMassStrip(describe, age, undefined, 48))!;
    assert.equal(strip.samples.length, 48);
    assert.equal(strip.walls.hydrogen_msun, 0.08);
    // Lightest first: a brown dwarf at the light end, a black hole at the heavy end; the colours redden then blue.
    assert.equal(strip.samples[0]!.phase, "brown-dwarf");
    assert.equal(strip.samples.at(-1)!.phase, "black-hole");
    assert.equal(strip.samples.at(-1)!.colour, null, "a black hole gives no light");
    const sunLike = (await kindMass(describe, kind("sun-like"), strip.samples, age))!;
    assert.ok(sunLike > 0.85 && sunLike < 1.15, `Sun-like near one solar mass: ${sunLike}`);
    const giant = (await kindMass(describe, kind("red-giant"), strip.samples, age))!;
    assert.ok(giant !== null, "a red giant at 4.57 billion years");
    const g = await s.describeStar(giant, age);
    assert.ok(g.ok && g.star.now.phase === "giant", `the mass found is a giant now: ${giant}`);
    assert.ok(giant > 1 && giant < 2.5, `born a little heavier than the Sun: ${giant}`);
    for (const id of ["white-dwarf", "neutron-star", "black-hole", "red-dwarf", "brown-dwarf"]) {
      const m = await kindMass(describe, kind(id), strip.samples, age);
      const d = m === null ? null : await s.describeStar(m, age);
      assert.ok(d?.ok && d.star.now.phase === kind(id).phase, `${id} at ${m}`);
    }
    assert.equal(await kindMass(describe, kind("blue-giant"), strip.samples, age), null, "no blue giant lives 4.57 billion years");
    const young = (await readMassStrip(describe, 5e6, undefined, 48))!;
    const blue = await kindMass(describe, kind("blue-giant"), young.samples, 5e6);
    assert.ok(blue !== null && blue > 2.5, `a blue giant at 5 million years: ${blue}`);
  } finally {
    await k.close();
  }
});

test("choosing a star for the Sun: one change, the worlds answer to it", async () => {
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
    const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sys = "u:kestrel/sys:sol";
    const { batches, addresses } = solBatches(sheet, sys);
    for (const b of batches) await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
    const sun = addresses.sun!;
    // A world of our own at 1 AU: the catalogued planets carry their measured temperatures.
    const mars = `${sys}/body:90`;
    await s.send({ kind: "set_values", edits: [
      { address: mars, field: "name", value: "Test world" },
      { address: mars, field: "body.kind", value: "planet" },
      { address: mars, field: "body.mass", value: 5.97e24 },
      { address: mars, field: "body.radius", value: 6.37e6 },
      { address: mars, field: "orbit.host", value: sun },
      { address: mars, field: "orbit.semi_major_axis", value: 1.496e11 },
    ], at_yr: 0 });
    const r0 = await s.resolvedSystem(sys);
    assert.ok(r0.ok);
    if (!r0.ok) return;
    const age = systemAgeYr(r0.result.facts[sys] as never)!;
    assert.ok(Math.abs(age / 4.5673e9 - 1) < 1e-6, `the Sun's age read back: ${age}`);
    const facts = r0.result.facts[sun] as Record<string, never>;
    assert.ok(Math.abs(birthMass(facts)! - 1) < 0.01, "a catalogued Sun is born at its mass now");
    const t0 = resultsOf(r0.result.surfaces, mars)["t_surface_now_k"] as number;
    // 1.1 solar masses is still on the main sequence at 4.57 billion years (1.5 would be a white dwarf by now).
    const d = await s.describeStar(1.1, age);
    assert.ok(d.ok && d.star.now.phase === "main-sequence");
    if (!d.ok) return;
    const ws = plainWorkspace();
    ws.desk = "build";
    const r = await commitFields(s, ws, { address: sun, edits: starEdits(d.star, facts), display: "1.1 solar masses", before_display: "1 solar mass" });
    assert.equal(r.status, "applied");
    assert.equal(ws.undo.length, 1, "one change, one Undo");
    const r1 = await s.resolvedSystem(sys);
    assert.ok(r1.ok);
    if (!r1.ok) return;
    assert.equal(birthMass(r1.result.facts[sun] as never), 1.1, "the strip reopens at the mass it was born with");
    const t1 = resultsOf(r1.result.surfaces, mars)["t_surface_now_k"] as number;
    assert.ok(t1 > t0 + 5, `a world at 1 AU warms under a brighter star: ${t0} K to ${t1} K`);
  } finally {
    await k.close();
  }
});
