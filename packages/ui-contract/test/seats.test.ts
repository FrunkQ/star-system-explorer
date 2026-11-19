// V10 acceptance in Node: seats and audiences. A three-empire table where
// each seat holds only its own discoveries, checked by inspecting the scenes
// the kernel builds (on the GM's machine and on a player's); disclosure is
// the GM's alone; auto-approve applies only inside its scope and fields.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, decide } from "../src/seat.ts";
import { enterFacts, type AuthoredFacts } from "../src/data-views.ts";
import { GmHost, PlayerReplica, SealedHost, SealedReplica, memoryRoom, type Manifest, type Wire } from "../src/p2p.ts";
import { addSeat, checkSeats, defaultPlan, discoverIntent, fogViews, isDefaultPlan, joinLink, removeSeat, startIntents, type SeatPlan } from "../src/seats.ts";
import { observeOf } from "../src/data-views.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const universe = { id: "kestrel", seed: "kestrel reach campaign" };
const SYS = "u:kestrel/sys:1";
const TARN = `${SYS}/body:3`;
const TARN_A = `${TARN}/body:0`;
const VAULT = `${SYS}/body:4`;
const VIEWS = ["long-range", "scanned", "survey", "rumour", "uncharted"];
const fog = { starts_with: "uncharted", discovered_view: "long-range" };

async function until(f: () => boolean, what: string) {
  for (let i = 0; i < 400; i++) {
    if (f()) return;
    await new Promise((r) => setTimeout(r, 10));
  }
  assert.fail(`timed out waiting for ${what}`);
}

test("the default plan is one table and needs no seat UI; plans are checked", () => {
  const p = defaultPlan();
  assert.ok(isDefaultPlan(p));
  assert.deepEqual(checkSeats(p, { views: VIEWS }), []);
  const three = addSeat(addSeat(addSeat(p, { label: "Empire: Vesh", fog }), { label: "Empire: Orun", fog }), { label: "Empire: Kel", fog });
  assert.ok(!isDefaultPlan(three));
  assert.deepEqual(three.seats.map((s) => s.id), ["table", "empire-vesh", "empire-orun", "empire-kel"]);
  assert.deepEqual(checkSeats(three, { views: VIEWS }), []);
  assert.equal(addSeat(three, { label: "Empire: Vesh" }).seats.at(-1)!.id, "empire-vesh-2", "ids stay unique");
  const bad: SeatPlan = { ...three, seats: [...three.seats, { ...three.seats[1]!, id: "x", audience: "nobody", lens: "nope", sealed: true }] };
  const errs = checkSeats(bad, { views: VIEWS });
  assert.ok(errs.some((e) => /no audience nobody/.test(e)) && errs.some((e) => /no data view nope/.test(e)) && errs.some((e) => /no audience nobody/.test(e)), errs.join("; "));
  assert.deepEqual(removeSeat(three, "empire-orun").audiences.map((a) => a.id), ["everyone", "empire-vesh", "empire-kel"]);
  assert.deepEqual(fogViews(observeOf([playerData])!.setup), fog, "the campaign's blank view starts the fog; its before view is a discovery");
  assert.equal(fogViews({ views: [], campaign: { default_view: "a", before: "a", after: "b" } }), null);
  assert.equal(joinLink("https://v4.example/", "abcd1234"), "https://v4.example/players.html?join=abcd1234");
  assert.equal(joinLink("https://v4.example/", "abcd1234", "empire-vesh", "players.survey.screen.json"), "https://v4.example/players.html?join=abcd1234&seat=empire-vesh&screen=players.survey.screen.json");
});

test("three empires: each seat sees only its own discoveries (state inspection, GM and player side)", async () => {
  const kernels: ReturnType<typeof nodeKernel>[] = [];
  const kernel = () => {
    const k = nodeKernel();
    kernels.push(k);
    return k;
  };
  try {
    const gk = kernel();
    const gm = await Session.open(gk.client, gmSeat(), universe, [base, playerData], gk.scratch);
    const version = await gm.kernelVersion();
    const room = memoryRoom();
    const host = new GmHost(gm, room.hub("gm"), { run: "r10", kernel: version, app: "a", universe, packs: [base, playerData], opening_hash: await gm.stateHash(), clock_yr: 0, seats: defaultPlan().seats });
    assert.equal(await enterFacts(gm, facts), null);
    const everyoneBefore = await gm.scene(SYS, { audience: "everyone" });
    assert.ok(everyoneBefore.ok);

    let plan = defaultPlan();
    for (const label of ["Empire: Vesh", "Empire: Orun", "Empire: Kel"]) plan = addSeat(plan, { label, fog, names: true });
    host.setState({ seats: plan.seats });
    for (const i of startIntents(plan, "u:kestrel", 0)) assert.equal((await gm.send(i)).status, "applied");
    const aud = (id: string) => plan.audiences.find((a) => a.id === id)!;
    const sees = async (s: Session, audience: string) => {
      const r = await s.scene(SYS, { audience });
      assert.ok(r.ok, r.ok ? "" : r.message);
      return r.scene.subjects.map((x) => x.address).sort();
    };
    for (const a of ["empire-vesh", "empire-orun", "empire-kel"]) assert.deepEqual(await sees(gm, a), [], `${a} starts knowing nothing`);

    // Vesh finds Tarn (its moon comes with it); Orun finds Vault; Kel charts the whole system.
    for (const [a, at] of [["empire-vesh", TARN], ["empire-orun", VAULT], ["empire-kel", SYS]] as const) {
      assert.equal((await gm.send(discoverIntent(aud(a), at, true, "long-range", 0))).status, "applied");
    }
    assert.deepEqual(await sees(gm, "empire-vesh"), [TARN, TARN_A]);
    assert.deepEqual(await sees(gm, "empire-orun"), [VAULT]);
    assert.deepEqual(await sees(gm, "empire-kel"), [SYS, TARN, TARN_A, VAULT].sort());
    const everyoneAfter = await gm.scene(SYS, { audience: "everyone" });
    assert.ok(everyoneAfter.ok);
    assert.equal(everyoneAfter.hash, everyoneBefore.hash, "the table's scene is untouched by the empires' grants");
    // The discovered view is the long-range one: names, not scan data.
    const vesh = await gm.scene(SYS, { audience: "empire-vesh" });
    assert.ok(vesh.ok && vesh.scene.subjects.every((s) => s.view === "long-range"));

    // A Vesh player's own kernel, replaying the GM's events, builds the same Vesh scene and nothing more.
    const open = async (m: Manifest) => {
      const k = kernel();
      return Session.open(k.client, m.seats.find((x) => x.id === "empire-vesh")!, m.universe, m.packs, k.scratch);
    };
    const p = new PlayerReplica(room.hub("vesh-1"), open, { kernel: version, app: "a", now: () => 0 });
    p.join();
    await until(() => p.status.state === "live", "Vesh live");
    assert.equal(p.session!.seat.audience, "empire-vesh");
    assert.deepEqual(await sees(p.session!, "empire-vesh"), [TARN, TARN_A]);
    const two = new PlayerReplica(room.hub("vesh-2"), open, { kernel: version, app: "a", now: () => 0 });
    two.join();
    await until(() => two.status.state === "live", "second Vesh window live");

    // Taking a discovery back is another event; both Vesh windows follow it in step.
    await gm.send(discoverIntent(aud("empire-vesh"), TARN, false, "long-range", 0));
    await until(() => (p.status as { seq?: number }).seq === host.state().seq && (two.status as { seq?: number }).seq === host.state().seq, "both windows in step");
    assert.deepEqual(await sees(p.session!, "empire-vesh"), []);
    assert.equal(await p.session!.stateHash(), await two.session!.stateHash());
    assert.equal(await p.session!.stateHash(), await gm.stateHash());

    // Disclosure is the GM's: a player seat can never grant itself a view, even through the GM.
    const grab = await p.request({ kind: "set_value", address: SYS, field: "observe.view.empire-vesh", value: "survey", at_yr: 0 });
    assert.equal(grab.ok, false);
    const forger = room.hub("forger");
    let forged: { status?: string; reason?: string } | null = null;
    forger.onMessage((m) => (m.t === "answer" && m.to === "forger" ? (forged = m) : null));
    forger.send({ t: "request", run: "r10", from: "forger", seat: "empire-vesh", id: "x", intent: { kind: "set_value", address: SYS, field: "observe.view.empire-vesh", value: "survey", at_yr: 0 } });
    await until(() => forged !== null, "forged answered");
    assert.equal(forged!.status, "declined");
    assert.match(forged!.reason!, /game master/);
  } finally {
    await Promise.all(kernels.map((k) => k.close()));
  }
});

test("auto-approve applies only inside its scope and fields; the rest waits for the GM", async () => {
  const kernels: ReturnType<typeof nodeKernel>[] = [];
  const kernel = () => {
    const k = nodeKernel();
    kernels.push(k);
    return k;
  };
  try {
    const gk = kernel();
    const gm = await Session.open(gk.client, gmSeat(), universe, [base, playerData], gk.scratch);
    const version = await gm.kernelVersion();
    const room = memoryRoom();
    const plan = addSeat(defaultPlan(), { label: "Survey ship", names: true, autoNames: [TARN] });
    const seat = plan.seats.at(-1)!;
    assert.deepEqual(decide(seat, { kind: "set_value", address: TARN_A, field: "name", value: "x", at_yr: 0 }), { action: "apply" }, "inside the scope (moons included)");
    assert.equal(decide(seat, { kind: "set_value", address: VAULT, field: "name", value: "x", at_yr: 0 }).action, "ask_gm");
    assert.equal(decide(seat, { kind: "set_value", address: TARN, field: "mass_kg", value: 1, at_yr: 0 }).action, "ask_gm", "only the named fields");
    assert.equal(decide({ ...seat, auto_approve: [{ intent: "set_value", scope: [SYS] }] }, { kind: "set_value", address: TARN, field: "observe.view", value: "survey", at_yr: 0 }).action, "refuse");

    const host = new GmHost(gm, room.hub("gm"), { run: "ra", kernel: version, app: "a", universe, packs: [base, playerData], opening_hash: await gm.stateHash(), clock_yr: 0, seats: plan.seats });
    const open = async (m: Manifest) => {
      const k = kernel();
      return Session.open(k.client, m.seats.find((x) => x.id === seat.id)!, m.universe, m.packs, k.scratch);
    };
    const p = new PlayerReplica(room.hub("ship"), open, { kernel: version, app: "a", now: () => 0 });
    p.join();
    await until(() => p.status.state === "live", "live");
    assert.ok((await p.request({ kind: "set_value", address: TARN, field: "name", value: "Haven", at_yr: 0 })).ok);
    await until(() => p.requests()[0]!.status === "approved" && host.state().seq === 1, "applied without the GM");
    assert.equal(host.requests().length, 0);
    assert.ok((await p.request({ kind: "set_value", address: VAULT, field: "name", value: "Anvil", at_yr: 0 })).ok);
    await until(() => host.requests().length === 1, "outside the scope it waits");
    assert.equal(host.state().seq, 1);
    await until(() => (p.status as { seq?: number }).seq === 1, "player replayed");
    assert.equal(await p.session!.stateHash(), await gm.stateHash());
  } finally {
    await Promise.all(kernels.map((k) => k.close()));
  }
});

test("a sealed seat's device receives no seed and no unfiltered fact", async () => {
  const gk = nodeKernel();
  try {
    const gm = await Session.open(gk.client, gmSeat(), universe, [base, playerData], gk.scratch);
    const version = await gm.kernelVersion();
    const shared = memoryRoom();
    const plan = addSeat(defaultPlan(), { label: "Empire: Vesh", fog, names: true, sealed: true });
    const seat = plan.seats.at(-1)!;
    assert.equal(seat.sealed, true);
    assert.deepEqual(checkSeats(plan, { views: VIEWS }), []);
    const host = new GmHost(gm, shared.hub("gm"), { run: "rs", kernel: version, app: "a", universe, packs: [base, playerData], opening_hash: await gm.stateHash(), clock_yr: 0, seats: plan.seats });
    assert.equal(await enterFacts(gm, facts), null);
    for (const i of startIntents(plan, "u:kestrel", 0)) assert.equal((await gm.send(i)).status, "applied");

    // The sealed seat finds the shared room: it is not served there.
    const sneak = shared.hub("sneak");
    const heardShared: Wire[] = [];
    sneak.onMessage((m) => heardShared.push(m));
    sneak.send({ t: "hello", from: "sneak", protocol: "sse.p2p/1", seat: seat.id });
    await new Promise((r) => setTimeout(r, 30));
    assert.ok(!heardShared.some((m) => m.t === "manifest"), "no manifest for a sealed seat");

    // Its own room: scenes only.
    const room = memoryRoom();
    const sealedHost = new SealedHost(gm, host, room.hub("gm"), { seat, scope: SYS, app: "a", clock_yr: 0, data_views: VIEWS });
    const device = room.hub("vesh-tablet");
    const heard: Wire[] = [];
    device.onMessage((m) => heard.push(structuredClone(m)));
    let t = 0;
    const p = new SealedReplica(device, seat.id, { app: "a", now: () => t });
    p.join();
    await until(() => p.status.state === "live", "sealed seat live");
    assert.deepEqual(p.scene!.subjects, [], "knows nothing yet");

    await gm.send(discoverIntent(plan.audiences.at(-1)!, TARN, true, "long-range", 0));
    await until(() => (p.scene?.subjects.length ?? 0) > 0, "discovery arrives as a scene");
    assert.deepEqual(p.scene!.subjects.map((x) => x.address).sort(), [TARN, TARN_A]);

    // A request goes to the GM's queue and its answer comes back through the sealed room.
    assert.ok((await p.request({ kind: "set_value", address: TARN, field: "name", value: "Haven", at_yr: 0 })).ok);
    await until(() => host.requests().length === 1, "GM sees it");
    await host.approve(host.requests()[0]!.id, "vesh-tablet");
    await until(() => p.requests()[0]?.status === "approved", "approved");
    await until(() => JSON.stringify(p.scene).includes("Haven"), "renamed scene");

    // Everything the device ever received: no seed, no packs, no log, no fact it was not shown.
    const all = JSON.stringify(heard);
    assert.ok(!all.includes(universe.seed), "no seed");
    assert.ok(!all.includes('"packs"') && !all.includes('"log"') && !all.includes('"entry"') && !all.includes('"intent"'), "no packs, log or intents");
    assert.ok(!all.includes("Vault") && !all.includes(VAULT), "nothing about what Vesh has not found");
    assert.ok(heard.every((m) => ["sealed", "scene", "answer", "beat", "bye", "hello", "request"].includes(m.t)), [...new Set(heard.map((m) => m.t))].join());

    // The GM goes: read only on the last scene.
    sealedHost.close();
    await until(() => p.status.state === "offline", "offline");
    assert.ok(JSON.stringify(p.scene).includes("Haven"));
  } finally {
    await gk.close();
  }
});
