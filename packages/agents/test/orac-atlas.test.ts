// AT11 and AT16 acceptance: Orac drafts Atlas editions, margin note sets and
// interface kits, each checked by the Atlas loader's own checks (with the test
// render for an edition) before the GM sees it. "Hauler but amber" gives a kit
// that passes every check; a figure typed into an edition, a note or a kit is
// refused; a shipped id is never overwritten. Accept keeps the document in the
// campaign's own pack, stacked through the kernel like any pack.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import type { EditionDoc, KitDoc, NoteSet, PackDoc, PartInfo, PlayerScene, Vocab } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { plainWorkspace } from "../../ui-contract/test/workspace.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { runOrac } from "../src/orac.ts";
import { ATLAS_PACK, atlasEntries, atlasPackKeep, editionTarget, kitTarget, noteSetTarget, type AtlasDocs, type AtlasHost, type AtlasKind } from "../src/orac-atlas.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const before: PlayerScene = json("tests/fixtures/observe/tarn-before.json");
const after: PlayerScene = json("tests/fixtures/observe/tarn-after.json");
const VOCABS: Record<string, Vocab> = { plain: json("packages/shell/src/documents/vocab.plain.json"), guide: json("packages/shell/src/documents/vocab.guide.json") };
const PARTS: PartInfo[] = [
  { id: "sse.view.atlas", controls: ["contents", "step", "edition", "print", "links"] },
  { id: "sse.view.chart", controls: ["view_style", "seen_from", "scale", "hud", "band", "turn", "select", "menu"] },
  { id: "sse.view.system-3d", controls: ["view_style", "seen_from", "scale", "hud", "band", "turn", "select", "menu"] },
  { id: "sse.view.card", controls: ["open", "why", "depth"] },
  { id: "sse.view.table", controls: ["sort", "filter"] },
  { id: "sse.view.readout", controls: [] },
  { id: "sse.view.text", controls: [] },
  { id: "sse.view.flow-editor", controls: [], gm_only: true },
];
const made = { kind: "", doc: null };

function memoryHost(): AtlasHost & { kept: AtlasDocs } {
  const kept: AtlasDocs = { editions: [], note_sets: [], kits: [] };
  return {
    kept,
    docs: () => kept,
    looks: () => ["chart", "paper", "deck", "phosphor", "glass", "guide", "guide-gadget"],
    vocabs: () => VOCABS,
    parts: () => PARTS,
    scenes: async () => [after, before],
    keep: async (kind: AtlasKind, doc) => {
      (kept[kind] as { id: string }[]).push(doc);
      return null;
    },
  };
}

const amber: KitDoc = { schema: "sse.kit/1", id: "mine.hauler-amber", name: "Hauler, amber", description: "Hauler on an amber screen, with the hazard light always on.", extends: "sse.kit.hauler", palettes: ["phosphor-amber"] } as KitDoc;

test("kits: Hauler but amber passes every check; refusals in the loader's words; a shipped id is never overwritten", async () => {
  const h = memoryHost();
  const t = kitTarget(h);
  const good = await t.check(amber, made);
  assert.deepEqual(good.problems, []);
  assert.equal(t.summary(good.value!), "Hauler, amber: Hauler on an amber screen, with the hazard light always on. Starts from Hauler; in Amber. Passed every check the kit loader has (parts, controls, contrast, deliveries, tier). Accept keeps it in the campaign's own pack; choose it on a role card.");
  assert.match((await t.check({ ...amber, id: "sse.kit.hauler" }, made)).problems[0]!, /starting "mine\."/);
  assert.match((await t.check({ ...amber, id: "Hauler Amber" }, made)).problems[0]!, /for example "mine\.hauler-amber"/);
  const muddy = await t.check({ ...amber, palettes: [{ id: "mine.mud", name: "Mud", tokens: { ground: "#333333", ink: "#444444", "ink-muted": "#444444", accent: "#555555", "accent-ink": "#444444" } }] }, made);
  assert.ok(muddy.problems.some((p) => /contrast|4\.5/.test(p)), muddy.problems.join("; "));
  const flow = await t.check({ ...amber, layouts: { system: { arrangement: "picture-and-side", items: [{ view: "sse.view.flow-editor", area: "a" }] } } }, made);
  assert.ok(flow.problems.some((p) => /GM's alone/.test(p)), flow.problems.join("; "));
  const fig = await t.check({ ...amber, description: "Hauler at 0.7 bar" }, made);
  assert.ok(fig.problems.some((p) => /may not state a fact/.test(p)), fig.problems.join("; "));
  const noPrint = await t.check({ ...amber, id: "mine.wall", name: "Wall screen", description: "Hauler without the print button.", palettes: undefined, layouts: { world: { arrangement: "picture-and-side", items: [{ view: "sse.view.atlas", area: "b", controls: { print: false, shout: true } }] } } } as unknown as KitDoc, made);
  assert.ok(noPrint.problems.some((p) => /switches "shout" on sse\.view\.atlas, which does not declare it/.test(p)), noPrint.problems.join("; "));
  assert.equal(await t.accept(good.value!), null);
  assert.equal(h.kept.kits[0]!.id, "mine.hauler-amber");
});

test("editions: the loader and the test render check Orac's edition; a figure or an unknown field is refused", async () => {
  const h = memoryHost();
  const t = editionTarget(h);
  const fleet: EditionDoc = { schema: "sse.edition/1", id: "mine.fleet-survey", name: "Fleet Survey Office", description: "The company edition on the Fleet Survey Office's paper.", extends: "sse.edition.company", letterhead: { use: "brand", lines: ["{brand.name} FLEET SURVEY OFFICE", "{doc.number}"], stamp: "FOR OFFICIAL USE" } } as EditionDoc;
  const good = await t.check(fleet, made);
  assert.deepEqual(good.problems, []);
  assert.match(t.summary(good.value!), /^Fleet Survey Office: The company edition on the Fleet Survey Office's paper\. Starts from sse\.edition\.company\. Drawn for 2 audiences beside the Survey atlas: every figure the same, nothing shown that they were not given\./);
  const fig = await t.check({ ...fleet, voice: { gap: "beyond 5 AU, unsurveyed" } }, made);
  assert.ok(fig.problems.some((p) => /states a figure/.test(p)), fig.problems.join("; "));
  const field = await t.check({ ...fleet, templates: { world: [{ key: "secret", lines: ["base.location"] }] } }, made);
  assert.ok(field.problems.some((p) => /"base\.location", which is not a fact the Atlas knows/.test(p)), field.problems.join("; "));
  assert.match((await t.check({ ...fleet, id: "sse.edition.company" }, made)).problems[0]!, /shipped edition/);
  assert.equal(await t.accept(good.value!), null);
  // A kit may now name the campaign's own edition.
  assert.deepEqual((await kitTarget(h).check({ ...amber, atlas: { edition: "mine.fleet-survey" } }, made)).problems, []);
});

test("note sets: a figure must be a slot its condition tests; a good set is kept", async () => {
  const h = memoryHost();
  const t = noteSetTarget(h);
  const set: NoteSet = { schema: "sse.margin-notes/1", id: "mine.church", name: "The Church's marginalia", notes: [{ id: "wet", about: "world", when: { fact: "ocean.fraction", at_least: 0.7 }, text: "With {ocean.fraction} of it under water, baptisms are never short of supply." }] } as NoteSet;
  const good = await t.check(set, made);
  assert.deepEqual(good.problems, []);
  assert.equal(t.summary(good.value!), "The Church's marginalia: 1 note about world, each shown only when what the audience knows fits it. Accept keeps it in the campaign's own pack; name it in an edition's margin to show it.");
  const typed = await t.check({ ...set, notes: [{ ...set.notes[0]!, text: "A full 81% of it is under water." }] }, made);
  assert.ok(typed.problems.some((p) => /states a figure/.test(p)), typed.problems.join("; "));
  const unread = await t.check({ ...set, notes: [{ id: "x", about: "world", text: "Waters cover {ocean.fraction} of it." }] }, made);
  assert.ok(unread.problems.some((p) => /"when" does not test/.test(p)), unread.problems.join("; "));
  await t.accept(good.value!);
  // An edition may now show the campaign's own set.
  assert.deepEqual((await editionTarget(h).check({ schema: "sse.edition/1", id: "mine.church-guide", name: "Church guide", description: "The Guide with the Church's notes.", extends: "sse.edition.guide", margin: { sets: ["mine.church"] } }, made)).problems, []);
});

test("the catalogue lists editions, note sets, kits, palettes, parts with their controls, and looks", () => {
  const e = atlasEntries(memoryHost());
  for (const k of ["edition:sse.edition.company", "notes:sse.notes.guide", "kit:sse.kit.hauler", "palettes", "parts", "looks"]) assert.ok(e.some((x) => x.key === k), k);
  assert.match(e.find((x) => x.key === "parts")!.full, /sse\.view\.flow-editor: controls none \(GM only: never on a player's screen\)/);
  assert.match(e.find((x) => x.key === "palettes")!.full, /phosphor-amber: Amber/);
});

test("accept keeps the document in campaign.atlas through the kernel; a second edit replaces its own id", async () => {
  const k = nodeKernel();
  try {
    const scene = json("packages/shell/src/documents/kestrel.scene.json");
    const s = await Session.open(k.client, gmSeat(), scene.universe, [json("packs/base-physics/pack.json")], k.scratch);
    const keep = atlasPackKeep(s, plainWorkspace(scene.now_yr));
    assert.equal(await keep("kits", amber), null);
    assert.equal(await keep("kits", { ...amber, name: "Hauler, amber (2)" }), null);
    const pack = s.stack().added.find((p: PackDoc) => p.id === ATLAS_PACK)!;
    const kits = (pack.provides!.other as { atlas: AtlasDocs }).atlas.kits;
    assert.deepEqual(kits.map((x) => x.name), ["Hauler, amber (2)"]);
    assert.equal(pack.version, "0.0.2");
  } finally {
    await k.close();
  }
});

test("Orac drafts Hauler but amber: an invented palette name goes back, the fixed kit is offered", async () => {
  const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
  const p = await mockProvider({ replies: (m: { content: string }[]) => fence({ note: "Hauler in amber.", make: [{ kind: "kit", title: "Hauler, amber", doc: { ...amber, palettes: [/not known/.test(m.at(-1)!.content) ? "phosphor-amber" : "amber"] } }] }) });
  try {
    const h = memoryHost();
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { building: { provider: "x", model: "m" } } };
    const r = await runOrac({ request: "Hauler but amber, with the hazard light always on.", targets: [kitTarget(h), editionTarget(h), noteSetTarget(h)], catalogue: atlasEntries(h), ask: asker(st, "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /The palette "amber" is not known/);
    assert.equal(r.passed.length, 1);
    assert.equal(h.kept.kits.length, 0, "nothing kept before accept");
  } finally {
    await p.close();
  }
});
