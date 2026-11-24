// The Playbook (PB3): chapters, pages, sections and presets; fire, and press again to go back.
import { test } from "node:test";
import assert from "node:assert/strict";
import { addChapter, addPage, addPreset, addSection, emptyPlaybook, findPage, findPreset, fire, isUp, movePreset, movePresetToPage, playbookFrom, presetWords, presetsInView, removePart, removePreset, rename, updatePreset, type Preset } from "../src/playbook.ts";
import { emptyBoard, resolve, send, type Board } from "../src/board.ts";
import { makeBookmark } from "../src/bookmark.ts";

const bm = [makeBookmark([], { scope: "u:k/sys:tarn", now_yr: 1, instrument: "globe", selected: "u:k/sys:tarn/iv" }, "Tarn IV", "Globe")];
const title = (b: Board, role: string) => {
  const r = resolve(b, role);
  return r.held ? "held" : "off" in r.shown ? "off" : r.shown.title;
};

test("a new playbook has one chapter, page and section; parts are added with fresh ids", () => {
  let pb = emptyPlaybook();
  assert.equal(findPage(pb, null)?.page.id, "p1");
  const c = addChapter(pb, "Arrival at Tarn");
  pb = c.pb;
  const p = addPage(pb, "c2", "The beacon");
  pb = p.pb;
  const s = addSection(pb, p.page, "Approach");
  pb = s.pb;
  assert.deepEqual([c.page, p.page, s.section], ["p2", "p3", "s4"]);
  pb = rename(pb, "s4", "  Going down ");
  assert.equal(findPage(pb, "p3")?.page.sections[1]?.title, "Going down");
  assert.equal(rename(pb, "s4", " ").chapters[1]!.pages[1]!.sections[1]!.title, "Going down");
  pb = removePart(pb, "c2");
  assert.equal(pb.chapters.length, 1);
  assert.equal(removePart(emptyPlaybook(), "c1").chapters.length, 1, "never empty");
});

test("presets: add, change, move between sections and pages, remove; keys count only unfolded sections", () => {
  let pb = addSection(emptyPlaybook(), "p1", "Approach").pb;
  const a = addPreset(pb, "s1", { title: "Tarn IV close up", to: "telescope", steps: [{ show: { bookmark: "b1" } }] });
  pb = a.pb;
  const b = addPreset(pb, "s2", { title: "Screens dark", to: "everyone", steps: [{ hold: { card: "blank" } }] });
  pb = b.pb;
  assert.deepEqual([a.id, b.id], ["k1", "k2"]);
  pb = updatePreset(pb, "k1", { to: "bridge", note: "wait for them to ask" });
  assert.equal(findPreset(pb, "k1")?.preset.to, "bridge");
  pb = movePreset(pb, "k1", "s2", 0);
  assert.deepEqual(findPage(pb, "p1")!.page.sections.map((s) => s.presets.map((k) => k.id)), [[], ["k1", "k2"]]);
  assert.deepEqual(presetsInView(findPage(pb, "p1")!.page, new Set(["s2"])).length, 0);
  const p2 = addPage(pb, "c1", "Later");
  pb = movePresetToPage(p2.pb, "k2", p2.page);
  assert.equal(findPreset(pb, "k2")?.page.id, p2.page);
  pb = removePreset(pb, "k2");
  assert.equal(findPreset(pb, "k2"), null);
});

test("fire a Show at a role, and press again to go back", () => {
  const p: Preset = { id: "k1", title: "Tarn IV close up", to: "telescope", steps: [{ show: { bookmark: "b1" } }] };
  let b = emptyBoard({ scope: "u:k/sys:kestrel", title: "Kestrel" });
  b = fire(b, p, bm);
  assert.equal(title(b, "telescope"), "Tarn IV close up");
  assert.equal(title(b, "bridge"), "Kestrel");
  assert.ok(isUp(b, p));
  b = fire(b, p, bm);
  assert.equal(title(b, "telescope"), "Kestrel");
  assert.ok(!isUp(b, p));
});

test("an Everyone preset over a set-up is a layer; pressed again it lifts", () => {
  const p: Preset = { id: "k1", title: "Look at this", to: "everyone", steps: [{ show: { bookmark: "b1" } }] };
  let b = send(emptyBoard({ scope: "k", title: "Kestrel" }), "role:bridge", { scope: "k", title: "Bridge view" });
  b = fire(b, p, bm);
  assert.equal(title(b, "bridge"), "Look at this");
  b = fire(b, p, bm);
  assert.equal(title(b, "bridge"), "Bridge view");
});

test("a hold preset holds and releases; words only send nothing; a gone bookmark sends nothing", () => {
  const h: Preset = { id: "k2", title: "Signal lost", to: "bridge", steps: [{ hold: { card: "plain" } }] };
  let b = fire(emptyBoard({ scope: "k", title: "Kestrel" }), h, bm, ["bridge"]);
  assert.equal(title(b, "bridge"), "held");
  b = fire(b, h, bm, ["bridge"]);
  assert.equal(title(b, "bridge"), "Kestrel");
  const say: Preset = { id: "k3", title: "The beacon", to: "everyone", steps: [{ say: { text: "It repeats every ninety seconds." } }] };
  assert.deepEqual(fire(b, say, bm), b);
  const gone: Preset = { id: "k4", title: "Gone", to: "bridge", steps: [{ show: { bookmark: "b9" } }] };
  assert.deepEqual(fire(b, gone, bm), b);
  assert.equal(presetWords(gone, bm), "Show a bookmark that has gone");
  assert.equal(presetWords(h, bm), "Hold: awaiting transmission");
});

test("read back from a save: bad steps and presets are dropped; nothing usable gives a fresh playbook", () => {
  const pb = addPreset(emptyPlaybook(), "s1", { title: "Tarn", to: "everyone", steps: [{ show: { bookmark: "b1" } }, { say: { text: "Hello" } }] }).pb;
  assert.deepEqual(playbookFrom(JSON.parse(JSON.stringify(pb))), pb);
  const odd = JSON.parse(JSON.stringify(pb));
  odd.chapters[0].pages[0].sections[0].presets.push({ id: "k9", title: "x", to: "everyone", steps: [{ explode: true }] });
  odd.chapters[0].pages[0].sections[0].presets[0].steps.push({ hold: { card: "sparkles" } });
  assert.deepEqual(playbookFrom(odd), pb);
  assert.deepEqual(playbookFrom(null), emptyPlaybook());
  assert.deepEqual(playbookFrom({ schema: "sse.playbook/1", chapters: [] }), emptyPlaybook());
});

test("a Reveal step: words, read back, never on the board", async () => {
  const { reveals, revealsOf } = await import("../src/playbook.ts");
  const p: Preset = { id: "k5", title: "The wreck", to: "bridge", steps: [{ reveal: { address: "u:k/sys:1/body:4", landmark: "known", name: "the wreck" } }] };
  assert.equal(presetWords(p, bm), "Reveal the wreck");
  assert.ok(reveals(p));
  assert.deepEqual(revealsOf(p), [{ address: "u:k/sys:1/body:4", landmark: "known" }]);
  const b = emptyBoard({ scope: "k", title: "Kestrel" });
  assert.deepEqual(fire(b, p, bm), b);
  assert.ok(!isUp(b, p));
  const pb = addPreset(emptyPlaybook(), "s1", { title: p.title, to: p.to, steps: p.steps }).pb;
  assert.deepEqual(playbookFrom(JSON.parse(JSON.stringify(pb))), pb);
});
