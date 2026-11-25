// "Could be…" (ED10 part 1): what a world is not yet, and the smallest move that would make it so, worked out on a
// scratch what-if; nothing reaches the campaign.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { readSol, solBatches, solSourceText } from "../src/sol.ts";
import { COULD_BE, couldBeNow, recipeFor } from "../src/could-be.ts";
import { resultsOf } from "../src/handles.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");

test("the records offered are those the world does not pass", () => {
  const ids = couldBeNow({ "core.dynamo_now": false, locked: false }).map((c) => c.id);
  assert.deepEqual(ids, ["field", "locked"]);
  assert.deepEqual(couldBeNow({ "core.dynamo_now": true }).map((c) => c.id), []);
  assert.ok(COULD_BE.every((c) => c.by.length));
});

test("Mars could have a field of its own: a heavier Mars, the smallest move first", async () => {
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
    const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sys = "u:kestrel/sys:sol";
    const { batches, addresses } = solBatches(sheet, sys);
    for (const b of batches) await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
    const mars = addresses.mars!;
    const r = await s.resolvedSystem(sys);
    assert.ok(r.ok);
    if (!r.ok) return;
    const facts = r.result.facts[mars] as Record<string, never>;
    const offered = couldBeNow(resultsOf(r.result.surfaces, mars));
    assert.ok(offered.some((c) => c.id === "field"));
    const h0 = await s.stateHash();
    const steps = await recipeFor(s, offered.find((c) => c.id === "field")!, { system: sys, address: mars, facts, at_yr: 4.5673e9 });
    assert.ok(steps.length >= 1, "at least one way");
    assert.equal(steps[0]!.handle.id, "mass");
    assert.ok(steps[0]!.ratio > 1 && steps[0]!.ratio < 2, `a modest change: ${steps[0]!.ratio}`);
    assert.equal(await s.stateHash(), h0, "nothing written");
  } finally {
    await k.close();
  }
});
