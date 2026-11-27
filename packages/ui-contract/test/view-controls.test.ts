// The standard view controls (rule 12): every option always listed, the gaps quiet with reasons, a flat chart turning
// into 3D, the kept choice read back safely.
import { test } from "node:test";
import assert from "node:assert/strict";
import { firstView, flatChart, pickScale, pickSeen, pickStyle, readViewChoice, viewOptions, WHY_TRUE_HELD, type ViewControlsSpec } from "../src/view-controls.ts";

const system: ViewControlsSpec = { styles: { chart: true, "3d": true, true: WHY_TRUE_HELD }, flat: true };

test("every option is listed in the same order; what a view cannot do says why", () => {
  const o = viewOptions(system, { style: "chart", seen: "tilted" });
  assert.deepEqual(o.styles.map((x) => x.label), ["Chart", "3D", "True", "Wireframe"]);
  assert.deepEqual(o.seen.map((x) => x.label), ["Above", "Tilted", "Edge-on", "Free"]);
  assert.equal(o.styles[2]!.unavailable, WHY_TRUE_HELD);
  assert.match(o.styles[3]!.unavailable!, /not drawn here yet/);
  assert.deepEqual(o.shown, { style: "chart", seen: "above", scale: "readable", hud: true, band: "visible", framed: 0, exposure: "eye" }, "a flat chart is seen from above");
  assert.ok(o.seen.every((x) => !x.unavailable), "turning a flat chart is offered when it has 3D");
});

test("turning a flat chart makes it 3D; a fixed view on 3D stays 3D", () => {
  const c = pickSeen(system, { style: "chart", seen: "tilted" }, "edge");
  assert.deepEqual(c, { style: "3d", seen: "edge", framed: 1 });
  assert.deepEqual(pickSeen(system, c, "free"), { style: "3d", seen: "free", framed: 1 });
  assert.deepEqual(pickSeen(system, { style: "chart", seen: "tilted" }, "above"), { style: "chart", seen: "tilted", framed: 1 });
  assert.deepEqual(pickStyle(c, "chart"), { style: "chart", seen: "edge", framed: 1 });
});

test("picking a fixed Seen from reframes, even the one shown; Free keeps the hands' view (Alex, 24 Nov)", () => {
  const a = pickSeen(system, { style: "3d", seen: "free" }, "tilted");
  const b = pickSeen(system, a, "tilted");
  assert.equal(b.seen, "tilted");
  assert.ok((b.framed ?? 0) > (a.framed ?? 0), "re-picking the view shown frames it again");
  assert.equal(pickSeen(system, b, "free").framed, b.framed, "Free does not move the camera");
  assert.equal(viewOptions(system, b).shown.framed, b.framed);
});

test("a style a view cannot draw falls back to Chart, and the choice is kept for the next view", () => {
  const galaxy = flatChart();
  const o = viewOptions(galaxy, { style: "3d", seen: "free" });
  assert.deepEqual(o.shown, { style: "chart", seen: "above", scale: "readable", hud: true, band: "visible", framed: 0, exposure: "eye" });
  assert.ok(o.seen.slice(1).every((x) => x.unavailable), "a flat-only view cannot be turned");
  assert.equal(o.turnable, false);
  assert.deepEqual(viewOptions(system, { style: "3d", seen: "free" }).shown, { style: "3d", seen: "free", scale: "readable", hud: true, band: "visible", framed: 0, exposure: "eye" });
});

test("a kept choice is read back safely", () => {
  assert.deepEqual(readViewChoice('{"style":"3d","seen":"edge"}'), { style: "3d", seen: "edge", scale: "readable" }, "a choice kept before Scale reads as Readable");
  assert.deepEqual(readViewChoice('{"style":"3d","seen":"edge","scale":"real"}'), { style: "3d", seen: "edge", scale: "real" });
  assert.deepEqual(readViewChoice('{"style":"sepia","seen":7,"scale":"huge"}'), { style: "chart", seen: "tilted", scale: "readable" });
  assert.deepEqual(readViewChoice("oops"), { style: "chart", seen: "tilted", scale: "readable" });
});

test("a first visit opens in True on every device (alpha), and nothing kept reads as the first view", () => {
  assert.equal(firstView().style, "true");
  assert.equal(firstView("low").style, "true", "phones are not gated during the alpha");
  assert.deepEqual(readViewChoice(null, firstView()), { style: "true", seen: "tilted", scale: "readable" });
  assert.deepEqual(readViewChoice('{"style":"chart","seen":"above"}', firstView()), { style: "chart", seen: "above", scale: "readable" }, "a kept choice wins");
  assert.deepEqual(readViewChoice(null), { style: "chart", seen: "tilted", scale: "readable" }, "views without a first view keep the default");
});

test("Scale: Readable by default, Real where a view draws it; Real turns a flat chart into 3D, which is always Readable", () => {
  const o = viewOptions(system, { style: "3d", seen: "free" });
  assert.deepEqual(o.scales.map((x) => x.label), ["Readable", "Real"]);
  assert.match(o.scales[1]!.unavailable!, /not drawn to scale/i, "a view that does not say it draws Real lists it quiet");
  assert.equal(viewOptions(system, { style: "3d", seen: "free", scale: "real" }).shown.scale, "readable");
  const both: ViewControlsSpec = { styles: { chart: true, "3d": true, true: true }, flat: true, scale: { readable: true, real: true } };
  assert.equal(viewOptions(both, { style: "3d", seen: "free", scale: "real" }).shown.scale, "real");
  assert.equal(viewOptions(both, { style: "chart", seen: "above", scale: "real" }).shown.scale, "readable", "the flat chart is a diagram");
  assert.deepEqual(pickScale(both, { style: "chart", seen: "above" }, "real"), { style: "3d", seen: "tilted", scale: "real" });
  assert.deepEqual(pickScale(both, { style: "true", seen: "free" }, "real"), { style: "true", seen: "free", scale: "real" });
});

test("HUD: off in True (a photograph), on in the diagrams, one choice through every style once made", () => {
  const spec: ViewControlsSpec = { styles: { chart: true, "3d": true, true: true }, flat: true, hud: { "3d": true, true: true } };
  assert.equal(viewOptions(spec, { style: "true", seen: "free" }).shown.hud, false);
  assert.equal(viewOptions(spec, { style: "3d", seen: "free" }).shown.hud, true);
  assert.equal(viewOptions(spec, { style: "3d", seen: "free", hud: false }).shown.hud, false);
  assert.equal(viewOptions(spec, { style: "true", seen: "free", hud: true }).shown.hud, true);
  const flat = viewOptions(spec, { style: "chart", seen: "above", hud: false });
  assert.equal(flat.shown.hud, true, "a style without the switch keeps its labels");
  assert.match(flat.huds[0]!.unavailable!, /always shows its labels/);
  assert.equal(readViewChoice('{"style":"true","seen":"free","hud":false}').hud, false);
  assert.equal(readViewChoice('{"style":"true","seen":"free"}').hud, undefined);
});

test("Band: Visible by default; the others listed quiet until a view declares them; a kept band reads back", () => {
  const o = viewOptions(system, { style: "chart", seen: "above", band: "xray" });
  assert.deepEqual(o.bands.map((b) => b.label), ["Visible", "Infrared", "Ultraviolet", "X-ray", "Radio", "Mag", "Composite"]);
  assert.match(o.bands[6]!.unavailable!, /needs two bands/, "Composite is listed on every view, quiet where only one band is drawn");
  assert.equal(o.shown.band, "visible");
  assert.match(o.bands[3]!.unavailable!, /X-ray is not drawn here yet/);
  const lit: ViewControlsSpec = { ...system, bands: { visible: true, xray: true, mag: true } };
  assert.equal(viewOptions(lit, { style: "chart", seen: "above", band: "xray" }).shown.band, "xray");
  assert.equal(readViewChoice('{"style":"chart","seen":"above","band":"mag"}').band, "mag");
  assert.equal(readViewChoice('{"style":"chart","seen":"above","band":"gamma"}').band, undefined);
});

test("Scale on a flat slice: Real is drawn flat when the view says it can; a flat chart that turns shows Real in 3D", () => {
  const slice: ViewControlsSpec = { ...flatChart("A slice"), scale: { readable: true, real: true } };
  assert.equal(viewOptions(slice, { style: "chart", seen: "above", scale: "real" }).shown.scale, "real");
  assert.deepEqual(pickScale(slice, { style: "chart", seen: "above" }, "real"), { style: "chart", seen: "above", scale: "real" });
  assert.equal(viewOptions(flatChart(), { style: "chart", seen: "above", scale: "real" }).shown.scale, "readable", "Real not declared");
  assert.equal(viewOptions({ ...system, scale: { readable: true, real: true } }, { style: "chart", seen: "above", scale: "real" }).shown.scale, "readable", "the system's flat log chart");
});

test("the band switch reads its words from the master sheet's Bands table, one view row per band", async () => {
  const { SHEET_BANDS } = await import("../src/sheet.ts");
  const { VIEW_BANDS, VIEW_BAND_WORDS, VIEW_BAND_PLAIN } = await import("../src/view-controls.ts");
  const views = SHEET_BANDS.filter((b) => b.kind === "view").map((b) => b.view_id);
  assert.deepEqual([...VIEW_BANDS].sort(), [...views].sort() as string[]);
  assert.equal(VIEW_BAND_WORDS.xray, "X-ray");
  assert.equal(VIEW_BAND_PLAIN.infrared, "heat light");
});

test("True opens at a long exposure where the view draws one (Alex, 24 Nov), Eye is the choice; elsewhere the eye's (VP2)", () => {
  const spec = { styles: { chart: true as const, true: true as const }, long: true as const };
  assert.equal(viewOptions(spec, { style: "true", seen: "free" }).shown.exposure, "long");
  assert.equal(viewOptions(spec, { style: "true", seen: "free", exposure: "eye" }).shown.exposure, "eye");
  assert.equal(readViewChoice(JSON.stringify({ style: "true", seen: "free", exposure: "eye" })).exposure, "eye");
  assert.equal(viewOptions(spec, { style: "true", seen: "free", exposure: "long" }).shown.exposure, "long");
  assert.equal(viewOptions(spec, { style: "chart", seen: "above", exposure: "long" }).shown.exposure, "eye");
  assert.ok(viewOptions({ styles: { true: true } }, { style: "true", seen: "free", exposure: "long" }).exposures.find((e) => e.value === "long")!.unavailable);
  assert.equal(readViewChoice(JSON.stringify({ style: "true", seen: "free", exposure: "long" })).exposure, "long");
});
