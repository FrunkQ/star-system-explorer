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
