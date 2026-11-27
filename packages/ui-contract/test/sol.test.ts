// U21 part 2 in Node: the Solar System enters from packs/base-sol as catalogued facts, one batch per source, and
// reads back as a system the chart can draw and the kernel can age (its spec): the Sun, eight planets, the Moon round
// Earth, Pluto and Ceres, two belts and a spherical Oort cloud.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { readSol, solBatches, solSourceText } from "../src/sol.ts";
import { sceneFromResolved } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";
import { anchorOf, drawnSecond } from "../src/sky-clock.ts";
import { constant } from "../src/sheet.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;

test("the sheet becomes batches by source, parents first, nothing typed", () => {
  const { batches, addresses, unverified } = solBatches(sheet, "u:k/sys:sol");
  assert.equal(addresses.sun, "u:k/sys:sol/star:0");
  assert.equal(addresses.mercury, "u:k/sys:sol/body:0");
  assert.equal(addresses.moon, `${addresses.earth}/body:0`);
  assert.match(addresses["oort-cloud"]!, /\/belt:2$/);
  const all = batches.flatMap((b) => b.edits);
  assert.ok(batches.every((b) => sheet.sources.some((s) => s.id === b.source)), "every batch names a source the sheet lists");
  assert.equal(all.find((e) => e.address === addresses.moon && e.field === "orbit.host")?.value, addresses.earth);
  assert.ok(all.some((e) => e.address === "u:k/sys:sol" && e.field === "system.age"));
  assert.ok(unverified.length >= 1, "the sheet's unverified values are named");
  assert.match(solSourceText(sheet, batches[0]!.source), /\S/);
  // Earth's air gas by gas, and Saturn's rings as regions hosted by Saturn.
  const at = (a: string, f: string) => all.find((e) => e.address === a && e.field === f)?.value;
  assert.equal(at(addresses.earth!, "atmosphere.composition.n2"), 0.7808);
  assert.equal(at(addresses.earth!, "atmosphere.composition.o2"), 0.2095);
  assert.equal(at(addresses["saturn-b"]!, "body.kind"), "ring");
  assert.equal(at(addresses["saturn-b"]!, "orbit.host"), addresses.saturn);
  assert.ok((at(addresses["saturn-b"]!, "ring.optical_depth") as number) > 1);
  assert.equal(at(addresses["main-belt"]!, "body.kind"), "belt");
});

test("entered as catalogued facts, Sol reads back whole, with its spec", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sys = "u:kestrel/sys:sol";
    const { batches } = solBatches(sheet, sys);
    for (const b of batches) {
      const r = await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
      assert.equal(r.status, "applied", `${b.source}: ${JSON.stringify(r)}`);
    }
    const r = await s.resolvedSystem(sys);
    assert.ok(r.ok, JSON.stringify(r));
    const scene = sceneFromResolved(r.result, "The Solar System");
    const names = scene.subjects.map((x) => x.label);
    for (const n of ["Sun", "Mercury", "Earth", "Moon", "Neptune", "Pluto", "Ceres", "Main asteroid belt", "Kuiper belt", "Oort cloud"]) assert.ok(names.includes(n), `${n} in ${names.join(", ")}`);
    const earth = scene.subjects.find((x) => x.label === "Earth")!;
    assert.ok(earth.orbit && Math.abs(earth.orbit.a_au - 1) < 0.001, JSON.stringify(earth.orbit));
    assert.equal(scene.subjects.find((x) => x.label === "Moon")!.parent, earth.address);
    assert.ok(earth.stamps.includes("catalogued"));
    assert.equal(scene.subjects.find((x) => x.label === "Oort cloud")!.shape, "sphere");
    assert.ok(Math.abs(scene.now_yr - 4.5673e9) < 1e6, String(scene.now_yr));
    assert.ok(r.result.spec, "the kernel can age it: a spec from its facts");
    // One epoch (RW17 part 2): Sol's age is its age at its anchor, J2000.0 (time.j2000_s), so its birth second is that
    // date less its age, in whole Julian years, exactly; the screen draws birth second + age.
    const spec = r.result.spec as { birth_year: number; age_myr: number };
    const anchor = anchorOf(r.result.facts)!;
    const yr = BigInt(constant("yr_julian"));
    assert.equal(spec.birth_year, Number(anchor / yr) - Math.round(spec.age_myr * 1e6));
    assert.equal(BigInt(r.result.birth_s!), BigInt(spec.birth_year) * yr);
    assert.equal(drawnSecond(BigInt(r.result.birth_s!), scene.now_yr) - anchor, 0n, "at its own age, Sol is drawn at J2000.0");
    assert.equal(r.result.evidence?.[earth.address]?.["body.mass"]?.evidence, "catalogued");
  } finally {
    await k.close();
  }
});

test("the door converts each value by its printed unit: degrees to radians, SI as it is, an unknown unit stops it", async () => {
  const { toSI } = await import("../src/sol.ts");
  assert.equal(toSI({ v: 180, unit: "deg", src: "x" } as never), Math.PI);
  assert.equal(toSI({ v: 5.97e24, unit: "kg", src: "x" } as never), 5.97e24);
  assert.throws(() => toSI({ v: 1, unit: "AU", src: "x" } as never), /cannot convert/);
});
