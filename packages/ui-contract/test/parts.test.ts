// The parts palette, CM13 (compound ships design 8A.4, 8A.5, 8B.2, rulings 14A.4 to 14A.6). Checks: every part in the
// base palette is in the one format, with a listed source for every figure, unit facings, port and hull points on the
// size standard and no warnings; real parts carry the year they first flew and their figures add up; names are
// physics names (never the Atomic Rockets brand or a named screen ship); points clip by place and size, never by
// pack; stretch and widen follow each part's own law and a fixed part says why it will not; the era reading keeps
// what had flown.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { clips, flownBy, partWarnings, shapeVolume, stretchPart, type PartsPalette, type Part } from "../src/parts.ts";

const PACK = JSON.parse(readFileSync(new URL("../../../packs/base-parts/pack.json", import.meta.url), "utf8"));
const PAL = PACK.provides.other.palette as PartsPalette;
const SOURCES = PACK.provides.other.sources as Record<string, string>;
const part = (id: string) => PAL.parts.find((p) => p.id === id)!;
const near = (a: number, b: number, rel: number, what: string) => assert.ok(Math.abs(a - b) <= rel * Math.abs(b), `${what}: ${a} against ${b}`);

test("every part is in the one format, every figure's source is listed, and the palette warns about nothing", () => {
  const ids = new Set<string>();
  for (const p of PAL.parts) {
    assert.ok(!ids.has(p.id), `duplicate ${p.id}`);
    ids.add(p.id);
    assert.ok(p.name && p.kind && p.shape && p.scale, p.id);
    assert.ok(p.shape.length_m > 0 && p.shape.radius_m > 0, `${p.id} has a size`);
    for (const [k, f] of Object.entries(p.figures)) {
      assert.ok(Number.isFinite(f.v), `${p.id} ${k} is a number`);
      assert.ok(f.src in SOURCES, `${p.id} ${k} cites ${f.src}`);
      assert.ok(["cited", "authored", "predicted", "derived", "estimated"].includes(f.mark), `${p.id} ${k} mark`);
    }
    for (const c of p.points) near(Math.hypot(...c.facing), 1, 1e-9, `${p.id} ${c.id} faces a unit direction`);
    for (const id of [...(p.scale.with_volume ?? []), ...(p.scale.with_area ?? [])]) assert.ok(id in p.figures, `${p.id} scales a figure it has (${id})`);
    // Trusses and booms carry no figures yet and say what they are waiting on; nothing else warns.
    const warn = partWarnings(p, PAL.port_sizes).filter((w) => !(p.kind === "truss" && /decoration/.test(w)));
    assert.deepEqual(warn, [], p.id);
  }
  assert.ok(PAL.credits.some((c) => /GCAT.*CC BY 4\.0/.test(c)), "GCAT is credited");
});

test("real parts carry the year they first flew, and their figures add up", () => {
  const real = PAL.parts.filter((p) => p.credit);
  assert.ok(real.length >= 30);
  for (const p of real) {
    assert.ok(p.year !== undefined && p.year >= 1942 && p.year <= 2026, `${p.id} year ${p.year}`);
    assert.ok(p.year_src, `${p.id} says where its year comes from`);
  }
  assert.equal(part("stage-saturn-s-ic").year, 1967);
  assert.equal(part("stage-et").year, 1981);
  for (const p of PAL.parts.filter((p) => p.kind === "stage")) {
    near(p.figures.propellant_t!.v, p.figures.wet_t!.v - p.figures.dry_t!.v, 1e-9, `${p.id} propellant`);
    assert.equal(p.alone, true, "a stage can fly alone, so it becomes a construct");
    assert.deepEqual(p.points.map((c) => c.place), ["front", "behind"]);
  }
  for (const p of PAL.parts.filter((p) => p.kind === "tank" || p.kind === "habitat" || p.kind === "bay"))
    if (p.figures.volume_m3) near(shapeVolume(p.shape), p.figures.volume_m3.v, 1e-6, `${p.id} holds what its shape holds`);
});

test("each era's drives and power plants name their future tech advance, whose figures the kernel reads", () => {
  const tech = JSON.parse(readFileSync(new URL("../../../packs/base-tech/pack.json", import.meta.url), "utf8")).provides.other.tech;
  const ids = new Set(tech.advances.map((a: { id: string }) => a.id));
  const drives = PAL.parts.filter((p) => p.advance);
  assert.ok(drives.some((p) => p.advance === "fusion-torch") && drives.some((p) => p.advance === "chemical-rocket") && drives.some((p) => p.advance === "rtg"));
  for (const p of drives) {
    assert.ok(ids.has(p.advance), `${p.id} names a real advance`);
    assert.deepEqual(p.figures, {}, `${p.id} copies no figure from base-tech`);
  }
});

test("names are physics names: never the Atomic Rockets brand or a named screen ship", () => {
  const refused = [/atomic\s*rockets?/i, /projectrho/i, /\bliberator\b/i, /millennium falcon/i, /\bserenity\b/i, /rocinante/i, /nostromo/i];
  for (const p of PAL.parts) for (const rx of refused) assert.ok(!rx.test(p.name), `${p.id}: ${p.name}`);
  const builder = readFileSync(new URL("../../../scripts/build-base-parts.py", import.meta.url), "utf8");
  assert.match(builder, /def refuse_names/, "the builder refuses them before it writes the palette");
});

test("points clip by place and size, never by pack, and pass the lines both carry", () => {
  const tank = part("tank-4m");
  const hab = part("habitat-4m");
  const front = tank.points.find((c) => c.place === "front")!;
  const behind = tank.points.find((c) => c.place === "behind")!;
  assert.deepEqual(clips(front, behind), { ok: true, lines: ["propellant"] });
  const habPort = hab.points.find((c) => c.place === "port")!;
  const node = part("node-4").points.find((c) => c.place === "port")!;
  assert.deepEqual(clips(habPort, node), { ok: true, lines: ["air", "people", "power"] });
  const big = part("stage-saturn-s-ii").points.find((c) => c.place === "front")!;
  const small = part("stage-saturn-s-ivb").points.find((c) => c.place === "behind")!;
  const r = clips(big, small);
  assert.equal(r.ok, false);
  assert.match(r.why!, /adapter/, "a 10 m stage meets a 6.6 m one through an adapter, as the real stack did");
  assert.equal(clips(front, front).ok, false, "two fronts do not meet");
});

test("stretch and widen follow the part's own law; a fixed part says why it will not", () => {
  const tank = part("tank-4m");
  const { part: long } = stretchPart(tank, { length_m: 2 * tank.shape.length_m });
  near(long.figures.volume_m3!.v, 2 * tank.figures.volume_m3!.v, 1e-9, "twice as long holds twice as much");
  assert.equal(long.figures.volume_m3!.mark, "derived");
  near(long.points.find((c) => c.id === "front")!.at[0], 2 * tank.shape.length_m, 1e-9, "the front point moves with the nose");
  assert.equal(long.points.find((c) => c.id === "front")!.size_m, tank.points.find((c) => c.id === "front")!.size_m, "sizes stay on the standard");
  const { part: wide } = stretchPart(tank, { radius_m: 2 * tank.shape.radius_m });
  near(wide.figures.volume_m3!.v, 4 * tank.figures.volume_m3!.v, 1e-9, "twice as wide holds four times as much");
  const panel = part("solar-panel");
  near(stretchPart(panel, { length_m: 3 * panel.shape.length_m }).part.figures.area_m2!.v, 3 * panel.figures.area_m2!.v, 1e-2, "a panel's area follows its size");
  const stage = part("stage-saturn-s-ivb");
  const fixed = stretchPart(stage, { length_m: 40 });
  assert.equal(fixed.part, stage);
  assert.match(fixed.note!, /comes in the sizes its pack offers/);
});

test("the era reading keeps what had flown by then, and every generic part", () => {
  const in1970 = flownBy(PAL.parts, 1970);
  assert.ok(in1970.some((p) => p.id === "stage-saturn-s-ivb"));
  assert.ok(!in1970.some((p) => p.id === "stage-et"), "the Shuttle's tank had not flown in 1970");
  assert.ok(in1970.some((p) => p.id === "tank-4m"), "generic parts have no year and are always offered");
  const parts: Part[] = flownBy(PAL.parts, 1941);
  assert.ok(parts.every((p) => p.year === undefined));
});
