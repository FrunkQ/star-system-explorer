// RB3 (ring and belt editing design 4.2 and 6): the Profile part against a fixture in the agreed ring_structure_is
// shape. Shaping never changes how much there is: every join and every height keeps the ring's mass.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { annulus, authoredZones, causeTag, edgesHeld, joins, lightThrough, markWords, moveJoin, profileMass, profileSpans, sculptorOf, setDensity, setShare, srgb, type RingStructure, type Sculptor } from "../src/profile.ts";

const fx = JSON.parse(readFileSync(new URL("./fixtures/ring-structure.json", import.meta.url), "utf8")) as { ask: { sculptors: Sculptor[] }; answer: RingStructure };
const ring = fx.answer;
const moons = fx.ask.sculptors;
const close = (a: number, b: number, rel = 1e-9) => Math.abs(a - b) <= rel * Math.max(Math.abs(a), Math.abs(b), 1);

test("the fixture keeps the seam's promise: zone masses sum to the ring's", () => {
  assert.ok(close(ring.zones.reduce((t, z) => t + z.mass_kg, 0), ring.mass_kg, 1e-6));
});

test("spans are inner first and their shares sum to one", () => {
  const s = profileSpans(ring);
  assert.equal(s.length, 7);
  for (let i = 1; i < s.length; i++) assert.ok(s[i]!.inner_m >= s[i - 1]!.inner_m);
  assert.ok(close(s.reduce((t, x) => t + x.share, 0), 1));
  const b = s.find((x) => x.name === "B ring")!;
  assert.ok(b.share > 0.5, `B holds most of it (${b.share})`);
});

test("each zone says its cause in words", () => {
  const s = profileSpans(ring);
  const tag = (n: string) => causeTag(s.find((x) => x.name === n)!.cause, moons);
  assert.equal(tag("B ring"), "Mimas 2:1");
  assert.equal(tag("A ring, outer"), "Janus 7:6");
  assert.equal(tag("Encke Gap"), "cleared by Pan");
  assert.equal(tag("Cassini Division"), "measured");
  assert.equal(tag("C ring"), "as generated");
  assert.equal(causeTag({ kind: "authored" }, moons), "drawn by you");
  assert.equal(causeTag({ kind: "resonance", sculptor: "nobody" }, moons), "held by nobody");
  assert.equal(markWords(ring.marks[0]!, moons), "edge held by Mimas (2:1)");
  assert.equal(markWords(ring.marks[2]!, moons), "cleared by Pan");
  assert.deepEqual(edgesHeld(s.find((x) => x.name === "B ring")!, ring.marks, moons), ["Outer edge held by Mimas."]);
});

test("every touching pair of zones has a join; the ring's own edges are not joins", () => {
  const s = profileSpans(ring);
  assert.deepEqual(joins(s), [1, 2, 3, 4, 5, 6]);
  const apart = s.map((x, i) => (i === 3 ? { ...x, inner_m: x.inner_m + 1e5 } : x));
  assert.deepEqual(joins(apart), [1, 2, 4, 5, 6], "a zone that does not touch the one before has no join there");
});

test("moving a join keeps the mass, keeps both zones' density relations, and stops short of the next join", () => {
  const s = profileSpans(ring);
  const total = profileMass(s);
  const out = moveJoin(s, 3, 119000e3); // B's outer edge out into the Cassini Division
  assert.ok(close(profileMass(out), total), "the whole is unchanged");
  assert.ok(close(out.reduce((t, x) => t + x.share, 0), 1));
  assert.equal(out[2]!.outer_m, 119000e3);
  assert.equal(out[3]!.inner_m, 119000e3);
  assert.ok(out[2]!.share > s[2]!.share, "B, wider, holds more");
  assert.ok(out[2]!.yours && out[3]!.yours && !out[0]!.yours);
  // B stays about forty times denser than the division, as it was: the eye sees only the edge move.
  assert.ok(close(out[2]!.sigma_kg_m2 / out[3]!.sigma_kg_m2, s[2]!.sigma_kg_m2 / s[3]!.sigma_kg_m2, 1e-9));
  // Masses and faces agree after the move.
  for (const x of out) assert.ok(close(x.mass_kg, x.sigma_kg_m2 * annulus(x.inner_m, x.outer_m), 1e-9), x.name);
  const far = moveJoin(s, 3, 200000e3);
  assert.ok(far[3]!.inner_m < s[3]!.outer_m, "the join stops before the division's own outer edge");
  assert.ok(close(profileMass(far), total));
  assert.deepEqual(moveJoin(s, 0, 70000e3), s, "the ring's inner edge is Where's, not a join");
});

test("a zone's share trades against the rest, which keep their proportions", () => {
  const s = profileSpans(ring);
  const total = profileMass(s);
  const out = setShare(s, 4, 0.4); // the A ring to 40% of the ring
  assert.ok(close(profileMass(out), total));
  assert.ok(close(out[4]!.share, 0.4));
  assert.ok(close(out[2]!.mass_kg / out[1]!.mass_kg, s[2]!.mass_kg / s[1]!.mass_kg), "B and C keep their proportion");
  assert.ok(out[4]!.tau > s[4]!.tau, "denser is more opaque");
  for (const x of out) assert.equal(x.inner_m, s[out.indexOf(x)]!.inner_m, "heights never move edges");
  const cleared = setShare(s, 5, 0);
  assert.equal(cleared[5]!.mass_kg, 0, "a zone may be cleared");
  assert.ok(close(profileMass(cleared), total));
});

test("dragging a zone to a density puts it exactly there, and the whole stays", () => {
  const s = profileSpans(ring);
  const total = profileMass(s);
  const out = setDensity(s, 1, 400); // the C ring ten times denser
  assert.ok(close(out[1]!.sigma_kg_m2, 400, 1e-9));
  assert.ok(close(profileMass(out), total));
  const all = setDensity(s, 1, 1e12);
  assert.ok(all[1]!.share < 1 && all[1]!.share > 0.999, "a drag never swallows the whole ring");
  assert.ok(close(profileMass(all), total));
});

test("a shaped ring is written as authored zones whose shares sum to one", () => {
  const z = authoredZones(moveJoin(profileSpans(ring), 2, 95000e3));
  assert.equal(z.length, 7);
  assert.ok(z.every((x) => x.source === "authored"));
  assert.ok(close(z.reduce((t, x) => t + x.share, 0), 1));
  assert.equal(z[1]!.outer_m, 95000e3);
});

test("light through a zone", () => {
  assert.equal(lightThrough(0), 1);
  assert.ok(close(lightThrough(1), Math.exp(-1)));
  assert.equal(lightThrough(-2), 1);
});

test("a mark names its sculptor by id or by name; colours are linear and drawn as sRGB", () => {
  assert.equal(sculptorOf("Mimas", moons)?.id, "mimas");
  assert.equal(sculptorOf("mimas", moons)?.name, "Mimas");
  assert.equal(srgb([0, 0, 0]), "rgb(0,0,0)");
  assert.equal(srgb([1, 1, 1]), "rgb(255,255,255)");
  assert.equal(srgb([0.2159, 0.2159, 0.2159]), "rgb(128,128,128)", "linear 0.216 is sRGB 128");
  assert.equal(ring.profile.zone.length, ring.profile.tau.length);
});
