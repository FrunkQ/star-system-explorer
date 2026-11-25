// Composite (exotic objects design 6A, XO17; Alex 24 Nov: "the composite view should be everywhere!"): the last entry
// of the band switch on every view, a tick list mixing bands, presets as data, honest legend, seat and phone limits.
import { test } from "node:test";
import assert from "node:assert/strict";
import { baseBand, readViewChoice, viewOptions, VIEW_BAND_WORDS, type ViewControlsSpec } from "../src/view-controls.ts";
import { DEFAULT_MIX, PHONE_LAYERS, WHY_PHONE_LAYERS, compositeLegend, compositePresets, fitMix, presetMix, recolour, toggleLayer, withCompositePresets } from "../src/composite.ts";

const all: ViewControlsSpec = { styles: { chart: true, "3d": true, true: true }, bands: { visible: true, infrared: true, ultraviolet: true, xray: true, radio: true, mag: true }, composite: { chart: true, "3d": true, true: true } };
const two: ViewControlsSpec = { styles: { chart: true }, bands: { visible: true, infrared: true }, composite: { chart: true } };

test("Composite is the band switch's last entry and shows the Observatory mix on its first press", () => {
  const o = viewOptions(all, { style: "true", seen: "tilted", band: "composite" });
  assert.equal(o.bands.at(-1)!.value, "composite");
  assert.equal(o.bands.at(-1)!.unavailable, undefined);
  assert.equal(o.composite, true);
  assert.deepEqual(o.layers.map((l) => l.band), ["xray", "visible", "infrared", "radio"]);
  assert.equal(o.shown.band, "visible", "a view that draws one band shows the mix's Visible layer");
  assert.deepEqual(DEFAULT_MIX, presetMix("observatory"));
});

test("a single band is one layer, so every view reads layers the same way", () => {
  const o = viewOptions(all, { style: "true", seen: "tilted", band: "xray" });
  assert.equal(o.composite, false);
  assert.deepEqual(o.layers, [{ band: "xray", colour: "#b36bff" }]);
});

test("a mix is cut to what the view draws, so the same choice works on every view", () => {
  const o = viewOptions(two, { style: "chart", seen: "above", band: "composite" });
  assert.deepEqual(o.layers.map((l) => l.band), ["visible", "infrared"], "Observatory on a view with two bands");
  const notYet = viewOptions({ ...two, composite: undefined }, { style: "chart", seen: "above", band: "composite" });
  assert.equal(notYet.composite, false, "a view that does not lay layers yet says so and shows one band");
  assert.match(notYet.bands.at(-1)!.unavailable!, /coming to this view/);
  const visibleOnly = viewOptions({ styles: { chart: true }, composite: { chart: true } }, { style: "chart", seen: "above", band: "composite" });
  assert.equal(visibleOnly.composite, false);
  assert.equal(visibleOnly.shown.band, "visible");
});

test("a seat ticks only the bands its sensors carry; Composite is never a way round that", () => {
  const o = viewOptions(all, { style: "true", seen: "tilted", band: "composite" }, { seat: ["visible", "infrared"] });
  assert.deepEqual(o.layers.map((l) => l.band), ["visible", "infrared"]);
  assert.match(o.bands.find((b) => b.value === "xray")!.unavailable!, /sensors/);
  assert.equal(viewOptions(all, { style: "true", seen: "tilted", band: "xray" }, { seat: ["visible"] }).shown.band, "visible");
});

test("a phone stops at two bands and says why on the third", () => {
  const o = viewOptions(all, { style: "true", seen: "tilted", band: "composite" }, { maxLayers: PHONE_LAYERS });
  assert.equal(o.layers.length, 2);
  const limits = { can: o.drawable, max: PHONE_LAYERS };
  const r = toggleLayer({ layers: o.layers }, "radio", limits);
  assert.deepEqual(r, { refused: WHY_PHONE_LAYERS });
  const off = toggleLayer({ layers: o.layers }, o.layers[0]!.band, limits);
  assert.ok(!("refused" in off) && off.layers.length === 1);
});

test("the legend lists every layer and its colour and says mapped once any band but Visible is in", () => {
  const l = compositeLegend(DEFAULT_MIX, VIEW_BAND_WORDS);
  assert.equal(l.text, "Composite: X-ray violet · Visible actual · Infrared red · Radio orange · each band scaled to its own brightest · mapped");
  assert.equal(l.mapped, true);
  assert.equal(compositeLegend({ layers: [{ band: "visible", colour: "natural" }] }, VIEW_BAND_WORDS).mapped, false);
});

test("colours change per layer; Visible keeps its own; bad colours are refused", () => {
  const m = recolour(DEFAULT_MIX, "radio", "#00FF00");
  assert.equal(m.layers.find((l) => l.band === "radio")!.colour, "#00ff00");
  assert.deepEqual(recolour(DEFAULT_MIX, "visible", "#00ff00"), DEFAULT_MIX);
  assert.deepEqual(recolour(DEFAULT_MIX, "radio", "red"), DEFAULT_MIX);
  assert.equal(fitMix({ layers: [{ band: "xray", colour: "nonsense" }, { band: "xray", colour: "#123456" }] }, { can: ["xray"] }).layers.length, 1);
});

test("presets are data a pack can add to", () => {
  assert.deepEqual(compositePresets().map((p) => p.name).slice(0, 3), ["Observatory", "Hot and cold", "Radio over visible"]);
  withCompositePresets([{ id: "test-pack", name: "Test", layers: ["visible", "mag"], why: "a pack's own" }]);
  assert.deepEqual(presetMix("test-pack")!.layers.map((l) => l.band), ["visible", "mag"]);
});

test("the mix is kept on the device and read back safely; baseBand gives the one band where only one is used", () => {
  const c = readViewChoice(JSON.stringify({ style: "true", seen: "tilted", band: "composite", composite: { preset: "hot-and-cold", layers: [{ band: "xray", colour: "#b36bff" }, { band: "gamma", colour: "#fff" }, { band: "infrared", colour: "oops" }] } }));
  assert.equal(c.band, "composite");
  assert.deepEqual(c.composite, { preset: "hot-and-cold", layers: [{ band: "xray", colour: "#b36bff" }, { band: "infrared", colour: "#ff4a3d" }] });
  assert.equal(baseBand(c), "xray");
  assert.equal(baseBand({ style: "chart", seen: "above", band: "composite" }), "visible");
  assert.equal(baseBand(undefined), "visible");
});
