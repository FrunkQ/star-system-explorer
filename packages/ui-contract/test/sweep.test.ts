// The sweep behind the strip (ED2): the kernel's K1 sweep of Mars's orbit, turned into the strip's marks; the main
// line does not move.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { readSol, solBatches, solSourceText } from "../src/sol.ts";
import { HANDLES } from "../src/handles.ts";
import { classNamesIn, sweepAsk, sweepMarks } from "../src/sweep.ts";
import { constant } from "../src/sheet.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;
const AU = constant("au");

test("the sweep behind Mars's orbit strip: where it stops lasting and where its day locks, nothing written", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sys = "u:kestrel/sys:sol";
    const { batches, addresses } = solBatches(sheet, sys);
    for (const b of batches) await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
    const mars = addresses.mars!;
    const orbit = HANDLES.find((h) => h.id === "orbit")!;
    const h0 = await s.stateHash();
    let progressed = 0;
    const r = await s.sweep(sweepAsk(orbit, { system: sys, address: mars }, 0.25 * AU, 8 * AU, 12), () => progressed++);
    assert.ok(r.ok, JSON.stringify(r));
    if (!r.ok) return;
    assert.equal(r.result.steps.length, 12);
    assert.ok(progressed > 0, "progress is told as it goes");
    const marks = sweepMarks(orbit, r.result);
    console.log(marks.map((m) => `${(m.at / AU).toFixed(3)} AU: ${m.says}`).join("; "));
    const lock = marks.find((m) => /turns freely/.test(m.says));
    assert.ok(lock && lock.at > 0.3 * AU && lock.at < 0.9 * AU, "Mars's day is locked inside about half an AU");
    assert.ok(marks.some((m) => /does not last|worth watching/.test(m.says)), "near Jupiter it does not last");
    assert.equal(await s.stateHash(), h0, "nothing reaches the campaign");
  } finally {
    k.close?.();
  }
});

test("the sweep behind a world's mass strip marks where its class changes, in the atlas's words", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sys = "u:kestrel/sys:sol";
    const { batches, addresses } = solBatches(sheet, sys);
    for (const b of batches) await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
    const mars = addresses.mars!;
    // The class regions read all five parts of the makeup (the Makeup box writes all five).
    const mk = { metal: 0.22, rock: 0.78, carbon: 0, ice: 0, gas: 0 };
    await s.send({ kind: "set_values", edits: Object.entries(mk).map(([p, v]) => ({ address: mars, field: `makeup.${p}`, value: v })), at_yr: 0 });
    const atlas = JSON.parse(text("packs/base-atlas/pack.json"));
    const mass = HANDLES.find((h) => h.id === "mass")!;
    const h0 = await s.stateHash();
    const r = await s.sweep(sweepAsk(mass, { system: sys, address: mars }, 1e22, 1e27, 12, { packs: [atlas], scheme: "physical" }));
    assert.ok(r.ok, JSON.stringify(r));
    if (!r.ok) return;
    assert.equal(r.result.steps[0]!.class, "rocky");
    const marks = sweepMarks(mass, r.result, undefined, classNamesIn([atlas], "physical"));
    console.log(marks.map((m) => `${(m.at / 5.972e24).toPrecision(3)} Earth masses: ${m.says}`).join("; "));
    const past = marks.find((m) => /^no longer rocky/.test(m.says));
    assert.ok(past && past.at > 8 * 5.972e24 && past.at < 16 * 5.972e24, "past the rocky class's mass, about ten Earths");
    assert.equal(await s.stateHash(), h0, "nothing reaches the campaign");
  } finally {
    k.close?.();
  }
});
