// U5 Packs screen: a pack is checked by the kernel and shown with every
// derived value it would change before it is stacked; stacking is one intent.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { deriveScene, type DerivationDoc } from "../src/derivation.ts";
import { checkPackDoc, packSummary, percentWords, previewStack, reportWords, sourceLink, stackPacks, unitWords } from "../src/packs-edit.ts";
import { adoptRequest, draftParam } from "../src/physics-edit.ts";
import type { PackDoc } from "../src/physics.ts";
import type { SystemScene } from "../src/scene.ts";
import { nodeKernel } from "./node-kernel.ts";
import { plainWorkspace } from "./workspace.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const warm: PackDoc = json("packages/shell/src/documents/sample.warm-worlds.pack.json");
const scene: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");
const doc: DerivationDoc = json("packages/shell/src/documents/derivations.base.json");
const TARN = "u:kestrel/sys:1/body:3";

test("pack summaries and schema checks", () => {
  const b = packSummary(base);
  // The summary counts what the base pack holds (it grows as physics packages add flows).
  assert.equal(b.flows.length, (base as { provides: { flows: unknown[] } }).provides.flows.length);
  assert.ok(b.flows.length >= 7, "P1's heat flows are in the base pack");
  assert.ok(b.quantity_kinds >= 53);
  assert.equal(packSummary(warm).fictional, true);
  assert.deepEqual(packSummary(warm).replaces, ["sse.base.planet-climate from sse.base-physics ^0.1.0"]);
  assert.equal(checkPackDoc(warm, ["sse.base-physics"]), null);
  assert.match(checkPackDoc(base, ["sse.base-physics"])!, /already/);
  assert.match(checkPackDoc({ format: "x" }, [])!, /sse.pack\/1/);
});

test("add a fictional pack: preview, stack, labels, remove", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base], k.scratch);
    const ws = plainWorkspace(scene.now_yr);
    const shipped = await deriveScene(s, scene, doc);
    const hash0 = s.stack().hash;
    const state0 = await s.stateHash();

    const p = await previewStack(s, shipped, [warm]);
    assert.ok(p.ok, p.problems.join("; "));
    assert.equal(s.stack().hash, hash0, "a preview leaves the stack in force");
    assert.equal(await s.stateHash(), state0, "a preview runs on the scratch kernel and chronicles nothing");
    const tarn = p.changes.filter((c) => c.address === TARN);
    assert.ok(tarn.some((c) => c.field === "surface_k" && c.fictional), JSON.stringify(tarn));
    assert.ok(!p.changes.some((c) => c.label.startsWith("Luminosity")), "the star's flow is untouched");

    assert.equal((await stackPacks(s, ws, [warm])).status, "applied");
    assert.deepEqual(s.stack().added.map((x) => x.id), ["sample.warm-worlds"]);
    assert.notEqual(await s.stateHash(), state0, "stacking a pack is chronicled");
    const now = await deriveScene(s, scene, doc);
    const t = now.subjects.find((x) => x.address === TARN)!;
    assert.ok(t.stamps.includes("fictional"));
    assert.equal(t.facts.find((f) => f.field === "surface_k")!.display, tarn.find((c) => c.field === "surface_k")!.after, "the preview showed what stacking did");

    // A campaign physics change written against the base flow now conflicts: the kernel says so.
    draftParam(s, ws, "sse.base.planet-climate", "teq", "redistribution", 0.5);
    const r = await adoptRequest(s, ws, "gm-draft.sse.base.planet-climate");
    assert.equal(r.status, "applied", "drafts are written against the flow in force, so they adopt over the added pack");

    // Removing the added pack would leave the campaign pack replacing a flow from the wrong pack: refused, nothing changes.
    const back = await previewStack(s, now, []);
    assert.equal(back.ok, false);
    assert.match(back.problems.join(" "), /replaces/);
    assert.equal((await stackPacks(s, ws, [])).status, "refused");
    assert.deepEqual(s.stack().added.map((x) => x.id), ["sample.warm-worlds"]);
  } finally {
    await k.close();
  }
});

test("players cannot stack packs", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, { ...tableSeat(), may_send: ["stack_packs"] }, scene.universe, [base]);
    assert.equal((await s.send({ kind: "stack_packs", packs: [warm as never] })).status, "pending");
    assert.equal(s.stack().added.length, 0);
  } finally {
    await k.close();
  }
});

// ND6: before a pack is added, its own sources, fudges and reproduction checks are shown, as the kernel worked them out.
test("a paper's pack shows its sources, fudges and reproduction before it is added", async () => {
  const paper: PackDoc = JSON.parse(readFileSync(new URL("./fixtures/paper.pack.json", import.meta.url), "utf8"));
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base], k.scratch);
    const shipped = await deriveScene(s, scene, doc);
    const p = await previewStack(s, shipped, [paper], { report: paper.id });
    assert.ok(p.ok, p.problems.join("; "));
    const r = p.report!;
    assert.equal(r.pack, "test.paper");
    assert.equal(r.reproduced, true);
    assert.deepEqual(r.fudges.map((f) => f.from), ["declared", "model_number"]);
    assert.equal(r.fudges[1]!.number?.grade, "fitted");
    const row = r.reproduction[0]!.rows[0]!;
    assert.equal(row.published, 9.8);
    assert.ok(row.pass && row.off! < 0.01, JSON.stringify(row));
    assert.equal(reportWords(r), "2 sources, 2 fudges, 1 of 1 check reproduced");
    assert.equal(sourceLink(r.sources[1]!), "https://doi.org/10.0000/example");
    assert.equal(sourceLink({ doi: "https://doi.org/10.1/x" }), "https://doi.org/10.1/x");
    assert.equal(percentWords(0.0012), "0.12 %");
    assert.deepEqual(["m_s2", "w_m2", "km_s", "1", "kg"].map(unitWords), ["m/s²", "W/m²", "km/s", "", "kg"]);
    assert.equal(s.stack().added.length, 0, "showing the report adds nothing");
    // Asked for nothing, the preview carries no report; outside its stated closeness the pack is refused, both numbers shown.
    assert.equal((await previewStack(s, shipped, [paper])).report, undefined);
    const tight = structuredClone(paper) as PackDoc & { reproduction: { expect: Record<string, { tolerance: number }> }[] };
    tight.reproduction[0]!.expect["body.surface_gravity"]!.tolerance = 0.0001;
    const no = await previewStack(s, shipped, [tight], { report: paper.id });
    assert.equal(no.ok, false);
    assert.match(no.problems.join(" "), /9\.8.*allows/);
  } finally {
    await k.close();
  }
});
