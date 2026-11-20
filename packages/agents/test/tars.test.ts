// A4 acceptance (build plan: answers cite the bundle or doc they came from).
// Tars answers from SSE's help passages and the kernel's explanation bundles;
// a model's answer must cite a passage it was given and keep to its numbers,
// or it goes back, and in the end Tars falls back on the passage itself. With
// no model he still answers, citing. The humour setting turns the jokes down.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { deriveScene, showReference, type DerivationDoc, type PackDoc, type SystemScene } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { askTars, checkAnswer, findPassages, rulesAnswer, SHELF, tarsSystem, whyPassage, DONT_KNOW } from "../src/tars.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const linked = (base: string): AgentsState => ({ ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base }], links: { support: { provider: "x", model: "m" } } });

test("the shelf: every passage has an id, a title and text; questions find the right one", () => {
  const ids = new Set<string>();
  for (const p of SHELF) {
    assert.ok(/^[a-z0-9-]+$/.test(p.id) && !ids.has(p.id), p.id);
    ids.add(p.id);
    assert.ok(p.title && p.text.length > 40, p.id);
  }
  assert.equal(findPassages("How do I change the physics of a flow?", SHELF)[0]!.id, "physics");
  assert.equal(findPassages("what can my players see, and the scanned button", SHELF)[0]!.id, "player-data");
  assert.equal(findPassages("where are my keys kept?", SHELF)[0]!.id, "agents");
  assert.deepEqual(findPassages("zzzz qqqq", SHELF), []);
});

test("with no model, Tars answers from the best passage and cites it", async () => {
  const r = await askTars({ question: "How do I age a system?", passages: SHELF, humour: 0.6 });
  assert.equal(r.answer.by, "rules");
  assert.equal(r.answer.sources[0]!.id, "lookahead");
  assert.match(r.answer.words, /^Look ahead ages the system on a branch/);
  assert.deepEqual(rulesAnswer("zzzz", []), { words: DONT_KNOW, sources: [], by: "rules" });
});

test("a model answer must cite what it was given and keep to its numbers", async () => {
  const found = findPassages("How do I undo an edit?", SHELF);
  assert.deepEqual(checkAnswer({ words: "Press Ctrl+Z.", sources: [] }, "q", found).problems, ["cite at least one passage you used, by its id"]);
  assert.match(checkAnswer({ words: "Press Ctrl+Z.", sources: ["made-up"] }, "q", found).problems[0]!, /you cited made-up, which you were not given/);
  assert.match(checkAnswer({ words: "Undo goes back 50 steps.", sources: ["chart"] }, "q", found).problems[0]!, /not in the passages you cited: 50/);
  assert.deepEqual(checkAnswer({ words: "Press Ctrl+Z.", sources: ["chart"] }, "q", found).problems, []);
  assert.deepEqual(checkAnswer({ words: DONT_KNOW, sources: [] }, "q", found).problems, [], "not knowing needs no citation");

  // First answer uncited, then cited: passes, by the model, with its source.
  const p = await mockProvider({ replies: [fence({ words: "Press Ctrl+Z, naturally.", sources: [] }), fence({ words: "Press Ctrl+Z: each undo is a new correction on the chart's history.", sources: ["chart"] })] });
  try {
    const st = linked(p.base);
    const r = await askTars({ question: "How do I undo an edit?", passages: SHELF, humour: 0.9, ask: asker(st, "support") });
    assert.equal(r.answer.by, "model");
    assert.deepEqual(r.answer.sources.map((x) => x.id), ["chart"]);
    assert.equal(r.run!.calls, 2);
    const sys = JSON.parse(p.seen.find((x: { path: string }) => x.path.endsWith("/chat/completions")).body).messages[0].content;
    assert.match(sys, /Be dryly funny, briefly/);
  } finally {
    await p.close();
  }
  // A model that never cites: Tars falls back on the passage, still cited.
  const q = await mockProvider({ replies: () => "Just press buttons until it works." });
  try {
    const r = await askTars({ question: "How do I undo an edit?", passages: SHELF, humour: 0, ask: asker(linked(q.base), "support") });
    assert.equal(r.answer.by, "rules");
    assert.ok(r.answer.sources.length >= 1);
    assert.match(JSON.parse(q.seen[0].body).messages[0].content, /Be plain and direct; no jokes/);
  } finally {
    await q.close();
  }
  assert.match(tarsSystem(0.5), /A light, dry remark/);
});

test("why? passages come from the kernel's explanation bundles, with their references", async () => {
  const k = nodeKernel();
  try {
    const pack: PackDoc = json("packs/base-physics/pack.json");
    const scene: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");
    const doc: DerivationDoc = json("packages/shell/src/documents/derivations.base.json");
    const s = await Session.open(k.client, gmSeat(), scene.universe, [pack], k.scratch);
    const derived = await deriveScene(s, scene, doc);
    const tarn = derived.subjects.find((x) => x.address === "u:kestrel/sys:1/body:3")!;
    const f = tarn.facts.find((x) => x.field === "surface_k")!;
    const e = await s.explain(f.why!, f.why!.kind);
    assert.ok(e.ok);
    const passage = whyPassage({ subject: tarn.label, field: f.field, label: f.label, display: f.display, curious: e.explanation.lenses.curious, science: e.explanation.lenses.science, validity: e.explanation.validity, references: e.explanation.references.map(showReference) });
    assert.equal(passage.id, `why:${tarn.label}:surface_k`);
    assert.ok(passage.text.length > 20);
    assert.ok(passage.references!.length >= 1 && passage.references!.every((r) => r.text));
    // Asked why, Tars finds and cites the explanation.
    const r = await askTars({ question: `Why is ${tarn.label}'s surface temperature what it is?`, passages: [...SHELF, passage], humour: 0.6 });
    assert.equal(r.answer.sources[0]!.id, passage.id);
  } finally {
    await k.close();
  }
});
