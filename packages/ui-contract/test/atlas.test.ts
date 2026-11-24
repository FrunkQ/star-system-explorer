// AT1: the Atlas's page model. Scenes are K9's own verified outputs (tests/fixtures/observe), so what holds here holds
// for the kernel's scene op.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { PlayerScene, Vocab } from "../src/player-scene.ts";
import { checkBrand } from "../src/brand.ts";
import {
  buildPage,
  chapterOf,
  checkEditionShape,
  atlasContents,
  isCatalogued,
  resolveEdition,
  shownFields,
  templateFields,
  FIGURE_FIELDS,
  COUNT_FIELDS,
  DEFAULT_EDITION,
  SHIPPED_EDITIONS,
  checkEdition,
  testRender,
  typedFigures,
  pageText,
  atlasHtml,
  TEXT_BAR,
  asciiLogo,
  logoLetters,
  docNumber,
  withLetterhead,
  BUILTIN_LOGOS,
  SHIPPED_NOTE_SETS,
  GUIDE_NOTES,
  withMargin,
  fillNote,
  meets,
  checkNoteSet,
  withVoice,
  type AtlasPage,
  type EditionDoc,
} from "../src/atlas/index.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const before: PlayerScene = json("tests/fixtures/observe/tarn-before.json");
const after: PlayerScene = json("tests/fixtures/observe/tarn-after.json");
const vocab: Vocab = json("packages/shell/src/documents/vocab.plain.json");
const SYS = "u:kestrel/sys:0";
const TARN = "u:kestrel/sys:0/body:3";
const survey = resolveEdition(DEFAULT_EDITION);
const opts = { vocab };

const page = (scene: PlayerScene, addr: string, ed = survey): AtlasPage => buildPage(scene, addr, ed, opts)!;
const facts = (p: AtlasPage) => p.blocks.flatMap((b) => (b.type === "facts" ? b.rows : []));

test("contents: the top of the book, then each subject the audience knows, in address order", () => {
  const c = atlasContents(after);
  assert.deepEqual(
    c.map((l) => [l.number, l.title, l.kind, l.level]),
    [
      [1, "Kestrel", "system", 0],
      [2, "Tarn", "world", 1],
      [3, "Tarn a", "moon", 2],
      [4, "Vault", "world", 1],
    ],
  );
  assert.deepEqual(chapterOf(c, TARN).map((l) => l.title), ["Tarn", "Tarn a"]);
  // A world the audience has not found has no line and no number.
  const less: PlayerScene = { ...after, subjects: after.subjects.filter((s) => s.address !== TARN) };
  assert.deepEqual(atlasContents(less).map((l) => l.title), ["Kestrel", "Tarn a", "Vault"], "its moon sits under the star; nothing names the unknown world");
});

test("the same scene, edition and date give the same block list", () => {
  const a = JSON.stringify(page(after, TARN));
  const b = JSON.stringify(buildPage(JSON.parse(JSON.stringify(after)), TARN, resolveEdition(DEFAULT_EDITION), { vocab }));
  assert.equal(a, b);
  const p = page(after, TARN);
  assert.equal(p.prev, SYS);
  assert.equal(p.next, `${TARN}/body:0`);
  assert.equal(p.number, 2);
});

test("a world page in the Survey atlas: figures through the figures door, gaps in plain words", () => {
  const p = page(after, TARN);
  const t = p.blocks[0]!;
  assert.deepEqual(t, { type: "title", text: "Tarn", subtitle: "World · terrestrial · of Kestrel", kind: "world" });
  const rows = Object.fromEntries(facts(p).map((r) => [r.field, r]));
  assert.equal(rows["orbit.semi_major_axis_m"]!.text, "0.63 AU");
  assert.equal(rows["mass_kg"]!.text, "1.1 M⊕");
  assert.equal(rows["orbit.period_s"]!.text, "214 d");
  assert.equal(rows["atmosphere.pressure_pa"]!.text, "1.3 bar");
  assert.equal(rows["ocean.fraction"]!.text, "71%");
  assert.equal(rows["surface.temp_mean_k"]!.text, "270 to 290 K");
  assert.equal(rows["surface.temp_mean_k"]!.state, "band");
  assert.equal(rows["hazards"]!.text, "storm belt, tides");
  assert.deepEqual(rows["mass_kg"]!.figure, { si: 6.56942e24, kind: "mass", about: { class: "planet", what: "mass" } });
  for (const f of ["interior.core_fraction", "resources", "settlements.count"]) {
    assert.deepEqual({ ...rows[f]!, label: "" }, { field: f, label: "", state: "gap", text: "not yet surveyed" }, f);
  }
  const prose = p.blocks.find((b) => b.type === "prose")!;
  assert.ok(prose.type === "prose" && prose.text.includes("It orbits 0.63 AU from its star."), prose.type === "prose" ? prose.text : "");
  const moons = p.blocks.find((b) => b.type === "table")!;
  assert.ok(moons.type === "table");
  assert.deepEqual(moons.rows, [{ link: `${TARN}/body:0`, cells: [{ text: "Tarn a", state: "known" }, { text: "small", state: "known" }, { text: "0.994 M☾", state: "known" }] }]);
});

test("a seat's block list holds nothing its scene lacks", () => {
  for (const scene of [before, after]) {
    for (const l of atlasContents(scene)) {
      const p = page(scene, l.address);
      const s = scene.subjects.find((x) => x.address === l.address)!;
      for (const f of shownFields(p)) {
        const got = s.facts.find((x) => x.field === f);
        assert.ok(got && (got.value !== undefined || got.band), `${l.address} shows ${f}, which its scene does not give`);
      }
      const known = new Set(scene.subjects.map((x) => x.address));
      for (const b of p.blocks) {
        if (b.type === "table") for (const r of b.rows) assert.ok(known.has(r.link));
        if (b.type === "plate") for (const a of b.addresses) assert.ok(known.has(a));
      }
    }
  }
  assert.equal(buildPage(after, `${SYS}/body:9`, survey, opts), null, "no page for a subject the audience does not know");
});

test("a gap is drawn from the template, never from the value: a placeholder and nothing read the same", () => {
  // The kernel writes a placeholder only where a value exists; the Atlas must not let that show.
  const stripped: PlayerScene = { ...before, subjects: before.subjects.map((s) => ({ ...s, facts: s.facts.filter((f) => f.value !== undefined || f.band) })) };
  for (const ed of [survey, resolveEdition("x.bars", [{ schema: "sse.edition/1", id: "x.bars", name: "Bars", description: "bars", gaps: "bar" }])]) {
    for (const l of atlasContents(before)) assert.deepEqual(page(stripped, l.address, ed), page(before, l.address, ed), `${ed.id} ${l.address}`);
  }
  const bars = resolveEdition("x.bars", [{ schema: "sse.edition/1", id: "x.bars", name: "Bars", description: "bars", gaps: "bar" }]);
  const gaps = facts(page(before, TARN, bars)).filter((r) => r.state === "gap");
  assert.ok(gaps.length > 0 && gaps.every((r) => r.bar === true && r.text === ""), "bars carry no words and no length of their own");
  const omit = resolveEdition("x.omit", [{ schema: "sse.edition/1", id: "x.omit", name: "Omit", description: "omit", gaps: "omit" }]);
  assert.ok(facts(page(before, TARN, omit)).every((r) => r.state !== "gap"));
});

test("on the GM's own scene a line with nothing behind it does not apply, and is left out", () => {
  const gm: PlayerScene = { ...after, audience: "gm" };
  const rows = facts(page(gm, TARN));
  assert.ok(rows.every((r) => r.state !== "gap"));
  assert.ok(!rows.some((r) => r.field === "interior.core_fraction"));
});

test("a heard-of subject has a page that is mostly gaps, with what was heard", () => {
  const p = page(before, `${SYS}/body:5`);
  const story = p.blocks.find((b) => b.type === "story")!;
  assert.ok(story.type === "story");
  assert.deepEqual(story.lines, [{ label: "Rumour", text: "Miners say the third moon sings on the comms." }]);
  assert.ok(facts(p).every((r) => r.state === "gap"));
});

test("the system page: summary, plate, star and the worlds in orbit", () => {
  const p = page(after, SYS);
  assert.equal(p.kind, "system");
  assert.deepEqual(p.blocks[0], { type: "title", text: "Kestrel", subtitle: "System · K1V", kind: "system" });
  const plate = p.blocks.find((b) => b.type === "plate")!;
  assert.ok(plate.type === "plate");
  assert.deepEqual(plate.addresses, [TARN, `${SYS}/body:5`]);
  const worlds = p.blocks.find((b) => b.type === "table")!;
  assert.ok(worlds.type === "table");
  assert.deepEqual(worlds.columns, ["Name", "Size", "Orbit"]);
  assert.deepEqual(worlds.rows[0]!.cells.map((c) => c.text), ["Tarn", "terrestrial", "0.63 AU"]);
  assert.deepEqual(worlds.rows[1]!.cells.map((c) => c.text), ["Vault", "—", "—"]);
});

test("editions: extends, overrides and refusals", () => {
  const navy: EditionDoc = {
    schema: "sse.edition/1",
    id: "my.navy",
    name: "Fleet Survey Office",
    description: "Dry naval sheets.",
    gaps: "voice",
    voice: { labels: { "section.air": "Air and water", "line.mass_kg": "Displacement" }, gap: "NOT HELD AT THIS STATION" },
  };
  const ed = resolveEdition("my.navy", [navy]);
  assert.deepEqual(ed.chain, ["my.navy", DEFAULT_EDITION]);
  assert.equal(ed.look, "paper", "the look comes from the parent");
  const p = page(after, TARN, ed);
  const air = p.blocks.find((b) => b.type === "facts" && b.key === "section.air")!;
  assert.ok(air.type === "facts" && air.title === "Air and water");
  const rows = Object.fromEntries(facts(p).map((r) => [r.field, r]));
  assert.equal(rows["mass_kg"]!.label, "Displacement");
  assert.equal(rows["resources"]!.text, "NOT HELD AT THIS STATION");
  // An edition changes words, never a figure.
  const plain = Object.fromEntries(facts(page(after, TARN)).filter((r) => r.state !== "gap").map((r) => [r.field, r.text]));
  for (const [f, t] of Object.entries(plain)) assert.equal(rows[f]!.text, t, f);
  assert.throws(() => resolveEdition("nope"), /there is no edition nope/);
  assert.throws(() => resolveEdition("a", [{ ...navy, id: "a", extends: "b" }, { ...navy, id: "b", extends: "a" }]), /extends itself/);
  assert.throws(() => resolveEdition("a", [{ ...navy, id: "a", extends: "ghost" }]), /extends ghost, which is not here/);
  assert.match(checkEditionShape({ ...navy, gaps: "loud" })!, /gaps is one of/);
  assert.match(checkEditionShape({ ...navy, templates: { planet: [] } })!, /no page kind "planet"/);
});

test("every field a shipped template names is catalogued, and every number in one has a figure kind", () => {
  for (const f of templateFields(survey)) {
    assert.ok(isCatalogued(f), `${f} is not in the Atlas's fact catalogue`);
    const w = vocab.fields[f];
    if (w?.scale !== undefined || w?.unit !== undefined || w?.percent) assert.ok(f in FIGURE_FIELDS || COUNT_FIELDS.includes(f), `${f} is a number with no figure kind`);
  }
});

// ---------------------------------------------------------------------------------------------------------- AT3

const guideVocab: Vocab = json("packages/shell/src/documents/vocab.guide.json");
const VOCABS = { plain: vocab, guide: guideVocab };
const gmAfter: PlayerScene = { ...after, audience: "gm" };
const STACK = { looks: ["chart", "paper", "deck", "phosphor", "glass", "guide", "guide-gadget"], vocabs: ["plain", "guide"], noteSets: ["sse.notes.guide"] };

test("the shipped editions load, and their test render matches the Survey atlas", () => {
  for (const d of SHIPPED_EDITIONS) {
    assert.deepEqual(checkEdition(d, STACK), [], d.id);
    assert.deepEqual(testRender(d, [gmAfter, before, after], VOCABS), [], d.id);
  }
});

test("switching edition changes no figure, compared block by block", () => {
  for (const scene of [gmAfter, before, after]) {
    for (const l of atlasContents(scene)) {
      const pages = SHIPPED_EDITIONS.map((d) => {
        const ed = resolveEdition(d.id);
        return buildPage(scene, l.address, ed, { vocab: VOCABS[ed.vocab as keyof typeof VOCABS] })!;
      });
      const figs = pages.map((p) => facts(p).filter((r) => r.figure).map((r) => [r.field, r.text, JSON.stringify(r.figure)]).sort());
      for (const f of figs.slice(1)) {
        const common = f.filter((x) => figs[0]!.some((y) => y[0] === x[0]));
        assert.deepEqual(common, figs[0]!.filter((y) => f.some((x) => x[0] === y[0])), l.address);
      }
      // The same gaps on every edition: from the template and the audience, never from the edition's taste.
      const gapsOf = (p: AtlasPage) => facts(p).filter((r) => r.state === "gap").map((r) => r.field).sort();
      for (const p of pages) if (resolveEdition(p.edition).gaps !== "omit") assert.deepEqual(gapsOf(p), gapsOf(pages[0]!), `${p.edition} ${l.address}`);
    }
  }
  const guide = page(after, TARN, resolveEdition("sse.edition.guide"));
  const rows = Object.fromEntries(facts(guide).map((r) => [r.field, r]));
  assert.equal(rows["resources"]!.text, "nobody has looked yet");
  const company = page(before, TARN, resolveEdition("sse.edition.company"));
  assert.ok(facts(company).filter((r) => r.state === "gap").every((r) => r.bar));
});

test("the loader refuses with sentences that say what to change", () => {
  const base: EditionDoc = { schema: "sse.edition/1", id: "x.bad", name: "Bad", description: "bad" };
  const says = (d: Partial<EditionDoc>, re: RegExp) => {
    const got = checkEdition({ ...base, ...d }, STACK);
    assert.ok(got.some((s) => re.test(s)), `${JSON.stringify(d)} gave ${JSON.stringify(got)}`);
  };
  assert.deepEqual(checkEdition({ schema: "nope" }), [`an edition's schema is "sse.edition/1".`]);
  says({ extends: "ghost" }, /extends ghost, which is not here/);
  says({ templates: { world: [{ key: "section.x", lines: ["magic.level"] }] } }, /names "magic\.level", which is not a fact/);
  says({ voice: { labels: { "section.air": "Air (about 1.2 bar)" } } }, /states a figure \("1\.2 bar\)"\)|states a figure/);
  says({ voice: { gap: "71% unknown" } }, /states a figure/);
  says({ letterhead: { use: "brand", lines: ["{brand.colour}"] } }, /unknown slot \{brand\.colour\}/);
  says({ voice: { labels: { "section.air": "{audience}'s air" } } }, /slots go only in the letterhead/);
  says({ look: "neon" }, /look "neon" is not in the pack stack/);
  says({ vocab: "pirate" }, /vocabulary "pirate"/);
  says({ description: "x".repeat(300) }, /keep it to 200/);
  assert.deepEqual(typedFigures("1. Summary, 2. Primary, 42 things"), []);
  assert.deepEqual(typedFigures("about 5 kg and 3 days"), ["5 kg", "3 days"]);
});


test("Copy as text says what the page says: same figures, gaps as words or a fixed bar, Markdown when asked", () => {
  const p = page(after, TARN);
  const plain = pageText(p, { footer: "Survey atlas · Kestrel" });
  for (const r of facts(p).filter((r) => r.state !== "gap")) assert.ok(plain.includes(r.text), `${r.field} is in the copy`);
  assert.match(plain, /^Tarn\n====/);
  assert.ok(plain.trimEnd().endsWith("Survey atlas · Kestrel"));
  const md = pageText(p, { markdown: true });
  assert.match(md, /^# Tarn\n/);
  assert.match(md, /- \*\*.+:\*\* 0\.63 AU/);
  // Company bars what the table has not learnt: the bar is the same length whatever is behind it.
  const company = resolveEdition("sse.edition.company");
  const seat = page(before, TARN, company);
  const t = pageText(seat);
  if (facts(seat).some((r) => r.bar)) assert.ok(t.includes(TEXT_BAR));
  for (const m of t.matchAll(/█+/g)) assert.equal(m[0], TEXT_BAR);
  assert.equal(pageText(p), pageText(page(after, TARN)), "the same page copies the same");
});

test("Save page is one self-contained HTML file: escaped, no script, nothing from the web", () => {
  const p = page(after, TARN);
  const odd = { ...p, blocks: [...p.blocks, { type: "note" as const, text: "<script>alert(1)</script> & co" }] };
  const html = atlasHtml([page(after, SYS), odd], { title: "Survey atlas: <Kestrel>", tokens: "--sse-ink: #222;", footer: "Survey atlas" });
  assert.match(html, /^<!doctype html>/);
  assert.ok(!/<script/i.test(html), "no script, not even an escaped page note");
  assert.ok(html.includes("&lt;script&gt;alert(1)&lt;/script&gt; &amp; co"));
  assert.ok(html.includes("<title>Survey atlas: &lt;Kestrel&gt;</title>"));
  assert.ok(!/(src|href)=["']?https?:/i.test(html), "nothing loaded from the web");
  assert.equal(html.match(/<article>/g)!.length, 2, "one article per page");
  assert.ok(html.includes("0.63 AU"));
  assert.ok(html.includes("--sse-ink: #222;"));
});

test("the placeholder logo: the same name always draws the same art, letters made of themselves, three seeded flaws", () => {
  const a = asciiLogo("Halden-Voss Deep Survey", "1098");
  assert.deepEqual(asciiLogo("Halden-Voss Deep Survey", "1098"), a, "the same every time");
  assert.equal(logoLetters("Halden-Voss Deep Survey"), "HVDS");
  assert.equal(logoLetters("Bob"), "BOB");
  assert.equal(logoLetters("Interstellar"), "INTERSTE", "one word, spelt out to eight letters");
  assert.equal(logoLetters("A B C D E F"), "ABCD", "at most four initials");
  const body = a.lines.slice(1, 6).join("\n");
  for (const ch of body.replace(/[\s|=>()*~o-]/g, "")) assert.ok("HVDS".includes(ch), `only the letters themselves (${ch})`);
  assert.notEqual(a.lines[0]!.length, a.lines[6]!.length, "the bottom edge is one dash out");
  assert.match(a.lines[7]!, /HALDEN-VOSS DEEP SURVEY \*est 1098\*/);
  assert.match(a.lines[8]!, /^\s+\(.+\)$/, "an apology");
  assert.match(a.label, /placeholder logo drawn in text/);
  // Different names may pick different flaws; no clock is involved, so a second call never differs.
  assert.notDeepEqual(asciiLogo("Kelido Mining").lines, asciiLogo("Nexum Freight").lines);
});

test("letterhead: the brand's paper on the Company edition, a logo replaces the drawn one, Survey has none", () => {
  const company = resolveEdition("sse.edition.company");
  const p = page(after, TARN, company);
  const ctx = { clearance: "GM EYES ONLY", date: "Year 1, day 1", place: "Kestrel", est: "1" };
  const drawn = withLetterhead(p, company, { ...ctx, brand: { name: "Halden-Voss Deep Survey", accent: "#aa3322", motto: "We go further" } });
  const lh = drawn.blocks[0]!;
  assert.equal(lh.type, "letterhead");
  if (lh.type !== "letterhead") return;
  assert.ok(lh.art && !lh.logo);
  assert.deepEqual(lh.lines, ["Halden-Voss Deep Survey", "SURVEY REPORT HVD-KES-02/TARN", "ISSUED Year 1, day 1", "CLEARANCE: GM EYES ONLY"]);
  assert.equal(lh.stamp, "COMPANY CONFIDENTIAL");
  assert.equal(lh.accent, "#aa3322");
  assert.equal(lh.motto, "We go further");
  assert.deepEqual(drawn.blocks.slice(1), p.blocks, "the letterhead adds paper, never a fact");
  const logo = BUILTIN_LOGOS.find((l) => l.id === "interspan")!;
  const withLogo = withLetterhead(p, company, { ...ctx, brand: { name: logo.name, logo: logo.logo } }).blocks[0]!;
  assert.ok(withLogo.type === "letterhead" && withLogo.logo === logo.logo && !withLogo.art);
  const noBrand = withLetterhead(p, company, ctx).blocks[0]!;
  assert.ok(noBrand.type === "letterhead" && noBrand.name === "Independent Survey Office", "no brand: the edition's fallback name");
  assert.equal(withLetterhead(page(after, TARN), survey, ctx).blocks[0]!.type, "title", "the Survey atlas has no letterhead");
  assert.ok(!withLetterhead(p, company, { ...ctx, brand: { name: "X", accent: "red; background: url(x)" } }).blocks.some((b) => b.type === "letterhead" && b.accent), "an accent is a colour or nothing");
  assert.equal(docNumber("Kelido", "Kestrel", 3, "Tarn a"), "KEL-KES-03/TARNA");
  assert.match(pageText(drawn), /COMPANY CONFIDENTIAL/);
});

test("the built-in logos: SSE3's starters, credited, each a brand logo the brand check accepts, no Weyland-Yutani", () => {
  assert.deepEqual(BUILTIN_LOGOS.map((l) => l.id), ["sse", "interspan", "kelido", "nexum", "terra", "tsec"]);
  for (const l of BUILTIN_LOGOS) {
    assert.deepEqual(checkBrand({ name: l.name, logo: l.logo }), [], l.id);
    assert.ok(l.credit.length > 10, `${l.id} is credited`);
  }
  assert.ok(!BUILTIN_LOGOS.some((l) => /weyland|yutani/i.test(l.id + l.name)));
});

test("margin notes: fit by what the audience knows, slots said by the figures door, the same pick every time", () => {
  const guide = resolveEdition("sse.edition.guide");
  const p = page(after, TARN, guide);
  const a = withMargin(p, after, guide, SHIPPED_NOTE_SETS, opts);
  const m = a.blocks.find((b) => b.type === "margin");
  assert.ok(m && m.type === "margin");
  if (!m || m.type !== "margin") return;
  assert.ok(m.notes.length >= 1 && m.notes.length <= 2, "at most two to a page");
  assert.equal(m.signature, "The Guide says");
  assert.deepEqual(withMargin(p, JSON.parse(JSON.stringify(after)), guide, SHIPPED_NOTE_SETS, opts), a, "the same page gives the same notes");
  assert.equal(a.blocks[a.blocks.length - 1]!.type, "note", "the line saying notes are opinions");
  assert.deepEqual(a.blocks.slice(0, p.blocks.length), p.blocks, "notes add to the page, never change a fact");
  // A slotted note says the audience's own figure through the figures door.
  const sea = GUIDE_NOTES.notes.find((n) => n.id === "guide.new.sea-market")!;
  const tarn = after.subjects.find((s) => s.address === TARN)!;
  assert.equal(fillNote(sea, tarn, opts), `With 71% of it under water, the local property market is best described as "liquid".`);
  // A note never fires on a fact the seat lacks: with ocean cover unknown, no note about the sea fits.
  const blind: PlayerScene = { ...after, subjects: after.subjects.map((s) => (s.address === TARN ? { ...s, facts: s.facts.filter((f) => f.field !== "ocean.fraction") } : s)) };
  const fits = (sc: PlayerScene) => {
    const all: string[] = [];
    const one = withMargin(page(sc, TARN, guide), sc, { ...guide, margin: { ...guide.margin!, max: 999 } }, SHIPPED_NOTE_SETS, opts);
    for (const b of one.blocks) if (b.type === "margin") all.push(...b.notes.map((n) => n.id));
    return all;
  };
  assert.ok(fits(after).includes("guide.new.sea-market"));
  for (const id of fits(blind)) {
    const n = GUIDE_NOTES.notes.find((x) => x.id === id)!;
    assert.ok(!JSON.stringify(n.when ?? {}).includes("ocean.fraction"), `${id} does not read the unknown sea`);
  }
  // A band meets a bound only when all of it does.
  assert.equal(meets({ address: "x", view: "gm", tags: [], facts: [{ field: "surface.temp_mean_k", band: [220, 240] }] }, { fact: "surface.temp_mean_k", at_most: 230 }), false);
  assert.equal(meets({ address: "x", view: "gm", tags: [], facts: [{ field: "surface.temp_mean_k", band: [200, 225] }] }, { fact: "surface.temp_mean_k", at_most: 230 }), true);
  // "Not this one" turns a note away and another may take its place, still deterministically.
  const first = m.notes[0]!.id;
  const again = withMargin(p, after, guide, SHIPPED_NOTE_SETS, { ...opts, exclude: [first] });
  assert.ok(!JSON.stringify(again.blocks).includes(`"${first}"`));
  // The Survey atlas has no margin.
  assert.deepEqual(withMargin(page(after, TARN), after, survey, SHIPPED_NOTE_SETS, opts), page(after, TARN));
});

test("the note sets load: Alex's list (no Douglas Adams names in the base set), the fan pack apart, the loader's refusals", () => {
  assert.deepEqual(checkNoteSet(GUIDE_NOTES), []);
  const fan = json("store-packs/guide-fan-notes/notes.json");
  assert.deepEqual(checkNoteSet(fan), []);
  const borrowed = /Zaphod|Vogon|Beeblebrox|Sirius Cybernetics|Improbability|Gargle Blaster|Ford Prefect|Megadodo|Eccentrica|petunias|\b42\b|forgot your towel|Bugblatter|Nutri-Matic|Genuine People|bypass/i;
  for (const n of GUIDE_NOTES.notes) assert.ok(!borrowed.test(n.text), `${n.id} is in the base set`);
  const alex = GUIDE_NOTES.notes.filter((n) => n.id.startsWith("guide.alex.")).length + fan.notes.length;
  assert.equal(alex, 131 - 5, "every line of Alex's list is in one set or held for him (five type a figure)");
  const ed = json("store-packs/guide-fan-notes/edition.json");
  assert.deepEqual(checkEdition(ed, { ...STACK, noteSets: ["sse.notes.guide", "fan.notes.guide-classic"] }), []);
  assert.equal(resolveEdition(ed.id, [ed]).cover?.publisher, "Megadodo Publications");
  assert.equal(resolveEdition("sse.edition.guide").cover?.publisher, undefined, "the base Guide's cover names no publisher");
  assert.equal(resolveEdition("sse.edition.guide").cover?.big, "DON'T PANIC");
  const bad = { schema: "sse.margin-notes/1", id: "x", name: "X", notes: [
    { id: "a", about: "world", text: "The sea covers 71% of it." },
    { id: "a", about: "planet", text: "At {gravity.surface} you bounce." },
    { id: "b", about: "world", text: "With {ocean.fraction} wet, bring a towel." },
  ] };
  const said = checkNoteSet(bad).join("\n");
  assert.match(said, /states a figure \("71%"\)/);
  assert.match(said, /appears twice/);
  assert.match(said, /about "planet"/);
  assert.match(said, /\{gravity\.surface\}, which is not a fact/);
  assert.match(said, /slot \{ocean\.fraction\} that its "when" does not test/);
});

test("the Guide's verdict and subtitle phrase are read off what the audience knows, with their Why", () => {
  const guide = resolveEdition("sse.edition.guide");
  const p = withVoice(page(after, TARN, guide), after, guide, opts);
  const t = p.blocks[0]!;
  assert.ok(t.type === "title");
  if (t.type !== "title") return;
  assert.match(t.subtitle, / · the damp one$/, "71% sea");
  assert.ok(t.verdict, "a verdict");
  assert.equal(t.verdict!.label, "Verdict");
  assert.ok(t.verdict!.why.length >= 1 && t.verdict!.why.every((w) => /: \S/.test(w)), `each Why names a fact and its figure (${t.verdict!.why})`);
  // Without the facts, nothing is said.
  const blind: PlayerScene = { ...after, subjects: after.subjects.map((s) => (s.address === TARN ? { ...s, facts: s.facts.filter((f) => !/pressure|temp|ocean|composition/.test(f.field)) } : s)) };
  const q = withVoice(page(blind, TARN, guide), blind, guide, opts).blocks[0]!;
  assert.ok(q.type === "title" && !q.verdict && !q.subtitle.includes("damp"));
  // The Survey atlas has no voice tables; switching edition still changes no figure (the test render passes).
  assert.deepEqual(withVoice(page(after, TARN), after, survey, opts), page(after, TARN));
  assert.match(pageText(p), /\[ Verdict: /);
});
