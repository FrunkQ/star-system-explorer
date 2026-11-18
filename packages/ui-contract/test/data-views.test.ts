// U5 data views (PlayerData board) against the live kernel: the campaign's
// recorded facts, scenes per audience, the Scanned button as one intent, and
// an edited disclosure map applied as the campaign's own player-data pack.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { applySetup, checkSetup, enterFacts, observeOf, setupChanges, PLAYER_DATA_PACK, type AuthoredFacts, type ObserveSetup } from "../src/data-views.ts";
import type { PackDoc } from "../src/physics.ts";
import type { PlayerScene } from "../src/player-scene.ts";
import { nodeKernel } from "./node-kernel.ts";
import { plainWorkspace } from "./workspace.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const universe = { id: "kestrel", seed: "kestrel reach campaign" };
const TARN = "u:kestrel/sys:1/body:3";
const fact = (s: PlayerScene, a: string, f: string) => s.subjects.find((x) => x.address === a)?.facts.find((x) => x.field === f);

test("scenes per audience, the Scanned button, and an applied disclosure change", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), universe, [base, playerData], k.scratch);
    assert.equal(await enterFacts(s, facts), null);
    const before = await s.scene(facts.scope, { audience: "everyone" });
    assert.ok(before.ok, !before.ok ? before.message : "");
    assert.equal(fact(before.scene, TARN, "mass_kg")?.placeholder, "not yet scanned");
    assert.equal(fact(before.scene, TARN, "gm.notes"), undefined, "secrets never reach a player scene");
    const gm = await s.scene(facts.scope);
    assert.ok(gm.ok && fact(gm.scene, TARN, "gm.notes")?.value === "Pirate cache under the north pole.");

    // The Scanned button: one chronicled intent; a second tap undoes it.
    const r = await s.send({ kind: "scan_toggle", address: TARN, audience: "everyone", at_yr: 0 });
    assert.equal(r.status, "applied");
    assert.equal(r.status === "applied" && r.view, "scanned");
    const after = await s.scene(facts.scope, { audience: "everyone" });
    assert.ok(after.ok && typeof fact(after.scene, TARN, "mass_kg")?.value === "number");
    assert.deepEqual(fact(after.scene, TARN, "surface.temp_mean_k")?.band, [270, 290]);
    await s.send({ kind: "scan_toggle", address: TARN, audience: "everyone", at_yr: 0 });
    const again = await s.scene(facts.scope, { audience: "everyone" });
    assert.ok(again.ok && before.ok && again.hash === before.hash, "a second tap gives the before scene again");

    // Looking through a view, whatever the grants say.
    const survey = await s.scene(facts.scope, { audience: "everyone", data_view: "survey" });
    assert.ok(survey.ok && fact(survey.scene, TARN, "resources")?.value === "rare earths");

    // Edit the disclosure: mass becomes obvious. Applied as the campaign's pack, chronicled.
    const cur = observeOf(s.stack().base)!;
    assert.equal(cur.pack, "kestrel.player-data");
    const next: ObserveSetup = structuredClone(cur.setup);
    next.disclosure.fields["mass_kg"] = "obvious";
    assert.equal(checkSetup(next), null);
    assert.deepEqual(setupChanges(cur.setup, next), ["mass_kg scanned to obvious"]);
    const state0 = await s.stateHash();
    const ws = plainWorkspace(0);
    assert.equal((await applySetup(s, ws, next)).status, "applied");
    assert.notEqual(await s.stateHash(), state0);
    assert.equal(observeOf(s.stack().base)!.pack, PLAYER_DATA_PACK);
    const now = await s.scene(facts.scope, { audience: "everyone" });
    assert.ok(now.ok && typeof fact(now.scene, TARN, "mass_kg")?.value === "number", "long range now shows mass");

    // Bad setups are refused before the kernel sees them.
    const bad = structuredClone(next);
    bad.views[0]!.show_tiers.push("secret");
    assert.match(checkSetup(bad)!, /GM-only/);
    bad.views[0]!.show_tiers.pop();
    bad.campaign.after = "nope";
    assert.match((await applySetup(s, ws, bad)).status === "refused" ? "refused" : "", /refused/);
  } finally {
    await k.close();
  }
});

test("a player seat sees only its own audience and asks the GM to scan", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, { ...tableSeat(), may_send: ["scan_toggle"] }, universe, [base, playerData]);
    const mine = await s.scene("u:kestrel/sys:1");
    assert.ok(mine.ok);
    assert.equal((await s.scene("u:kestrel/sys:1", { audience: "gm" })).ok, false);
    assert.equal((await s.scene("u:kestrel/sys:1", { data_view: "survey" })).ok, false);
    assert.equal((await s.send({ kind: "scan_toggle", address: TARN, audience: "everyone", at_yr: 0 })).status, "pending");
  } finally {
    await k.close();
  }
});
