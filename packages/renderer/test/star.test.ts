// R5a: a star drawn from its physics. Limb darkening follows the grey
// atmosphere; spots cover what the kernel says and sit where Sporer's law puts
// them; the Sun's corona has a scale height near 0.1 solar radii at 1.5 MK;
// pixels: the limb is darker than the centre, a spot darker than the
// photosphere, and in X-rays the rim outshines the disc's centre.
import { test } from "node:test";
import assert from "node:assert/strict";
import { coronaScaleHeight, drawStar, GRANULE_DT, granuleCount, granuleDeltaT, granulesFor, granulesShow, limbDarkening, narrateGranules, spotsFor, type StarDraw } from "../src/star.ts";

const SUN = { mass: 1.98847e30, radius: 6.957e8 };

test("grey-atmosphere limb darkening: 1 at the centre, 0.4 at the limb", () => {
  assert.equal(limbDarkening(1), 1);
  assert.ok(Math.abs(limbDarkening(0) - 0.4) < 1e-12);
  assert.ok(Math.abs(limbDarkening(0.5) - 0.7) < 1e-12);
});

test("spots cover what the kernel says, follow Sporer's law, and are the same each time", () => {
  const early = spotsFor("sun", 0.003, { phase: 0.05, number: 3 });
  const late = spotsFor("sun", 0.003, { phase: 0.9, number: 3 });
  const area = (s: { radius_deg: number }[]) => s.reduce((a, x) => a + (1 - Math.cos((x.radius_deg * Math.PI) / 180)) / 2, 0);
  assert.ok(Math.abs(area(early) / 0.003 - 1) < 0.35, `coverage ${area(early)}`);
  const meanLat = (s: { lat_deg: number }[]) => s.reduce((a, x) => a + Math.abs(x.lat_deg), 0) / s.length;
  assert.ok(meanLat(early) > 22 && meanLat(late) < 14, `${meanLat(early)} then ${meanLat(late)}`);
  assert.deepEqual(spotsFor("sun", 0.003, { phase: 0.05, number: 3 }), early);
  assert.deepEqual(spotsFor("sun", 0, null), []);
  const sat = spotsFor("ev-lac", 0.2, null);
  assert.ok(sat.some((x) => Math.abs(x.lat_deg) > 50), "saturated stars carry high-latitude spots");
});

test("the Sun's coronal scale height is about 0.1 solar radii at 1.5 MK", () => {
  const h = coronaScaleHeight(SUN.mass, SUN.radius);
  assert.ok(h > 0.09 && h < 0.13, `H = ${h} R`);
});

function px(s: StarDraw, n = 101, disc = 0.8) {
  const img = { width: n, height: n, data: new Uint8ClampedArray(n * n * 4) };
  drawStar(img, s, disc);
  const at = (fx: number, fy: number) => {
    const o = (Math.round(fy * (n - 1)) * n + Math.round(fx * (n - 1))) * 4;
    return [img.data[o]!, img.data[o + 1]!, img.data[o + 2]!, img.data[o + 3]!];
  };
  return at;
}
const lum = (c: number[]) => c[0]! + c[1]! + c[2]!;

test("pixels: limb darker than centre, a spot darker still, X-ray rim brighter than the disc", () => {
  const plain: StarDraw = { teff_k: 5772, spots: [], lon_deg: 0, lat_deg: 0, light: "visible", corona_h: 0.1 };
  const at = px(plain);
  assert.ok(lum(at(0.5, 0.5)) > lum(at(0.5, 0.5 - 0.39)) + 30, `${at(0.5, 0.5)} vs ${at(0.5, 0.11)}`);
  assert.equal(at(0.02, 0.02)[3], 0, "space is clear in visible light");
  const spotted = px({ ...plain, spots: [{ lat_deg: 0, lon_deg: 0, radius_deg: 6 }] });
  assert.ok(lum(spotted(0.5, 0.5)) < lum(at(0.5, 0.5)) - 40, `spot ${spotted(0.5, 0.5)} vs ${at(0.5, 0.5)}`);
  const hot = px({ ...plain, teff_k: 9600 });
  assert.ok(hot(0.5, 0.5)[2]! >= hot(0.5, 0.5)[0]!, `a 9600 K star is bluish white: ${hot(0.5, 0.5)}`);
  const x = px({ ...plain, light: "xray" }, 101, 0.6);
  const rim = lum(x(0.5, 0.5 - 0.305));
  assert.ok(rim > lum(x(0.5, 0.5)), `rim ${rim} vs centre ${lum(x(0.5, 0.5))}`);
});

// O10 convection, as the kernel gives it for the Sun and for a Betelgeuse-like supergiant.
const SUN_CELLS = { granule_size_m: 1.346e6, granule_lifetime_s: 336, supergranule_size_m: 3.4e7, supergranule_lifetime_s: 83000, speed_m_s: 2000, sound_speed_m_s: 7000, granules_across: 1034, network_cells: 5400 };
const BETELGEUSE_CELLS = { ...SUN_CELLS, granule_size_m: 2.7e10, granule_lifetime_s: 2.2e6, granules_across: 39, network_cells: 8 };

test("granules: drawn only where the disc resolves them, the same each time, and the mean light unchanged", () => {
  assert.equal(granulesShow(SUN_CELLS.granules_across, 300), false, "the Sun's granules are sub-pixel on a 300 px disc");
  assert.equal(granulesShow(BETELGEUSE_CELLS.granules_across, 300), true);
  assert.equal(granulesShow(BETELGEUSE_CELLS.granules_across, 60), false, "39 across needs at least 117 px");
  const n = granuleCount(39);
  assert.ok(n > 4000 && n < 7000, `a 39-across star has about 5,500 granules: ${n}`);
  const g = granulesFor("betelgeuse", 39, 0);
  assert.deepEqual(granulesFor("betelgeuse", 39, 0).pts, g.pts, "same star and generation, same cells");
  assert.notDeepEqual(granulesFor("betelgeuse", 39, 1).pts, g.pts, "a new generation is new cells");
  // Temperature spread: rms about 3.1% of the surface temperature, mean zero.
  let s1 = 0;
  let s2 = 0;
  const m = 3000;
  for (let i = 0; i < m; i++) {
    const z = 1 - (2 * (i + 0.37)) / m;
    const q = Math.sqrt(1 - z * z);
    const t = granuleDeltaT(g, 3600, q * Math.cos(i * 2.39996), z, q * Math.sin(i * 2.39996));
    s1 += t;
    s2 += t * t;
  }
  assert.ok(Math.abs(s1 / m) < 15, `mean dT ${s1 / m} K`);
  const rms = Math.sqrt(s2 / m);
  assert.ok(Math.abs(rms / (GRANULE_DT * 3600) - 1) < 0.1, `rms dT ${rms} K`);
});

test("pixels: a supergiant's disc is mottled by its cells, its mean light kept; the Sun at this size is plain", () => {
  const n = 241;
  const draw = (s: StarDraw) => {
    const img = { width: n, height: n, data: new Uint8ClampedArray(n * n * 4) };
    drawStar(img, s, 0.9);
    // Green channel, decoded to linear light, over the inner disc.
    const vals: number[] = [];
    for (let py = 0; py < n; py++)
      for (let px = 0; px < n; px++) {
        const x = (px + 0.5 - n / 2) / (n * 0.45);
        const y = (py + 0.5 - n / 2) / (n * 0.45);
        if (x * x + y * y > 0.25) continue;
        const c = img.data[(py * n + px) * 4 + 1]! / 255;
        vals.push(c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
      }
    const mean = vals.reduce((a, v) => a + v, 0) / vals.length;
    const sd = Math.sqrt(vals.reduce((a, v) => a + (v - mean) ** 2, 0) / vals.length);
    return { mean, sd };
  };
  const plain: StarDraw = { teff_k: 3600, spots: [], lon_deg: 0, lat_deg: 0, light: "visible", corona_h: 0.1 };
  const flat = draw(plain);
  const boil = draw({ ...plain, granules: granulesFor("betelgeuse", 39, 0) });
  assert.ok(boil.sd > flat.sd + 0.02, `mottled: sd ${boil.sd} vs ${flat.sd}`);
  assert.ok(Math.abs(boil.mean / flat.mean - 1) < 0.03, `mean light kept: ${boil.mean} vs ${flat.mean}`);
  assert.ok(narrateGranules(BETELGEUSE_CELLS, 240).includes("drawn here"));
  assert.ok(narrateGranules(SUN_CELLS, 240).includes("too small to draw"));
  assert.ok(narrateGranules(null, 240).includes("too hot to churn"));
});
