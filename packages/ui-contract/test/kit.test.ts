// AT14: interface kits as data. The loader's checks are the same for a starter, a pack's kit and Orac's draft.
import { test } from "node:test";
import assert from "node:assert/strict";
import { atlasDocsOf, checkKit, checkPalette, contrastOf, controlsOn, resolveKit, SHIPPED_PALETTES, STARTER_KITS, type KitDoc, type PartInfo } from "../src/atlas/index.ts";
import { SHIPPED_EDITIONS } from "../src/atlas/index.ts";

// The parts' controls as the design lists them (11A.4), until the parts declare their own (AT13).
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
const STACK = {
  parts: PARTS,
  looks: ["chart", "paper", "deck", "phosphor", "glass", "guide", "guide-gadget"],
  editions: SHIPPED_EDITIONS.map((e) => e.id),
  vocabs: ["plain", "guide"],
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
    layouts: {
      system: { arrangement: "report", items: [{ view: "sse.view.atlas", area: "c", controls: { edition: false, teleport: true } }, { view: "sse.view.flow-editor", area: "a" }] },
      bridge: { arrangement: "one" },
    },
  };
  const said = checkKit(bad, STACK).join("\n");
  for (const m of [/extends sse\.kit\.nowhere, which is not here/, /state a figure \("5 kg"\)/, /look "neon"/, /palette "no-such"/, /murky, ink on ground has a contrast of 1\.\d to 1/, /A Turn lasts at most 0\.4 s/, /"explode" is not in the toolkit/, /names a kit a weaker device falls back to/, /area "c", which report does not have/, /switches "teleport" on sse\.view\.atlas/, /the GM's alone/, /"bridge" is not a situation/]) {
    assert.match(said, m);
  }
});

test("controls: none is a picture, all is every declared one, unstated take the default", () => {
  const atlas = PARTS[0]!;
  assert.deepEqual(controlsOn({ view: atlas.id, area: "a", controls: "none" }, atlas), []);
  assert.deepEqual(controlsOn({ view: atlas.id, area: "a", controls: "all" }, atlas), atlas.controls);
  assert.deepEqual(controlsOn({ view: atlas.id, area: "a", controls: { edition: false, contents: false } }, atlas), ["step", "print", "links"]);
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
