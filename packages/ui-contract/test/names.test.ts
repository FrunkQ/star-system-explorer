// NM6: the Names page reads the stack's schemes and defaults from the packs, and samples names through the kernel,
// on the stack or in a trial when the base names pack is only offered.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { layerWords, meaningWords, namesOnStack, sampleKinds, saysMore, sortWords } from "../src/names.ts";
import type { PackDoc } from "../src/physics.ts";
import type { SystemScene } from "../src/scene.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const names: PackDoc = json("packs/base-names/pack.json");
const scene: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");

test("schemes and defaults come from the stack, a later pack winning", () => {
  assert.deepEqual(namesOnStack([base]), { schemes: [], defaults: [] });
  const n = namesOnStack([base, names]);
  assert.ok(n.schemes.length >= 10);
  assert.deepEqual(n.defaults, [{ layer: "designation", scheme: "catalogue", pack: "sse.base-names" }]);
  const vesh = n.schemes.find((s) => s.id === "vesh")!;
  assert.equal(vesh.sort, "invented");
  assert.equal(vesh.pack, "sse.base-names");
  assert.deepEqual(sampleKinds(vesh).slice(0, 3), ["planet", "moon", "star"]);
  const mix = n.schemes.find((s) => s.id === "many-nations")!;
  assert.ok(mix.mix?.length && mix.kinds.includes("planet"), "a mix has its members' kinds");
  // A later pack's scheme with the same id replaces the earlier one, and its default wins.
  const later = { ...names, id: "fan.names", provides: { other: { names: { schemes: [{ id: "vesh", label: "Vesh, retold", kind: "invented", forms: { planet: "{word}" } }], defaults: { common: "vesh" } } } } } as unknown as PackDoc;
  const m = namesOnStack([base, names, later]);
  assert.equal(m.schemes.filter((s) => s.id === "vesh").length, 1);
  assert.equal(m.schemes.find((s) => s.id === "vesh")!.label, "Vesh, retold");
  assert.deepEqual(m.defaults.find((d) => d.layer === "common"), { layer: "common", scheme: "vesh", pack: "fan.names" });
  assert.deepEqual(["designation", "common", "local_vesh_hive"].map(layerWords), ["Designation", "Common", "Local (vesh hive)"]);
  assert.equal(sortWords("real"), "real languages");
});

test("a sample is the same in a trial and on the stack, and never changes the campaign", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base], k.scratch);
    const state0 = await s.stateHash();
    const ask = { scheme: "vesh", kind: "planet", count: 12, start: 0 };
    assert.equal((await s.nameSample(ask)).ok, false, "no names pack on the stack: no such scheme");
    const t = await s.trial({ added: [names] }, (p) => p.nameSample(ask));
    assert.ok(t.ok && t.result.ok, JSON.stringify(t));
    const tried = t.result.value;
    assert.equal(tried.names.length, 12);
    assert.ok(tried.names.every((n) => n.text && saysMore(n) && n.hints), "every Vesh name says how to say it, with hints for its sounds");
    const english = await s.trial({ added: [names] }, (p) => p.nameSample({ ...ask, scheme: "english" }));
    assert.ok(english.ok && english.result.ok);
    assert.ok(english.result.value.names.every((n) => meaningWords(n) && !saysMore(n)), "English-sounding names carry meanings; saying them is no help");
    assert.equal(await s.stateHash(), state0, "a trial sample records nothing");
    const more = await s.trial({ added: [names] }, (p) => p.nameSample({ ...ask, start: 12 }));
    assert.ok(more.ok && more.result.ok && more.result.value.names[0]!.text !== tried.names[0]!.text, "another twelve are other names");
    // Stacked, the same ask gives the same names: keyed draws, not a sequence.
    const s2 = await Session.open(k.client, gmSeat(), scene.universe, [base, names], k.scratch);
    const on = await s2.nameSample(ask);
    assert.ok(on.ok);
    assert.equal(on.value.digest, tried.digest);
  } finally {
    await k.close();
  }
});

test("rename and set_naming are GM intents that become the kernel's own requests", async () => {
  const { checkIntent, intentToRequest } = await import("../src/intent.ts");
  const { decide, tableSeat } = await import("../src/seat.ts");
  const r = { kind: "rename" as const, address: "u:k/sys:1/body:3", layer: "local_vesh", text: " Zakqu ", at_yr: 4.6e9 };
  assert.equal(checkIntent(r), null);
  const req = intentToRequest(r) as { op: string; text?: string; layer: string };
  assert.equal(req.op, "rename");
  assert.equal(req.text, "Zakqu");
  assert.equal((intentToRequest({ ...r, text: null }) as { text?: string }).text, undefined, "null takes the typed name away");
  assert.match(checkIntent({ ...r, layer: "Local Vesh" })!, /layer/);
  assert.match(checkIntent({ ...r, text: "  " })!, /some text/);
  const n = { kind: "set_naming" as const, address: "u:k/sys:1", layer: "common", scheme: "vesh", of_kind: "moon", at_yr: 4.6e9 };
  assert.deepEqual(Object.keys(intentToRequest(n)).sort(), ["address", "at_s", "kind", "layer", "op", "scheme"]);
  assert.equal((intentToRequest({ ...n, scheme: null }) as { scheme?: string }).scheme, undefined);
  assert.equal(decide(gmSeat(), r).action, "apply");
  assert.equal(decide({ ...tableSeat(), may_send: ["rename"] }, r).action, "ask_gm", "a player's rename goes to the GM");
});

test("layer choice in lenses: a lens names a layer, the chart's labels follow it, nothing else changes", async () => {
  const { chartNameLayers, namesInLayer, relabel } = await import("../src/names.ts");
  const { checkLens, saved, SHIPPED_LENSES } = await import("../src/lens.ts");
  assert.deepEqual(
    chartNameLayers().map((l) => l.id),
    ["", "designation", "local"],
  );
  assert.ok(chartNameLayers([{ layer: "common" }, { layer: "traveller" }]).some((l) => l.id === "traveller"));
  const lens = { ...SHIPPED_LENSES[0]!, id: "mine.survey", names: "designation" };
  assert.deepEqual(checkLens(lens), []);
  assert.equal(saved(lens).names, "designation", "the layer is kept when saved");
  assert.equal(saved({ ...lens, names: undefined }).names, undefined);
  assert.ok(checkLens({ ...lens, names: "Not A Layer" }).length);
  const asked: string[] = [];
  const session = {
    readNames: async (a: string) => {
      asked.push(a);
      return a.endsWith("1") ? { ok: true as const, value: { address: a, kind: "planet", names: [{ layer: "designation", text: "KS 1 b", source: "drawn" as const }] } } : { ok: false as const, code: "x", message: "no" };
    },
  };
  assert.equal((await namesInLayer(session, ["a/1", "a/2"], "common")).size, 0, "common needs no reading");
  assert.deepEqual(asked, []);
  const m = await namesInLayer(session, ["a/1", "a/2"], "designation");
  assert.deepEqual([...m], [["a/1", "KS 1 b"]]);
  const sc = { subjects: [{ address: "a/1", label: "Tarn", facts: [1] }, { address: "a/2", label: "Ember", facts: [2] }] };
  const out = relabel(sc, m);
  assert.deepEqual(
    out.subjects.map((x) => x.label),
    ["KS 1 b", "Ember"],
  );
  assert.equal(out.subjects[0]!.facts, sc.subjects[0]!.facts, "only the label changes");
  assert.equal(relabel(sc, new Map()), sc);
});
