// Plumes above the limb (third audit R10): grains from the kernel's polar cap, on ballistic arcs, lit forward.
import { test } from "node:test";
import assert from "node:assert/strict";
import { plumeBrightness, plumeFacts, plumeGrains, plumeNote, PLUME_GRAINS } from "../src/plumes.ts";

const R = 252e3;
const enceladus = { plumes: { strength: 0.77, pole: -1, cap_deg: 41.4, grain_speed_m_s: 120.5, escape_speed_m_s: 239, grain_height_m: 85847, gas_escapes: true, vapour_kg_s: 256, why: ["…"] } };

test("plume facts are read from a surface record, and a record without them gives none", () => {
  assert.equal(plumeFacts({ regime_now: "cold-ice-lid" }), null);
  assert.equal(plumeFacts(null), null);
  const p = plumeFacts(enceladus)!;
  assert.equal(p.pole, -1);
  assert.equal(p.gas_escapes, true);
});

test("grains rise over the kernel's polar cap, to about the kernel's height", () => {
  const p = plumeFacts(enceladus)!;
  const g = plumeGrains(p, R);
  const n = g.length / 3;
  assert.equal(n, Math.round(PLUME_GRAINS * 0.77));
  let top = 0;
  for (let i = 0; i < n; i++) {
    const [x, y, z] = [g[i * 3]!, g[i * 3 + 1]!, g[i * 3 + 2]!];
    const r = Math.hypot(x, y, z);
    assert.ok(r >= 1 - 1e-6, `grain under the ground: ${r}`);
    // South pole: every grain over the southern cap (within the cap plus the jets' spread).
    assert.ok(z / r < -Math.cos(((41.4 + 20) * Math.PI) / 180), `grain outside the cap: ${z / r}`);
    top = Math.max(top, (r - 1) * R);
  }
  // Launch speeds spread ±40% about the grain speed, so the tallest stand above the mean height but below escape.
  assert.ok(top > 85847 && top < 6 * 85847, `${top}`);
  // The same picture every time.
  assert.deepEqual(plumeGrains(p, R), g);
});

test("lit from behind a plume shines; lit from the front it is faint", () => {
  const p = plumeFacts(enceladus)!;
  const back = plumeBrightness(p, 1);
  const front = plumeBrightness(p, -1);
  assert.ok(Math.abs(back - 0.77) < 1e-9);
  assert.ok(front < back / 50, `${front} ${back}`);
});

test("the note says which pole, how high and how much", () => {
  const s = plumeNote("Enceladus", plumeFacts(enceladus)!);
  assert.match(s, /south pole/);
  assert.match(s, /86 km/);
  assert.match(s, /256 kg/);
});

test("trueLook draws a world's plumes turned to its pole, and a hot world's glow", async () => {
  const { trueLook } = await import("../src/true-body.ts");
  const facts = plumeFacts(enceladus)!;
  // Pole along +x in the drawing: grains go to -x (a south-pole plume). Lit from behind: the light beyond the plume.
  const look = trueLook({ key: "e", star: false, colour: "#888", radius_m: R, pole: [1, 0, 0], plumes: { facts, toLight: [0, 0, -1], toEye: [0, 0, 1] } });
  assert.ok(look.plumes && look.plumes.brightness > 0.7);
  const p = look.plumes!.points;
  for (let i = 0; i < p.length; i += 3) assert.ok(p[i]! < -0.5, `grain at ${p[i]}`);
  // Front-lit: faint.
  const front = trueLook({ key: "e", star: false, colour: "#888", radius_m: R, pole: [1, 0, 0], plumes: { facts, toLight: [0, 0, 1], toEye: [0, 0, 1] } });
  assert.ok(front.plumes!.brightness < 0.02);
  // A brown dwarf far out glows; a cold moon does not.
  const hot = trueLook({ key: "b", star: false, colour: "#888", radius_m: 7e7, pole: [0, 0, 1], heat: { t_k: 1300, star: { t_k: 5772, r_m: 6.957e8, d_m: 50 * 1.496e11 } } });
  assert.ok(hot.glow && hot.glow[0] === 1);
  const cold = trueLook({ key: "c", star: false, colour: "#888", radius_m: R, pole: [0, 0, 1], heat: { t_k: 75, star: { t_k: 5772, r_m: 6.957e8, d_m: 9.5 * 1.496e11 } } });
  assert.equal(cold.glow, undefined);
});
