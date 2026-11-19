// U10 in Node: a campaign kept and reopened. The GM's universe (facts
// entered, an edit, the sample Sol adopted) is exported, written out as
// text, and imported into a fresh kernel: the same state hash and the same
// systems. A player's screen joining the reopened campaign imports the same
// export from the manifest and agrees by state hash. Saves that belong to
// another universe, or are not saves, are refused before anything changes.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { enterFacts, type AuthoredFacts } from "../src/data-views.ts";
import { GmHost, PlayerReplica, memoryRoom, type Manifest } from "../src/p2p.ts";
import { checkCampaignSave, makeCampaignSave, publicExport, saveFileName, savedEvents } from "../src/campaign.ts";
import type { Json } from "../src/json.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const json = (p: string) => JSON.parse(text(p));
const base: PackDoc = json("packs/base-physics/pack.json");
const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const universe = { id: "kestrel", seed: "kestrel reach campaign" };
const TARN = "u:kestrel/sys:1/body:3";
async function until(f: () => boolean, what: string) {
  for (let i = 0; i < 600; i++) {
    if (f()) return;
    await new Promise((r) => setTimeout(r, 10));
  }
  assert.fail(`timed out waiting for ${what}`);
}

test("a campaign exported as text reopens in a fresh kernel with the same state hash, and a player joins it", async () => {
  const ks = [nodeKernel(), nodeKernel(), nodeKernel()];
  try {
    const gm = await Session.open(ks[0]!.client, gmSeat(), universe, [base, playerData], ks[0]!.scratch);
    assert.equal(await enterFacts(gm, facts), null);
    const edit = await gm.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.6, at_yr: 0 });
    assert.equal(edit.status, "applied");
    assert.ok((await gm.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") })).ok);
    const h0 = await gm.stateHash();
    const ex = await gm.exportChronicle();
    assert.ok(ex.ok, JSON.stringify(ex).slice(0, 200));
    const save = makeCampaignSave({ title: "Kestrel Reach", root: "u:kestrel", state_hash: h0, export: ex.export, workspace: { applied: {}, undo: [], redo: [], now_yr: 4.6e9 }, saved_at: "2026-11-19T12:00:00.000Z" });
    assert.deepEqual(checkCampaignSave(save, "u:kestrel"), []);
    assert.ok(savedEvents(save) > 10);
    const kept = JSON.parse(JSON.stringify(save)) as typeof save;

    // Reopen in a fresh kernel.
    const again = await Session.open(ks[1]!.client, gmSeat(), universe, [base, playerData], ks[1]!.scratch);
    const im = await again.importChronicle(kept.export);
    assert.ok(im.ok, JSON.stringify(im));
    assert.equal(im.stack_matches, true);
    assert.equal(im.state_hash, h0);
    assert.equal(await again.stateHash(), h0);
    const listed = await again.listSystems();
    assert.ok(listed.ok && listed.systems.some((x) => x.name === "Sol"), "the adopted system is still in the campaign");
    // The reopened campaign keeps working: a further edit applies.
    assert.equal((await again.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.55, at_yr: 0 })).status, "applied");

    // A player joining the reopened campaign starts from the same export.
    const room = memoryRoom();
    const version = await again.kernelVersion();
    const host = new GmHost(again, room.hub("gm"), { run: "r", kernel: version, app: "a", universe, packs: [base, playerData], opening_export: publicExport(kept.export), opening_hash: im.state_hash, clock_yr: 0 });
    assert.equal(await again.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.58, at_yr: 0 }).then((r) => r.status), "applied");
    const open = async (m: Manifest) => Session.open(ks[2]!.client, tableSeat(), m.universe, m.packs, ks[2]!.scratch);
    const p = new PlayerReplica(room.hub("p"), open, { kernel: version, app: "a", now: () => 0 });
    p.join();
    await until(() => p.status.state === "live" || p.status.state === "reload", "the player's screen settles");
    assert.equal(p.status.state, "live", JSON.stringify(p.status));
    assert.equal((p.status as { state_hash: string }).state_hash, host.state().state_hash, "the player agrees by state hash");
    host.close?.();
  } finally {
    for (const k of ks) k.close();
  }
});

test("saves that are not saves, or belong to another universe, are refused before anything changes", async () => {
  assert.ok(checkCampaignSave(null).length);
  assert.ok(checkCampaignSave({ schema: "sse.something/1" }).length);
  const ok = makeCampaignSave({ title: "T", root: "u:kestrel", state_hash: "h", export: { root: "u:kestrel", events: [] } as Json, workspace: { applied: {}, undo: [], redo: [], now_yr: 0 }, saved_at: "2026-11-19T00:00:00Z" });
  assert.deepEqual(checkCampaignSave(ok, "u:kestrel"), []);
  assert.match(checkCampaignSave(ok, "u:other").join(), /belongs to u:kestrel, not this campaign/);
  assert.match(checkCampaignSave({ ...ok, export: { root: "u:elsewhere", events: [] } }).join(), /another universe/);
  // The campaign record never travels on the players' manifest.
  const withRecord = { root: "u:kestrel", events: [], record: [{ note: "the players peed on an alien statue" }], record_head: "abc" } as unknown as Json;
  assert.deepEqual(publicExport(withRecord), { root: "u:kestrel", events: [] });
  assert.equal(saveFileName("Kestrel Reach", "2026-11-19T12:00:00Z"), "sse-campaign-kestrel-reach-2026-11-19.json");

  // A damaged export is refused by the kernel and the open universe is untouched.
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), universe, [base], k.scratch);
    const h = await s.stateHash();
    const ex = await s.exportChronicle();
    assert.ok(ex.ok);
    const bad = JSON.parse(JSON.stringify(ex.export)) as { head: string };
    bad.head = "0".repeat(bad.head.length);
    const r = await s.importChronicle(bad as unknown as Json);
    assert.ok(!r.ok && r.code === "bad_export", JSON.stringify(r));
    assert.equal(await s.stateHash(), h);
    // Players' seats cannot save the campaign.
    const t = await Session.open(k.client, tableSeat(), universe, [base], k.scratch).catch(() => null);
    if (t) assert.equal((await t.exportChronicle()).ok, false);
  } finally {
    k.close();
  }
});
