// AT1: the Atlas's page model. Scenes are K9's own verified outputs (tests/fixtures/observe), so what holds here holds
// for the kernel's scene op.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { PlayerScene, Vocab } from "../src/player-scene.ts";
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
