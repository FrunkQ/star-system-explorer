// V4 acceptance in Node (H9.2): two kernels, one GM, players replaying the
// GM's events over a room, agreeing by state hash; a late joiner; a player
// with a different app version told to reload; a GM drop going read-only;
// a divergence repaired from a snapshot.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, sharedTableSeat, tableSeat } from "../src/seat.ts";
import { enterFacts, type AuthoredFacts } from "../src/data-views.ts";
import { GmHost, PlayerReplica, memoryRoom, type Manifest, type ReplicaStatus } from "../src/p2p.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";
import type { PartyShip } from "../src/party.ts";
import type { Json } from "../src/json.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const universe = { id: "kestrel", seed: "kestrel reach campaign" };
const TARN = "u:kestrel/sys:1/body:3";
const settle = () => new Promise((r) => setTimeout(r, 30));
async function until(f: () => boolean, what: string) {
  for (let i = 0; i < 400; i++) {
    if (f()) return;
    await new Promise((r) => setTimeout(r, 10));
  }
  assert.fail(`timed out waiting for ${what}`);
}

test("players replay the GM's events and agree by state hash; GM drop goes read-only; version mismatch reloads", async () => {
  const kernels: ReturnType<typeof nodeKernel>[] = [];
  const kernel = () => {
    const k = nodeKernel();
    kernels.push(k);
    return k;
  };
  try {
    const room = memoryRoom();
    const gk = kernel();
    const gm = await Session.open(gk.client, gmSeat(), universe, [base, playerData], gk.scratch);
    const version = await gm.kernelVersion();
    const host = new GmHost(gm, room.hub("gm"), { run: "run-1", kernel: version, app: "test-app", universe, packs: [base, playerData], opening_hash: await gm.stateHash(), clock_yr: facts.at_yr });
    assert.equal(await enterFacts(gm, facts), null);

    let clock = 0;
    const open = async (m: Manifest) => {
      const k = kernel();
      return Session.open(k.client, tableSeat(), m.universe, m.packs, k.scratch);
    };
    const player = (id: string, app = "test-app") => {
      const p = new PlayerReplica(room.hub(id), open, { kernel: version, app, now: () => clock, offlineAfterMs: 6000 });
      const seen: ReplicaStatus["state"][] = [];
      p.onStatus = (s) => seen.push(s.state);
      p.join();
      return { p, seen };
    };

    // A phone joins after the facts were entered: it replays the log.
    const phone = player("phone");
    await until(() => phone.p.status.state === "live", "phone live");
    assert.equal((phone.p.status as { state_hash: string }).state_hash, await gm.stateHash());
    assert.equal(await phone.p.session!.stateHash(), await gm.stateHash(), "phone and GM agree by state hash");

    // The GM edits: a desktop already in the room and the phone both follow.
    const desk = player("desk");
    await until(() => desk.p.status.state === "live", "desk live");
    const r = await gm.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.55, at_yr: facts.at_yr });
    assert.equal(r.status, "applied");
    const r2 = await gm.send({ kind: "scan_toggle", address: TARN, audience: "everyone", at_yr: facts.at_yr });
    assert.equal(r2.status, "applied");
    const want = await gm.stateHash();
    await until(() => phone.p.status.state === "live" && (phone.p.status as { seq: number }).seq === host.state().seq && desk.p.status.state === "live" && (desk.p.status as { seq: number }).seq === host.state().seq, "both caught up");
    assert.equal(await phone.p.session!.stateHash(), want);
    assert.equal(await desk.p.session!.stateHash(), want);
    // The players' scene shows the scanned disclosure, computed on their own kernel.
    const gs = await gm.scene(facts.scope, { audience: "everyone" });
    const ps = await phone.p.session!.scene(facts.scope);
    assert.ok(gs.ok && ps.ok);
    assert.equal(ps.hash, gs.hash, "the player's own scene equals what the GM's kernel says that audience sees");
    assert.equal(host.state().players, 2);

    // Data-view state: a player screen may look through a view only when the GM grants it.
    const view = (playerData as unknown as { provides: { other: { observe: { views: { id: string }[] } } } }).provides.other.observe.views[0]!.id;
    assert.equal((await phone.p.session!.scene(facts.scope, { data_view: view })).ok, false);
    host.setState({ data_views: [view], clock_yr: facts.at_yr });
    await settle();
    assert.equal((await phone.p.session!.scene(facts.scope, { data_view: view })).ok, true);

    // What the table is shown (the landing run-through): nothing sent yet, then a place, then screens off; a late joiner
    // gets the latest on its manifest.
    assert.equal(phone.p.shown(), undefined);
    host.setState({ shown: { scope: facts.scope, title: "Kestrel" } });
    await settle();
    assert.deepEqual(phone.p.shown(), { scope: facts.scope, title: "Kestrel" });
    host.setState({ shown: { off: true } });
    await settle();
    assert.deepEqual(desk.p.shown(), { off: true });
    assert.deepEqual(host.manifest().shown, { off: true });
    host.setState({ clock_yr: facts.at_yr });
    await settle();
    assert.deepEqual(phone.p.shown(), { off: true }, "a clock tick keeps what is shown");
    // The party's ship travels as its own key: the table always knows where it is.
    const kittiwake: PartyShip = { ship: "u:kestrel/sys:1/c:1", name: "Kittiwake", nav: "fusion-torch", at: TARN, voyage: { from: "u:kestrel/sys:1/body:1", to: TARN, days: 12 } };
    host.setState({ party: kittiwake as unknown as Json });
    await settle();
    assert.equal((phone.p.party() as { name: string }).name, "Kittiwake");
    assert.equal((phone.p.party() as unknown as PartyShip).voyage?.days, 12);
    assert.equal((host.manifest().party as { at: string }).at, TARN);
    host.setState({ party: null });
    await settle();
    assert.equal(phone.p.party(), undefined, "no party ship any more");

    // The Switchboard (PB1): a screen says hello with its own id and name; one screen can be sent its own place or held
    // with a card, while the rest of the table carries on.
    const hellos: { id: string; name: string; role: string }[] = [];
    host.onScreen = (h) => hellos.push(h);
    const wall = new PlayerReplica(room.hub("wall"), open, { kernel: version, app: "test-app", now: () => clock, offlineAfterMs: 6000 });
    wall.join({ id: "scr-wall", name: "Wall TV", role: "table" });
    await until(() => wall.status.state === "live", "wall live");
    assert.deepEqual(hellos, [{ id: "scr-wall", name: "Wall TV", role: "table" }]);
    host.setState({ shown: { scope: facts.scope, title: "Kestrel" }, screens: { "scr-wall": { off: true, card: "Signal lost" } } });
    await settle();
    assert.deepEqual(wall.shownFor("table", "scr-wall"), { off: true, card: "Signal lost" });
    assert.deepEqual(phone.p.shownFor("table"), { scope: facts.scope, title: "Kestrel" }, "the other screens carry on");
    assert.deepEqual(host.manifest().screens, { "scr-wall": { off: true, card: "Signal lost" } }, "a late joiner gets it on the manifest");
    host.setState({ screens: {} });
    await settle();
    assert.deepEqual(wall.shownFor("table", "scr-wall"), { scope: facts.scope, title: "Kestrel" }, "back with the table");

    // A system the GM adds (a V3 import) reaches players' screens as an event: their kernels hold it too.
    const added = await gm.adopt({ import_v3: readFileSync(new URL("tests/fixtures/v3/synthetic-sol.json", root), "utf8") });
    assert.ok(added.ok, added.ok ? "" : added.message);
    const sys = added.ok ? added.result.systems[0]! : "";
    await until(() => phone.p.status.state === "live" && (phone.p.status as { seq: number }).seq === host.state().seq, "phone has the adopt");
    assert.equal(await phone.p.session!.stateHash(), await gm.stateHash(), "the adopt replays to the same state");
    const theirs = await phone.p.session!.scene(sys, { audience: "everyone" });
    assert.ok(theirs.ok, theirs.ok ? "" : `the player's kernel has the imported system: ${theirs.message}`);
    assert.equal((await phone.p.session!.adopt({ import_v3: "{}" })).ok, false, "a player seat never adds a system");

    // A player tries to change the universe: its seat cannot (V5 carries requests).
    const own = await phone.p.session!.send({ kind: "set_value", address: TARN, field: "a_au", value: 9, at_yr: facts.at_yr });
    assert.notEqual(own.status, "applied");

    // Heartbeats keep players live; silence makes them read-only offline.
    clock = 1000;
    host.beat();
    await settle();
    phone.p.tick();
    assert.equal(phone.p.status.state, "live");
    clock = 9000;
    phone.p.tick();
    assert.equal(phone.p.status.state, "offline");
    // Wait for delivery, not a fixed 30 ms: on a loaded runner the hub's
    // message can take longer. The clock stands still meanwhile, so only the
    // GM's beat or goodbye can change the state, never a heartbeat timeout.
    host.beat();
    await until(() => phone.p.status.state === "live", "the GM back: live again");
    host.close();
    await until(() => desk.p.status.state === "offline", "the GM's goodbye: read-only at once");

    // A player built from a different app version is told to reload, and never drifts.
    const host2 = new GmHost(gm, room.hub("gm2"), { run: "run-2", kernel: version, app: "test-app", universe, packs: [base, playerData], opening_hash: "", clock_yr: 0 });
    const old = player("old", "older-app");
    await until(() => old.p.status.state === "reload", "old reload");
    assert.match((old.p.status as { reason: string }).reason, /Reload/);
    assert.equal(old.p.session, null);
    host2.close();
  } finally {
    await Promise.all(kernels.map((k) => k.close()));
  }
});

test("a player that diverges asks for a snapshot and rebuilds in step", async () => {
  const kernels: ReturnType<typeof nodeKernel>[] = [];
  const kernel = () => {
    const k = nodeKernel();
    kernels.push(k);
    return k;
  };
  try {
    const room = memoryRoom();
    const gk = kernel();
    const gm = await Session.open(gk.client, gmSeat(), universe, [base], gk.scratch);
    const version = await gm.kernelVersion();
    const host = new GmHost(gm, room.hub("gm"), { run: "r", kernel: version, app: "a", universe, packs: [base], opening_hash: await gm.stateHash(), clock_yr: 0 });
    const open = async (m: Manifest) => {
      const k = kernel();
      return Session.open(k.client, tableSeat(), m.universe, m.packs, k.scratch);
    };
    let sessions = 0;
    const p = new PlayerReplica(room.hub("p"), open, { kernel: version, app: "a", now: () => 0 });
    p.onSession = () => (sessions += 1);
    const states: string[] = [];
    p.onStatus = (s) => states.push(s.state);
    p.join();
    await until(() => p.status.state === "live", "live");
    // The player misses an event (a dropped message): the next beat shows it is behind.
    room.drop("gm");
    await gm.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.6, at_yr: 0 });
    await settle();
    assert.equal((p.status as { seq: number }).seq, 0, "the event never arrived");
    const room2 = room; // same room, GM reachable again through a fresh hub
    void room2;
    // Re-attach: a new hub for the GM that serves the same log.
    const hub2 = room.hub("gm-again");
    hub2.onMessage((m) => {
      if (m.t === "snapshot") hub2.send({ t: "manifest", to: m.from, manifest: host.manifest() });
    });
    hub2.send({ t: "beat", run: "r", seq: host.state().seq, state_hash: host.state().state_hash });
    await until(() => p.status.state === "live" && (p.status as { seq: number }).seq === 1, "rebuilt in step");
    assert.ok(states.includes("resync"));
    assert.equal(sessions, 2, "a fresh session was built from the snapshot");
    assert.equal(await p.session!.stateHash(), await gm.stateHash());
  } finally {
    await Promise.all(kernels.map((k) => k.close()));
  }
});

test("V5: a player's request becomes an event only after the GM approves it", async () => {
  const kernels: ReturnType<typeof nodeKernel>[] = [];
  const kernel = () => {
    const k = nodeKernel();
    kernels.push(k);
    return k;
  };
  try {
    const room = memoryRoom();
    const gk = kernel();
    const gm = await Session.open(gk.client, gmSeat(), universe, [base, playerData], gk.scratch);
    const version = await gm.kernelVersion();
    const host = new GmHost(gm, room.hub("gm"), { run: "r5", kernel: version, app: "a", universe, packs: [base, playerData], opening_hash: await gm.stateHash(), clock_yr: 0, seats: [sharedTableSeat()] });
    const open = async (m: Manifest) => {
      const k = kernel();
      return Session.open(k.client, m.seats.find((x) => x.id === "table")!, m.universe, m.packs, k.scratch);
    };
    const p = new PlayerReplica(room.hub("p"), open, { kernel: version, app: "a", now: () => 0 });
    let answers = 0;
    p.onRequests = () => (answers += 1);
    p.join();
    await until(() => p.status.state === "live", "live");

    // The request travels; nothing changes until the GM acts.
    const name = { kind: "set_value" as const, address: TARN, field: "name", value: "Haven", at_yr: 0 };
    const sent = await p.request(name);
    assert.ok(sent.ok);
    await until(() => host.requests().length === 1, "GM sees the request");
    assert.equal(host.state().seq, 0, "no event before approval");
    assert.equal(await p.session!.stateHash(), await gm.stateHash());
    assert.equal(p.requests()[0]!.status, "waiting");

    // Approved: the GM's session applies it; it reaches the player as an ordinary event.
    await host.approve(host.requests()[0]!.id, "p");
    await until(() => p.requests()[0]!.status === "approved" && (p.status as { seq: number }).seq === 1, "approved and replayed");
    assert.equal(host.state().seq, 1);
    assert.equal(await p.session!.stateHash(), await gm.stateHash());
    const hist = await gm.history(TARN);
    assert.ok(hist.length >= 1, "chronicled on the GM's side");

    // Declined: no event, a reason comes back.
    const again = await p.request({ ...name, value: "Nope" });
    assert.ok(again.ok);
    await until(() => host.requests().length === 1, "second request");
    host.decline(host.requests()[0]!.id, "p", "Not tonight.");
    await until(() => p.requests()[1]!.status === "declined", "declined");
    assert.equal(p.requests()[1]!.reason, "Not tonight.");
    assert.equal(host.state().seq, 1);

    // A request the seat may never make is refused on the player's side; a forged one is declined by the GM.
    const never = await p.request({ kind: "scan_toggle", address: TARN, audience: "everyone", at_yr: 0 });
    assert.equal(never.ok, false);
    const forger = room.hub("forger");
    let forged: unknown = null;
    forger.onMessage((m) => (m.t === "answer" && m.to === "forger" ? (forged = m) : null));
    forger.send({ t: "request", run: "r5", from: "forger", seat: "gm", id: "x", intent: name });
    await until(() => forged !== null, "forged answered");
    assert.equal((forged as { status: string }).status, "declined");
    assert.equal(host.requests().length, 0);

    // While the GM is away, requests are disabled.
    host.close();
    await until(() => p.status.state === "offline", "offline");
    assert.equal((await p.request(name)).ok, false);
    assert.ok(answers >= 4);
  } finally {
    await Promise.all(kernels.map((k) => k.close()));
  }
});

// Record writes (a ship made, a note, a kept tank level, a flight log entry) travel as `record` intents, so players stay
// in step with the GM's state hash through them and through the changes after (party run-through, 24 Nov).
test("record writes travel to players: a ship made, its tanks and its flight log keep the replica in step", async () => {
  const ks: ReturnType<typeof nodeKernel>[] = [];
  const kernel = () => {
    const k = nodeKernel();
    ks.push(k);
    return k;
  };
  try {
    const manifest: PackDoc = json("packs/base-manifest/pack.json");
    const rpg: PackDoc = json("tests/fixtures/packs/rpg-templates.json");
    const room = memoryRoom();
    const gk = kernel();
    const packs = [base, playerData, manifest, rpg];
    const gm = await Session.open(gk.client, gmSeat(), universe, packs, gk.scratch);
    const version = await gm.kernelVersion();
    const host = new GmHost(gm, room.hub("gm"), { run: "run-rec", kernel: version, app: "test-app", universe, packs, opening_hash: await gm.stateHash(), clock_yr: facts.at_yr });
    assert.equal(await enterFacts(gm, facts), null);
    const open = async (m: Manifest) => {
      const k = kernel();
      return Session.open(k.client, tableSeat(), m.universe, m.packs, k.scratch);
    };
    const p = new PlayerReplica(room.hub("phone"), open, { kernel: version, app: "test-app", now: () => 0, offlineAfterMs: 6000 });
    p.join();
    await until(() => p.status.state === "live", "phone live");
    const made = await gm.createFrom({ template: "freighter", under: "u:kestrel/sys:1", name: "Beowulf", at_s: 0 });
    assert.ok(made.ok, JSON.stringify(made));
    const ship = (made as { ok: true; result: { address: string } }).result.address;
    // Sent together, as Voyages does: they apply one at a time, in order.
    const [kept, log] = await Promise.all([
      gm.setKept!({ address: ship, kind: "fuel.propellant_t", value: 1, reason: "voyage to Tarn", at_s: 0 }),
      gm.logFlight({ construct: ship, at_s: 0, event: { at_s: 0, kind: "stop", type: "depart", place_id: "u:kestrel/sys:1", mark: "planned" } }),
    ]);
    assert.ok(kept.ok, JSON.stringify(kept));
    assert.ok(log.ok, JSON.stringify(log));
    assert.ok((await gm.addNote!({ address: ship, text: "the crew renamed the galley", audience: "gm", at_s: 0 })).ok);
    // A field change after them still replays to the same state.
    assert.equal((await gm.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.6, at_yr: facts.at_yr })).status, "applied");
    await until(() => p.status.state === "live" && (p.status as { seq: number }).seq === host.state().seq, "phone caught up");
    assert.equal(await p.session!.stateHash(), await gm.stateHash(), "in step after record writes");
    // A player seat cannot send one.
    const t = await p.session!.send({ kind: "record", op: "set_kept", ask: { address: ship, kind: "fuel.propellant_t", value: 9, at_s: 0 } });
    assert.equal(t.status, "refused");
  } finally {
    for (const k of ks) await k.close();
  }
});
