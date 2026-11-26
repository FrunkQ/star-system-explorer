// The travel contract (GZ1, design/SSE4-grand-zoom-design.md section 5): the ladder of rungs, the way in and its
// link form, the plan and the camera at any journey time.
import { test } from "node:test";
import assert from "node:assert/strict";
import { PLACE_SCALES, parsePlaceHash, placeHash, type Place } from "../src/place.ts";
import { RUNGS, RUNG_INFO, ZOOM_IN_FROM, approachFrom, checkApproach, fullApproach, oneSmoothZoom, rungDrawn, rungOfPlace, sharedRung, splitApproach, withApproach, SITE_SCREEN, type Approach, type LadderRung } from "../src/approach.ts";
import { PACE_S_PER_DECADE, REDUCED_HOLD_S, SHORT_RUNG_S, journeyAt, journeyEnd, journeyHold, journeyPlan, poseWidthM, stopBefore, widthAt, type JourneyAsk, type JourneyPlan } from "../src/journey.ts";
import type { Pose, Vec3 } from "../src/pose.ts";

const plan = (ask: JourneyAsk): JourneyPlan => {
  const r = journeyPlan(ask);
  if (!r.ok) throw new Error(r.why);
  return r.plan;
};
const rungs = (p: JourneyPlan) => p.path.map((e) => e.rung);
const angle = (a: Vec3, b: Vec3) => Math.acos(Math.max(-1, Math.min(1, a[0] * b[0] + a[1] * b[1] + a[2] * b[2])));
const close = (a: number, b: number, rel = 1e-6) => assert.ok(Math.abs(a - b) <= rel * Math.max(Math.abs(a), Math.abs(b)), `${a} is not ${b}`);
const tarnSite: Place = { scale: "world", system: "", world: "u:k/sys:1/body:3", screen: SITE_SCREEN, site: { lat_deg: 10, lon_deg: 20 } };
const pose: Pose = { about: "u:k/sys:1/body:3", eye_dir: [0, 0.6, 0.8], eye_m: 1800, up: [0, -0.8, 0.6], fov_deg: 60, exposure: { mode: "long", ev: 0 }, keep: [0, 0, 1, 1] };

test("seven rungs, Local stars declared and held, the place line itself unchanged", () => {
  assert.deepEqual([...RUNGS], ["web", "cluster", "galaxy", "stars", "system", "world", "site"]);
  assert.deepEqual([...PLACE_SCALES], ["web", "cluster", "galaxy", "system", "world"]);
  assert.deepEqual(RUNGS.filter((r) => !rungDrawn(r)), ["stars"]);
  assert.match(RUNG_INFO.stars.held!, /conserved budget chain/);
  for (let i = 1; i < RUNGS.length; i++) assert.ok(RUNG_INFO[RUNGS[i]!].stop_m < RUNG_INFO[RUNGS[i - 1]!].stop_m, "each rung narrower than the one above");
});

test("a place's rung, and the rung two places share", () => {
  assert.equal(rungOfPlace(tarnSite), "site");
  assert.equal(rungOfPlace({ ...tarnSite, screen: "globe" }), "world");
  const tarn: Place = { scale: "world", system: "", world: "u:k/sys:1/body:3" };
  const moon: Place = { scale: "world", system: "", world: "u:k/sys:1/body:3/body:0" };
  const other: Place = { scale: "world", system: "u:k/sys:@seed", world: "u:k/sys:@seed/body:1" };
  assert.equal(sharedRung(tarnSite, tarn), "world");
  assert.equal(sharedRung(tarnSite, tarnSite), "site");
  assert.equal(sharedRung(tarn, moon), "system");
  assert.equal(sharedRung(tarn, other), "galaxy");
  assert.equal(sharedRung(tarn, { scale: "cluster", system: "" }), "cluster");
});

test("a way in: defaults, the checks, and reading one back", () => {
  assert.equal(fullApproach(null, "system").from, "galaxy", "the rung above, passing over the held Local stars");
  assert.equal(fullApproach(undefined, "world").from, "system");
  assert.equal(fullApproach({}, "site").pace, "short");
  assert.equal(checkApproach(ZOOM_IN_FROM), null);
  assert.deepEqual(oneSmoothZoom(ZOOM_IN_FROM), { ...ZOOM_IN_FROM, stops: "none", hud: "never", words: "none" });
  assert.match(checkApproach({ from: "moon" })!, /no rung/);
  assert.match(checkApproach({ pace: { total_s: -1 } })!, /pace/);
  assert.match(checkApproach({ go_on: { after_s: 0 } })!, /go on/);
  assert.match(checkApproach({ colour: "red" })!, /unknown key/);
  assert.equal(approachFrom([1]), null);
  assert.deepEqual(approachFrom({ stops: ["site", "galaxy", "galaxy"] }), { stops: ["galaxy", "site"] }, "stops in ladder order");
});

test("the link form rides after the place and comes back the same", () => {
  const a: Approach = { from: "web", stops: ["galaxy", "system"], go_on: { after_s: 4 }, hud: "at_stops", words: "captions", style: "true", pace: { total_s: 60 }, ends: "landing" };
  const site = placeHash(tarnSite);
  const linked = withApproach(site, a);
  assert.equal(linked, `${site}&from=web&stops=galaxy.system&go_on=4&hud=at_stops&words=captions&style=true&pace=60&ends=landing`);
  const back = splitApproach(linked);
  assert.equal(back.hash, site);
  assert.deepEqual(back.approach, a);
  assert.deepEqual(parsePlaceHash(back.hash), parsePlaceHash(site), "the place still reads");
  const galaxy = placeHash({ scale: "galaxy", system: "" });
  assert.equal(withApproach(galaxy, { from: "web", stops: "each" }), `${galaxy}?from=web&stops=each`);
  assert.deepEqual(splitApproach(`${galaxy}?from=web&stops=each`), { hash: galaxy, approach: { from: "web", stops: "each" } });
  assert.equal(withApproach(`${galaxy}?from=web`, null), galaxy, "a way in is replaced or taken off");
  assert.deepEqual(splitApproach(`${galaxy}?from=moon`), { hash: galaxy, approach: null }, "a bad way in is dropped");
});

test("the grand zoom: web to a site, stately, about 45 s, passing over Local stars", () => {
  const p = plan({ to: "site", approach: { from: "web", pace: "stately" } });
  assert.deepEqual(rungs(p), ["web", "cluster", "galaxy", "system", "world", "site"]);
  assert.ok(p.notes.some((n) => n.code === "held" && /Local stars/.test(n.words)));
  const decades = Math.log10(RUNG_INFO.web.stop_m / RUNG_INFO.site.stop_m);
  close(p.total_s, PACE_S_PER_DECADE.stately * (decades + 0.6));
  assert.ok(p.total_s > 40 && p.total_s < 50, `${p.total_s} s`);
  assert.equal(p.seams.length, 5);
  assert.deepEqual(p.stops, []);
  // Width falls all the way, never jumps, and each seam is crossed once, in order.
  let prev = widthAt(p, 0);
  close(prev, RUNG_INFO.web.stop_m);
  for (let t = 0.01; t <= p.total_s; t += 0.01) {
    const w = widthAt(p, t);
    assert.ok(w <= prev * (1 + 1e-12), `width grew at ${t}`);
    assert.ok(Math.log10(prev / w) < 0.01 / PACE_S_PER_DECADE.stately + 1e-6, `width jumped at ${t}`);
    prev = w;
  }
  close(widthAt(p, p.total_s), RUNG_INFO.site.stop_m);
  assert.deepEqual(p.seams.map((s) => [rungs(p)[s.from_entry], rungs(p)[s.to_entry]]), [["web", "cluster"], ["cluster", "galaxy"], ["galaxy", "system"], ["system", "world"], ["world", "site"]]);
  for (let i = 1; i < p.seams.length; i++) assert.ok(p.seams[i]!.t > p.seams[i - 1]!.t);
});

test("paces: brisk, stately and slow are rates; a length is exact; the short form is 0.6 s a rung", () => {
  const len = (pace: Approach["pace"]) => plan({ to: "site", approach: { from: "web", pace } }).total_s;
  assert.ok(len("brisk") < len("stately") && len("stately") < len("slow"));
  assert.ok(len("brisk") > 20 && len("brisk") < 30 && len("slow") > 80 && len("slow") < 100, "about 25, 45 and 90 s (question 8)");
  close(len({ total_s: 60 }), 60);
  close(plan({ to: "world" }).total_s, SHORT_RUNG_S, 1e-9);
  close(plan({ to: "world", approach: { from: "galaxy" } }).total_s, 2 * SHORT_RUNG_S, 1e-9);
});

test("the seam: two views, the outgoing one fading over the incoming one, never at the first or last frame", () => {
  const p = plan({ to: "world", approach: { from: "galaxy", pace: "stately" } });
  const s = p.seams[0]!;
  assert.equal(journeyAt(p, 0).views.length, 1);
  assert.equal(journeyAt(p, p.total_s).views.length, 1);
  const mid = journeyAt(p, s.t);
  assert.deepEqual(mid.views.map((v) => v.rung), ["system", "galaxy"], "incoming under, outgoing over");
  assert.equal(mid.views[0]!.opacity, 1);
  close(mid.views[1]!.opacity, 0.5, 1e-9);
  assert.equal(mid.views[0]!.width_m, mid.views[1]!.width_m, "both zoom onto the same picture width");
  assert.ok(journeyAt(p, s.fade[0] + 0.01).views[1]!.opacity > 0.99);
  assert.equal(journeyAt(p, s.t - 0.01).rung, "galaxy");
  assert.equal(journeyAt(p, s.t + 0.01).rung, "system");
  assert.ok(journeyAt(p, s.t + 0.01).moving, "the zoom never stops at a seam");
  assert.ok(p.path[1]!.t_in < s.t && p.path[0]!.t_out > s.t, "each view is drawn through the whole fade");
});

test("directions: each rung opens, views and leaves as aimed, with no jump, and the end is the bookmark's pose", () => {
  const ax = (x: number, y: number, z: number): Vec3 => {
    const l = Math.hypot(x, y, z);
    return [x / l, y / l, z / l];
  };
  const down = {
    galaxy: { target: "halo:home", good: ax(1, 0, 1), leave: ax(0, 1, 1) },
    system: { target: "u:k/sys:1", arrive: ax(1, 1, 0.2), good: ax(0, 0, 1), leave: ax(1, 0, 0.3) },
    world: { target: "u:k/sys:1/body:3", arrive: ax(0.2, 1, 0.1) },
  };
  const p = plan({ to: "world", approach: { from: "galaxy", pace: "stately" }, down, end: journeyEnd(pose) });
  // At each seam the outgoing view is on its leave and the incoming one on its arrive.
  const s1 = journeyAt(p, p.seams[0]!.t);
  assert.ok(angle(s1.views[1]!.eye_dir, down.galaxy.leave) < 1e-6);
  assert.ok(angle(s1.views[0]!.eye_dir, down.system.arrive) < 1e-6);
  const s2 = journeyAt(p, p.seams[1]!.t);
  assert.ok(angle(s2.views[1]!.eye_dir, down.system.leave) < 1e-6);
  // The system's good angle is reached at its stop width.
  const passing = p.path[1]!;
  let best = Infinity;
  for (let t = passing.t_in; t <= passing.t_out; t += 0.005) {
    const f = journeyAt(p, t);
    const v = f.views.find((x) => x.rung === "system")!;
    if (Math.abs(Math.log10(f.width_m / passing.stop_m)) < 0.01) best = Math.min(best, angle(v.eye_dir, down.system.good));
  }
  assert.ok(best < 0.01, `good angle missed by ${best}`);
  // No jump in any view's line of sight; up stays square to it.
  let last = new Map<string, Vec3>();
  for (let t = 0; t <= p.total_s; t += 0.005) {
    const f = journeyAt(p, t);
    const now = new Map<string, Vec3>();
    for (const v of f.views) {
      const before = last.get(v.rung);
      if (before) assert.ok(angle(before, v.eye_dir) < 0.05, `${v.rung} turned ${angle(before, v.eye_dir)} at ${t}`);
      assert.ok(Math.abs(v.up[0] * v.eye_dir[0] + v.up[1] * v.eye_dir[1] + v.up[2] * v.eye_dir[2]) < 1e-9);
      now.set(v.rung, v.eye_dir);
    }
    last = now;
  }
  const end = journeyAt(p, p.total_s);
  assert.equal(end.views[0]!.target, "u:k/sys:1/body:3");
  close(end.width_m, poseWidthM(pose));
  assert.ok(angle(end.views[0]!.eye_dir, pose.eye_dir) < 1e-9, "ends exactly on the pose");
  assert.ok(angle(end.views[0]!.up, pose.up) < 1e-9);
  assert.ok(end.done);
});

test("stops: each rung on the way down but the last, held for Go on or by themselves", () => {
  const p = plan({ to: "site", approach: { ...ZOOM_IN_FROM, from: "web" } });
  assert.deepEqual(p.stops.map((s) => s.rung), ["web", "cluster", "galaxy", "system", "world"]);
  assert.ok(p.stops.every((s) => s.hold_s === null));
  const at = journeyAt(p, p.stops[2]!.t);
  assert.equal(at.stop?.rung, "galaxy");
  assert.equal(at.hud, true);
  assert.equal(at.words, "captions");
  assert.ok(at.ring && !at.moving);
  close(at.width_m, RUNG_INFO.galaxy.stop_m);
  const between = journeyAt(p, (p.stops[2]!.t + p.stops[3]!.t) / 2);
  assert.ok(between.moving && between.width_line && between.hud === false && between.words === "none" && !between.ring);
  // The player may go no further than the next stop not yet pressed past.
  assert.equal(journeyHold(p, 30, 0), 0, "held at the start");
  assert.equal(journeyHold(p, 30, 2), p.stops[2]!.t);
  assert.equal(journeyHold(p, 999, 5), p.total_s);
  assert.deepEqual(stopBefore(p, p.stops[3]!.t), { t: p.stops[2]!.t, released: 2 });
  assert.deepEqual(stopBefore(p, 0), { t: 0, released: 0 });
  // Going on by itself lengthens the journey by the waits; named stops only stop there.
  const timed = plan({ to: "site", approach: { ...ZOOM_IN_FROM, from: "web", go_on: { after_s: 3 } } });
  close(timed.total_s, p.total_s + 5 * 3);
  assert.deepEqual(plan({ to: "site", approach: { from: "web", stops: ["galaxy", "stars", "site"] } }).stops.map((s) => s.rung), ["galaxy"]);
  // One smooth zoom has no stops and no HUD.
  const smooth = plan({ to: "site", approach: oneSmoothZoom({ ...ZOOM_IN_FROM, from: "web" }) });
  assert.equal(smooth.stops.length, 0);
  assert.equal(journeyAt(smooth, 10).hud, false);
  assert.equal(journeyAt(smooth, 10).width_line, false);
  assert.equal(journeyAt(plan({ to: "world" }), 0.3).hud, null, "the short form leaves the HUD as the viewer has it");
});

test("from here: rise to the rung both places share, turn there, come down to the other place", () => {
  const up = { world: { target: "u:k/sys:1/body:3" }, system: { target: "u:k/sys:1" } };
  const down = { galaxy: { target: "halo:home" }, system: { target: "u:k/sys:@seed" }, world: { target: "u:k/sys:@seed/body:1" } };
  const p = plan({ to: "world", here: "world", shared: "galaxy", approach: { from: "here", pace: "brisk" }, up, down, start_m: 5e7 });
  assert.deepEqual(p.path.map((e) => [e.rung, e.side, e.target]), [["world", "up", up.world.target], ["system", "up", up.system.target], ["galaxy", "turn", "halo:home"], ["system", "down", down.system.target], ["world", "down", down.world.target]]);
  close(widthAt(p, 0), 5e7);
  // Out to the galaxy's width, at rest there, and back in.
  const turn = p.legs.findIndex((l, i) => i > 0 && l.kind === "move" && p.legs[i - 1]!.kind === "move");
  assert.ok(turn > 0);
  close(widthAt(p, p.legs[turn]!.t0), RUNG_INFO.galaxy.stop_m);
  assert.equal(p.stops.length, 0, "the turn is a rest, not a stop, unless stops say so");
  const each = plan({ to: "world", here: "world", shared: "galaxy", approach: { from: "here", stops: "each" } });
  assert.deepEqual(each.stops.map((s) => s.rung), ["galaxy", "system"], "never on the way up");
  // Down from here inside one system: no rising, no stop at the start.
  const inside = plan({ to: "site", here: "system", approach: { from: "here", stops: "each" } });
  assert.deepEqual(rungs(inside), ["system", "world", "site"]);
  assert.deepEqual(inside.stops.map((s) => s.rung), ["world"]);
  assert.equal(journeyPlan({ to: "world", approach: { from: "here" } }).ok, false);
});

test("a role travels only from as high as it knows", () => {
  const p = plan({ to: "world", ceiling: "system", approach: { from: "web", stops: "each" } });
  assert.equal(p.from, "system");
  assert.deepEqual(rungs(p), ["system", "world"]);
  assert.deepEqual(p.notes.map((n) => n.code), ["knowledge"]);
  assert.match(p.notes[0]!.words, /Starts at System/);
  const none = plan({ to: "system", ceiling: "system", approach: { from: "web" } });
  assert.equal(none.total_s, 0, "nothing above it known: it simply arrives");
  assert.equal(journeyAt(none, 0).done, true);
});

test("held rungs: never arrived at, passed over, and a journey asked to start there starts above", () => {
  const to = journeyPlan({ to: "stars" });
  assert.equal(to.ok, false);
  assert.match((to as { why: string }).why, /held/);
  const from = plan({ to: "system", approach: { from: "stars" } });
  assert.equal(from.from, "galaxy");
  assert.ok(from.notes.some((n) => n.code === "held"));
  // When Local stars has a view (GZ7, after the star field settles), it is simply one more rung.
  const drawn = plan({ to: "system", approach: { from: "galaxy" }, drawn: () => true });
  assert.deepEqual(rungs(drawn), ["galaxy", "stars", "system"]);
  assert.equal(drawn.notes.length, 0);
});

test("reduced motion: nothing zooms; rungs are held about two seconds and change by the plain fade", () => {
  const p = plan({ to: "site", approach: { from: "web", pace: "stately" }, reduced_motion: true, end: journeyEnd(pose) });
  assert.ok(p.legs.every((l) => l.kind === "rest"));
  assert.equal(p.seams.length, 5);
  assert.ok(Math.abs(p.total_s - (5 * REDUCED_HOLD_S + 5 * 0.15)) < 1e-9);
  const f = journeyAt(p, (p.seams[0]!.fade[0] + p.seams[0]!.fade[1]) / 2);
  assert.deepEqual(f.views.map((v) => v.width_m), [RUNG_INFO.cluster.stop_m, RUNG_INFO.web.stop_m], "each view keeps its own width");
  close(journeyAt(p, p.total_s).width_m, poseWidthM(pose));
  const short = plan({ to: "world", approach: { from: "galaxy" }, reduced_motion: true });
  assert.equal(short.seams.length, 1, "the short form is one fade");
  assert.deepEqual(short.seams.map((s) => [short.path[s.from_entry]!.rung, short.path[s.to_entry]!.rung]), [["galaxy", "world"]]);
  close(short.total_s, 0.15);
});

test("a pure function of journey time: the same frame every time, and the plan survives being sent", () => {
  const p = plan({ to: "site", approach: { ...ZOOM_IN_FROM, from: "cluster", go_on: { after_s: 2 } }, end: journeyEnd(pose) });
  const sent = JSON.parse(JSON.stringify(p)) as JourneyPlan;
  for (const t of [0, 0.1, 3.3, 7, 12.5, p.total_s / 2, p.total_s, p.total_s + 5, -1]) {
    assert.deepEqual(journeyAt(p, t), journeyAt(p, t));
    assert.deepEqual(journeyAt(sent, t), journeyAt(p, t));
  }
  // Played backwards is the same journey the other way: widths at mirrored times step through the same values.
  const ts = [1, 2, 3].map((x) => (x * p.total_s) / 4);
  assert.deepEqual(ts.map((t) => widthAt(p, t)), ts.map((t) => widthAt(sent, t)));
});

test("ends with the landing only at a site", () => {
  assert.equal(plan({ to: "site", approach: { from: "world", ends: "landing" } }).landing, true);
  const w = plan({ to: "world", approach: { from: "system", ends: "landing" } });
  assert.equal(w.landing, false);
  assert.equal(w.notes[0]!.code, "landing");
});

test("every rung the ladder names can be planned to, except the held one", () => {
  for (const r of RUNGS as readonly LadderRung[]) {
    const p = journeyPlan({ to: r, approach: r === "web" ? { from: "here" } : {}, here: "web" });
    assert.equal(p.ok, rungDrawn(r), r);
  }
});
