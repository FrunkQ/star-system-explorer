// Close-up (Alex, 25 Nov): how near the camera comes, how a zoom step moves near the ground, where it looks, and the
// ground under it for the detail ladder.
import { test } from "node:test";
import assert from "node:assert/strict";
import { CLOSE_FLOOR_M, CLOSE_REACH, STAR_CLOSE_SHARE, closeMin, grazeLook, outlineRadius, groundUnder, orbitEye, orbitOf, zoomNear } from "../src/close-up.ts";

test("the camera comes down to a few km over a big world, a small share of the radius over a small one", () => {
  const earth = 6.371e6;
  assert.ok(Math.abs(closeMin(1, earth) - (1 + CLOSE_FLOOR_M / earth)) < 1e-12);
  assert.ok(Math.abs(closeMin(1, 2e5) - (1 + CLOSE_FLOOR_M / 2e5)) < 1e-12, "a 200 km moon: 3 km is 1.5% of it");
  assert.equal(closeMin(1, 7e7), 1 + 2e-4, "a giant: never under 0.02% of the radius (14 km over Jupiter)");
  assert.equal(closeMin(2), 2.02, "size unknown: 1% of the radius");
});

test("near the world a zoom step scales the height; far out, the distance", () => {
  assert.equal(zoomNear(10, 1, 0.5), 5);
  assert.equal(zoomNear(3, 1, 0.5), 2, "height 2 halves to 1");
  // Ten steps in from two radii still leave the camera over the ground, never through it.
  let d = 2;
  for (let i = 0; i < 40; i++) d = zoomNear(d, 1, 0.8);
  assert.ok(d > 1 && d - 1 < 1e-3);
});

test("the view looks at the centre from afar and tips toward the horizon low down", () => {
  const fov = Math.PI / 4;
  assert.deepEqual(grazeLook([0, -5, 0], [0, 0, 0], 1, fov), [0, 0, 0]);
  const eye: [number, number, number] = [0, -1.002, 0];
  const look = grazeLook(eye, [0, 0, 0], 1, fov);
  const dir = [look[0] - eye[0], look[1] - eye[1], look[2] - eye[2]];
  const n = Math.hypot(dir[0]!, dir[1]!, dir[2]!);
  // Straight down would be +y; low over the ground it points mostly along the frame's up, just under the horizon.
  const down = dir[1]! / n;
  const horizon = Math.cos(Math.asin(1 / 1.002));
  assert.ok(down > horizon, `still below the horizon: ${down} > ${horizon}`);
  assert.ok(down < 0.5, `tipped well toward the horizon: ${down}`);
  assert.ok(dir[2]! > 0, "toward the up side");
});

test("the ground under the camera: its direction, height and metres per pixel; none when far", () => {
  const g = groundUnder("moon", [0, 0, 1.01], [0, 0, 0], 1, 1.7374e6, Math.PI / 4, 1000)!;
  assert.deepEqual(g.toward, [0, 0, 1]);
  assert.ok(Math.abs(g.height_m - 17374) < 1e-6);
  assert.ok(Math.abs(g.m_per_px - (17374 * 2 * Math.tan(Math.PI / 8)) / 1000) < 1e-9);
  assert.equal(groundUnder("moon", [0, 0, CLOSE_REACH + 0.1], [0, 0, 0], 1, 1.7e6, 1, 100), null);
  assert.equal(groundUnder("x", [0, 0, 1.5], [0, 0, 0], 1, undefined, 1, 100), null);
});

test("the orbit camera goes to a postcard's camera and back", () => {
  const o = { yaw: -0.6, pitch: 0.55, dist: 2.2 };
  const e = orbitEye(o, 0.55)!;
  assert.ok(Math.abs(Math.hypot(...e.dir) - 1) < 1e-12);
  assert.ok(Math.abs(e.radii - 4) < 1e-12);
  const back = orbitOf(e.dir, e.radii, 0.55);
  for (const k of ["yaw", "pitch", "dist"] as const) assert.ok(Math.abs(back[k] - o[k]) < 1e-12, k);
  assert.equal(orbitEye(o, 0), null);
});

test("over a star the camera stops where the whole disc fills the view, its face drawn to the sphere's outline", () => {
  assert.equal(closeMin(2, 6.96e8, true), 2 * (1 + STAR_CLOSE_SHARE));
  // At the floor the disc's angular radius is about half a 45-degree view: the limb is in sight top and bottom.
  const d = 1 + STAR_CLOSE_SHARE;
  const deg = (Math.asin(1 / d) * 180) / Math.PI;
  assert.ok(deg > 21 && deg < 24, `${deg}`);
  // Far off the outline is the radius; up close a picture at the centre must be wider to cover asin(r/d).
  assert.ok(Math.abs(outlineRadius(1, 1e6) - 1) < 1e-9);
  const r = outlineRadius(1, 1.5);
  assert.ok(Math.abs(Math.atan(r / 1.5) - Math.asin(1 / 1.5)) < 1e-12, `${r}`);
  assert.ok(r > 1.3 && r < 1.4, `${r}`);
});
