// U4 acceptance in Node: what-if branches. Edits on a branch leave the main
// line's state hash alone; the branch has its own; throwing it away leaves
// the main line exactly as it was; keeping it inks the edits on the main line
// as the GM's own intents (so they replicate and can be undone); comparing
// lists what was set and what follows from it. Players cannot open one.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { commitEdit, compareScenes, dropWhatIf, keepWhatIf, openWhatIf, undoEdit } from "../src/edit.ts";
import { applyEdits, deriveScene, type DerivationDoc } from "../src/derivation.ts";
import type { PackDoc } from "../src/physics.ts";
import type { SystemScene } from "../src/scene.ts";
import { plainWorkspace } from "./workspace.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const TARN = "u:kestrel/sys:1/body:3";
const mass = (v: number) => ({ address: TARN, field: "mass_earth", value: v, display: `${v} × Earth`, before: { value: 0.82, display: "0.82 × Earth" } });

test("a what-if moves only its branch; throw it away and nothing happened", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const ws = plainWorkspace();
    const h0 = await s.stateHash();
    assert.equal(await openWhatIf(s, ws, "Heavier Tarn"), null);
    assert.match((await openWhatIf(s, ws, "again"))!, /already open/);
    const r = await commitEdit(s, ws, mass(1.4));
    assert.equal(r.status, "applied");
    assert.equal(await s.stateHash(), h0, "the main line does not move");
    assert.equal(ws.undo.length, 0, "branch edits are not on the undo stack");
    const hb = await s.branchStateHash(ws.branch!.id);
    assert.notEqual(hb, h0, "the branch has its own state");
    assert.equal(await undoEdit(s, ws), null, "undo is for the main line");
    await dropWhatIf(s, ws);
    assert.equal(ws.branch, null);
    assert.equal(await s.stateHash(), h0, "thrown away: the main line is exactly as it was");
    assert.equal((await s.history(TARN)).length, 0);
  } finally {
    await k.close();
  }
});

test("keeping a what-if inks its edits on the main line, in order, undoable", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const ws = plainWorkspace();
    await openWhatIf(s, ws, "Heavier Tarn");
    await commitEdit(s, ws, mass(1.4));
    await commitEdit(s, ws, { ...mass(1.6), before: { value: 1.4, display: "1.4 × Earth" } });
    const hb = await s.branchStateHash(ws.branch!.id);
    assert.equal(await keepWhatIf(s, ws), null);
    assert.equal(ws.branch, null);
    assert.equal(ws.applied[`${TARN}#mass_earth`]!.value, 1.6);
    assert.equal((await s.history(TARN)).length, 2, "two edits, inked on the main line");
    assert.equal(ws.undo.length, 2);
    // The same edits sent straight to the main line give the same state as the kept branch.
    const k2 = nodeKernel();
    try {
      const s2 = await Session.open(k2.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
      const w2 = plainWorkspace();
      await commitEdit(s2, w2, mass(1.4));
      await commitEdit(s2, w2, mass(1.6));
      assert.equal(await s.stateHash(), await s2.stateHash(), "kept equals edited directly: every replica agrees");
    } finally {
      await k2.close();
    }
    assert.ok(hb.length > 0);
  } finally {
    await k.close();
  }
});

test("compare lists what was set and what follows from it", async () => {
  const k = nodeKernel();
  try {
    const base = json("packs/base-physics/pack.json") as PackDoc;
    const scene = json("packages/shell/src/documents/kestrel.scene.json") as SystemScene;
    const der = json("packages/shell/src/documents/derivations.base.json") as DerivationDoc;
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base]);
    const main = await deriveScene(s, scene, der);
    const set = { [`${TARN}#mass_earth`]: { value: 1.6, display: "1.6 × Earth", event_id: "x" } };
    const what = await deriveScene(s, applyEdits(scene, set), der);
    const d = compareScenes(main, what, set);
    const tarn = d.filter((x) => x.address === TARN);
    assert.ok(tarn.some((x) => x.field === "mass_earth" && x.how === "set"));
    assert.ok(tarn.some((x) => x.field === "gravity_g" && x.how === "follows"), JSON.stringify(tarn));
    assert.ok(d.every((x) => x.address === TARN), "nothing else changed");
  } finally {
    await k.close();
  }
});

test("players cannot open or edit a what-if", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const r = await s.openBranch("sneaky");
    assert.equal(r.ok, false);
    const e = await s.branchSend("x", { kind: "set_value", address: TARN, field: "mass_earth", value: 2, at_yr: 0 });
    assert.equal(e.status, "refused");
  } finally {
    await k.close();
  }
});
