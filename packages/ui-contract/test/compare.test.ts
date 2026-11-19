// V14 in Node: the reference objects check; items come only from radius
// facts; the lineup sits on one true scale, marks what is under a pixel,
// draws a body that dwarfs the rest as a limb; the size words; and the
// chart's measurement from the scene's orbits.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkReferences, itemFromReference, itemFromSubject, layoutLineup, scaleBar, sizeWords, starColour, type ReferenceDoc } from "../src/compare.ts";
import { distanceWords, lightWords, separation } from "../src/measure.ts";
import type { SystemScene } from "../src/scene.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const refs = json("packages/shell/src/documents/reference-objects.json") as ReferenceDoc;
const kestrel = json("packages/shell/src/documents/kestrel.scene.json") as SystemScene;
const ref = (id: string) => itemFromReference(refs.objects.find((o) => o.id === id)!);

test("the reference objects are well formed: one of each planet and star type, each with a linked reference", () => {
  assert.deepEqual(checkReferences(refs), []);
  const groups = new Set(refs.objects.map((o) => o.group));
  assert.deepEqual([...groups].sort(), ["moons", "remnants", "stars", "worlds"]);
  for (const t of ["gas-giant", "ice-giant", "sub-neptune", "rocky", "iron-rich", "brown-dwarf"]) assert.ok(refs.objects.some((o) => o.tags.includes(`class:${t}`)), t);
  for (const k of ["M", "K", "G", "F", "A", "B"]) assert.ok(refs.objects.some((o) => o.tags.includes(`spectral:${k}`)), k);
  assert.ok(checkReferences({ ...refs, objects: [{ ...refs.objects[0]!, radius_km: -1 }] }).length > 0);
  assert.ok(checkReferences({ schema: "x" }).length > 0);
});

test("items come only from radius facts; the view never guesses one", () => {
  const items = kestrel.subjects.map((s) => itemFromSubject(s, kestrel.title));
  const tarn = items.find((i) => i?.label === "Tarn")!;
  assert.ok(Math.abs(tarn.radius_km - 0.95 * 6371) < 1e-6);
  assert.equal(tarn.where, kestrel.title);
  const withoutRadius = kestrel.subjects.filter((s) => !s.facts.some((f) => f.field === "radius_earth" || f.field === "radius_sun"));
  for (const s of withoutRadius) assert.equal(itemFromSubject(s, "x"), null, s.label);
});

test("one true scale on one baseline; tiny bodies marked, never dropped; slots keep labels apart", () => {
  const items = [ref("earth"), ref("jupiter"), ref("moon"), ref("neutron-star")];
  const l = layoutLineup(items, 200, 800, "true", 12, 70);
  assert.equal(l.limb, null);
  assert.equal(l.placed.length, 4);
  const by = (id: string) => l.placed.find((p) => p.item.key === `ref:${id}`)!;
  // Same scale: radius ratio is the true ratio where both are above a pixel.
  assert.ok(Math.abs(by("jupiter").r / by("earth").r - 69911 / 6371) < 1e-9);
  assert.ok(by("jupiter").r * 2 <= 200 + 1e-9);
  assert.ok(by("neutron-star").tiny && by("neutron-star").r >= 1.5);
  // Slots: centres at least the slot apart.
  const xs = l.placed.map((p) => p.x);
  for (let i = 1; i < xs.length; i++) assert.ok(xs[i]! - xs[i - 1]! >= 70);
});

test("a body that dwarfs the rest becomes a limb, and the scale is set by the rest", () => {
  const l = layoutLineup([ref("sun"), ref("earth"), ref("moon")], 200, 800, "true", 12, 70);
  assert.equal(l.limb?.item.key, "ref:sun");
  assert.ok(l.limb!.x + l.limb!.r > 0 && l.limb!.x < 0, "the limb's edge shows at the left");
  const e = l.placed.find((p) => p.item.key === "ref:earth")!;
  assert.ok(Math.abs(e.r * 2 - 200) < 1, "Earth, the largest of the rest, fills the height");
  assert.ok(Math.abs(l.limb!.r / e.r - 695700 / 6371) < 1e-9, "the limb is at the same true scale");
  // The Sun is under twelve times Jupiter's radius, so with Jupiter in the row the Sun stays in the row too.
  assert.equal(layoutLineup([ref("sun"), ref("jupiter"), ref("earth")], 200, 800, "true", 12).limb, null);
  // At the default (five times the next) the Sun is a limb beside Jupiter.
  assert.equal(layoutLineup([ref("sun"), ref("jupiter"), ref("earth")], 200, 800).limb?.item.key, "ref:sun");
  // Two stars of similar size: no limb.
  assert.equal(layoutLineup([ref("sun"), ref("alpha-cen-b"), ref("earth")], 200, 800).limb, null);
});

test("log mode orders by size and is marked not to scale", () => {
  const l = layoutLineup([ref("neutron-star"), ref("earth"), ref("betelgeuse")], 200, 800, "log");
  const [a, b, c] = l.placed.map((p) => p.r);
  assert.ok(a! < b! && b! < c!);
  assert.ok(Number.isNaN(l.km_per_px));
});

test("size words, scale bar and star colours", () => {
  assert.equal(sizeWords(6371), "1.00 × Earth");
  assert.equal(sizeWords(69911), "1.00 × Jupiter");
  assert.equal(sizeWords(695700), "1.00 × Sun");
  assert.equal(sizeWords(531500000), "764 × Sun");
  assert.equal(sizeWords(1737.4), "0.27 × Earth");
  const b = scaleBar(100, 120);
  assert.equal(b.km, 10000);
  assert.equal(b.px, 100);
  assert.notEqual(starColour(3000), starColour(20000));
});

test("measure: distances between Kestrel's bodies from the scene's orbits", () => {
  const sub = (n: string) => kestrel.subjects.find((s) => s.label === n)!;
  const tarn = sub("Tarn");
  const star = kestrel.subjects.find((s) => s.kind === "star" && !s.orbit)!;
  const s1 = separation(kestrel, star.address, tarn.address)!;
  assert.ok(Math.abs(s1.now_au - tarn.orbit!.a_au) < 1e-12, "from the star: the orbit's radius");
  assert.ok(Math.abs(s1.light_s - (tarn.orbit!.a_au * 149_597_870.7) / 299_792.458) < 1e-6);
  const other = kestrel.subjects.find((s) => s.orbit && s.parent === tarn.parent && s !== tarn)!;
  const s2 = separation(kestrel, tarn.address, other.address)!;
  assert.ok(s2.range_au);
  assert.ok(s2.now_au >= s2.range_au[0] - 1e-12 && s2.now_au <= s2.range_au[1] + 1e-12);
  assert.equal(separation(kestrel, tarn.address, tarn.address), null);
  assert.equal(distanceWords(1), "1 AU (150 million km)");
  assert.equal(distanceWords(384400 / 149_597_870.7), "384,400 km");
  assert.equal(lightWords(499), "8.3 light minutes");
});
