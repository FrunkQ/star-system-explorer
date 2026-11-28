// The journey played (GZ3/GZ8 first part): the hold while a view reads, the place line as progress, the width line,
// and the two upper ports' cameras, which must agree at a seam.
import { test } from "node:test";
import assert from "node:assert/strict";
import { journeyAt, journeyPlan, type JourneyPlan } from "../src/journey.ts";
import { homeInCube, journeySteps, nearbyAim, readyHold, webEye, widthLine, SEAM_HOLD_S } from "../src/journey-play.ts";
import { projectNearby } from "../src/galaxies-nearby.ts";
import { constant } from "../src/sheet.ts";
import type { Vec3 } from "../src/pose.ts";

const MPC = 1e6 * constant("pc");
const webToCluster = (): JourneyPlan => {
  const r = journeyPlan({ to: "cluster", approach: { from: "web", stops: "none", pace: "stately" }, drawn: (x) => x === "web" || x === "cluster" });
  if (!r.ok) throw new Error(r.why);
  return r.plan;
};

test("the player holds before a seam while the incoming view reads, and at the start while the first does", () => {
  const p = webToCluster();
  assert.equal(p.seams.length, 1);
  const tIn = p.path[1]!.t_in;
  assert.deepEqual(readyHold(p, 0.5, [false, false]), { t: 0, waiting: 0 });
  const held = readyHold(p, tIn + 1, [true, false]);
  assert.equal(held.waiting, 1);
  assert.ok(Math.abs(held.t - (tIn - SEAM_HOLD_S)) < 1e-12);
  // Held there, one view only is drawn: never half-faded.
  assert.equal(journeyAt(p, held.t).views.length, 1);
  assert.deepEqual(readyHold(p, tIn + 1, [true, true]), { t: tIn + 1, waiting: null });
  // Before the seam nothing waits on the second view.
  assert.deepEqual(readyHold(p, tIn / 2, [true, false]), { t: tIn / 2, waiting: null });
});

test("the place line as progress: passed, current, coming; a held rung between says why; the rest off", () => {
  const r = journeyPlan({ to: "system", approach: { from: "web", stops: "none" }, drawn: (x) => x !== "stars" && x !== "galaxy" });
  assert.ok(r.ok);
  const p = r.ok ? r.plan : null!;
  const steps = journeySteps(p, journeyAt(p, p.seams[0]!.t + 0.01));
  assert.deepEqual(steps.map((s) => s.state), ["passed", "current", "held", "held", "coming", "off", "off"]);
  assert.match(steps[3]!.why!, /conserved budget chain/);
});

test("the width line: light years and a power of ten", () => {
  const w = widthLine(10 * MPC);
  assert.match(w.words, /million light years across$/);
  assert.equal(w.power, "3.1 × 10²³ m");
  assert.equal(widthLine(2e3).power, "2 × 10³ m");
  // 51 million light years is 4.8 × 10²³ m, not 10²⁴ (the first capture's rounding).
  assert.equal(widthLine(4.83e23).power, "4.8 × 10²³ m");
  assert.equal(widthLine(9.97e23).power, "1 × 10²⁴ m");
});

test("at a seam the web cube and the galaxies nearby show the same width and the same line of sight", () => {
  const p = webToCluster();
  const s = p.seams[0]!;
  const f = journeyAt(p, s.t);
  assert.equal(f.views.length, 2);
  const [incoming, outgoing] = f.views;
  assert.equal(incoming!.rung, "cluster");
  assert.equal(outgoing!.rung, "web");
  assert.ok(Math.abs(incoming!.width_m / outgoing!.width_m - 1) < 1e-12);
  // The web cube: a point one reach across from the target, square to the line of sight, lands at the drawing's edge.
  const fov = (40 * Math.PI) / 180;
  const target: Vec3 = [10, -20, 5];
  const { eye, dist_mpc } = webEye(outgoing!, target, fov);
  assert.ok(Math.abs(Math.hypot(eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]) - dist_mpc) < 1e-9);
  assert.ok(Math.abs(2 * dist_mpc * Math.tan(fov / 2) - outgoing!.width_m / MPC) < 1e-9);
  // The galaxies nearby, flat: its drawing spans the same width, and its line of sight is the journey's.
  const S = 560;
  const { reach_mpc, turn } = nearbyAim(incoming!, S);
  const a = projectNearby([0, 0, 0], "free", S, reach_mpc, 24, turn);
  const half = incoming!.width_m / MPC / 2;
  // A point half the picture's width along the page's across line sits on the drawing's edge.
  const yaw = (turn.yaw_deg * Math.PI) / 180;
  const across: Vec3 = [Math.cos(yaw) * half, -Math.sin(yaw) * half, 0];
  const b = projectNearby(across, "free", S, reach_mpc, 24, turn);
  assert.ok(Math.abs(a.x - S / 2) < 1e-9 && Math.abs(a.y - S / 2) < 1e-9, "the target at the middle");
  assert.ok(Math.abs(b.x - S) < 1e-6, `edge at ${b.x}`);
  // Depth in projectNearby grows toward the viewer: a point along the line of sight is nearest.
  const d = incoming!.eye_dir;
  const toward = projectNearby([d[0], d[1], d[2]], "free", S, reach_mpc, 24, turn);
  assert.ok(Math.abs(toward.x - S / 2) < 1e-6 && Math.abs(toward.y - S / 2) < 1e-6, "the line of sight projects onto the middle");
  assert.ok(toward.depth > 0.999, `depth ${toward.depth}`);
});

test("home in the web cube: the halo of home's mass nearest home's own place, through the cube's step", () => {
  const home = { position_mpc: [1, 2, 3] as const, mass_msun: 1.2e12 };
  const gs = [
    { position: [40, 0, 0] as const, halo_msun: 1.2e12 },
    { position: [4, 2, 3] as const, halo_msun: 1.2e12 },
    { position: [1, 2, 3.1] as const, halo_msun: 3e11 },
  ];
  const h = homeInCube(gs, home)!;
  assert.deepEqual(h.position, [4, 2, 3]);
  assert.ok(Math.abs(h.moved_mpc - 3) < 1e-12);
  assert.equal(homeInCube([gs[2]!], home), null);
});

// --- FL5: travelling on a coarse picture, and the warm-up of the way down ---------------------------------------
import { BuildTimes } from "../src/settle.ts";
import { coarseFits, journeyReadNext, journeyWaitLine, journeyWarm, levelPasses, levelShows, startTurn, type LevelAnswer } from "../src/journey-play.ts";

const HOST = { hot: false, data_saver: false, memory_over: false, asked: true };

test("FL5: gone early, the journey travels on a level's coarse picture and settles onto the proper one at rest", () => {
  const p = webToCluster();
  const tIn = p.path[1]!.t_in;
  const coarse: LevelAnswer = { state: "coarse", things: 70, of: 70 };
  // Waiting (the default) it holds before the seam; gone early it travels on.
  assert.equal(readyHold(p, tIn + 1, [true, coarse]).waiting, 1);
  assert.deepEqual(readyHold(p, tIn + 1, [true, coarse], { early: true, film: false }), { t: tIn + 1, waiting: null });
  assert.equal(levelShows(coarse, { early: true, film: false }, null, false), "coarse");
  // The proper picture arrives mid-flight: the coarse one is kept until the journey rests, then it settles.
  assert.equal(levelShows(true, { early: true, film: false }, "coarse", false), "coarse");
  assert.equal(levelShows(true, { early: true, film: false }, "coarse", true), "proper");
  assert.equal(levelShows(true, { early: true, film: false }, null, false), "proper");
});

test("FL5: a coarse picture with fewer galaxies than the proper one (or one that cannot say) makes the journey hold", () => {
  const p = webToCluster();
  const tIn = p.path[1]!.t_in;
  const early = { early: true, film: false };
  for (const c of [{ state: "coarse", things: 57, of: 70 }, { state: "coarse", things: 57, of: null }, { state: "coarse", things: 0, of: 0 }] as LevelAnswer[]) {
    assert.equal(coarseFits(c), false);
    assert.equal(levelPasses(c, early), false);
    assert.equal(readyHold(p, tIn + 1, [true, c], early).waiting, 1, JSON.stringify(c));
    assert.equal(levelShows(c, early, null, false), "hold");
  }
});

test("FL5: film never travels on, or shows, a coarse picture", () => {
  const p = webToCluster();
  const tIn = p.path[1]!.t_in;
  const coarse: LevelAnswer = { state: "coarse", things: 70, of: 70 };
  const film = { early: true, film: true };
  assert.equal(readyHold(p, tIn + 1, [true, coarse], film).waiting, 1);
  assert.equal(levelShows(coarse, film, null, false), "hold");
  // Even a level already showing coarse turns to the proper picture (or holds) under film.
  assert.equal(levelShows(coarse, film, "coarse", false), "hold");
  assert.equal(levelShows(true, film, "coarse", false), "proper");
  // The start level too: a journey for film holds at its start until the first level is proper.
  assert.deepEqual(readyHold(p, 1, [coarse, true], film), { t: 0, waiting: 0 });
});

test("FL5: the way down reads one level at a time in the order it is needed, the one on screen first", () => {
  const p = webToCluster();
  const at0 = { entry: 0, t: 0 };
  assert.equal(journeyReadNext(p, at0, [false, false], HOST), 0);
  assert.equal(journeyReadNext(p, at0, [true, false], HOST), 1);
  assert.equal(journeyReadNext(p, at0, [true, true], HOST), null);
  // Backed up to the web with the lower level still unread: the one on screen still comes first.
  const w = journeyWarm(p, at0, [false, false]);
  assert.deepEqual(w.map((x) => [x.kind, x.on_screen]), [["journey.web", true], ["journey.cluster", false]]);
  assert.ok(w[1]!.needed_in_s > 0);
  // A hot device still reads the next level of a journey the viewer asked for, and nothing it did not ask for.
  assert.equal(journeyReadNext(p, at0, [true, false], { ...HOST, hot: true }), 1);
  assert.equal(journeyReadNext(p, at0, [true, false], { ...HOST, hot: true, asked: false }), null);
});

test("FL5: the wait's line tells the time left from this device's own times once it has them", () => {
  const p = webToCluster();
  const times = new BuildTimes();
  assert.equal(journeyWaitLine(p, [false, false], times), "Getting the way down ready: about 15 seconds, 0 of 2 levels.");
  assert.equal(journeyWaitLine(p, [true, false], times, 2000), "Getting the way down ready: about 7 seconds, 1 of 2 levels.");
  times.record("journey.cluster", 37000);
  assert.equal(journeyWaitLine(p, [true, false], times), "Getting the way down ready: 37 seconds, 1 of 2 levels.");
  assert.equal(journeyWaitLine(p, [true, true], times), "The way down is ready.");
});

test("FL5: the start level's slow turn eases back to the journey's line of sight within half a second", () => {
  let a = startTurn(0, 10, true);
  assert.ok(Math.abs(a - 1.2) < 1e-12);
  a = 3; // nearly half a turn round
  for (let i = 0; i < 30; i++) a = startTurn(a, 1 / 60, false);
  assert.equal(a, 0);
});
