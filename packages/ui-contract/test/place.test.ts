// The place line (U24): the ladder of scales, the trail back down, recent places and deep links.
import { test } from "node:test";
import assert from "node:assert/strict";
import { nextTrail, parsePlaceHash, placeHash, placeSteps, readRecent, rememberPlace, type Place } from "../src/place.ts";

const names = { system: "Kestrel", world: "Tarn" };
const sys: Place = { scale: "system", system: "" };
const tarn: Place = { scale: "world", system: "", world: "u:kestrel/sys:1/body:3" };
const galaxy: Place = { scale: "galaxy", system: "" };

test("the line runs from the cosmic web down to the place in view", () => {
  const s = placeSteps(tarn, null, names);
  assert.deepEqual(s.map((x) => x.label), ["Cosmic web", "Galaxies nearby", "Galaxy", "Kestrel", "Tarn"]);
  assert.deepEqual(s.map((x) => x.current), [false, false, false, false, true]);
  assert.ok(s.every((x) => !x.below));
});

test("going up keeps the way back down on the line", () => {
  const trail = nextTrail(nextTrail(null, tarn), galaxy);
  assert.deepEqual(trail, tarn);
  const s = placeSteps(galaxy, trail, names);
  assert.deepEqual(s.map((x) => x.label), ["Cosmic web", "Galaxies nearby", "Galaxy", "Kestrel", "Tarn"]);
  assert.deepEqual(s.filter((x) => x.below).map((x) => x.label), ["Kestrel", "Tarn"]);
  assert.equal(s.find((x) => x.current)!.label, "Galaxy");
  assert.deepEqual(s.at(-1)!.place, { scale: "world", system: "", world: tarn.world });
});

test("going down, or to another system, starts a new trail", () => {
  assert.deepEqual(nextTrail(tarn, sys), tarn);
  const moon: Place = { scale: "world", system: "", world: "u:kestrel/sys:1/body:3/body:0" };
  assert.deepEqual(nextTrail(tarn, moon), moon);
  const other: Place = { scale: "galaxy", system: "u:kestrel/sys:@seed" };
  assert.deepEqual(nextTrail(tarn, other), other);
  assert.deepEqual(placeSteps(other, tarn, { system: "Seed" }).map((x) => x.label), ["Cosmic web", "Galaxies nearby", "Galaxy"]);
});

test("recent places: newest first, no repeats, at most six", () => {
  let r: Place[] = [];
  for (const p of [sys, tarn, galaxy, { ...sys, screen: "life" }]) r = rememberPlace(r, p);
  assert.deepEqual(r.map((p) => p.scale), ["system", "galaxy", "world"]);
  for (let i = 0; i < 9; i++) r = rememberPlace(r, { scale: "world", system: "", world: `w${i}` });
  assert.equal(r.length, 6);
  assert.deepEqual(readRecent(JSON.stringify([tarn, { scale: "nowhere", system: "" }, 7, { scale: "world", system: "" }])), [tarn]);
  assert.deepEqual(readRecent("not json"), []);
});

test("a place is a deep link and back", () => {
  assert.equal(placeHash(sys), "#chart");
  assert.equal(placeHash(galaxy), "#chart/galaxy");
  for (const p of [sys, tarn, galaxy, { scale: "web", system: "u:kestrel/sys:@seed", screen: "explore" } as Place, { ...tarn, screen: "cutaway" }]) assert.deepEqual(parsePlaceHash(placeHash(p)), p);
  assert.deepEqual(parsePlaceHash("#chart"), sys);
  assert.equal(parsePlaceHash("#elsewhere"), null);
  assert.equal(parsePlaceHash("#chart/nowhere"), null);
  assert.equal(parsePlaceHash("#chart/world"), null);
});

test("above a system, a place is one of that scale's own (a cell, a galaxy), named by its view", () => {
  const cell: Place = { scale: "cluster", system: "", id: "cell:0.0.0" };
  const halo: Place = { scale: "galaxy", system: "", id: "u:kestrel/cell:0.0.0/halo:3" };
  for (const p of [cell, halo, { ...halo, screen: "galaxy" }]) assert.deepEqual(parsePlaceHash(placeHash(p)), p);
  assert.equal(placeHash(cell), `#chart/cluster//${encodeURIComponent("cell:0.0.0")}`);
  const s = placeSteps(halo, null, { system: "Kestrel", above: { cluster: { id: "cell:0.0.0", name: "Cell 0 · 0 · 0" }, galaxy: { id: halo.id!, name: "Halo 3, cell 0.0.0" } } });
  assert.deepEqual(s.map((x) => x.label), ["Cosmic web", "Cell 0 · 0 · 0", "Halo 3, cell 0.0.0"]);
  assert.deepEqual(s[1]!.place, cell);
  assert.ok(!rememberPlace([cell], { ...cell, id: "cell:1.0.0" }).every((p) => p.id === "cell:0.0.0"));
  assert.equal(rememberPlace([cell], { ...cell, id: "cell:1.0.0" }).length, 2);
});

test("a landing site rides in the link after its screen (Voyages' Land here)", () => {
  const p: Place = { scale: "world", system: "u/sys:sol", world: "u/sys:sol/planet:3", screen: "surface", site: { lat_deg: 10.5, lon_deg: 75.97 } };
  const h = placeHash(p);
  assert.equal(h, "#chart/world/u%2Fsys%3Asol/u%2Fsys%3Asol%2Fplanet%3A3~surface?site=10.5,75.97");
  assert.deepEqual(parsePlaceHash(h), p);
  assert.deepEqual(parsePlaceHash("#chart/world/s/w~surface?site=-12.25,200")?.site, { lat_deg: -12.25, lon_deg: -160 });
  assert.equal(parsePlaceHash("#chart/world/s/w~surface?site=95,0")?.site, undefined, "no such latitude");
  assert.equal(parsePlaceHash("#chart/world/s/w~surface")?.screen, "surface");
  assert.equal(placeHash({ scale: "world", system: "s", world: "w", site: { lat_deg: 1, lon_deg: 2 } }), "#chart/world/s/w", "a site needs its screen");
});
