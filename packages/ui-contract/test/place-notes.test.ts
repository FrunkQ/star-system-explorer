// A description and GM notes for the campaign and each system (V3 gap 42), on the real kernel's record.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { campaignAddress, placeNoteEdit, placeNoteOf } from "../src/place-notes.ts";
import { nodeKernel } from "./node-kernel.ts";

const base = JSON.parse(readFileSync(new URL("../../../packs/base-physics/pack.json", import.meta.url), "utf8"));

test("the campaign's notes live at the universe's root", () => {
  assert.equal(campaignAddress("u:kestrel/sys:1/body:3"), "u:kestrel");
  assert.equal(campaignAddress("u:kestrel"), "u:kestrel");
});

test("an edit writes only a change; clearing withdraws; an empty note with nothing to withdraw writes nothing", () => {
  const cur = { id: "n1", text: "Old", at_s: 0 };
  assert.equal(placeNoteEdit("u:k/sys:1", "description", cur, "Old  "), null);
  assert.deepEqual(placeNoteEdit("u:k/sys:1", "description", cur, "New"), { address: "u:k/sys:1", text: "New", audience: "everyone", tags: ["description"], supersedes: "n1" });
  assert.deepEqual(placeNoteEdit("u:k/sys:1", "gm-notes", null, "Pirates at Tarn")!.audience, "gm");
  assert.equal(placeNoteEdit("u:k/sys:1", "gm-notes", cur, " ")!.text, "");
  assert.equal(placeNoteEdit("u:k/sys:1", "gm-notes", null, "  "), null);
});

test("on the kernel: write, change and clear a system's description and the campaign's GM notes; the state does not move", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [base]);
    const h0 = await s.stateHash();
    const sys = "u:kestrel/sys:1";
    const read = async (a: string) => {
      const r = await s.record!(a);
      assert.ok(r.ok, JSON.stringify(r));
      return r.record.notes;
    };
    const write = async (a: string, kind: "description" | "gm-notes", text: string) => {
      const e = placeNoteEdit(a, kind, placeNoteOf(await read(a), kind), text);
      if (!e) return;
      const r = await s.addNote!(e);
      assert.ok(r.ok, JSON.stringify(r));
    };
    await write(sys, "description", "A quiet K star with an ocean world.");
    await write(sys, "description", "A quiet K star with an ocean world and a ruined moon.");
    await write(sys, "gm-notes", "The moon's ruins are a trap.");
    let notes = await read(sys);
    assert.equal(placeNoteOf(notes, "description")!.text, "A quiet K star with an ocean world and a ruined moon.");
    assert.equal(placeNoteOf(notes, "gm-notes")!.text, "The moon's ruins are a trap.");
    assert.equal(notes.filter((n) => n.tags?.includes("description")).length, 1, "the change superseded the first");
    await write(sys, "gm-notes", "");
    notes = await read(sys);
    assert.equal(placeNoteOf(notes, "gm-notes"), null, "cleared");
    const camp = campaignAddress(sys);
    await write(camp, "gm-notes", "Session 3: the party reaches Kestrel.");
    assert.equal(placeNoteOf(await read(camp), "gm-notes")!.text, "Session 3: the party reaches Kestrel.");
    assert.equal(await s.stateHash(), h0, "notes never change the universe's state");
  } finally {
    await k.close();
  }
});
