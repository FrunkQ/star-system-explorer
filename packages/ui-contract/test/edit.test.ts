// Undo and redo are intents too: each is a new authored event, and undoing
// returns the state hash to what it was before the edit's value.
import { test } from "node:test";
import assert from "node:assert/strict";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { commitEdit, redoEdit, undoEdit } from "../src/edit.ts";
import { plainWorkspace } from "./workspace.ts";
import { nodeKernel } from "./node-kernel.ts";

test("edit, undo, redo: each an event; values follow", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" });
    const ws = plainWorkspace();
    const a = "u:kestrel/sys:1/body:3";
    const r = await commitEdit(s, ws, { address: a, field: "a_au", value: 0.9, display: "0.9 AU", before: { value: 0.63, display: "0.63 AU" } });
    assert.equal(r.status, "applied");
    assert.equal(ws.undo.length, 1);
    await undoEdit(s, ws);
    assert.equal(ws.applied[`${a}#a_au`]!.value, 0.63);
    assert.deepEqual([ws.undo.length, ws.redo.length], [0, 1]);
    await redoEdit(s, ws);
    assert.equal(ws.applied[`${a}#a_au`]!.value, 0.9);
    assert.deepEqual([ws.undo.length, ws.redo.length], [1, 0]);
    assert.equal((await s.history(a)).length, 3, "edit, undo and redo are three inked events");
    assert.equal(await undoEdit(s, plainWorkspace()), null, "nothing to undo");
  } finally {
    await k.close();
  }
});

test("a value read back from the universe is typed in its own units and sent to its kernel field in SI", async () => {
  const { factEdit } = await import("../src/edit.ts");
  const fixture = { field: "mass_earth", label: "Mass", value: 0.82, unit: "M⊕", display: "0.82 M⊕", editable: true, source: "authored" as const };
  assert.deepEqual(factEdit(fixture, 0.9), { field: "mass_earth", value: 0.9, before: 0.82 }, "the fixture's own field, as typed");
  const read = { ...fixture, kernel: { field: "body.mass", per_unit: 2 } };
  assert.deepEqual(factEdit(read, 0.9), { field: "body.mass", value: 1.8, before: 1.64 }, "the universe's field, scaled to SI");
  assert.deepEqual(factEdit({ ...read, value: "Tarn", display: "Tarn" }, "Mere"), { field: "body.mass", value: "Mere", before: "Tarn" }, "text is not scaled");
});
