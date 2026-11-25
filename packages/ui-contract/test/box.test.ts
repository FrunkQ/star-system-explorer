// ED12: a handle's box, the Orbit and Spin boxes (editing design section 11F). The sheet declares the parts and chips;
// the helpers are presentation arithmetic (the orbit's geometry, the sun's day, the ratios of years), checked here
// against the real Solar System's numbers.
import { test } from "node:test";
import assert from "node:assert/strict";
import { boxAt, epochMeanFor, handlesFor, mirrorAngle, orbitShape, partHandle, resonances, solarDay, stabilityFor, stripDomain, trueAnomalyFrom, trueFromMean, meanFromTrueRad, wrap } from "../src/index.ts";

const orbit = handlesFor("world").find((h) => h.id === "orbit")!;
const spin = handlesFor("world").find((h) => h.id === "spin")!;
const DAY = 86400;

test("the Orbit box: Curious shows where it is now; Science adds shape, lean, the two rings and the chips", () => {
  assert.deepEqual(boxAt(orbit, "curious").parts.map((p) => p.id), ["now"]);
  assert.deepEqual(boxAt(orbit, "science").parts.map((p) => p.id), ["shape", "now", "lean", "peri", "node"]);
  assert.deepEqual(boxAt(orbit, "science").chips.map((c) => c.id), ["retrograde", "resonance"]);
  assert.deepEqual(boxAt(spin, "curious").parts.map((p) => p.id), ["tilt"]);
  assert.deepEqual(boxAt(spin, "curious").chips.map((c) => c.id), ["retrograde", "locked"]);
  // A part is a handle of its own for the strip, with its fixed ends.
  const lean = partHandle(orbit, orbit.box!.parts.find((p) => p.id === "lean")!);
  assert.equal(lean.part.field, "orbit.inclination");
  const d = stripDomain(lean, 0.1, []);
  assert.equal(d.scale, "linear");
  assert.equal(d.lo, 0);
  assert.equal(d.hi, Math.PI);
  // Every box field is a real kernel field, and every handle without a box still opens its plain strip.
  for (const h of handlesFor("world")) for (const p of h.box?.parts ?? []) assert.match(p.part.field, /^(orbit|body)\./);
  assert.equal(boxAt(handlesFor("world").find((h) => h.id === "mass")!, "engine").parts.length, 0);
});

test("Retrograde mirrors the lean through 90 degrees, and back", () => {
  const ten = (10 * Math.PI) / 180;
  assert.ok(Math.abs(mirrorAngle(ten) - (170 * Math.PI) / 180) < 1e-12);
  assert.ok(Math.abs(mirrorAngle(mirrorAngle(ten)) - ten) < 1e-12);
});

test("true and mean anomaly round trip, at any shape", () => {
  for (const e of [0, 0.0167, 0.2056, 0.9])
    for (const nu of [0, 0.3, 1.5, Math.PI, 4, 6.2]) assert.ok(Math.abs(wrap(trueFromMean(meanFromTrueRad(nu, e), e) - nu + 1e-9) - 1e-9) < 1e-9, `e ${e} nu ${nu}`);
});

test("the place round the orbit from the kernel's position, at any lean", () => {
  const o = { inc: 0.4, node: 1.1, peri: 2.3 };
  // A point at true anomaly 0.7 on that orbit, built the kernel's way (perifocal to reference frame).
  const nu = 0.7;
  const [xp, yp] = [Math.cos(nu), Math.sin(nu)];
  const [co, so, cw, sw, ci, si] = [Math.cos(o.node), Math.sin(o.node), Math.cos(o.peri), Math.sin(o.peri), Math.cos(o.inc), Math.sin(o.inc)];
  const r: [number, number, number] = [(co * cw - so * sw * ci) * xp + (-co * sw - so * cw * ci) * yp, (so * cw + co * sw * ci) * xp + (-so * sw + co * cw * ci) * yp, sw * si * xp + cw * si * yp];
  assert.ok(Math.abs(trueAnomalyFrom(r, o) - nu) < 1e-12);
});

test("the ring writes the epoch's mean anomaly that puts the world there now", () => {
  const P = 3.15e7;
  const m0 = epochMeanFor(Math.PI, 0.1, P, 2.5 * P + 1000, 0);
  const now = m0 + (2 * Math.PI * (2.5 * P + 1000)) / P;
  assert.ok(Math.abs(trueFromMean(now, 0.1) - Math.PI) < 1e-9);
});

test("closest and farthest, and the sunlight's swing: Earth gets about 7% more in January", () => {
  const s = orbitShape(149.598e9, 0.0167);
  assert.ok(Math.abs(s.closest - 147.1e9) < 0.1e9);
  assert.ok(Math.abs(s.farthest - 152.1e9) < 0.1e9);
  assert.ok(Math.abs(s.swing - 0.069) < 0.002);
});

test("the sun's day: Earth's 24 hours from its 23 h 56 m turn; Venus backwards; a locked world has none", () => {
  const year = 365.256 * DAY;
  assert.ok(Math.abs(solarDay(86164.1, year)! - DAY) < 1);
  const venus = solarDay(-243.0226 * DAY, 224.701 * DAY)!;
  assert.ok(Math.abs(venus / DAY - 116.75) < 0.1, String(venus / DAY));
  assert.equal(solarDay(year, year), null);
});

test("In step with: Pluto's 3:2 with Neptune, and the size that would put it exactly there", () => {
  const nep = { label: "Neptune", a: 30.07, year: 164.8 };
  const pluto = { a: 39.48, year: 248.0 };
  const r = resonances(pluto, [nep, { label: "Uranus", a: 19.2, year: 84 }]);
  assert.equal(r.length, 1);
  assert.equal(r[0]!.label, "3:2 with Neptune");
  assert.match(r[0]!.says, /Neptune goes round 3 times while this goes round 2/);
  assert.ok(Math.abs(r[0]!.a - 39.48 * ((164.8 * 1.5) / 248) ** (2 / 3)) < 1e-9);
  assert.equal(r[0]!.on, false);
  // Exactly there, it says so.
  assert.equal(resonances({ a: r[0]!.a, year: 164.8 * 1.5 }, [nep])[0]!.on, true);
});

test("the stability verdict names this world's worst check", () => {
  const checks = [
    { check: "hill", bodies: ["a", "b"], level: 0, says: "lasts" },
    { check: "amd", bodies: ["a", "b"], level: 1, says: "they could, in time" },
    { check: "moon", around: "a", bodies: ["a/m"], level: 2, says: "lost" },
  ];
  assert.deepEqual(stabilityFor(checks, "a"), { level: 1, word: "Watch", says: "they could, in time", check: "amd" });
  assert.equal(stabilityFor(checks, "c"), null);
});

test("the outer wall on a moon's orbit strip is the kernel's widest lasting moon orbit", async () => {
  const { outerWall } = await import("../src/index.ts");
  const checks = [
    { check: "moon", around: "p", bodies: ["p/m"], level: 0, says: "inside the widest lasting moon orbit (708950 km): stays", results: { limit: 708949815.6, verdict: 0 } },
    { check: "hill", bodies: ["p", "q"], level: 0, says: "lasts", results: { spacing: 26 } },
  ];
  assert.deepEqual(outerWall(checks, "p/m"), { at: 708949815.6, says: "the star pulls it away beyond" });
  assert.equal(outerWall(checks, "p"), null);
});
