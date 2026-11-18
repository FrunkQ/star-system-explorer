// V12 acceptance (H8.4A): symbols are separate from themes. The same system
// draws in both grammars and a mix under every shipped theme; a one-colour
// theme still tells every planet class apart; the legend matches the marks;
// symbols never touch the kernel.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkSymbology, legendFor, markerFor, sameLook, type Mark, type MarkSubject, type SymbologyDoc } from "../src/symbology.ts";
import { subjectOf } from "../src/mark.ts";
import { ALL_THEMES } from "../../skins/player/src/index.ts";

const doc = (n: string) => JSON.parse(readFileSync(new URL(`../../shell/src/documents/${n}`, import.meta.url), "utf8"));
const REG: SymbologyDoc[] = ["symbology.diagrammatic.json", "symbology.colour-coded.json", "symbology.mixed.json"].map(doc);
const CLASSES: { id: string; kind: string }[] = JSON.parse(readFileSync(new URL("../../../packs/sse3-classic/pack.json", import.meta.url), "utf8")).provides.other.classes;
const BASE = CLASSES.filter((c) => c.kind === "base").map((c) => c.id);
const scene = doc("kestrel.scene.json") as { subjects: { kind: string; tags: string[]; address: string; label: string }[] };
const planet = (c: string): MarkSubject => ({ kind: "planet", classes: [c], tags: [] });

function distinct(marks: Mark[]): boolean {
  return marks.every((a, i) => marks.every((b, j) => i === j || !sameLook(a, b)));
}

test("the shipped symbologies are well formed and bind only real classes", () => {
  for (const d of REG) assert.equal(checkSymbology(d, REG), null, d.id);
  for (const d of REG) for (const b of d.bindings) if (b.from === "class" && "table" in b.scale) for (const k of Object.keys(b.scale.table)) assert.ok(CLASSES.some((c) => c.id === k), `${d.id}: ${k} is not a C1 class`);
  // Every base class has a look in both base grammars.
  for (const id of ["sse.symbology.diagrammatic", "sse.symbology.colour-coded"]) {
    const m = markerFor(id, REG, { supports: { colours: "full", patterns: true, shapes: true } });
    const own = m.bindings.filter((b) => b.from === "class" && b.channel !== "ring" && "table" in b.scale);
    for (const c of BASE) assert.ok(own.some((b) => "table" in b.scale && c in b.scale.table), `${id} has no look for ${c}`);
  }
  // Colour-coded's colours are flagged as placeholders until V3's table is read.
  assert.match(String(REG[1]!.placeholder), /placeholder/i);
});

test("both grammars and the mix draw the system under every shipped theme, telling every class apart", () => {
  for (const t of ALL_THEMES) {
    for (const d of REG) {
      const m = markerFor(d.id, REG, t);
      for (const s of scene.subjects) {
        const mk = m.mark(subjectOf(s));
        assert.ok(mk.shape && mk.fill_pattern && mk.size > 0, `${t.id}/${d.id}/${s.label}`);
        if (t.supports.colours !== "full") assert.equal(mk.colour, null, `${t.id} cannot carry colour`);
      }
      assert.ok(distinct(BASE.map((c) => m.mark(planet(c)))), `${d.id} under ${t.id} draws two classes alike`);
    }
  }
});

test("a one-colour theme still distinguishes every planet class, by a stated rule", () => {
  const one = ALL_THEMES.filter((t) => t.supports.colours !== "full" && t.supports.colours <= 1);
  assert.ok(one.length >= 1, "a one-colour theme ships");
  for (const t of one) {
    const coded = markerFor("sse.symbology.colour-coded", REG, t);
    assert.match(coded.collapse ?? "", /colour becomes a pattern/);
    const marks = BASE.map((c) => coded.mark(planet(c)));
    assert.ok(marks.every((x) => x.colour === null));
    assert.ok(new Set(marks.map((x) => x.fill_pattern)).size > 1, "colours became patterns");
    assert.ok(distinct(marks));
    // The diagrammatic grammar already speaks in one ink: nothing collapses but colour.
    const diag = markerFor("sse.symbology.diagrammatic", REG, t);
    assert.ok(distinct(BASE.map((c) => diag.mark(planet(c)))));
  }
  // The class codes are a one-colour stand-in for colour-coded only.
  const full = markerFor("sse.symbology.colour-coded", REG, { supports: { colours: "full", patterns: true, shapes: true } });
  assert.equal(full.mark(planet("planet/ocean")).glyph, null);
});

test("the legend is generated from the marks and matches them", () => {
  const subjects = scene.subjects.map(subjectOf);
  for (const t of ALL_THEMES) {
    for (const d of REG) {
      const m = markerFor(d.id, REG, t);
      const legend = legendFor(m, { present: subjects, order: Array.isArray(d.legend) ? d.legend : undefined });
      const classes = [...new Set(subjects.filter((s) => s.kind === "planet").flatMap((s) => s.classes))];
      assert.deepEqual([...legend.map((e) => e.key)].sort(), classes.sort(), `${d.id}: one entry per class on the chart`);
      for (const s of subjects.filter((x) => x.kind === "planet")) {
        const e = legend.find((x) => x.key === s.classes[0])!;
        const drawn = m.mark(s);
        // The body's mark is the legend's, plus what its modifiers add (a ring).
        assert.ok(sameLook(e.mark, { ...drawn, ring: e.mark.ring }), `${d.id} under ${t.id}: ${s.classes[0]}`);
      }
    }
  }
});

test("mixing is per channel and an edited copy overrides one class", () => {
  const mixed = markerFor("sse.symbology.mixed", REG, { supports: { colours: "full", patterns: true, shapes: true } });
  const diag = markerFor("sse.symbology.diagrammatic", REG, { supports: { colours: "full", patterns: true, shapes: true } });
  const coded = markerFor("sse.symbology.colour-coded", REG, { supports: { colours: "full", patterns: true, shapes: true } });
  for (const c of BASE) {
    const a = mixed.mark(planet(c));
    assert.equal(a.fill_pattern, diag.mark(planet(c)).fill_pattern);
    assert.equal(a.colour, coded.mark(planet(c)).colour);
  }
  const mine: SymbologyDoc = { schema: "sse.symbology/1", id: "campaign.symbology", name: "Mine", inherits: "sse.symbology.mixed", bindings: [{ applies_to: "planet", channel: "colour", from: "class", scale: { table: { "planet/ocean": "#123456" } }, fallback: null }] };
  assert.equal(checkSymbology(mine, REG), null);
  const m = markerFor(mine.id, [...REG, mine], { supports: { colours: "full", patterns: true, shapes: true } });
  assert.equal(m.mark(planet("planet/ocean")).colour, "#123456");
  assert.equal(m.mark(planet("planet/ocean")).fill_pattern, "band");
  assert.match(checkSymbology({ ...mine, inherits: "campaign.symbology" }, REG) ?? "", /loop/);
  assert.match(checkSymbology({ ...mine, bindings: [{ ...mine.bindings[0]!, scale: { table: { "planet/ocean": "blue" } } }] }, REG) ?? "", /#RRGGBB/);
});

test("palette colours take the theme's palette", () => {
  const d: SymbologyDoc = { schema: "sse.symbology/1", id: "p", name: "p", bindings: [{ applies_to: "planet", channel: "colour", from: "class", scale: { table: { "planet/ocean": "palette:1" } } }] };
  const withPalette = ALL_THEMES.find((t) => t.palette?.length)!;
  assert.equal(markerFor("p", [d], withPalette).mark(planet("planet/ocean")).colour, withPalette.palette![1]);
  assert.equal(markerFor("p", [d], { supports: { colours: "full", patterns: true, shapes: true } }).mark(planet("planet/ocean")).colour, null);
});
