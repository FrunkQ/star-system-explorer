// Stars that show what they do (V3 gap 19): a star's mark reads its power and activity from the kernel's facts.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkSymbology, markerFor, type SymbologyDoc } from "../src/symbology.ts";
import { luminosityLsun, readStarFacts, starFactsFrom, withStarFacts } from "../src/star-marks.ts";

const doc = (n: string) => JSON.parse(readFileSync(new URL(`../../shell/src/documents/${n}`, import.meta.url), "utf8"));
const REG: SymbologyDoc[] = ["symbology.diagrammatic.json", "symbology.colour-coded.json", "symbology.mixed.json"].map(doc);
const FULL = { supports: { colours: "full" as const, patterns: true, shapes: true } };
const SUN = { "body.radius": 6.957e8, "star.effective_temperature": 5772, "body.mass": 1.989e30 };

test("the Sun's power from its radius and temperature is one Sun", () => {
  assert.ok(Math.abs(luminosityLsun(6.957e8, 5772)! - 1) < 0.01);
  assert.equal(luminosityLsun(0, 5772), null);
});

test("a star's mark grows with its power and flares only when its corona is busy", () => {
  for (const id of ["sse.symbology.diagrammatic", "sse.symbology.colour-coded", "sse.symbology.mixed"]) {
    const m = markerFor(id, REG, FULL);
    const star = (f: Record<string, number>, regime?: string) => m.mark(withStarFacts({ kind: "star", classes: [], tags: [] }, starFactsFrom(f, regime ? { regime, flares: true } : undefined)));
    const sun = star(SUN, "moderate");
    const dwarf = star({ ...SUN, "body.radius": 1.4e8, "star.effective_temperature": 3100 }, "saturated");
    const giant = star({ ...SUN, "body.radius": 5e10, "star.effective_temperature": 4000 });
    assert.equal(sun.size, 7, id);
    assert.ok(dwarf.size < sun.size && giant.size > sun.size, id);
    assert.equal(sun.aura, "none", id);
    assert.equal(dwarf.aura, "flares", id);
    // With no facts, the plain star as before.
    const plain = m.mark({ kind: "star", classes: [], tags: [] });
    assert.equal(plain.size, 7);
    assert.equal(plain.aura, "none");
    // A planet never takes a star's aura.
    assert.equal(m.mark({ kind: "planet", classes: [], tags: ["activity:saturated"] }).aura, "none");
  }
});

test("an aura must be one the chart can draw", () => {
  const bad: SymbologyDoc = { schema: "sse.symbology/1", id: "bad", name: "Bad", bindings: [{ applies_to: "star", channel: "aura", from: "tag:activity:", scale: { table: { saturated: "sparkles" } } }] };
  assert.match(checkSymbology(bad, [bad]) ?? "", /aura sparkles/);
});

test("a system's stars are read in one resolved_system and one read_activity", async () => {
  const calls: string[] = [];
  const session = {
    resolvedSystem: async (sys: string) => (calls.push(`resolved ${sys}`), { ok: true as const, result: { facts: { s: { "system.age": 1e17 }, "s/a": SUN, "s/b": {} } } as never }),
    readActivity: async (ask: { stars: unknown[] }) => (calls.push(`activity ${ask.stars.length}`), { ok: true as const, stars: [{ activity: { regime: "active", flares: true } }] }),
  };
  const f = await readStarFacts(session, "s", ["s/a", "s/b"]);
  assert.deepEqual(calls, ["resolved s", "activity 1"]);
  assert.deepEqual(f.get("s/a")!.tags, ["activity:active", "flares"]);
  assert.ok(Math.abs(f.get("s/a")!.numbers.luminosity_lsun! - 1) < 0.01);
  assert.equal(f.has("s/b"), false, "a star the kernel cannot describe keeps the plain mark");
});
