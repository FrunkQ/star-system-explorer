// U20 part 2 in Node: a paste goes into the universe as one change (the kernel's set_authored_batch), reads back as
// the bodies it copied, and one Undo withdraws them all; Redo brings them back with every fact. A cut is Copy, then
// a withdraw, undone the same way. Malformed batches are refused before they reach the kernel.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, sharedTableSeat, decide } from "../src/seat.ts";
import { checkIntent } from "../src/intent.ts";
import { copyClip, placeClip } from "../src/clip.ts";
import { placeBodies, withdrawBodies, undoEdit, redoEdit, PRESENCE } from "../src/edit.ts";
import { sceneFromResolved } from "../src/make.ts";
import type { SystemScene } from "../src/scene.ts";
import { nodeKernel } from "./node-kernel.ts";
import { plainWorkspace } from "./workspace.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
const scene = JSON.parse(text("packages/shell/src/documents/kestrel.scene.json")) as SystemScene;
const SYS = "u:kestrel/sys:1";
const STAR = `${SYS}/body:0`;
const TARN = `${SYS}/body:3`;

test("a paste is one change: read back as copied, one Undo withdraws it all, Redo brings it back", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const ws = plainWorkspace();
    const h0 = await s.stateHash();
    const c = copyClip(scene, TARN)!;
    const p = placeClip(c.clip, { system: SYS, host: STAR, hostKind: "star", taken: scene.subjects.map((x) => x.address) });
    const r = await placeBodies(s, ws, { edits: p.edits, roots: [p.placed[0]!.address], words: c.words });
    assert.equal(r.status, "applied", JSON.stringify(r));
    assert.equal(ws.undo.length, 1, "one entry on the undo stack");
    assert.equal(ws.undo[0]!.field, PRESENCE);
    const read = await s.resolvedSystem(SYS);
    assert.ok(read.ok);
    const subs = sceneFromResolved(read.result).subjects;
    assert.deepEqual(subs.map((x) => [x.address, x.kind, x.label]), [[`${SYS}/body:6`, "planet", "Tarn"], [`${SYS}/body:6/body:0`, "moon", "Tarn a"]]);
    assert.equal(read.result.facts[`${SYS}/body:6`]!["orbit.host"], STAR, "written as orbit.host, which the kernel's spec reads");
    const h1 = await s.stateHash();

    assert.equal((await undoEdit(s, ws))!.status, "applied");
    assert.equal(ws.undo.length, 0);
    assert.equal(ws.redo.length, 1);
    assert.ok(!(await s.resolvedSystem(SYS)).ok, "Undo withdrew the planet and its moon");
    assert.notEqual(await s.stateHash(), h0, "the chronicle only grows");

    assert.equal((await redoEdit(s, ws))!.status, "applied");
    const back = await s.resolvedSystem(SYS);
    assert.ok(back.ok && Object.keys(back.result.facts).length === 2, "Redo brought both back");
    assert.notEqual(await s.stateHash(), h1);

    // Cut: withdrawn as one change, and Undo puts it back.
    assert.equal((await withdrawBodies(s, ws, { roots: [`${SYS}/body:6`], words: c.words })).status, "applied");
    assert.ok(!(await s.resolvedSystem(SYS)).ok);
    await undoEdit(s, ws);
    assert.ok((await s.resolvedSystem(SYS)).ok);
  } finally {
    await k.close();
  }
});

test("impossible values go in their own batch, still one Undo", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const ws = plainWorkspace();
    const a = `${SYS}/body:6`;
    const r = await placeBodies(s, ws, {
      edits: [
        { address: a, field: "body.kind", value: "planet" },
        { address: a, field: "orbit.host", value: STAR },
        { address: a, field: "orbit.semi_major_axis", value: 1.5e11 },
        { address: a, field: "body.mass", value: 6e24, impossible: true },
      ],
      roots: [a],
      words: "a world",
    });
    assert.equal(r.status, "applied", JSON.stringify(r));
    assert.equal(ws.undo.length, 1);
    const read = await s.resolvedSystem(SYS);
    assert.ok(read.ok);
    assert.equal(read.result.evidence?.[a]?.["body.mass"]?.evidence, "authored-impossible");
    await undoEdit(s, ws);
    assert.ok(!(await s.resolvedSystem(SYS)).ok);
  } finally {
    await k.close();
  }
});

test("batches are checked before the kernel, and only the GM places or withdraws", () => {
  const e = { address: TARN, field: "body.mass", value: 1 };
  assert.match(checkIntent({ kind: "set_values", edits: [], at_yr: 0 })!, /at least one/);
  assert.match(checkIntent({ kind: "set_values", edits: [e, { ...e, impossible: true }], at_yr: 0 })!, /separate/);
  assert.match(checkIntent({ kind: "set_values", edits: [e], at_yr: 0, evidence: "catalogued" })!, /source/);
  assert.equal(checkIntent({ kind: "set_values", edits: [e], at_yr: 0, evidence: "catalogued", source: "doi:10.1/x" }), null);
  assert.match(checkIntent({ kind: "withdraw", address: "tarn", at_yr: 0 })!, /u:/);
  assert.equal(decide(gmSeat(), { kind: "withdraw", address: TARN, at_yr: 0 }).action, "apply");
  assert.equal(decide(sharedTableSeat(), { kind: "set_values", edits: [e], at_yr: 0 }).action, "refuse");
});
