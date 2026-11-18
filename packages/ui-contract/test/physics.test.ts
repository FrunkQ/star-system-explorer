// U3 acceptance: open a real flow, change a parameter, see the diff and the
// badge. Physics changes are documents (a campaign pack with an explicit
// `replaces`), validated by the kernel before they can be adopted, and
// adopted by one intent through the same session every seat uses.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { deriveScene, type DerivationDoc } from "../src/derivation.ts";
import { campaignPack, diffFlow, editFlow, flowColumns, packFor, providerOf, CAMPAIGN_PACK, type PackDoc } from "../src/physics.ts";
import { adoptRequest, benchRef, declineRequest, draftId, draftParam, previewRequest, withdrawRequest } from "../src/physics-edit.ts";
import type { SystemScene } from "../src/scene.ts";
import { nodeKernel } from "./node-kernel.ts";
import { plainWorkspace } from "./workspace.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const pack: PackDoc = json("packs/base-physics/pack.json");
const scene: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");
const doc: DerivationDoc = json("packages/shell/src/documents/derivations.base.json");
const CLIMATE = "sse.base.planet-climate";
const TARN = "u:kestrel/sys:1/body:3";

test("flows draw left to right by depth", () => {
  const f = providerOf([pack], CLIMATE)!.doc;
  assert.deepEqual(flowColumns(f), [["sun"], ["teq"], ["surf"]]);
  const star = providerOf([pack], "sse.base.star-main-sequence")!.doc;
  assert.equal(flowColumns(star).flat().length, star.nodes.length);
});

test("a campaign pack replaces the base flow explicitly and diffs cleanly", () => {
  const f = providerOf([pack], CLIMATE)!.doc;
  const g = editFlow(f, [{ node: "teq", param: "redistribution", before: 0.25, after: 0.5 }]);
  assert.equal(f.nodes[1]!.params!.redistribution, 0.25, "the base flow is never touched");
  assert.deepEqual(diffFlow(f, g), { params: [{ node: "teq", param: "redistribution", before: 0.25, after: 0.5 }], structure: [] });
  const c = campaignPack([pack], [g])!;
  assert.equal(c.id, CAMPAIGN_PACK);
  assert.deepEqual(c.replaces, [{ flow: CLIMATE, pack: "sse.base-physics", range: "^0.1.0" }]);
  assert.equal(packFor([pack], []), null, "no requests: the base physics as shipped");
  assert.throws(() => editFlow(f, [{ node: "nope", param: "x", before: 0, after: 1 }]));
});

test("open a real flow, change a parameter, see diff and badge", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [pack], k.scratch);
    const ws = plainWorkspace(scene.now_yr);
    const shipped = await deriveScene(s, scene, doc);
    const tarn0 = shipped.subjects.find((x) => x.address === TARN)!;
    const surf0 = tarn0.facts.find((f) => f.field === "surface_k")!;
    assert.equal(surf0.changed, false);
    assert.ok(!tarn0.stamps.includes("changed-physics"));
    const hash0 = await s.stateHash();
    const stack0 = s.stack().hash;

    // Open the flow from the value's why, with Tarn's real inputs.
    ws.openFlow(CLIMATE, { node: "teq", ref: { flow: CLIMATE, inputs: surf0.why!.inputs }, label: "Tarn" });
    assert.equal(ws.screen, "physics");
    const bench = benchRef(shipped, ws, CLIMATE)!;
    assert.equal(s.flowDoc(CLIMATE)!.pack, "sse.base-physics");

    // Change a parameter: a draft request with a diff; nothing changes yet.
    const draft = draftParam(s, ws, CLIMATE, "teq", "redistribution", 0.5)!;
    assert.equal(draft.id, draftId(CLIMATE));
    assert.deepEqual(draft.edits, [{ node: "teq", param: "redistribution", before: 0.25, after: 0.5 }]);
    assert.equal(s.stack().campaign, null);
    const p = await previewRequest(s, ws, draft, bench.ref);
    assert.ok(p.ok, p.problems.join("; "));
    const teq = p.values.find((v) => v.kind === "planet.equilibrium_temperature")!;
    assert.ok(teq.after!.value > teq.before!.value * 1.18 && teq.after!.value < teq.before!.value * 1.19, "f 1/4 → 1/2 raises T_eq by 2^¼");
    assert.equal(teq.after!.publisher.split("/")[0], CAMPAIGN_PACK);
    assert.equal((await s.derive(bench.ref)).ok && s.stack().hash, stack0, "a preview restores the stack in force");

    // Out of the atom's range: the kernel rejects it, so it can never be adopted.
    const bad = draftParam(s, ws, CLIMATE, "teq", "redistribution", 7)!;
    const pb = await previewRequest(s, ws, bad, bench.ref);
    assert.equal(pb.ok, false);
    assert.match(pb.problems.join(" "), /redistribution/);
    assert.equal((await adoptRequest(s, ws, bad.id)).status, "refused");
    assert.equal(s.stack().campaign, null);

    // Adopt: one intent, chronicled as one stack.adopt event; values are badged.
    draftParam(s, ws, CLIMATE, "teq", "redistribution", 0.5);
    const r = await adoptRequest(s, ws, draftId(CLIMATE));
    assert.equal(r.status, "applied");
    assert.equal(ws.requests.filter((x) => x.status === "adopted").length, 1);
    assert.equal(ws.requests.find((x) => x.id === draftId(CLIMATE)), undefined, "the next edit starts a new draft");
    assert.notEqual(s.stack().hash, stack0);
    assert.equal(s.flowDoc(CLIMATE)!.pack, CAMPAIGN_PACK);
    assert.notEqual(await s.stateHash(), hash0, "adopting physics is chronicled");
    assert.equal(r.status === "applied" && (await s.history(s.universe.root)).some((e) => e.id === r.event_id && e.kind === "stack.adopt"), true);
    const changed = await deriveScene(s, scene, doc);
    const tarn1 = changed.subjects.find((x) => x.address === TARN)!;
    const surf1 = tarn1.facts.find((f) => f.field === "surface_k")!;
    assert.equal(surf1.changed, true);
    assert.ok(tarn1.stamps.includes("changed-physics"));
    assert.ok(Number(surf1.value) > Number(surf0.value));
    const star = changed.subjects.find((x) => x.kind === "star")!;
    assert.ok(!star.stamps.includes("changed-physics"), "the star's flow is untouched");
    const e = await s.explain(surf1.why!, "planet.equilibrium_temperature");
    assert.ok(e.ok && e.explanation.params.redistribution === 0.5);

    // Withdraw: back to the base physics as shipped, same values as before.
    const adoptedId = ws.requests.find((x) => x.status === "adopted")!.id;
    assert.equal((await withdrawRequest(s, ws, adoptedId)).status, "applied");
    assert.equal(s.stack().campaign, null);
    assert.equal(s.stack().hash, stack0);
    const back = await deriveScene(s, scene, doc);
    assert.equal(back.subjects.find((x) => x.address === TARN)!.facts.find((f) => f.field === "surface_k")!.display, surf0.display);

    declineRequest(ws, adoptedId);
    assert.equal(ws.requests.find((x) => x.id === adoptedId)!.status, "declined");
  } finally {
    await k.close();
  }
});

test("players cannot change the physics: their request waits for the GM", async () => {
  const k = nodeKernel();
  try {
    const seat = { ...tableSeat(), may_send: ["adopt_physics" as const], auto_approve: [{ intent: "adopt_physics" as const, scope: ["u:kestrel"] }] };
    const s = await Session.open(k.client, seat, scene.universe, [pack], k.scratch);
    const c = packFor([pack], [{ flow: CLIMATE, edits: [{ node: "teq", param: "redistribution", before: 0.25, after: 0.5 }] }]);
    const r = await s.send({ kind: "adopt_physics", pack: c as never, requests: ["x"] });
    assert.equal(r.status, "pending");
    assert.equal(s.stack().campaign, null);
    assert.equal((await Session.open(k.client, tableSeat(), scene.universe, [pack], k.scratch).then((t) => t.send({ kind: "adopt_physics", pack: null, requests: [] }))).status, "refused");
  } finally {
    await k.close();
  }
});

test("Orac's sample requests are valid and say they are samples", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [pack], k.scratch);
    const ws = plainWorkspace(scene.now_yr);
    const { requests } = json("packages/shell/src/documents/orac.requests.json");
    for (const r of requests) {
      assert.equal(r.fixture, true);
      ws.fileRequest(r);
      const p = await previewRequest(s, ws, r, null);
      assert.ok(p.ok, `${r.id}: ${p.problems.join("; ")}`);
    }
  } finally {
    await k.close();
  }
});
