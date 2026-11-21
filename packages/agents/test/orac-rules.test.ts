// A2 part 3 acceptance: Orac drafts programs (J6). The kernel checks each
// draft by trying the stack it would make; its refusals go back in its own
// words; nothing is stacked until accept, and then only through the GM's
// own stack change.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import type { PackDoc } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { plainWorkspace } from "../../ui-contract/test/workspace.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { runOrac } from "../src/orac.ts";
import { PROGRAMS_PACK, programTarget, ruleEntries } from "../src/orac-rules.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const rpg: PackDoc = json("tests/fixtures/packs/rpg-programs.json");
const scene = json("packages/shell/src/documents/kestrel.scene.json");
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const made = { kind: "", doc: null };
const hauler = (kind: string, to = "home") => ({
  id: "hauler",
  label: "Ore hauler",
  params: { depot: "depot" },
  start: "home",
  states: [
    { id: "home", act: { kind: "unload", payload: { address: "{self}" } }, transitions: [{ when: { if: "range", field: "drone.cargo_t", max: 0 }, to: "out" }] },
    { id: "out", act: { kind, payload: { address: "{self}", to: "{param.depot}" } }, transitions: [{ when: { if: "is", field: "drone.at", value: "depot" }, to }] },
  ],
  budget: { max_steps: 4 },
  explain: "empties its hold, then flies back to the depot",
});

test("programs: the kernel's load refusals go back; accept stacks the campaign's programs pack", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, rpg], k.scratch);
    const ws = plainWorkspace(scene.now_yr);
    const t = programTarget(s, ws);
    const stack0 = s.stack().hash;

    assert.deepEqual((await t.check({ ...hauler("go"), id: "miner" }, made)).problems, ["there is already a program miner"]);
    const undeclared = await t.check(hauler("fly"), made);
    assert.ok(undeclared.problems.some((p) => /fly.*not declared/.test(p)), undeclared.problems.join("; "));
    const nowhere = await t.check(hauler("go", "dock"), made);
    assert.ok(nowhere.problems.some((p) => /dock/.test(p)), nowhere.problems.join("; "));
    const good = await t.check(hauler("go"), made);
    assert.deepEqual(good.problems, []);
    assert.equal(s.stack().hash, stack0, "checking stacks nothing");
    assert.match(t.summary(good.value!), /^Ore hauler: home \(unload\) → out \(go\), starting at home/);

    assert.equal(await t.accept(good.value!), null);
    assert.ok(s.stack().added.some((p) => p.id === PROGRAMS_PACK), "accepted: the campaign's programs pack is stacked");
    assert.notEqual(s.stack().hash, stack0);
    // A second program goes into the same pack, beside the first.
    const again = await t.check({ ...hauler("go"), id: "hauler-2" }, made);
    assert.deepEqual(again.problems, []);
    assert.equal(again.value!.added.filter((p) => p.id === PROGRAMS_PACK).length, 1);

    const entries = ruleEntries(s);
    assert.match(entries.find((e) => e.key === "intents")!.full, /^go: fly to a body/m);
    assert.ok(entries.some((e) => e.key === "program:hauler") && entries.some((e) => e.key === "template:drone"));
  } finally {
    await k.close();
  }
});

test("Orac drafts a program: an undeclared intent goes back, the fix is offered", async () => {
  const k = nodeKernel();
  const p = await mockProvider({ replies: (m: { content: string }[]) => fence({ note: "A hauler.", make: [{ kind: "program", title: "Ore hauler", doc: hauler(/not declared/.test(m.at(-1)!.content) ? "go" : "fly") }] }) });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, rpg], k.scratch);
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { building: { provider: "x", model: "m" } } };
    const r = await runOrac({ request: "A drone that empties its hold and flies home.", targets: [programTarget(s, plainWorkspace(scene.now_yr))], catalogue: ruleEntries(s), ask: asker(st, "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /not declared/);
    assert.equal(r.passed.length, 1);
    assert.ok(!s.stack().added.some((x) => x.id === PROGRAMS_PACK), "nothing stacked before accept");
  } finally {
    await p.close();
    await k.close();
  }
});
