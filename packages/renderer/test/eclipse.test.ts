// Shadows and eclipses in True (eclipse.ts): the processor twin of the GPU's shadow sums.
import { test } from "node:test";
import assert from "node:assert/strict";
import { discCover, GLINT_FALLBACK, glintBrdf, ringPassAt, seenFrom, shadeFor, starLightAt, turnOnto, umbraLight, type Placed } from "../src/eclipse.ts";

const near = (a: number, b: number, tol: number, what: string) => assert.ok(Math.abs(a - b) <= tol, `${what}: ${a} not within ${tol} of ${b}`);

test("the share of the star's disc a body covers: none apart, all inside, a lens between, a ring for a small one", () => {
  assert.equal(discCover(1, 2), 0);
  assert.equal(discCover(1.2, 0.1), 1);
  near(discCover(0.5, 0.2), 0.25, 1e-12, "annular");
  // Two equal discs a radius apart overlap by 2π/3 − √3/2 (over π).
  near(discCover(1, 1), (2 * Math.PI / 3 - Math.sqrt(3) / 2) / Math.PI, 1e-9, "lens");
  // The cover falls smoothly through the edge: a soft edge, not a step.
  const steps = [0.9, 1.0, 1.1, 1.2, 1.3].map((d) => discCover(0.4, d));
  for (let i = 1; i < steps.length; i++) assert.ok(steps[i]! < steps[i - 1]!, `falling: ${steps}`);
});

// The Sun, Earth and Moon at a total solar eclipse: the Sun along +x, the Moon on the line between.
const AU = 1.496e11;
const sun: Placed = { id: "sun", at_m: [AU, 0, 0], radius_m: 6.957e8 };
const earth: Placed = { id: "earth", at_m: [0, 0, 0], radius_m: 6.371e6 };
const moonAt = (y_m: number): Placed => ({ id: "moon", at_m: [3.75e8, y_m, 0], radius_m: 1.7374e6 });

test("a total eclipse of the Sun: dark at the spot under the Moon, a penumbra a few thousand km round it, daylight beyond", () => {
  const sh = shadeFor(earth, [moonAt(0)], sun, [1, 0, 0])!;
  assert.ok(sh, "the Moon stands in the way");
  assert.equal(sh.casters.length, 1);
  near(sh.star, 6.957e8 / AU, 1e-6, "the Sun's angular radius");
  const toStar = [1, 0, 0];
  // Under the Moon (the sub-solar point): the Moon's disc (0.2655°) is just larger than the Sun's here, so none left.
  assert.equal(starLightAt([1, 0, 0], toStar, sh)[0], 0);
  // 2,000 km away the Sun is part covered (deep penumbra); 6,000 km away (beyond the Moon's ~3,500 km penumbra), full.
  const at = (km: number) => {
    const y = (km * 1e3) / 6.371e6;
    return starLightAt([Math.sqrt(1 - y * y), y, 0], toStar, sh)[1];
  };
  const mid = at(2000);
  assert.ok(mid > 0.05 && mid < 0.95, `penumbra ${mid}`);
  assert.equal(at(6000), 1);
});

test("a caster is read where it was when the light passed it: the Moon 1.25 s and 1.28 km back along its path", () => {
  // The Moon on the Sun–Earth line now, moving at 1.022 km/s across it: the light reaching Earth passed it 3.75e8/c s ago.
  const moving: Placed = { ...moonAt(0), v_m_s: [0, 1022, 0] };
  const back = seenFrom(moving, earth);
  const tau = 3.75e8 / 299792458;
  near(back[1], -1022 * tau, 1e-9, "the light time's step back");
  near(back[1], -1278.4, 0.5, "about 1.28 km");
  // In the shade the caster sits that far off the line, in the world's radii (Earth's here, turned onto +x light).
  const sh = shadeFor(earth, [moving], sun, [1, 0, 0])!;
  near(sh.casters[0]!.at[1], (-1022 * tau) / 6.371e6, 1e-12, "drawn where the light passed it");
  // No velocity, no step: the caster stays where it is.
  assert.deepEqual(seenFrom(moonAt(0), earth), moonAt(0).at_m);
});

test("no eclipse when the Moon passes 5° to one side, as it does most months", () => {
  assert.equal(shadeFor(earth, [moonAt(3.75e8 * Math.tan((5 * Math.PI) / 180))], sun, [1, 0, 0]), null);
});

test("a body behind the world, or the star itself, casts nothing on it", () => {
  const behind: Placed = { id: "moon", at_m: [-3.84e8, 0, 0], radius_m: 1.7374e6 };
  assert.equal(shadeFor(earth, [behind, sun], sun, [1, 0, 0]), null);
});

test("Earth's shadow on the Moon leaves the umbra's own light, copper and faint, in the middle", () => {
  const umbra = umbraLight({ lit: true, colour: { linear_srgb: [1, 0.0906, 0] }, star_colour: { linear_srgb: [1, 0.879, 0.821] }, share: 2.56e-4 })!;
  assert.ok(umbra[0] > umbra[1] && umbra[1] >= umbra[2], `red over green over blue: ${umbra}`);
  const lum = 0.2126 * umbra[0] * 1 + 0.7152 * umbra[1] * 0.879 + 0.0722 * umbra[2] * 0.821;
  near(lum / (0.2126 + 0.7152 * 0.879 + 0.0722 * 0.821), 2.56e-4, 1e-9, "the kernel's share");
  const moon: Placed = { id: "moon", at_m: [-3.844e8, 0, 0], radius_m: 1.7374e6 };
  const sh = shadeFor(moon, [{ ...earth, umbra }], sun, [1, 0, 0])!;
  const light = starLightAt([1, 0, 0], [1, 0, 0], sh);
  for (let k = 0; k < 3; k++) near(light[k]!, umbra[k]!, 1e-12, `umbra channel ${k}`);
  // An airless world leaves nothing, and a kernel giving a colour but no brightness is not drawn as daylight.
  assert.equal(umbraLight({ lit: false, star_colour: { linear_srgb: [1, 1, 1] } }), null);
  assert.equal(umbraLight({ lit: true, colour: { linear_srgb: [1, 0.1, 0] }, star_colour: { linear_srgb: [1, 1, 1] } }), null);
});

test("Jupiter and Io: Io's shadow on Jupiter at conjunction, Jupiter's on Io half an orbit later, in the drawing's frame", () => {
  const jup: Placed = { id: "jupiter", at_m: [0, 0, 0], radius_m: 6.9911e7 };
  const sunJ: Placed = { id: "sun", at_m: [7.785e11, 0, 0], radius_m: 6.957e8 };
  const io = (x: number): Placed => ({ id: "io", at_m: [x, 0, 0], radius_m: 1.8216e6 });
  // The drawing lights Jupiter from +y: the shadow comes with it.
  const sh = shadeFor(jup, [io(4.217e8)], sunJ, [0, 1, 0])!;
  near(sh.casters[0]!.at[1], 4.217e8 / 6.9911e7, 1e-9, "Io turned onto the drawn light");
  near(sh.casters[0]!.at[0], 0, 1e-9, "on the line");
  assert.ok(starLightAt([0, 1, 0], [0, 1, 0], sh)[0] < 0.01, "Io's umbra on Jupiter's sub-solar point");
  assert.equal(shadeFor(jup, [io(-4.217e8)], sunJ, [0, 1, 0]), null);
  const shIo = shadeFor(io(-4.217e8), [jup], sunJ, [1, 0, 0])!;
  assert.equal(starLightAt([1, 0, 0], [1, 0, 0], shIo)[0], 0);
});

test("a turn takes one direction onto another and keeps lengths and angles", () => {
  const t = turnOnto([1, 0, 0], [0, 0, 1]);
  const a = t([1, 0, 0]);
  near(a[2], 1, 1e-12, "onto");
  const b = t([0, 2, 0]);
  near(Math.hypot(...b), 2, 1e-12, "length");
  const o = turnOnto([1, 0, 0], [-1, 0, 0])([1, 0, 0]);
  near(o[0], -1, 1e-12, "a half turn");
});

test("a ring's shadow on its world: (1 − cover)^(1/μ₀) where the ray to the star crosses it, none past its edges", () => {
  const n = [0, 0, 1];
  const toStar = [Math.cos(0.35), 0, Math.sin(0.35)];
  const cover = (rho: number) => (rho > 1.5 && rho < 2 ? 0.8 : 0.1);
  // From a point in the southern hemisphere the ray rises through the ring plane: find where and check the pass.
  const z = -0.312;
  const x = Math.sqrt(1 - z * z);
  const p = [x, 0, z];
  const t = -z / Math.sin(0.35);
  const rho = x + Math.cos(0.35) * t;
  assert.ok(rho > 1.5 && rho < 2, `crosses the dense ring: ${rho}`);
  const pass = ringPassAt(p, toStar, n, 1.2, 2.3, cover);
  near(pass, Math.pow(1 - cover(rho), 1 / Math.sin(0.35)), 1e-12, "slant pass");
  assert.ok(pass < 0.5, `a dense ring darkens the ground: ${pass}`);
  // The northern, sunward side sees the star straight, and a ray crossing inside the inner edge is clear.
  assert.equal(ringPassAt([0, 0, 1], toStar, n, 1.2, 2.3, cover), 1);
  assert.equal(ringPassAt([0, 0, -1], [0, 0, 1], n, 1.2, 2.3, cover), 1);
});

test("the sun's glint: brightest where the sea mirrors it, spread wider and dimmer by rougher waves", () => {
  const n = [0, 0, 1];
  const l = [Math.sin(0.5), 0, Math.cos(0.5)];
  const mirror = [-Math.sin(0.5), 0, Math.cos(0.5)];
  const aside = [-Math.sin(0.9), 0, Math.cos(0.9)];
  // The kernel's slope variances (sse.light.sea_glint, Cox and Munk) for a 2 m/s and a 15 m/s wind.
  const calm = 0.01324;
  const windy = 0.0798;
  // The stand-in is the kernel's answer for water under 7 m/s: 0.003 + 5.12e-3 × 7 and ((1.333 − 1)/(1.333 + 1))².
  near(GLINT_FALLBACK.slope_variance, 0.003 + 5.12e-3 * 7, 1e-12, "the stand-in slope variance");
  near(GLINT_FALLBACK.normal_reflectance, ((1.333 - 1) / (1.333 + 1)) ** 2, 1e-4, "the stand-in reflectance");
  assert.ok(glintBrdf(n, l, mirror, calm) > glintBrdf(n, l, mirror, windy), "a calm sea's glint is brighter at its centre");
  assert.ok(glintBrdf(n, l, aside, windy) > glintBrdf(n, l, aside, calm), "a rough sea's glint reaches further out");
  // Energy: over the sky the glint gives back about Fresnel's share (2 to 10% at these angles), never more than the light.
  let sum = 0;
  const N = 400;
  for (let i = 0; i < N; i++) {
    const th = ((i + 0.5) / N) * (Math.PI / 2);
    for (let j = 0; j < N; j++) {
      const ph = ((j + 0.5) / N) * 2 * Math.PI;
      const v = [Math.sin(th) * Math.cos(ph), Math.sin(th) * Math.sin(ph), Math.cos(th)];
      sum += glintBrdf(n, l, v, windy) * Math.cos(th) * Math.sin(th) * (Math.PI / 2 / N) * ((2 * Math.PI) / N);
    }
  }
  assert.ok(sum > 0.015 && sum < 0.12, `reflected share ${sum}`);
});
