// SV7 part 1 acceptance: Orac's bulk reveals are change requests. The check
// reads the kernel's own known_to; nothing is revealed until accept, which
// sends the same set_known the "Known to" eye sends, as one undo for a list.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { enterFacts, yearsToSeconds, type AuthoredFacts, type PackDoc } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { plainWorkspace } from "../../ui-contract/test/workspace.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { runOrac } from "../src/orac.ts";
import { placeEntries, revealIntents, revealTarget } from "../src/orac-known.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const observe: PackDoc = json("tests/fixtures/packs/tarn-observe.json");
const scene = json("packages/shell/src/documents/kestrel.scene.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const places = () => (scene.subjects as { address: string; label: string }[]).map((s) => ({ address: s.address, name: s.label }));
const made = { kind: "", doc: null };
const TARN = "u:kestrel/sys:1/body:3";
const VAULT = "u:kestrel/sys:1/body:4";
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";

test("reveals: the kernel's known_to checks the draft; accept is one set_known over the list; nothing moves before", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, observe], k.scratch);
    assert.equal(await enterFacts(s, facts), null);
    const ws = plainWorkspace(scene.now_yr);
    const t = revealTarget(s, ws, places);
    const tell = { audience: "everyone", landmark: "known", things: [TARN, VAULT], explain: "the crew flew past both" };
    assert.match((await t.check({ ...tell, audience: "crew" }, made)).problems[0]!, /"crew" is not an audience here: use one of everyone \(The table\)/);
    assert.match((await t.check({ ...tell, things: ["u:kestrel/sys:1/body:9"] }, made)).problems[0]!, /not among the things/);
    assert.match((await t.check({ ...tell, landmark: "seen" }, made)).problems[0]!, /landmark is one of gm_only, heard_of, known/);
    assert.match((await t.check({ ...tell, landmark: "heard_of" }, made)).problems[0]!, /nothing would change: The table already has every one of them at Heard of/);
    const hash = await s.stateHash();
    const good = await t.check(tell, made);
    assert.deepEqual(good.problems, []);
    assert.equal(await s.stateHash(), hash, "checking reveals nothing");
    assert.equal(t.summary(good.value!), "The table: Tarn (Heard of to Known), Vault (Heard of to Known). The crew flew past both. Accept sends it as one change with one undo; no fact changes, only who knows it.");
    const [i, ...more] = revealIntents(good.value!.doc, { at_yr: 1, at_s: "1" });
    assert.equal(more.length, 0);
    assert.deepEqual((i as { scope: unknown }).scope, { list: [TARN, VAULT] });
    assert.equal(await t.accept(good.value!), null);
    const after = await s.knownTo(VAULT, undefined, yearsToSeconds(scene.now_yr));
    assert.ok(after.ok && after.value.audiences[0]!.landmark === "known", JSON.stringify(after));
    assert.match((await t.check(tell, made)).problems[0]!, /nothing would change/);
    assert.ok(placeEntries(places()).find((e) => e.key === "things")!.full.includes(`${TARN}: Tarn`));
  } finally {
    await k.close();
  }
});

test("Orac drafts a reveal: a wrong audience goes back with the kernel's list, the fix is offered, nothing revealed", async () => {
  const k = nodeKernel();
  const draft = (audience: string) => ({ note: "Tell the table.", make: [{ kind: "reveal", title: "Tarn", doc: { audience, landmark: "heard_of", things: [TARN], with_inside: true } }] });
  const p = await mockProvider({ replies: (m: { content: string }[]) => fence(draft(/not an audience/.test(m.at(-1)!.content) ? "everyone" : "crew")) });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, observe], k.scratch);
    assert.equal(await enterFacts(s, facts), null);
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { building: { provider: "x", model: "m" } } };
    const r = await runOrac({ request: "The crew have heard of Tarn and its moon.", targets: [revealTarget(s, plainWorkspace(scene.now_yr), places)], catalogue: placeEntries(places()), ask: asker(st, "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /not an audience/);
    assert.equal(r.passed.length, 1);
    const k2 = await s.knownTo(TARN);
    assert.ok(k2.ok && k2.value.audiences[0]!.landmark === "heard_of");
  } finally {
    await p.close();
    await k.close();
  }
});
