import { test } from "node:test";
import assert from "node:assert/strict";
import { leadingArmWords, cellName, placeholderGalaxyName, type GalaxyMark } from "../src/galaxy.ts";

test("GH8: the leading arm in words, the latest made by the chart's time", () => {
  const fade = (m: { strength: number; began_gyr: number; fades_gyr: number | null }, t: number) => (t < m.began_gyr ? 0 : m.fades_gyr ? m.strength * Math.exp(-(t - m.began_gyr) / m.fades_gyr) : m.strength);
  const arm = (began_gyr: number, shape: Record<string, number>) => ({ kind: "leading_arm", caused_by: "u/halo:1/prog:3", began_gyr, strength: 0.5, fades_gyr: 0.9, shape });
  const marks: GalaxyMark[] = [arm(3, { prograde: -1 }), arm(13.5, { r_kpc: 21.6, prograde: -0.8, speed_kms: 300 }), { ...arm(1, {}), kind: "warp" }];
  assert.equal(leadingArmWords([marks[2]!], 13.8, fade), null, "no leading arm, no line");
  const now = leadingArmWords(marks, 13.8, fade)!;
  assert.equal(now.made, 2);
  assert.ok(now.live);
  assert.equal(now.line, "A leading arm, from a backwards pass 22 kpc out 300 Myr ago: it still winds against the spin.");
  const early = leadingArmWords(marks, 4.2, fade)!;
  assert.equal(early.made, 1, "one not yet made is not counted");
  assert.ok(!early.live);
  assert.match(early.line, /^A leading arm, from a backwards merger 1\.2 Gyr ago: the rotation has wound it back/);
  assert.equal(leadingArmWords(marks, 10, fade), null, "long wound back: only the history list keeps it");
  assert.equal(leadingArmWords([{ ...marks[1]!, strength: 0.1 }], 13.8, fade), null, "too weak ever to be seen");
});

test("a cell is named after its most important galaxy: ours at home, a keyed stand-in elsewhere", () => {
  assert.equal(cellName([0, 0, 0], "u/web/cell:0.0.0/halo:7"), "Milky Way neighbourhood");
  const a = "u/web/cell:1.0.0/halo:3";
  const n = cellName([1, 0, 0], a);
  assert.match(n, /^[A-Z][a-z]+ neighbourhood$/);
  assert.equal(n, cellName([1, 0, 0], a), "same galaxy, same name");
  assert.equal(placeholderGalaxyName(a), n.replace(" neighbourhood", ""));
  const names = new Set(Array.from({ length: 50 }, (_, i) => placeholderGalaxyName(`u/web/cell:${i}.0.0/halo:0`)));
  assert.ok(names.size >= 45, `stand-in names rarely repeat (${names.size} of 50)`);
  assert.equal(cellName([2, 0, 0], null), "Cell 2 · 0 · 0", "no halo drawn, keeps its number");
});
