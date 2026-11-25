// V12 acceptance (H8.4A): symbols are separate from themes. The same system
// draws in both grammars and a mix under every shipped theme; a one-colour
// theme still tells every planet class apart; the legend matches the marks;
// symbols never touch the kernel.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { checkSymbology, legendFor, markerFor, sameLook, type Mark, type MarkSubject, type SymbologyDoc } from "../src/symbology.ts";
import { subjectOf } from "../src/mark.ts";
import { ALL_THEMES } from "../../skins/player/src/index.ts";

const doc = (n: string) => JSON.parse(readFileSync(new URL(`../../shell/src/documents/${n}`, import.meta.url), "utf8"));
const REG: SymbologyDoc[] = ["symbology.diagrammatic.json", "symbology.colour-coded.json", "symbology.mixed.json"].map(doc);
const CLASSES: { id: string; kind: string }[] = JSON.parse(readFileSync(new URL("../../../packs/sse3-classic/pack.json", import.meta.url), "utf8")).provides.other.classes;
const BASE = CLASSES.filter((c) => c.kind === "base").map((c) => c.id);
// SSE4's own atlas: its drawn schemes (what a world is, what its surface is like) each have a mark of their own.
const ATLAS: { id: string; kind: string; scheme: string }[] = JSON.parse(readFileSync(new URL("../../../packs/base-atlas/pack.json", import.meta.url), "utf8")).provides.other.classes;
const DRAWN = ATLAS.filter((c) => c.kind === "base" && (c.scheme === "physical" || c.scheme === "surface")).map((c) => c.id);
const scene = doc("kestrel.scene.json") as { subjects: { kind: string; tags: string[]; address: string; label: string }[] };
const planet = (c: string): MarkSubject => ({ kind: "planet", classes: [c], tags: [] });

function distinct(marks: Mark[]): boolean {
  return marks.every((a, i) => marks.every((b, j) => i === j || !sameLook(a, b)));
}

test("the shipped symbologies are well formed and bind only real classes", () => {
  for (const d of REG) assert.equal(checkSymbology(d, REG), null, d.id);
  for (const d of REG) for (const b of d.bindings) if (b.from === "class" && "table" in b.scale) for (const k of Object.keys(b.scale.table)) assert.ok(CLASSES.some((c) => c.id === k) || ATLAS.some((c) => c.id === k), `${d.id}: ${k} is not a C1 class`);
  // Every base class has a look in both base grammars.
  for (const id of ["sse.symbology.diagrammatic", "sse.symbology.colour-coded"]) {
    const m = markerFor(id, REG, { supports: { colours: "full", patterns: true, shapes: true } });
    const own = m.bindings.filter((b) => b.from === "class" && b.channel !== "ring" && "table" in b.scale);
    for (const c of BASE) assert.ok(own.some((b) => "table" in b.scale && c in b.scale.table), `${id} has no look for ${c}`);
  }
  // Colour-coded carries V3's own colours, no longer placeholders.
  assert.equal(REG[1]!.placeholder, undefined);
});

test("colour-coded uses V3's colour table and V3's first-match rule", () => {
  const v3 = JSON.parse(readFileSync(new URL("../../../docs/v3-colours.json", import.meta.url), "utf8")) as { bodies: { key: string; hex: string }[]; stars: { key: string; hex: string; token: string | null }[] };
  const hex = (k: string) => v3.bodies.find((b) => b.key === k)!.hex;
  const m = markerFor("sse.symbology.colour-coded", REG, { supports: { colours: "full", patterns: true, shapes: true } });
  const col = (c: string, tags: string[] = []) => m.mark({ kind: "planet", classes: [c], tags }).colour;
  assert.equal(col("planet/gas-giant"), hex("gasGiant"));
  assert.equal(col("planet/hot-jupiter"), hex("gasGiant"));
  assert.equal(col("planet/ice-giant"), hex("iceGiant"));
  assert.equal(col("planet/brown-dwarf"), hex("brownDwarf"));
  assert.equal(col("planet/forest"), hex("biosphere"));
  assert.equal(col("planet/desert"), hex("terrestrial"));
  // A habitability tag comes first, as in V3.
  assert.equal(col("planet/forest", ["habitability/earth-like"]), hex("habitable"));
  assert.equal(col("planet/gas-giant", ["habitability/human"]), hex("habitable"));
  // Every colour is one of V3's.
  const all = new Set(v3.bodies.map((b) => b.hex));
  for (const c of BASE) assert.ok(all.has(col(c)!), `${c} is not drawn in a V3 colour`);
  // Stars by spectral key, belts and constructs by kind.
  assert.equal(m.mark({ kind: "star", classes: [], tags: ["spectral:K"] }).colour, v3.stars.find((x) => x.key === "K")!.hex);
  assert.equal(m.mark({ kind: "belt", classes: [], tags: [] }).colour, hex("belt"));
  assert.equal(m.mark({ kind: "station", classes: [], tags: [] }).colour, hex("construct"));
});

test("both grammars and the mix draw the system under every shipped theme; diagrammatic and the mix tell every class apart", () => {
  for (const t of ALL_THEMES) {
    for (const d of REG) {
      const m = markerFor(d.id, REG, t);
      for (const s of scene.subjects) {
        const mk = m.mark(subjectOf(s));
        assert.ok(mk.shape && mk.fill_pattern && mk.size > 0, `${t.id}/${d.id}/${s.label}`);
        if (t.supports.colours !== "full") assert.equal(mk.colour, null, `${t.id} cannot carry colour`);
      }
      // Colour-coded is V3's colours: by colour it tells V3's groups apart, not every class
      // (unless the theme is one-colour, where class codes come in; tested below).
      if (d.id !== "sse.symbology.colour-coded") assert.ok(distinct(BASE.map((c) => m.mark(planet(c)))), `${d.id} under ${t.id} draws two classes alike`);
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

test("SSE4's own atlas classes have marks of their own, kept in step by scripts/add-atlas-marks.py", () => {
  // The script fails when a class of a drawn scheme has no mark, so a new class cannot draw as a plain disc.
  execFileSync("python3", ["scripts/add-atlas-marks.py", "--check"], { cwd: new URL("../../../", import.meta.url) });
  const full = { supports: { colours: "full" as const, patterns: true, shapes: true } };
  const diag = markerFor("sse.symbology.diagrammatic", REG, full);
  const coded = markerFor("sse.symbology.colour-coded", REG, full);
  assert.ok(DRAWN.length >= 30);
  for (const c of DRAWN) for (const b of diag.bindings.filter((x) => x.from === "class" && x.channel !== "ring" && "table" in x.scale)) assert.ok("table" in b.scale && c in b.scale.table, `diagrammatic ${b.channel} has no look for ${c}`);
  // An SSE4 class looks like its nearest SSE 3 class, by its own code.
  const rocky = diag.mark(planet("rocky"));
  const te = diag.mark(planet("planet/terrestrial"));
  assert.equal(rocky.shape, te.shape);
  assert.equal(rocky.fill_pattern, te.fill_pattern);
  assert.notEqual(rocky.glyph, te.glyph);
  assert.equal(coded.mark(planet("gas-giant/ammonia-clouds")).colour, coded.mark(planet("planet/ammonia-clouds-gas-giant")).colour);
  assert.equal(diag.mark(planet("small-body/icy")).shape, "square");
  assert.equal(diag.mark(planet("brown-dwarf")).shape, "diamond");
  assert.equal(diag.mark({ kind: "planet", classes: ["gas-giant", "physical/ringed"], tags: [] }).ring, true);
  // Every drawn SSE4 class is told apart in the diagrammatic grammar and the mix under every theme, and by
  // colour-coded under a one-colour theme (where class codes come in).
  for (const t of ALL_THEMES) {
    for (const id of ["sse.symbology.diagrammatic", "sse.symbology.mixed"]) assert.ok(distinct(DRAWN.map((c) => markerFor(id, REG, t).mark(planet(c)))), `${id} under ${t.id} draws two SSE4 classes alike`);
    if (t.supports.colours !== "full" && t.supports.colours <= 1) assert.ok(distinct(DRAWN.map((c) => markerFor("sse.symbology.colour-coded", REG, t).mark(planet(c)))), `colour-coded under ${t.id}`);
  }
  // Their plain names come from the atlas.
  const names = doc("symbology.names.json") as { names: Record<string, string> };
  assert.equal(names.names["surface/temperate-ocean"], "Temperate ocean");
  for (const c of DRAWN) assert.ok(names.names[c], `${c} has a name`);
});
