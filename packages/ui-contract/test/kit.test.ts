// AT14: interface kits as data. The loader's checks are the same for a starter, a pack's kit and Orac's draft.
import { test } from "node:test";
import assert from "node:assert/strict";
import { atlasDocsOf, checkKit, checkControlSet, controlSetOf, controlWords, drawingOf, kitScreen, STARTER_CONTROL_SETS, checkPalette, contrastOf, partsOf, resolveKit, SHIPPED_PALETTES, STARTER_KITS, type KitDoc, type PartInfo } from "../src/atlas/index.ts";
import { SHIPPED_EDITIONS } from "../src/atlas/index.ts";
import { STARTER_DELIVERIES } from "../src/delivery.ts";

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
    delivery: { turn: { kind: "type_on", ms: 1500 }, arrive: { kind: "explode" } },
    needs: { tier: "gpu-high" },
    banners: { at: "middle", sets: ["sse.notes.nowhere"] },
    layouts: {
      system: { arrangement: "four", items: [{ view: "sse.view.atlas", area: "c", controls: { step: false, teleport: true } }, { view: "sse.view.flow-editor", area: "1" }] },
      bridge: { arrangement: "one" },
    },
  };
  const said = checkKit(bad, STACK).join("\n");
  for (const m of [/extends sse\.kit\.nowhere, which is not here/, /state a figure \("5 kg"\)/, /look "neon"/, /palette "no-such"/, /murky, ink on ground has a contrast of 1\.\d to 1/, /A Turn lasts at most 0\.4 s/, /"explode" is not a delivery/, /names a kit a weaker device falls back to/, /area "c", which four does not have \(1, 2, 3, 4\)/, /switches "teleport" on sse\.view\.atlas/, /"sse\.view\.flow-editor", which is not a part on a player's screen/, /"bridge" is not a situation/, /banners sit at "middle"/, /note set "sse\.notes\.nowhere" is not in the pack stack/]) {
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

test("a kit's deliveries are AT15a's: a starter by name, or a look's own moment by moment", () => {
  assert.deepEqual(resolveKit("sse.kit.hauler").delivery, STARTER_DELIVERIES.hauler, "Hauler warms up and builds up, as the Hauler look does");
  assert.deepEqual(resolveKit("sse.kit.guide").delivery, STARTER_DELIVERIES.guide, "the Guide redraws in bands");
  const quiet = { ...STARTER_KITS[0]!, id: "mine.kit.quiet", extends: "sse.kit.hauler", delivery: { power_on: { kind: "fade" as const } } };
  const k = resolveKit(quiet.id, [quiet]);
  assert.deepEqual(k.delivery, { ...STARTER_DELIVERIES.hauler, power_on: { kind: "fade" } }, "a kit that extends a starter changes one moment");
  assert.match(checkKit({ ...quiet, delivery: "disco" as never }, {}).join(" "), /"disco" is not a starter's/);
});

// ---- AT15: control sets and the kit's screens.
test("control sets: every starter's set loads, each kit names one, and a kind a set leaves out comes from plain", () => {
  for (const s of STARTER_CONTROL_SETS) assert.deepEqual(checkControlSet(s), [], s.id);
  for (const k of STARTER_KITS) {
    assert.ok(typeof k.control_set === "string" && STARTER_CONTROL_SETS.some((s) => s.id === k.control_set), `${k.id} names a set`);
    assert.equal(controlSetOf(k).id, k.control_set);
  }
  assert.equal(drawingOf("hauler", "button").frame, "bracket");
  assert.equal(drawingOf("hauler", "dial").frame, "plain", "Hauler draws no dial of its own: the plain one");
  assert.equal(drawingOf("deck", "button").frame, "cap");
  assert.equal(drawingOf("nobody", "tab").frame, "plain");
  const variant = { id: "amber-hauler", name: "Amber hauler", extends: "hauler", kinds: {} };
  assert.deepEqual(checkControlSet(variant), []);
  assert.equal(drawingOf("amber-hauler", "button", [...STARTER_CONTROL_SETS, variant]).frame, "bracket", "a variant costs nothing");
});

test("control words follow the set: Print is [P]RINT on Hauler's command line and Print in the Guide", () => {
  assert.deepEqual(controlWords("Print", drawingOf("hauler", "button")), { text: "[P]RINT", key: "P" });
  assert.deepEqual(controlWords("Next page", drawingOf("hauler", "step"), "N"), { text: "[N]EXT PAGE", key: "N" });
  assert.deepEqual(controlWords("Contents", drawingOf("hauler", "button"), "X"), { text: "[X] CONTENTS", key: "X" }, "a key not in the word goes in front");
  assert.deepEqual(controlWords("Print", drawingOf("guide", "button")), { text: "Print" });
  assert.deepEqual(controlWords("Print", drawingOf("deck", "button")), { text: "PRINT" });
});

test("checkControlSet says what to change", () => {
  const bad = { id: "x", name: "X", extends: "nope", kinds: { lever: { frame: "plain" }, button: { frame: "round", lit: "sparkle" }, tab: { frame: "pill", hotkey: true } } };
  const errs = checkControlSet(bad);
  assert.ok(errs.some((e) => /extends "nope"/.test(e)));
  assert.ok(errs.some((e) => /"lever", which is not a kind of control/.test(e)));
  assert.ok(errs.some((e) => /button needs a frame/.test(e)));
  assert.ok(errs.some((e) => /lit as "sparkle"/.test(e)));
  assert.ok(errs.some((e) => /tab shows its key, which needs a frame of bracket or key/.test(e)));
  const k = { ...STARTER_KITS[0]!, id: "mine.k", control_set: "missing" };
  assert.ok(checkKit(k).some((e) => /control set "missing" is not loaded/.test(e)));
  assert.ok(!checkKit({ ...STARTER_KITS[0]!, id: "mine.k2", control_set: bad }).every((e) => !/lever/.test(e)));
});

test("kitScreen: a starter's layout for a situation is an ordinary screen document; a holding card is not", () => {
  const sys = kitScreen("sse.kit.hauler", "system")!;
  assert.equal(sys.schema, "sse.screen/1");
  assert.equal(sys.audience, "players");
  assert.equal(sys.id, "sse.kit.hauler.system");
  assert.equal(sys.layout, "grid");
  assert.ok(sys.items.some((i) => i.view === "sse.view.atlas" && typeof i.controls === "object"), "the Atlas part with its controls");
  const world = kitScreen("sse.kit.hauler", "world")!;
  assert.deepEqual(world.items, sys.items, "world extends system");
  assert.equal(kitScreen("sse.kit.hauler", "holding"), null);
  assert.equal(kitScreen("no.such.kit", "system"), null);
  for (const k of STARTER_KITS) for (const s of ["sector", "system", "world", "landing"] as const) {
    const d = kitScreen(k, s);
    assert.ok(d, `${k.id} has a ${s} screen`);
    const areas = d.items.map((i) => i.area);
    assert.equal(new Set(areas).size, areas.length, `${k.id} ${s}: one part to an area`);
  }
});
