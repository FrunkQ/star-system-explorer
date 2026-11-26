import { test } from "node:test";
import assert from "node:assert/strict";
import { galaxyColour, haloR200, lightTravelWords, nearbyAt, nearbyRadius, nearbyScaleBar, projectNearby, readNearby, readableFactor, withinReach } from "../src/galaxies-nearby.ts";
import type { Json } from "../src/json.ts";

// The shape of the kernel's `sky_from_here` with galaxies and no stars (fields cut to what the view reads).
const sky = {
  galaxy: "u:k/halo:milky-way-like",
  galaxy_walk: { reach_mpc: 12, home_mpc: [4, 4, 4], satellites: { r200_kpc: 211, host_mass_msun: 1e12, merged: 11, remains: { stream: 1, shell: 2 } } },
  galaxies: [
    { address: "u:k/cell:0.0.-1/halo:1", morphology: "disc", halo_mass_msun: 1.4e12, stellar_mass_now_msun: 3.3e10, sfr_msun_yr: 1.6, distance_mpc: 8.6, dir: [0.31, 0.21, -0.93], half_light_kpc: 13.5, axis_ratio: 0.19, inclination_deg: 79, position_angle_deg: 149, colours: [{ name: "B-V", value: 0.7 }], absolute: [{ band: "V", m_vega: -20.3 }] },
    { address: "u:k/halo:milky-way-like/prog:m3", satellite: true, morphology: "barred", halo_mass_msun: 3e11, stellar_mass_now_msun: 8.2e9, distance_mpc: 0.107, dir: [0.82, 0.16, 0.55], half_light_kpc: 3.6, axis_ratio: 0.49, orbit: { labels: ["being torn apart", "stream"] } },
    { address: "broken: no direction", distance_mpc: 1 },
  ],
} as unknown as Json;

test("the kernel's sky reads into galaxies around home, with what is counted but not drawn", () => {
  const r = readNearby(sky)!;
  assert.equal(r.galaxies.length, 2, "a galaxy with no direction is left out");
  assert.equal(r.home, "u:k/halo:milky-way-like");
  assert.equal(r.home_r200_mpc, 0.211);
  assert.equal(r.remains, 3);
  assert.equal(r.merged, 11);
  const [disc, sat] = r.galaxies;
  assert.equal(disc!.b_v, 0.7);
  assert.equal(disc!.abs_v, -20.3);
  assert.equal(sat!.satellite, true);
  assert.deepEqual(sat!.orbit_words, ["being torn apart", "stream"]);
  assert.equal(readNearby({} as Json), null);
});

test("a galaxy's place is its direction from home times its distance, so the view and the sky agree", () => {
  const p = nearbyAt({ dir: [0, 0.6, 0.8], distance_mpc: 5 });
  assert.deepEqual(p, [0, 3, 4]);
  const r = readNearby(sky)!;
  assert.deepEqual(withinReach(r.galaxies, 1.5).map((g) => g.address), ["u:k/halo:milky-way-like/prog:m3"]);
  assert.equal(withinReach(r.galaxies, 12).length, 2);
});

test("projection: above keeps the plane, edge-on shows height, tilted stands a stalk to the plane", () => {
  const S = 500;
  const a = projectNearby([1, 1, 1], "above", S, 2);
  assert.equal(a.x, a.fx);
  assert.equal(a.y, a.fy);
  assert.ok(a.x > S / 2 && a.y < S / 2);
  const e = projectNearby([1, 1, 1], "edge", S, 2);
  assert.equal(e.fy, S / 2, "the stalk's foot is on our disc's line");
  assert.ok(e.y < S / 2, "above our disc is up the page");
  const t = projectNearby([0, 0, 1], "tilted", S, 2);
  assert.equal(t.fy, S / 2);
  assert.ok(t.y < t.fy, "a galaxy above the plane stands up from its foot");
  const c = projectNearby([0, 0, 0], "tilted", S, 2);
  assert.deepEqual([c.x, c.y], [S / 2, S / 2], "home is the middle");
});

test("Real sizes are true and mostly points; Readable is one stated factor larger", () => {
  const g = { half_light_kpc: 10 };
  const real = nearbyRadius(g, 500, 12, "real", readableFactor(12));
  assert.equal(real, 1.1, "a 10 kpc galaxy at 12 Mpc reach is a point of light");
  const big = nearbyRadius(g, 500, 1.5, "readable", readableFactor(1.5));
  const trueSize = nearbyRadius(g, 500, 1.5, "real", 1);
  assert.ok(Math.abs(big / trueSize - readableFactor(1.5)) < 1e-9);
  assert.deepEqual([0.4, 1.5, 12].map(readableFactor), [2, 8, 20], "smaller at our satellites, so our disc leaves their orbits clear");
});

test("a halo's r200 scales from home's own as the cube root of mass", () => {
  const home = { r200_mpc: 0.211, mass_msun: 1e12 };
  assert.ok(Math.abs(haloR200(8e12, home) - 0.422) < 1e-12);
});

test("colours, light travel and the scale bar say round, plain things", () => {
  assert.notEqual(galaxyColour(0.2), galaxyColour(1.0), "a blue galaxy and a red one differ");
  assert.equal(galaxyColour(null), galaxyColour(null));
  assert.equal(lightTravelWords(0.78), "2.5 million years");
  assert.equal(lightTravelWords(0.05), "163 thousand years");
  const bar = nearbyScaleBar(500, 1.5);
  assert.match(bar.words, /light years$/);
  assert.ok(bar.px > 50 && bar.px < 200);
});

test("bound pairs come from the cells' own neighbours, once each, both on view", async () => {
  const { boundPairs, meetWords } = await import("../src/galaxies-nearby.ts");
  const n = (address: string, bound: boolean, approach_gyr: number | null = 4.2) => ({ address, distance_mpc: 0.8, bound, approach_gyr });
  const cells = [
    { haloes: [{ address: "a", neighbour: n("b", true) }, { address: "b", neighbour: n("a", true) }, { address: "c", neighbour: n("a", false) }, { address: "d", neighbour: n("zz", true) }] },
  ];
  const p = boundPairs(cells, new Set(["a", "b", "c", "d"]));
  assert.deepEqual(p.map((x) => [x.a, x.b]), [["a", "b"]], "one pair; unbound and off-view neighbours left out");
  assert.equal(meetWords(4.2), "meet in about 4 Gyr");
  assert.equal(meetWords(0.43), "meet in about 0.4 Gyr");
  assert.equal(meetWords(0), "already meeting");
});

// GZ6: the kernel's satellite paths (sky_from_here with satellite_paths), kpc from the observer.
const withPaths = {
  ...(sky as object),
  t_myr: 13800,
  galaxy_walk: {
    reach_mpc: 12,
    satellites: {
      r200_kpc: 211,
      host_mass_msun: 1e12,
      merged: 1,
      remains: { stream: 1 },
      centre_kpc: [8, 0, 0],
      paths: [
        { address: "s/prog:a", fate: "survives", labels: ["satellite"], infall_gyr: 10, merged_gyr: null, disrupted_gyr: null, infall_halo_mass_msun: 1e10, infall_stars_msun: 1e7, path: [[10, 200, 0, 0], [11, 0, 100, 0], [13.8, -50, 0, 0]] },
        { address: "s/prog:b", fate: "merged", labels: ["merged"], infall_gyr: 2, merged_gyr: 4, disrupted_gyr: null, infall_halo_mass_msun: 3e10, infall_stars_msun: 6e7, path: [[2, 100, 0, 0], [3, 20, 0, 0], [4, 8, 0, 0]] },
        { address: "s/prog:c", fate: "torn apart", labels: ["stream"], infall_gyr: 6, merged_gyr: null, disrupted_gyr: 9, infall_halo_mass_msun: 2e10, infall_stars_msun: 3e7, path: [[6, 150, 0, 0], [8, 0, 60, 0], [9, 0, 0, 60], [11, -60, 0, 0], [13, 0, -60, 0], [13.8, 0, 0, -60]] },
        { address: "no points", fate: "survives", path: [] },
      ],
    },
  },
} as unknown as Json;

test("satellite paths read in Mpc, and a satellite is where its path says at a time", async () => {
  const { pathAt, firstInfall } = await import("../src/galaxies-nearby.ts");
  const r = readNearby(withPaths)!;
  assert.equal(r.paths.length, 3, "a path with no points is left out");
  assert.deepEqual(r.centre_mpc, [0.008, 0, 0]);
  assert.equal(r.t_gyr, 13.8);
  const [a, b] = r.paths;
  assert.equal(pathAt(a!, 9.9), null, "not yet fallen in");
  assert.deepEqual(pathAt(a!, 10), [0.2, 0, 0]);
  assert.deepEqual(pathAt(a!, 10.5), [0.1, 0.05, 0]);
  assert.deepEqual(pathAt(a!, 20), [-0.05, 0, 0], "held at its last point");
  assert.deepEqual(pathAt(b!, 3), [0.02, 0, 0]);
  assert.equal(pathAt(b!, 4), null, "gone once it reached our centre");
  assert.equal(firstInfall(r.paths), 2);
  assert.equal(firstInfall([]), null);
  // Without paths asked, none are read and the centre is the observer.
  assert.deepEqual(readNearby(sky)!.paths, []);
});

test("a stream is drawn along its own path behind its stars, only once torn apart", async () => {
  const { streamArc, satelliteStateWords, STREAM_ARC_GYR } = await import("../src/galaxies-nearby.ts");
  const c = readNearby(withPaths)!.paths[2]!;
  assert.equal(streamArc(c, 8.9), null, "still whole");
  // At 9.5: from half a Gyr before it was torn apart, to where it has got to.
  const early = streamArc(c, 9.5)!;
  assert.deepEqual(early[0], [0, 0, 0.06]);
  assert.equal(early.length, 2);
  // Late: no longer than the stated arc.
  const late = streamArc(c, 13.8)!;
  assert.ok(STREAM_ARC_GYR === 1.5);
  assert.deepEqual(late[0], [0, -0.06, 0], "the point at 13 Gyr starts the last 1.5 Gyr");
  assert.equal(streamArc(readNearby(withPaths)!.paths[0]!, 13.8), null, "a survivor has none");
  assert.equal(satelliteStateWords(c, 5), "not yet fallen in");
  assert.equal(satelliteStateWords(c, 7), "a satellite on its orbit");
  assert.equal(satelliteStateWords(c, 10), "torn apart, its stars in a stream");
  assert.equal(satelliteStateWords(readNearby(withPaths)!.paths[1]!, 5), "merged into our galaxy");
});

test("the group's edge: Sandage's zero-velocity radius of the mass inside it", async () => {
  const { zeroVelocityRadius, groupEdge } = await import("../src/galaxies-nearby.ts");
  // An absolute check, by hand: 8 G M t^2 / pi^2 with G = 4.4985e-15 Mpc^3 / (Msun Gyr^2), M = 3e12, t = 13.8 Gyr
  // gives 2.0835 Mpc^3, so R0 = 1.277 Mpc (about the measured Local Group's 1 Mpc for its mass of 2 to 3e12).
  assert.ok(Math.abs(zeroVelocityRadius(3e12, 13.8) - 1.277) < 0.002, String(zeroVelocityRadius(3e12, 13.8)));
  assert.equal(zeroVelocityRadius(0, 13.8), 0);
  // Grows as the cube root of mass and as t^(2/3).
  assert.ok(Math.abs(zeroVelocityRadius(8e12, 13.8) / zeroVelocityRadius(1e12, 13.8) - 2) < 1e-9);
  const home = 1e12;
  const r0 = zeroVelocityRadius(home, 13.8);
  // A big neighbour just inside home's own sphere is taken in, which widens it to take in the next; one far
  // outside is not; a satellite is not counted twice.
  const gs = [
    { halo_mass_msun: 1.5e12, distance_mpc: r0 * 0.9, satellite: false },
    { halo_mass_msun: 1e11, distance_mpc: r0 * 1.15, satellite: false },
    { halo_mass_msun: 5e12, distance_mpc: 5, satellite: false },
    { halo_mass_msun: 1e11, distance_mpc: 0.1, satellite: true },
  ];
  const e = groupEdge(home, gs, 13.8);
  assert.equal(e.members, 2);
  assert.equal(e.mass_msun, 2.6e12);
  assert.ok(Math.abs(e.radius_mpc - zeroVelocityRadius(2.6e12, 13.8)) < 1e-12);
  assert.equal(groupEdge(home, [], 13.8).radius_mpc, r0);
});
