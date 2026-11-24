// The units engine (ED1a): one way to say every figure and read it back, from figures-said.json over the master
// data sheet's Units table. Examples are the editing design's own (section 3C).
import { test } from "node:test";
import assert from "node:assert/strict";
import table from "../src/figures-said.json" with { type: "json" };
import { constant } from "../src/sheet.ts";
import {
  FIGURE_KINDS,
  SCALE_CLASSES,
  UNIT_FAMILIES,
  chosenReading,
  nextReading,
  readFigure,
  readingsFor,
  say,
  sayText,
  significantFigures,
  waitingUnits,
  withChoice,
  type About,
} from "../src/figures.ts";

const AU = constant("au");
const YR = constant("yr_julian");
const planetOrbit: About = { class: "planet", what: "orbit" };
const moonOrbit: About = { class: "moon", what: "orbit" };

// Units the table names that the sheet's Units table does not hold yet. The data sheet thread is adding them; when a
// row lands, take it off this list. A new unit the table names must be on the sheet or listed here, so a typo fails.
// Units the table names that the sheet does not hold yet. Empty since the data sheet's MP31 (PR 419) added them.
const WAITING: string[] = [];

test("every unit the table names is on the sheet, or is a known wait", () => {
  for (const u of waitingUnits()) assert.ok(WAITING.includes(u), `"${u}" is neither a sheet unit nor a listed wait`);
});

test("the table is well formed: families, classes and keys", () => {
  const fams = new Set(UNIT_FAMILIES.map((f) => f.id));
  const classes = new Set(SCALE_CLASSES.map((c) => c.id));
  for (const [kind, k] of Object.entries(table.kinds)) {
    for (const [key, rs] of Object.entries(k.say as Record<string, { id: string; family: string }[]>)) {
      const cls = key.split(".")[0]!;
      assert.ok(key === "default" || cls === "*" || classes.has(cls), `${kind} ${key}: no scale class "${cls}"`);
      assert.equal(new Set(rs.map((r) => r.id)).size, rs.length, `${kind} ${key}: reading ids repeat`);
      for (const r of rs) assert.ok(fams.has(r.family), `${kind} ${key} ${r.id}: no family "${r.family}"`);
      // Imperial is there for those who want it, never a default (Alex, 24 Nov 14:56).
      assert.notEqual(rs[0]!.family, "imperial", `${kind} ${key}: imperial first`);
    }
  }
});

test("figures read comfortably and climb their rungs as they grow", () => {
  assert.equal(sayText(384_400e3, "length", moonOrbit), "384 thousand km");
  assert.equal(sayText(1.2e9, "length", moonOrbit), "1.2 million km");
  assert.equal(sayText(0.52 * AU, "length", planetOrbit), "0.52 AU");
  assert.equal(sayText(1.2e9, "length", planetOrbit), "1.2 million km");
  assert.equal(sayText(AU, "length", planetOrbit), "1 AU");
  // A world's size in km, metres only for what you could stand beside; never megametres.
  assert.equal(sayText(6_371e3, "length", { class: "planet", what: "size" }), "6,370 km");
  assert.equal(sayText(250, "length", { class: "site", what: "size" }), "250 m");
  assert.equal(sayText(5.2e3, "length", { class: "site", what: "size" }), "5.2 km");
  // Masses by scale class: Earths for a planet, Jupiters for a giant, Suns for a star.
  assert.equal(sayText(0.82 * constant("M_earth"), "mass", { class: "planet", what: "mass" }), "0.82 M⊕");
  assert.equal(sayText(constant("M_jup"), "mass", { class: "giant", what: "mass" }), "1 MJ");
  assert.equal(sayText(constant("M_sun"), "mass", { class: "star", what: "mass" }), "1 M☉");
  // Air in bar and millibar; Pluto's in microbar; inside a world in GPa.
  assert.equal(sayText(101_325, "pressure"), "1.01 bar");
  assert.equal(sayText(600, "pressure"), "6 mbar");
  assert.equal(sayText(1.15, "pressure"), "11.5 µbar");
  assert.equal(sayText(3.6e11, "pressure", { what: "inside" }), "360 GPa");
  // Kelvin, and °C for places people stand.
  assert.equal(sayText(288, "temperature"), "288 K");
  assert.equal(sayText(288.15, "temperature", { class: "site" }), "15 °C");
  // Time spans climb hours, days, years, Myr, Gyr.
  assert.equal(sayText(31 * 3600, "time"), "31 h");
  assert.equal(sayText(687 * 86_400, "time"), "687 d");
  assert.equal(sayText(4.57e9 * YR, "time"), "4.57 Gyr");
  assert.equal(sayText(45, "time"), "45 s");
  // Light of a star in Suns; watts take prefixes, then an exponent on the plain unit.
  assert.equal(sayText(constant("L_sun"), "power", { class: "star", what: "light" }), "1 L☉");
  assert.equal(sayText(constant("L_sun"), "power"), "3.83 × 10²⁶ W");
  assert.equal(sayText(2.5e6, "power"), "2.5 MW");
  // Small bodies: kg with an exponent.
  assert.equal(sayText(9.38e20, "mass", { class: "small", what: "mass" }), "9.38 × 10²⁰ kg");
  // Angles in degrees, or degrees minutes seconds.
  assert.equal(sayText((23.44 * Math.PI) / 180, "angle"), "23.4°");
  assert.equal(sayText((23.4392811 * Math.PI) / 180, "angle", undefined, { reading: "dms" }), "23° 26′ 21.4″");
});

test("depth: words at Curious, exponents at Engine, 'about' on computed figures", () => {
  assert.equal(sayText(384_400e3, "length", moonOrbit, { depth: "curious" }), "about 384 thousand km");
  assert.equal(sayText(384_400e3, "length", moonOrbit, { depth: "engine" }), "3.84 × 10⁵ km");
  assert.equal(sayText(0.82 * constant("M_earth"), "mass", { class: "planet", what: "mass" }, { depth: "curious" }), "about 0.82 Earth masses");
  assert.equal(sayText(constant("M_earth"), "mass", { class: "planet", what: "mass" }, { depth: "curious", source: "catalogued", sig: 1 }), "1 Earth mass");
  assert.equal(sayText(4.57e9 * YR, "time", undefined, { depth: "curious" }), "about 4.57 billion years");
});

test("honest precision: typed figures keep every digit, nothing gains digits by conversion", () => {
  assert.equal(significantFigures("0.01671022"), 7);
  assert.equal(significantFigures("149.6"), 4);
  assert.equal(significantFigures("1,200"), 2);
  assert.equal(significantFigures("1.000"), 4);
  assert.equal(sayText(0.01671022, "share", undefined, { reading: "fraction", source: "typed", sig: 7 }), "0.01671022");
  assert.equal(sayText(AU, "length", planetOrbit, { source: "typed", sig: 4 }), "1.000 AU");
  const r = readFigure("149.6 million km", "length");
  assert.ok(r.ok);
  if (r.ok) assert.equal(sayText(r.si, "length", planetOrbit, { source: "typed", sig: r.sig }), "1.000 AU");
});

test("boundaries are sticky, and a drag holds its rung", () => {
  const near = 0.0099 * AU; // just under where AU takes over for a planet's orbit
  const a = say(near, "length", planetOrbit);
  assert.equal(a.unit, "million km");
  const up = say(0.0104 * AU, "length", planetOrbit, { prev: a.rung });
  assert.equal(up.unit, "million km", "a small step past the line does not flip the unit");
  const far = say(0.0115 * AU, "length", planetOrbit, { prev: a.rung });
  assert.equal(far.unit, "AU");
  const back = say(0.0095 * AU, "length", planetOrbit, { prev: far.rung });
  assert.equal(back.unit, "AU", "and not on the way back either");
  const held = say(3 * AU, "length", planetOrbit, { hold: a.rung });
  assert.equal(held.unit, "million km", "held for the length of a drag");
});

test("click steps the reading, and the choice is remembered per kind and class", () => {
  const giant: About = { class: "giant", what: "mass" };
  const ids = readingsFor("mass", giant).map((r) => r.id);
  assert.deepEqual(ids.slice(0, 2), ["jupiter", "earth"]);
  assert.equal(nextReading("mass", giant, "jupiter"), "earth");
  assert.equal(nextReading("mass", giant, ids[ids.length - 1]!), "jupiter", "round again");
  assert.equal(sayText(constant("M_jup"), "mass", giant, { reading: "earth" }), "318 M⊕");
  let c = withChoice(undefined, "length", planetOrbit, "si");
  c = withChoice(c, "length", moonOrbit, "scientific");
  assert.equal(chosenReading(c, "length", planetOrbit), "si");
  assert.equal(chosenReading(c, "length", { class: "planet", what: "size" }), "scientific", "size is its own choice");
  assert.equal(chosenReading(c, "length", moonOrbit), "scientific");
  assert.deepEqual(withChoice(c, "length", planetOrbit, null), {}, "Use the scheme's");
});

test("typing reads any unit of the kind, with words, prefixes and exponents", () => {
  const near = (got: ReturnType<typeof readFigure>, si: number) => {
    assert.ok(got.ok, JSON.stringify(got));
    if (got.ok) assert.ok(Math.abs(got.si - si) <= Math.abs(si) * 1e-12, `${got.si} against ${si}`);
  };
  near(readFigure("149.6 million km", "length"), 149.6e9);
  near(readFigure("1.2 Gm", "length"), 1.2e9);
  near(readFigure("0.005 AU", "length"), 0.005 * AU);
  near(readFigure("1.5", "length", { unit: "au" }), 1.5 * AU);
  near(readFigure("3.2 × 10¹⁹ kg", "mass"), 3.2e19);
  near(readFigure("3.2e19 kg", "mass"), 3.2e19);
  near(readFigure("3.2 x 10^19 kg", "mass"), 3.2e19);
  near(readFigure("500 g", "mass"), 0.5);
  near(readFigure("2 Earth masses", "mass"), 2 * constant("M_earth"));
  near(readFigure("20 °C", "temperature"), 293.15);
  near(readFigure("23.934 h", "time"), 23.934 * 3600);
  near(readFigure("365.256 d", "time"), 365.256 * 86_400);
  near(readFigure("1 day 2 h", "time"), 93_600);
  near(readFigure("4.57 billion years", "time"), 4.57e9 * YR);
  near(readFigure("11.5 µbar", "pressure"), 1.15);
  near(readFigure("2 GPa", "pressure"), 2e9);
  near(readFigure("about 384 thousand km", "length"), 384e6);
  near(readFigure("−12 km", "length"), -12e3);
  near(readFigure("23.4°", "angle"), (23.4 * Math.PI) / 180);
  assert.equal(readFigure("12 parsnips", "length").ok, false);
  assert.equal(readFigure("12", "length").ok, false, "a bare number needs the unit on screen");
});

test("a figure of another kind is told apart, so a period can become a distance", () => {
  const r = readFigure("300 d", "length");
  assert.equal(r.ok, false);
  if (!r.ok) assert.deepEqual(r.as, { kind: "time", si: 300 * 86_400 });
});

test("the Moon's tilt as published, '23d 26m'", () => {
  const r = readFigure("23d 26m", "angle");
  assert.ok(r.ok);
  if (r.ok) assert.ok(Math.abs(r.si - ((23 + 26 / 60) * Math.PI) / 180) < 1e-12);
});

test("everything said reads back to the value, within what was shown", () => {
  const cases: [string, About | undefined, number[]][] = [
    ["length", planetOrbit, [3e7, 1e9, 0.39 * AU, 30 * AU, 5e4 * AU]],
    ["length", moonOrbit, [9.4e6, 384.4e6, 2.3e10]],
    ["length", { class: "planet", what: "size" }, [2.4e6, 6.371e6, 7e7]],
    ["length", { class: "neighbourhood" }, [4.2 * 9.4607e15, 3e18]],
    ["mass", { class: "planet", what: "mass" }, [3.3e23, 5.97e24, 1e26]],
    ["mass", { class: "star", what: "mass" }, [2e29, 2e30, 3e31]],
    ["pressure", undefined, [1, 600, 101_325, 9.2e6]],
    ["temperature", undefined, [40, 288, 5772]],
    ["temperature", { class: "site" }, [250, 300]],
    ["time", undefined, [30, 7200, 2e5, 6e7, 3e14, 1.4e17]],
    ["power", undefined, [3, 2e6, 4e26]],
    ["density", undefined, [5514, 687]],
    ["gravity", undefined, [3.7, 9.81, 24.8]],
    ["speed", undefined, [11, 29_780]],
    ["field", undefined, [3e-5, 4e-4]],
    ["angle", undefined, [0.1, 3.0]],
  ];
  for (const [kind, about, values] of cases) {
    for (const reading of readingsFor(kind, about)) {
      for (const depth of ["curious", "science", "engine"] as const) {
        for (const v of values) {
          const s = say(v, kind, about, { reading: reading.id, depth });
          const back = readFigure(s.text, kind);
          assert.ok(back.ok, `${kind} ${reading.id} ${depth}: "${s.text}" did not read back: ${JSON.stringify(back)}`);
          if (back.ok) assert.ok(Math.abs(back.si - v) <= Math.abs(v) * 0.006, `${kind} ${reading.id} ${depth}: "${s.text}" read back as ${back.si}, not ${v}`);
        }
      }
    }
  }
});

test("every kind has a reading on the sheet today", () => {
  for (const kind of FIGURE_KINDS) assert.ok(readingsFor(kind).length > 0, kind);
});

// ED1a part 2: a card's facts through the engine.
import { factUnit, readFact, sayFact } from "../src/figures.ts";
import { subjectClass } from "../src/handles.ts";

test("a scene's unit symbols are the sheet's units, a unit's own symbol first", () => {
  assert.equal(factUnit("AU")?.unit, "au");
  assert.equal(factUnit("M⊕")?.kind, "mass");
  assert.equal(factUnit("g")?.unit, "gee", "g on a card is the gee, not a gram");
  assert.equal(factUnit("d")?.unit, "day", "d is a day, not a degree");
  assert.equal(factUnit("μT")?.unit, "ut", "Greek mu and the micro sign are one");
  assert.equal(factUnit("dex"), null);
  assert.equal(factUnit(undefined), null);
});

test("a card's facts are said like the same value anywhere else", () => {
  const f = (field: string, value: number, unit: string, source = "derived") => ({ field, value, unit, source });
  assert.equal(sayFact(f("a_au", 1.524, "AU", "catalogued"), "planet", { depth: "science" })?.text, "1.524 AU");
  assert.equal(sayFact(f("a_au", 1.524, "AU", "catalogued"), "planet", { depth: "curious" })?.text, "about 1.52 AU");
  assert.equal(sayFact(f("a_au", 0.00257, "AU"), "moon", { depth: "science" })?.text, "384 thousand km", "a moon's orbit in thousand km");
  assert.equal(sayFact(f("mass_earth", 0.10700000000000001, "M⊕", "catalogued"), "planet", { depth: "science" })?.text, "0.107 M⊕", "float noise gains no figures");
  assert.equal(sayFact(f("surface_k", 288.15, "K"), "planet", { depth: "science" })?.text, "15 °C", "a world's surface reads in °C");
  assert.equal(sayFact(f("teff_k", 5772, "K"), "star", { depth: "science" })?.text, "5,770 K");
  assert.equal(sayFact(f("luminosity_sun", 1, "L☉"), "star", { depth: "curious" })?.text, "about 1 Sun");
  assert.equal(sayFact({ field: "metallicity", value: -0.08, unit: "dex" }, "star"), null, "a unit the sheet lacks keeps the scene's text");
  assert.equal(sayFact({ field: "name", value: "Tarn" }, "planet"), null);
});

test("typing over a card's fact takes any unit of its kind, and a bare number in the unit on screen", () => {
  const orbit = { field: "a_au", value: 1, unit: "AU" };
  const r = readFact(orbit, "412 thousand km");
  assert.ok(r?.ok && Math.abs(r.value * constant("au") - 4.12e8) < 1);
  const c = readFact({ field: "surface_k", value: 288, unit: "K" }, "15", "degc");
  assert.ok(c?.ok && Math.abs(c.value - 288.15) < 1e-9);
  const bad = readFact(orbit, "3 kg");
  assert.ok(bad && !bad.ok);
});

test("a subject's scale class", () => {
  assert.equal(subjectClass({ kind: "planet", facts: [{ field: "mass_earth", value: 317.8 }] }), "giant");
  assert.equal(subjectClass({ kind: "planet", facts: [{ field: "mass_earth", value: 0.8 }] }), "planet");
  assert.equal(subjectClass({ kind: "moon" }), "moon");
  assert.equal(subjectClass({ kind: "belt" }), "ring");
});
