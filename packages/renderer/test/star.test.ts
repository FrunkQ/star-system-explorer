// R5a: a star drawn from its physics. Limb darkening follows the grey
// atmosphere; spots cover what the kernel says and sit where Sporer's law puts
// them; the Sun's corona has a scale height near 0.1 solar radii at 1.5 MK;
// pixels: the limb is darker than the centre, a spot darker than the
// photosphere, and in X-rays the rim outshines the disc's centre.
import { test } from "node:test";
import assert from "node:assert/strict";
import { coronaScaleHeight, drawStar, limbDarkening, spotsFor, type StarDraw } from "../src/star.ts";

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
