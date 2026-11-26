// TM4 (three desks design, question 3): on Explore every change is a try. The first change opens a what-if by
// itself, the change goes to it and the main line does not move; Keep it inks it on the main line without opening a
// fresh what-if; Throw away leaves the main line exactly as it was. Build writes to the campaign as before.
import { test } from "node:test";
import assert from "node:assert/strict";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { commitEdit, commitFields, dropWhatIf, keepWhatIf, TRY_LABEL } from "../src/edit.ts";
import { deskChanges, deskTries } from "../src/desk.ts";
import { plainWorkspace } from "./workspace.ts";
import { nodeKernel } from "./node-kernel.ts";

const TARN = "u:kestrel/sys:1/body:3";
const mass = (v: number) => ({ address: TARN, field: "mass_earth", value: v, display: `${v} × Earth`, before: { value: 0.82, display: "0.82 × Earth" } });

test("which desks try and which change at all", () => {
  assert.equal(deskTries("explore"), true);
  assert.equal(deskTries("build"), false);
  assert.equal(deskTries("run"), false);
  assert.equal(deskTries(undefined), false, "no desk (a player) tries nothing");
  assert.deepEqual([deskChanges("explore"), deskChanges("build"), deskChanges("run"), deskChanges(undefined)], [true, true, false, false]);
});

test("on Explore the first change opens a what-if by itself; the main line does not move", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const ws = plainWorkspace();
    ws.desk = "explore";
    const h0 = await s.stateHash();
    assert.equal((await commitEdit(s, ws, mass(1.4))).status, "applied");
    assert.equal(ws.branch?.label, TRY_LABEL, "the what-if opened by itself");
    const id = ws.branch!.id;
    assert.equal(await s.stateHash(), h0, "the main line does not move");
    assert.equal(ws.undo.length, 0);
    // A second change goes to the same what-if, not a new one; several fields at once too.
    assert.equal((await commitEdit(s, ws, { ...mass(1.6), before: { value: 1.4, display: "1.4 × Earth" } })).status, "applied");
    assert.equal((await commitFields(s, ws, { address: TARN, edits: [{ field: "mass_earth", value: 1.7, before: 1.6 }], display: "1.7 × Earth", before_display: "1.6 × Earth" })).status, "applied");
    assert.equal(ws.branch!.id, id, "one what-if for the whole try");
    assert.equal(ws.branch!.edits.length, 3);
    assert.equal(await s.stateHash(), h0);
    await dropWhatIf(s, ws);
    assert.equal(ws.branch, null);
    assert.equal(await s.stateHash(), h0, "thrown away: exactly as it was");
  } finally {
    await k.close();
  }
});

test("on Explore Keep it inks the try on the main line and opens no fresh what-if", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const ws = plainWorkspace();
    ws.desk = "explore";
    const h0 = await s.stateHash();
    await commitEdit(s, ws, mass(1.4));
    assert.equal(await keepWhatIf(s, ws), null);
    assert.equal(ws.branch, null, "kept: no what-if left open");
    assert.notEqual(await s.stateHash(), h0, "the main line moved");
    assert.equal(ws.applied[`${TARN}#mass_earth`]!.value, 1.4);
    assert.equal(ws.undo.length, 1, "one Undo");
  } finally {
    await k.close();
  }
});

test("on Build a change goes to the campaign, as before", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const ws = plainWorkspace();
    ws.desk = "build";
    const h0 = await s.stateHash();
    assert.equal((await commitEdit(s, ws, mass(1.4))).status, "applied");
    assert.equal(ws.branch, null);
    assert.notEqual(await s.stateHash(), h0);
  } finally {
    await k.close();
  }
});
