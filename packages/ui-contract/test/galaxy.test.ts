import { test } from "node:test";
import assert from "node:assert/strict";
import { leadingArmWords, type GalaxyMark } from "../src/galaxy.ts";

test("GH8: the leading arm in words, the latest made by the chart's time", () => {
  const fade = (m: { strength: number; began_gyr: number; fades_gyr: number | null }, t: number) => (t < m.began_gyr ? 0 : m.fades_gyr ? m.strength * Math.exp(-(t - m.began_gyr) / m.fades_gyr) : m.strength);
  const arm = (began_gyr: number, shape: Record<string, number>) => ({ kind: "leading_arm", caused_by: "u/halo:1/prog:3", began_gyr, strength: 0.5, fades_gyr: 0.9, shape });
  const marks: GalaxyMark[] = [arm(3, { prograde: -1 }), arm(13.5, { r_kpc: 21.6, prograde: -0.8, speed_kms: 300 }), { ...arm(1, {}), kind: "warp" }];
  assert.equal(leadingArmWords([marks[2]!], 13.8, fade), null, "no leading arm, no line");
  const now = leadingArmWords(marks, 13.8, fade)!;
  assert.equal(now.made, 2);
  assert.ok(now.live);
  assert.equal(now.line, "A leading arm, from a backwards pass 22 kpc out 300 Myr ago: it still winds against the spin.");
  const early = leadingArmWords(marks, 10, fade)!;
  assert.equal(early.made, 1, "one not yet made is not counted");
  assert.ok(!early.live);
  assert.match(early.line, /^A leading arm, from a backwards merger 7\.0 Gyr ago: the rotation has wound it back/);
});
