// A2 part 6 acceptance: Orac drafts triggers (track J). The kernel loads the
// candidate stack's triggers and its refusals go back in its own words;
// nothing is stacked until accept.
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
import { TRIGGERS_PACK, triggerEntries, triggerTarget } from "../src/orac-triggers.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const scene = json("packages/shell/src/documents/kestrel.scene.json");
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const made = { kind: "", doc: null };
const mayday = (id = "mine.mayday") => ({
  id,
  when: { on_condition: { condition: { if: "tag", tag: "status.damaged" } } },
  scope: { tag: "construct" },
  effects: [{ effect: "add_note", text: "A distress call goes out." }],
  explain: "a damaged ship calls for help",
  tags: [
    { id: "construct", label: "construct" },
    { id: "status.damaged", label: "Damaged" },
  ],
});

test("triggers: the kernel's load refusals go back; accept stacks the campaign's triggers pack", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base], k.scratch);
    const t = triggerTarget(s, plainWorkspace(scene.now_yr));
    const stack0 = s.stack().hash;
    const badId = await t.check(mayday("Mine Mayday"), made);
    assert.ok(badId.problems.some((p) => /ids are lowercase/.test(p)), badId.problems.join("; "));
    const unknownWhen = await t.check({ ...mayday(), when: { on_whim: {} } }, made);
    assert.ok(unknownWhen.problems.some((p) => /on_whim/.test(p)), unknownWhen.problems.join("; "));
    const { tags: _t, ...undeclared } = mayday();
    const noTags = await t.check(undeclared, made);
    assert.ok(noTags.problems.some((p) => /names tag construct, which no pack in the stack declares/.test(p)), noTags.problems.join("; "));
    const good = await t.check(mayday(), made);
    assert.deepEqual(good.problems, []);
    assert.equal(s.stack().hash, stack0, "checking stacks nothing");
    assert.match(t.summary(good.value!), /^mine\.mayday: when its condition holds, add note\. a damaged ship calls for help/);
    assert.equal(await t.accept(good.value!), null);
    assert.ok(s.stack().added.some((p) => p.id === TRIGGERS_PACK));
    assert.deepEqual((await t.check(mayday(), made)).problems, ["there is already a trigger mine.mayday"]);
    assert.ok(triggerEntries(s).some((e) => e.key === "trigger:mine.mayday"));
    assert.match(triggerEntries(s).find((e) => e.key === "tags")!.full, /^status\.damaged: Damaged/m);
  } finally {
    await k.close();
  }
});

test("Orac drafts a trigger: the kernel's refusal goes back, the fix is offered", async () => {
  const k = nodeKernel();
  const p = await mockProvider({ replies: (m: { content: string }[]) => fence({ note: "A distress call.", make: [{ kind: "trigger", title: "Mayday", doc: mayday(/lowercase/.test(m.at(-1)!.content) ? "mine.mayday" : "Mine Mayday") }] }) });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base], k.scratch);
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { building: { provider: "x", model: "m" } } };
    const r = await runOrac({ request: "A damaged ship sends a distress call.", targets: [triggerTarget(s, plainWorkspace(scene.now_yr))], catalogue: triggerEntries(s), ask: asker(st, "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /lowercase/);
    assert.equal(r.passed.length, 1);
    assert.ok(!s.stack().added.some((x) => x.id === TRIGGERS_PACK));
  } finally {
    await p.close();
    await k.close();
  }
});
