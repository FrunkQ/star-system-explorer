// VP-AU5 part 1: aurorae planned from read_field and the air, under rule 5 of the visual polish design (16.1): switched
// on by the kernel's numbers, never by a name; scaled by them; off where the physics says so; proven on a Solar System
// body, a world outside it and a world where it stays off (tests/fixtures/aurora/proving-worlds.json).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { exposeAurora, kiloRayleighRadiance, lineRgb, planAurora, type AuroraJson, type AuroraWorld } from "../src/aurora.ts";
import type { FieldReadJson } from "../src/field.ts";

type Read = FieldReadJson & { aurora?: AuroraJson | null };
const fx = JSON.parse(readFileSync(new URL("../../../tests/fixtures/aurora/proving-worlds.json", import.meta.url), "utf8")) as { worlds: Record<string, { read: Read; world: AuroraWorld }> };
const W = (k: string) => fx.worlds[k]!;
const plan = (k: string) => planAurora(W(k).read, W(k).world);

test("Earth's own numbers: oxygen green below red, round the magnetic pole at the kernel's oval", () => {
  const p = plan("earth-handed");
  assert.equal(p.layers.length, 2);
  const [g, r] = p.layers;
  assert.match(g!.line, /557\.7/);
  assert.match(r!.line, /630\.0/);
  assert.ok(g!.height_m < r!.height_m, "green below red");
  assert.ok(Math.abs(g!.height_m - 110e3) < 1, "at Earth's own scale height, Earth's measured heights");
  assert.ok(Math.abs(90 - (g!.colatitude_rad * 180) / Math.PI - 70.2) < 0.1, "at the kernel's oval");
  assert.ok(g!.rgb[1] === 1 && g!.rgb[0] < 0.9, "green is green");
  assert.ok(r!.rgb[0] === 1 && r!.rgb[1] < 0.3, "red is red");
  assert.ok(p.stand_in && /Stand-in/.test(p.note), "the stand-in says so");
  assert.ok(Math.abs(Math.acos(p.axis[2]) * (180 / Math.PI) - 9.4) < 1e-9, "the axis carries the tilt");
});

test("a world outside the Solar System: taller curtains from a taller air, ovals far from the spin poles", () => {
  const e = plan("earth-handed");
  const t = plan("tall-tilted");
  assert.equal(t.layers.length, 2);
  const k = W("tall-tilted").world.scale_height_m! / W("earth-handed").world.scale_height_m!;
  assert.ok(Math.abs(t.layers[0]!.height_m / e.layers[0]!.height_m - k) < 1e-9, "heights scale with the scale height");
  assert.ok(Math.abs(t.layers[1]!.thickness_m / e.layers[1]!.thickness_m - k) < 1e-9, "and so does the thickness");
  assert.ok(Math.abs(Math.acos(t.axis[2]) * (180 / Math.PI) - 40) < 1e-9, "tilted 40°");
  // In globe radii the small world's aurora stands far higher: 250 km × 2.5 over 3,600 km against 250 km over 6,371 km.
  const de = exposeAurora(e, W("earth-handed").world.radius_m, "long", 6, 1361)!;
  const dt = exposeAurora(t, W("tall-tilted").world.radius_m, "long", 6, 1361)!;
  assert.ok(dt.top - 1 > 3 * (de.top - 1), `top ${dt.top} against ${de.top}`);
});

test("where it must stay off: an airless world has no aurora and says why", () => {
  const p = plan("airless-lava");
  assert.equal(p.layers.length, 0);
  assert.match(p.note, /no air to glow/);
  assert.equal(exposeAurora(p, 5.2e6, "long", 6, 1361), null);
});

test("no oxygen, no green: a hydrogen giant's ovals are placed but wait on the air's own lines", () => {
  const p = plan("tipped-giant");
  assert.equal(p.layers.length, 0, "nothing borrowed from oxygen");
  assert.match(p.note, /2 ovals here, but its colours wait on the air's own lines/);
  assert.match(p.note, /no oxygen/);
  // The moon's spot is placed by the kernel and said, not drawn yet.
  assert.ok(p.caveats.some((c) => /moon's spot/.test(c)));
});

test("the kernel's own lines win over the stand-in the moment they arrive (MF9 part 2's shape)", () => {
  const r = structuredClone(W("tipped-giant").read);
  r.aurora!.ovals[0]!.width_deg = 2;
  r.aurora!.ovals[0]!.emissions = [{ line: "H I 656.3 nm", matter: "H", wavelength_m: 656.3e-9, power_w: 1e11, peak_height_m: 600e3, thickness_m: 300e3, colour: { linear_srgb: [1, 0.05, 0.08] } }];
  const p = planAurora(r, W("tipped-giant").world);
  assert.equal(p.layers.length, 1);
  assert.equal(p.layers[0]!.grade, "kernel");
  assert.equal(p.stand_in, false);
  assert.equal(p.layers[0]!.height_m, 600e3);
  assert.deepEqual(p.layers[0]!.rgb, [1, 0.05, 0.08]);
  // Power over the two bands' area, into 4π.
  const R = 4e7;
  const c = ((90 - r.aurora!.ovals[0]!.magnetic_latitude_deg) * Math.PI) / 180;
  const w = (2 * Math.PI) / 180;
  const area = 2 * 2 * Math.PI * R * R * (Math.cos(c - w / 2) - Math.cos(c + w / 2));
  assert.ok(Math.abs(p.layers[0]!.radiance_w_m2_sr / (1e11 / (4 * Math.PI * area)) - 1) < 1e-9);
  assert.match(p.note, /waits on the air's own lines/, "the other oval still waits");
});

test("long exposure lifts all faint light by one ratio, so ratios between lines stay true", () => {
  const p = plan("earth-handed");
  const d = exposeAurora(p, 6.371e6, "long", 6, 1361)!;
  const lum = (x: [number, number, number]) => 0.2126 * x[0] + 0.7152 * x[1] + 0.0722 * x[2];
  const ratio = lum(d.layers[1]!.rgb) / lum(d.layers[0]!.rgb);
  assert.ok(Math.abs(ratio - p.layers[1]!.radiance_w_m2_sr / p.layers[0]!.radiance_w_m2_sr) < 1e-9);
  assert.ok(d.lift > 1e6, `lift ${d.lift}`);
  assert.match(d.says, /Long exposure: faint light is lifted/);
});

test("eye: true brightness beside the sunlit world, and a faint aurora goes grey", () => {
  const p = plan("earth-handed");
  const d = exposeAurora(p, 6.371e6, "eye", 6, 1361)!;
  assert.equal(d.lift, 1);
  const g = d.layers[0]!.rgb;
  assert.ok(g[1] < 1e-6, "far too faint to see beside daylight");
  // 10 kR of green is about 0.002 cd/m²: inside the mesopic range, so partly grey.
  const s = (x: [number, number, number]) => (Math.max(...x) - Math.min(...x)) / Math.max(...x);
  const full = exposeAurora(p, 6.371e6, "long", 6, 1361)!.layers[0]!.rgb;
  assert.ok(s(g) < s(full), "less colour to the eye");
  assert.match(d.says, /grey where it is too faint/);
});

test("units: a kilorayleigh of oxygen green", () => {
  // 10⁷ photons s⁻¹ m⁻² sr⁻¹ ÷ 4π × hc/λ at 557.7 nm.
  assert.ok(Math.abs(kiloRayleighRadiance(1, 557.7e-9) - 2.837e-7) < 1e-9);
  const b = lineRgb(427.8e-9);
  assert.ok(b[2] === 1 && b[1] < 0.2, "nitrogen's 427.8 nm is blue");
});
