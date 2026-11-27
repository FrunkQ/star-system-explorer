import { test } from "node:test";
import assert from "node:assert/strict";
import { addProminences, prominenceColour, prominencePoints, prominencesAt, prominenceShown, type Prominence } from "../src/star-prominences.ts";
import { turnAt, type LiveStar } from "../src/live-star.ts";
import { drawStarNow } from "../src/star-now.ts";

const DAY = 86400;
const R_SUN = 6.957e8;
const sun = (prominences: Prominence[], over: Partial<LiveStar> = {}): LiveStar => ({
  address: "sys/star",
  teff: 5772,
  mass: 1.989e30,
  radius: R_SUN,
  lum: 3.828e26,
  age_s: 1e9,
  activity: { rotation_period_s: 25.4 * DAY },
  moment: { spot_coverage: 0.002, xray_w: 2e20 },
  flares: [],
  prominences,
  corona: { surface_field_t: 0.0095, slingshots: false, corotation_radii: 37 },
  ...over,
});
const prom = (over: Partial<Prominence> = {}): Prominence => ({
  kind: "quiescent",
  born_s: 1e9 - DAY,
  life_s: 40 * DAY,
  lat_deg: 0,
  lon_deg: 90,
  tilt_deg: 0,
  height_m: 0.1 * R_SUN,
  length_m: 0.4 * R_SUN,
  thickness_m: 0.01 * R_SUN,
  mass_kg: 1e12,
  temperature_k: 8000,
  share_visible: 1.1e-4,
  light: { lines: [{ line: "ha", power_w: 1 }, { line: "hb", power_w: 0.5 }] },
  ...over,
});

test("a prominence is drawn only where it stands off the disc: at the limb it shows, facing us it does not", () => {
  const draw = (p: Prominence) => {
    const s = sun([p]);
    const at = s.age_s;
    // Put its middle 90° from the viewer (the limb) or at the disc's centre, at this moment.
    const o = { width: 96, data: new Uint8ClampedArray(96 * 96 * 4) };
    const n = addProminences(o, prominencesAt(s, at, 2), s.radius, 2, 0);
    let lit = 0;
    for (let i = 3; i < o.data.length; i += 4) if (o.data[i]! > 0) lit++;
    return { n, lit };
  };
  const t0 = turnAt(sun([]), 1e9);
  const limb = draw(prom({ lon_deg: 90 + t0 }));
  const face = draw(prom({ lon_deg: 0 + t0 }));
  assert.equal(limb.n, 1);
  assert.ok(limb.lit > 20, `${limb.lit} pixels lit at the limb`);
  assert.equal(face.n, 0, "on the disc it is a filament, lost in white light");
  assert.equal(face.lit, 0);
});

test("it is red from its own hydrogen lines, grows in and fades out, and an eruption carries it off fading", () => {
  const c = prominenceColour(prom());
  assert.ok(c[0] === 1 && c[0] > c[2] && c[0] > c[1], `${c}`);
  const s = sun([prom({ born_s: 1e9, life_s: 10 * DAY })]);
  const k = (t: number) => prominencesAt(s, t, 3)[0]?.k ?? 0;
  assert.ok(k(1e9 + 0.2 * DAY) < k(1e9 + 5 * DAY));
  assert.ok(k(1e9 + 9.9 * DAY) < k(1e9 + 5 * DAY));
  assert.equal(k(1e9 + 11 * DAY), 0, "gone after its life");
  const e = sun([prom({ born_s: 1e9, life_s: 10 * DAY, eruption: { at_s: 1e9 + 10 * DAY, speed_m_s: 300e3 } })]);
  const up = prominencesAt(e, 1e9 + 10 * DAY + 3600, 3)[0]!;
  assert.ok(up.erupting && up.h > 0.1 + (300e3 * 3600) / R_SUN - 1e-9 && up.k < 1);
  assert.equal(prominencesAt(e, 1e9 + 10 * DAY + 3 * DAY, 3).length, 0, "risen out of the picture");
});

test("brightness follows the kernel's share on a stated log scale; reduced motion holds the present", () => {
  assert.ok(prominenceShown(1e-4) > prominenceShown(1e-5));
  assert.equal(prominenceShown(1e-7), 0);
  const s = sun([prom()]);
  const a = prominencesAt(s, s.age_s + 3 * DAY, 3, true)[0]!;
  const b = prominencesAt(s, s.age_s, 3, true)[0]!;
  assert.equal(a.lon_deg, b.lon_deg);
});

test("the living star draws its prominences into the halo and says so", () => {
  const t0 = turnAt(sun([]), 1e9);
  const s = sun([prom({ lon_deg: 90 + t0 }), prom({ lon_deg: -90 + t0, kind: "active", height_m: 0.04 * R_SUN })]);
  const face = { width: 32, data: new Uint8ClampedArray(32 * 32 * 4) };
  const halo = { width: 160, data: new Uint8ClampedArray(160 * 160 * 4) };
  const now = drawStarNow(face, halo, s, { t: s.age_s, clock: DAY / 4, span: DAY / 40, lat_deg: 0, disc_deg: 10 });
  assert.equal(now.limb, 2);
  assert.ok(now.words.some((w) => /2 prominences at the limb/.test(w) && /95 gauss/.test(w)), now.words.join(" | "));
});

test("an erupting prominence balloons out as it rises, feet on the surface: a bulb, not a rod, even edge-on", () => {
  const t0 = turnAt(sun([]), 1e9);
  // At the limb with its spine pointing at us: the loop is seen edge-on, the view that drew a rod before.
  const p = prom({ lon_deg: 90 + t0, born_s: 1e9 - 10 * DAY, life_s: 10 * DAY, eruption: { at_s: 1e9, speed_m_s: 800e3 } });
  const s = sun([p]);
  const at = 1e9 + (2 * R_SUN) / 800e3;
  const [up] = prominencesAt(s, at, 4);
  assert.ok(up && up.erupting && up.h > 1.9, `${up?.h}`);
  const pts = prominencePoints(up, R_SUN);
  const r = (q: number[]) => Math.hypot(q[0]!, q[1]!, q[2]!);
  // The axis's feet near the surface (within the rope's own thickness), its top risen by h.
  assert.ok(Math.abs(r(pts[0]!.at) - 1) < 0.02, `${r(pts[0]!.at)}`);
  assert.ok(Math.abs(Math.max(...pts.map((q) => r(q.at))) - (1 + up.h)) < 0.45 * up.h);
  // Seen from us (x across, y up), as wide as a good share of its height (absolute: a rod is under a tenth).
  const ys = pts.map((q) => q.at[1]!);
  const xs = pts.map((q) => q.at[0]!);
  const width = Math.max(...ys) - Math.min(...ys);
  const tall = Math.max(...xs) - 1;
  assert.ok(width > 0.4 * tall, `${width} across for ${tall} tall`);
});

test("a risen loop is drawn as one patch of light, not a row of beads", () => {
  const t0 = turnAt(sun([]), 1e9);
  for (const tilt_deg of [0, 90]) {
    const p = prom({ lon_deg: 90 + t0, tilt_deg, born_s: 1e9 - 10 * DAY, life_s: 10 * DAY, eruption: { at_s: 1e9, speed_m_s: 800e3 } });
    const s = sun([p]);
    const at = 1e9 + (1.5 * R_SUN) / 800e3;
    // Close in (150 px a radius), where the old 48 points along a risen arch lay further apart than their spots.
    const W = 1200;
    const o = { width: W, data: new Uint8ClampedArray(W * W * 4) };
    addProminences(o, prominencesAt(s, at, 4), s.radius, 4, 0);
    // Walk straight out from the limb along the equator (the row through the star's middle): the light is unbroken
    // from the limb to the loop's top.
    const y = W / 2;
    const lit: boolean[] = [];
    for (let x = W / 2 + 151; x < W; x++) lit.push(o.data[(y * W + x) * 4 + 3]! > 0);
    let runs = 0;
    for (let i = 0; i < lit.length; i++) if (lit[i] && !lit[i - 1]) runs++;
    assert.ok(runs >= 1 && runs <= 2, `tilt ${tilt_deg}: ${runs} separate lit runs out from the limb`);
  }
});
