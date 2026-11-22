// The Field layer's tracer (MF14a part 1): coefficients to field, field to lines, lines against the bubble.
import { test } from "node:test";
import assert from "node:assert/strict";
import { FIELD_LINES, bubbleOutline, chartCut, compressed, inBubble, cutPaths, drawField3D, project3D, dipole, dipoleAxis, fieldAt, insideBubble, lineBuffers, narrateField, schmidt, traceField, traceLine, type V3 } from "../src/field.ts";

const near = (a: number, b: number, tol: number, what: string) => assert.ok(Math.abs(a - b) <= tol, `${what}: ${a} against ${b}`);

test("Schmidt semi-normalised functions match their closed forms, and dP/dθ matches a finite difference", () => {
  const th = 0.7;
  const { P, dP } = schmidt(4, th);
  const c = Math.cos(th);
  const s = Math.sin(th);
  near(P[1]![0]!, c, 1e-14, "P10");
  near(P[1]![1]!, s, 1e-14, "P11");
  near(P[2]![0]!, 1.5 * c * c - 0.5, 1e-14, "P20");
  near(P[2]![1]!, Math.sqrt(3) * c * s, 1e-14, "P21");
  near(P[2]![2]!, (Math.sqrt(3) / 2) * s * s, 1e-14, "P22");
  near(P[3]![3]!, Math.sqrt(10) / 4 * s * s * s, 1e-14, "P33");
  const e = 1e-6;
  const up = schmidt(4, th + e).P;
  const dn = schmidt(4, th - e).P;
  for (let n = 1; n <= 4; n++) for (let m = 0; m <= n; m++) near(dP[n]![m]!, (up[n]![m]! - dn[n]![m]!) / (2 * e), 1e-8, `dP${n}${m}`);
});

test("an aligned dipole: B0 at the equator, twice that at the poles, falling as 1/r³", () => {
  const f = dipole(6.4e6, 3e-5);
  const eq = fieldAt(f, [1, 0, 0]);
  near(Math.hypot(...eq), 3e-5, 1e-18, "equator");
  assert.ok(eq[2] > 0, "Earth's sign: at the equator the field points north");
  const pole = fieldAt(f, [0, 0, 1]);
  near(pole[2], -6e-5, 1e-18, "north pole, pointing in");
  near(Math.hypot(...fieldAt(f, [2, 0, 0])), 3e-5 / 8, 1e-18, "two radii");
  // The field is the gradient of the potential: its curl vanishes (checked as a loop integral around a small square).
  const loop = (p: V3, h: number) => {
    const pts: V3[] = [[p[0], p[1], p[2]], [p[0] + h, p[1], p[2]], [p[0] + h, p[1] + h, p[2]], [p[0], p[1] + h, p[2]]];
    let sum = 0;
    for (let i = 0; i < 4; i++) {
      const a = pts[i]!;
      const b = pts[(i + 1) % 4]!;
      const mid: V3 = [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (a[2] + b[2]) / 2];
      const B = fieldAt({ ...f, g: [[], [1, 0.3, 0.1], [0.2, 0.1, 0.05]], h: [[], [0, 0.2], [0, 0.3, 0.1]] }, mid);
      sum += B[0] * (b[0] - a[0]) + B[1] * (b[1] - a[1]) + B[2] * (b[2] - a[2]);
    }
    return sum;
  };
  assert.ok(Math.abs(loop([1.3, 0.4, 0.5], 1e-3)) < 1e-9, "curl-free");
});

test("a traced dipole line follows r = L cos² λ and lands at the mirror footpoint", () => {
  const f = dipole(1, 1);
  for (const lat of [30, 50, 70]) {
    const l = traceLine(f, null, lat, 20, { step: 0.005 });
    assert.equal(l.kind, "closed");
    const L = 1 / Math.cos((lat * Math.PI) / 180) ** 2;
    let worst = 0;
    for (const p of l.points) {
      const r = Math.hypot(...p);
      const cl = Math.hypot(p[0], p[1]) / r;
      worst = Math.max(worst, Math.abs(r - L * cl * cl) / L);
    }
    assert.ok(worst < 1e-4, `lat ${lat}: worst ${worst}`);
    // The default step (4% of the distance) stays within a thousandth of the true line.
    const d = traceLine(f, null, lat, 20);
    for (const p of d.points) {
      const r = Math.hypot(...p);
      const cl = Math.hypot(p[0], p[1]) / r;
      assert.ok(Math.abs(r - L * cl * cl) / L < 1e-3, `default step at lat ${lat}`);
    }
    const end = l.points.at(-1)!;
    near((Math.asin(end[2]) * 180) / Math.PI, -lat, 0.05, `lands at ${-lat}`);
    near((Math.atan2(end[1], end[0]) * 180) / Math.PI, 20, 0.05, "same longitude");
  }
});

test("the wind's field opens the polar lines and keeps the low ones closed; a tilt carries the axis", () => {
  // A southward wind field about Earth's share of its own (5 nT against 31 µT).
  const bubble = { standoff_radii: 10, flaring: 0.58, sunward: [1, 0, 0] as V3, wind_field_t: [0, 0, -1.6e-4] as V3 };
  assert.ok(insideBubble(bubble, [9.9, 0, 0]) && !insideBubble(bubble, [10.1, 0, 0]));
  // Earth's oval maps near the nose: a footpoint at 60° (L = 4) is closed, one at 80° opens.
  const f = dipole(1, 1);
  const inside = inBubble(f, bubble);
  assert.equal(traceLine(inside, bubble, 60, 0).kind, "closed");
  assert.equal(traceLine(inside, bubble, 80, 0).kind, "open");
  const t = traceField(f, bubble);
  // Without the wind's field (no reconnection) the polar cap closes: only lines near the cusps and far on the night side
  // reach Shue's surface, which the image dipole (a flat mirror at the nose) fits only near the nose.
  const shut = traceField(f, { ...bubble, wind_field_t: undefined });
  assert.ok(shut.counts.open < t.counts.open, `${JSON.stringify(shut.counts)} against ${JSON.stringify(t.counts)}`);
  assert.ok(t.counts.closed > 0 && t.counts.open > 0 && t.counts.unfinished === 0, JSON.stringify(t.counts));
  const tilted = dipole(1, 1, 11, 290);
  const ax = dipoleAxis(tilted);
  near((Math.acos(ax[2]) * 180) / Math.PI, 11, 1e-9, "tilt");
  // Earth's north magnetic pole leans toward longitude 290°E (about 72°W).
  near(((Math.atan2(ax[1], ax[0]) * 180) / Math.PI + 360) % 360, 290, 1e-9, "toward");
  // The squeeze: at the nose the field along the boundary doubles and nothing crosses it (Chapman and Ferraro).
  const sq = compressed(f, { ...bubble, wind_field_t: undefined });
  const nose = fieldAt(sq, [10, 0, 0]);
  near(nose[0], 0, 1e-15, "no field through the nose");
  near(nose[2], 2 / 1000, 1e-15, "doubled at the nose");
  const flank = fieldAt(sq, [10, 3, 4]);
  near(flank[0], 0, 1e-15, "none through the plane at the flank either");
  const o = bubbleOutline(bubble);
  near(o[Math.floor(o.length / 2)]!.x, 10, 1e-12, "nose");
});

test("Chart cut, 3D buffers and words: one trace, three readings", () => {
  const f = dipole(6.4e6, 3.1e-5, 11, 290);
  const bubble = { standoff_radii: 10, flaring: 0.58, sunward: [1, 0, 0] as V3, wind_field_t: [0, 0, -5e-9] as V3 };
  const c = chartCut(f, bubble);
  assert.ok(c.lines.length > 10 && c.bubble!.length > 50);
  near(c.tilt_deg, 11, 1e-9, "tilt in the cut");
  near(Math.hypot(c.magneticAxis.x, c.magneticAxis.y), 1, 1e-12, "axis in plane");
  assert.ok(c.lines.some((l) => l.kind === "open") && c.lines.some((l) => l.kind === "closed"));
  const t = traceField(f, bubble);
  const b = lineBuffers(t);
  assert.equal(b.starts.length, t.lines.length);
  assert.equal(b.positions.length, 3 * b.brightness.length);
  assert.ok(Math.max(...b.brightness) <= 1 + 1e-6 && Math.min(...b.brightness) >= -1e-6);
  assert.match(narrateField(f, bubble, t), /^Field 31 µT at the equator, tilted 11° from the spin axis; holds the wind off at 10 radii\. \d+ of the lines drawn loop back to the body and \d+ open into the wind\.$/);
});

test("cost: the default trace stays within a few milliseconds on this machine (budget in design 9B)", () => {
  const f = { radius_m: 1, g: [[], [-1, 0.1], [0.1, 0.05, 0.02], [0.03, 0.02, 0.01, 0.01]], h: [[], [0, 0.08], [0, 0.04, 0.02], [0, 0.01, 0.01, 0.005]] };
  const bubble = { standoff_radii: 10, flaring: 0.58, sunward: [1, 0, 0] as V3 };
  traceField(f, bubble);
  const t0 = performance.now();
  const t = traceField(f, bubble);
  const ms = performance.now() - t0;
  const pts = t.lines.reduce((a, l) => a + l.points.length, 0);
  console.log(`trace: ${t.lines.length} lines, ${pts} points, degree 3, ${ms.toFixed(1)} ms`);
  assert.ok(ms < 200, `${ms} ms`);
});

test("3D: lines behind the body are cut, those in front and beside it stay; the cut's paths fit the frame", () => {
  const f = dipole(1, 1);
  const t = traceField(f, { standoff_radii: 10, flaring: 0.58, sunward: [1, 0, 0] });
  const cam = { lon_deg: 0, lat_deg: 0, scale: 50, cx: 200, cy: 200 };
  const runs = project3D(t, cam);
  // A line's point straight behind the body (x < 0, inside the disc) never appears.
  const behind = t.lines.flatMap((l) => l.points).filter((p) => p[0] < -0.2 && p[1] * p[1] + p[2] * p[2] < 0.8);
  assert.ok(behind.length > 0);
  for (const r of runs)
    for (let i = 0; i < r.xy.length; i += 2) {
      const dx = (r.xy[i]! - 200) / 50;
      const dy = (200 - r.xy[i + 1]!) / 50;
      if (dx * dx + dy * dy < 0.81) {
        // Inside the disc: must be a point in front (toward the camera at +x).
        const l = t.lines[r.line]!;
        assert.ok(l.points.some((p) => p[0] > 0 && Math.abs(p[1] - dx) < 0.05 && Math.abs(p[2] - dy) < 0.05), "a point over the disc is in front of it");
      }
    }
  let calls = 0;
  const pen = { beginPath() {}, moveTo() {}, lineTo() {}, stroke() { calls++; }, arc() {}, strokeStyle: "", globalAlpha: 1, lineWidth: 1 };
  drawField3D(pen, runs, cam, { out: "orange", in: "blue", body: "grey" });
  assert.ok(calls > runs.length, `${calls} strokes for ${runs.length} runs`);
  const c = cutPaths(chartCut(f, { standoff_radii: 10, flaring: 0.58, sunward: [1, 0, 0] }), 400, 14);
  assert.ok(c.bubble!.startsWith("M") && c.lines.every((l) => l.d === "" || l.d.startsWith("M")));
  near(c.body.r, 400 / 28, 1e-12, "body radius in px");
});

test("the views' fixture: a dipole Earth inside its bubble traces closed and open lines", async () => {
  const { earthField } = await import("./field-fixture.ts");
  const { model, bubble } = earthField();
  const t = traceField(model, bubble);
  assert.ok(t.counts.closed > 50 && t.counts.open > 0, JSON.stringify(t.counts));
  assert.match(narrateField(model, bubble, t), /^Field 31 µT at the equator, tilted 9° from the spin axis; holds the wind off at 9\.3 radii\./);
});

test("part 1b: read_field's Solar System fixture becomes the layer's model and bubble", async () => {
  const { readFileSync } = await import("node:fs");
  const { Session } = await import("../../ui-contract/src/session.ts");
  const { gmSeat } = await import("../../ui-contract/src/seat.ts");
  const { nodeKernel } = await import("../../ui-contract/test/node-kernel.ts");
  const { fieldFromRead, dipoleB0 } = await import("../src/field.ts");
  const k = nodeKernel();
  try {
    const pack = (n: string) => JSON.parse(readFileSync(new URL(`../../../packs/${n}/pack.json`, import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "mag", seed: "mag" }, [pack("base-physics"), pack("base-reference")]);
    const r = await s.readField({ fixture: "solar-system" });
    assert.ok(r.ok, JSON.stringify(r));
    const bodies = r.bodies as never as Parameters<typeof fieldFromRead>[0][];
    const earth = fieldFromRead(bodies.find((b) => b.name === "Earth")!)!;
    // IGRF's g10 is about −29.4 µT; the fixture's dipole from the measured moment lands near it.
    near(dipoleB0(earth.model) * 1e6, 30.6, 1.5, "Earth's B0, µT");
    assert.ok(earth.bubble && earth.bubble.standoff_radii > 7 && earth.bubble.standoff_radii < 12);
    // Parker's spiral at Earth: about 45°, so the radial and azimuthal parts are about equal.
    const wf = earth.bubble.wind_field_t!;
    near(Math.abs(wf[0]) / Math.abs(wf[1]), 1.1, 0.2, "spiral");
    const t = traceField(earth.model, earth.bubble);
    assert.ok(t.counts.closed > 50 && t.counts.open > 0, JSON.stringify(t.counts));
    for (const name of ["Mercury", "Jupiter", "Saturn", "Uranus", "Neptune"]) {
      const b = fieldFromRead(bodies.find((x) => x.name === name)!)!;
      assert.ok(b && b.bubble, name);
      const tr = traceField(b.model, b.bubble, FIELD_LINES.canvas!);
      assert.ok(tr.counts.closed > 0, `${name}: ${JSON.stringify(tr.counts)}`);
    }
  } finally {
    await k.close();
  }
});
