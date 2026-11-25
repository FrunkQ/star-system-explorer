// AT14: interface kits as data. The loader's checks are the same for a starter, a pack's kit and Orac's draft.
import { test } from "node:test";
import assert from "node:assert/strict";
import { atlasDocsOf, checkKit, checkPalette, contrastOf, partsOf, resolveKit, SHIPPED_PALETTES, STARTER_KITS, type KitDoc, type PartInfo } from "../src/atlas/index.ts";
import { SHIPPED_EDITIONS } from "../src/atlas/index.ts";

// The players' parts as they declare their controls (AT13; tests/ui/kit-parts.test.ts reads the real declarations).
const PARTS: PartInfo[] = [
  { id: "sse.view.atlas", controls: ["contents", "step", "print", "links"] },
  { id: "sse.view.chart", controls: ["select"] },
  { id: "sse.view.system-3d", controls: ["turn"] },
  { id: "sse.view.card", controls: ["open"] },
  { id: "sse.view.table", controls: ["sort", "select"] },
  { id: "sse.view.readout", controls: [] },
  { id: "sse.view.text", controls: [] },
];
const STACK = {
  parts: PARTS,
  looks: ["chart", "paper", "deck", "phosphor", "glass", "guide", "guide-gadget"],
  editions: SHIPPED_EDITIONS.map((e) => e.id),
  vocabs: ["plain", "guide"],
  noteSets: ["sse.notes.guide"],
};

test("the four starters load, with every palette readable", () => {
  assert.deepEqual(STARTER_KITS.map((k) => k.name), ["Hauler", "Corporate", "Deck", "The Guide"]);
  for (const k of STARTER_KITS) assert.deepEqual(checkKit(k, STACK), [], k.id);
  for (const p of SHIPPED_PALETTES) assert.deepEqual(checkPalette(p), [], p.id);
  assert.deepEqual(resolveKit("sse.kit.hauler").palettes, ["phosphor-green", "phosphor-amber", "phosphor-white"], "Hauler's green, amber and white");
});

test("extends: a kit made from a starter takes what it leaves out; a layout may extend another situation", () => {
  const salvage: KitDoc = { schema: "sse.kit/1", id: "mine.kit.salvage", name: "Salvage console", description: "A tired amber terminal.", extends: "sse.kit.hauler", palettes: ["phosphor-amber", "phosphor-green"] };
  const k = resolveKit(salvage.id, [salvage]);
  assert.equal(k.look, "phosphor");
  assert.deepEqual(k.palettes, ["phosphor-amber", "phosphor-green"]);
  assert.deepEqual(k.layouts!.world, k.layouts!.system, "world extends system");
  assert.equal(k.layouts!.holding!.card, "Signal lost");
  assert.deepEqual(checkKit(salvage, STACK), []);
  assert.throws(() => resolveKit("x.loop", [{ ...salvage, id: "x.loop", extends: "x.loop" }]), /extends itself/);
});

test("the loader refuses with sentences that say what to change", () => {
  const bad = {
    schema: "sse.kit/1",
    id: "mine.kit.bad",
    name: "Bad",
    description: "Holds 5 kg of cargo.",
    extends: "sse.kit.nowhere",
    look: "neon",
    palettes: ["no-such", { id: "murky", name: "Murky", tokens: { ground: "#333333", ink: "#444444", "ink-muted": "#555555", accent: "#666666", "accent-ink": "#777777" } }],
    delivery: { turn: { use: "type-on", seconds: 1.5 }, arrive: { use: "explode" } },
    needs: { tier: "gpu-high" },
    banners: { at: "middle", sets: ["sse.notes.nowhere"] },
    layouts: {
      system: { arrangement: "four", items: [{ view: "sse.view.atlas", area: "c", controls: { step: false, teleport: true } }, { view: "sse.view.flow-editor", area: "1" }] },
      bridge: { arrangement: "one" },
    },
  };
  const said = checkKit(bad, STACK).join("\n");
  for (const m of [/extends sse\.kit\.nowhere, which is not here/, /state a figure \("5 kg"\)/, /look "neon"/, /palette "no-such"/, /murky, ink on ground has a contrast of 1\.\d to 1/, /A Turn lasts at most 0\.4 s/, /"explode" is not in the toolkit/, /names a kit a weaker device falls back to/, /area "c", which four does not have \(1, 2, 3, 4\)/, /switches "teleport" on sse\.view\.atlas/, /"sse\.view\.flow-editor", which is not a part on a player's screen/, /"bridge" is not a situation/, /banners sit at "middle"/, /note set "sse\.notes\.nowhere" is not in the pack stack/]) {
    assert.match(said, m);
  }
});

test("partsOf reads a view's switchable declarations; a contrast is a WCAG ratio", () => {
  assert.deepEqual(partsOf([{ id: "v", switchable: [{ id: "sort", words: "Sort", kind: "button", default: true }] }, { id: "w" }]), [{ id: "v", controls: ["sort"] }, { id: "w", controls: [] }]);
  assert.ok((contrastOf("#000000", "#FFFFFF") ?? 0) > 20.9);
});

test("atlasDocsOf gathers editions, note sets and kits from provides.other.atlas; a later pack replaces by id", () => {
  const pack = (id: string, atlas: unknown) => ({ format: "sse.pack/1", id, version: "0.1.0", kind: ["other"], kernel_range: "^0.1", licence: "CC0", provides: { other: { atlas } } });
  const hauler = STARTER_KITS[0]!;
  const docs = atlasDocsOf([
    pack("a", { kits: [hauler, { ...hauler, id: "mine.kit.salvage" }] }),
    { format: "sse.pack/1", id: "b", version: "0.1.0", kind: ["other"], kernel_range: "^0.1", licence: "CC0" },
    pack("campaign.atlas", { kits: [{ ...hauler, name: "Hauler (ours)" }], note_sets: "oops", charts: [] }),
  ]);
  assert.deepEqual(docs.kits.map((k) => [k.id, k.name]), [["sse.kit.hauler", "Hauler (ours)"], ["mine.kit.salvage", "Hauler"]]);
  assert.match(docs.refused.join("\n"), /atlas\.charts; the lists are editions, note_sets and kits/);
  assert.match(docs.refused.join("\n"), /atlas\.note_sets that is not a list/);
});
