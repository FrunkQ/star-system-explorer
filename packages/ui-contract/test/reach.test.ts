// Reach it (ED5 part 1): the input that gives the result asked for, searched on a scratch branch the kernel resolves
// (BC1); nothing reaches the campaign.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { readSol, solBatches, solSourceText } from "../src/sol.ts";
import { reachIt, surfaceKelvin } from "../src/reach.ts";
import { constant } from "../src/sheet.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
const sheet = readSol(JSON.parse(text("packs/base-sol/pack.json")))!;
const AU = constant("au");

test("Reach it: move Mars until its surface is 260 K, on a scratch branch; the main line does not move", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sys = "u:kestrel/sys:sol";
    const { batches, addresses } = solBatches(sheet, sys);
    for (const b of batches) await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
    const mars = addresses.mars!;
    const h0 = await s.stateHash();
    const r = await reachIt(s, { system: sys, address: mars, field: "orbit.semi_major_axis", at_yr: 4.5673e9, lo: 0.3 * AU, hi: 3 * AU, log: true, target: 260, read: surfaceKelvin(mars), within: 2e-3 });
    assert.ok(r.ok, JSON.stringify(r));
    if (!r.ok) return;
    assert.ok(Math.abs(r.got - 260) < 0.6, `got ${r.got} K`);
    assert.ok(r.value > 1 * AU && r.value < 1.52 * AU, `Mars must move in to warm: ${r.value / AU} AU`);
    assert.ok(r.tries < 26, `${r.tries} tries`);
    assert.equal(await s.stateHash(), h0, "nothing reaches the campaign");
    const no = await reachIt(s, { system: sys, address: mars, field: "orbit.semi_major_axis", at_yr: 4.5673e9, lo: 0.3 * AU, hi: 3 * AU, log: true, target: 5000, read: surfaceKelvin(mars) });
    assert.ok(!no.ok && no.closest && no.closest.value === 0.3 * AU, "how close it can come: the nearest end");
  } finally {
    k.close?.();
  }
});
