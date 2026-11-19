// A2 acceptance (build plan: invalid candidates never reach the accept button;
// nothing mutates without accept). Orac runs Ovid's loop against the real
// kernel: a first draft the kernel rejects is sent back with the kernel's own
// words, the fixed draft passes, and only that one is offered. Nothing is
// filed, adopted or saved until accept is called, and the state hash and
// stack never move during the run.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { SHIPPED_LENSES, type Lens, type PackDoc } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { plainWorkspace } from "../../ui-contract/test/workspace.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { draftLoop } from "../src/loop.ts";
import { atomIssueTarget, flowEntries, lensEntries, lensTarget, physicsTarget } from "../src/orac-targets.ts";
import { atomIssueUrl, checkAtomIssue, oracMay, oracSystem, runOrac } from "../src/orac.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const pack: PackDoc = json("packs/base-physics/pack.json");
const scene = json("packages/shell/src/documents/kestrel.scene.json");
const CLIMATE = "sse.base.planet-climate";
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";

const linkedTo = (base: string): AgentsState => ({ ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base }], links: { building: { provider: "x", model: "mock/large" } } });

test("Orac: a rejected draft is fixed in the loop; only the passing one is offered; nothing changes before accept", async () => {
  const k = nodeKernel();
  const replies = [
    fence({ need: [`flow:${CLIMATE}`] }),
    fence({ note: "Doubled the heat carried to the night side.", make: [{ kind: "physics", title: "Keep the heat on the dayside", rationale: "Tidally locked worlds", doc: { flow: CLIMATE, edits: [{ node: "teq", param: "redistribution", after: 7 }] } }] }),
    fence({ note: "Within the allowed range now.", make: [{ kind: "physics", title: "Keep the heat on the dayside", rationale: "Tidally locked worlds", doc: { flow: CLIMATE, edits: [{ node: "teq", param: "redistribution", after: 0.5 }] } }] }),
  ];
  const p = await mockProvider({ replies });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [pack], k.scratch);
    const ws = plainWorkspace(scene.now_yr);
    const hash0 = await s.stateHash();
    const stack0 = s.stack().hash;
    const target = physicsTarget(s, ws);
    const st = linkedTo(p.base);
    const r = await runOrac({ request: "Tidally locked worlds should keep more heat on their day side.", targets: [target], catalogue: flowEntries(s), ask: asker(st, "building") });

    assert.equal(r.ok, true, r.message);
    assert.equal(r.calls, 3, "lookup, rejected draft, fixed draft");
    assert.deepEqual(r.rounds.map((x) => (x.lookup ? "lookup" : x.problems.length ? "problems" : "passed")), ["lookup", "problems", "passed"]);
    assert.match(r.rounds[1]!.problems[0]!, /outside the allowed range 0\.25 to 0\.666/, "the kernel's own words go back to Orac");
    // The second request carried the full flow entry; the third carried the kernel's problems.
    const bodies = p.seen.filter((x: { path: string }) => x.path.endsWith("/chat/completions")).map((x: { body: string }) => JSON.parse(x.body).messages);
    assert.match(bodies[1].at(-1).content, /node teq: atom .*redistribution=0\.25/);
    assert.match(bodies[2].at(-1).content, /outside the allowed range/);
    assert.equal(r.passed.length, 1);
    assert.equal(r.failed.length, 0);
    const offered = r.passed[0]!.value!;
    assert.equal(offered.target.summary(offered.value), `${CLIMATE}: teq.redistribution 0.25 → 0.5`);

    // Nothing filed, adopted or chronicled until accept.
    assert.equal(ws.requests.length, 0);
    assert.equal(await s.stateHash(), hash0);
    assert.equal(s.stack().hash, stack0);
    assert.equal(await offered.target.accept(offered.value), null);
    assert.equal(ws.requests.length, 1);
    assert.equal(ws.requests[0]!.author, "orac");
    assert.equal(ws.requests[0]!.status, "draft", "filed for the GM to adopt on the Physics screen, not adopted");
    assert.equal(s.stack().hash, stack0);
    assert.equal(st.ledger.length, 3, "every call is in the ledger");
  } finally {
    await p.close();
    await k.close();
  }
});

test("Orac: a draft still wrong after three fixes is never offered", async () => {
  const k = nodeKernel();
  const bad = fence({ note: "x", make: [{ kind: "physics", title: "Bad", doc: { flow: CLIMATE, edits: [{ node: "teq", param: "redistribution", after: 9 }] } }] });
  const p = await mockProvider({ replies: () => bad });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [pack], k.scratch);
    const ws = plainWorkspace(scene.now_yr);
    const r = await runOrac({ request: "Break it", targets: [physicsTarget(s, ws)], catalogue: flowEntries(s), ask: asker(linkedTo(p.base), "building") });
    assert.equal(r.ok, false);
    assert.equal(r.calls, 4, "the first answer and three fixes");
    assert.equal(r.passed.length, 0, "nothing reaches accept");
    assert.equal(r.failed.length, 1);
    assert.equal(r.failed[0]!.value, undefined, "a failed candidate carries nothing to accept");
    assert.match(r.message, /still has problems.*cannot be accepted/);
    assert.equal(ws.requests.length, 0);
  } finally {
    await p.close();
    await k.close();
  }
});

test("Orac: lenses and atom issues use SSE's own checks; unknown nodes, kinds and fields are refused", async () => {
  const saved: Lens[] = [];
  const fields = { planet: ["a_au", "surface_k", "mass_earth"], star: ["mass_sun"] };
  const lens = lensTarget({ taken: () => SHIPPED_LENSES.map((l) => l.id), fields: () => fields, save: (l) => saved.push(l) });
  const opened: string[] = [];
  const issue = atomIssueTarget((u) => opened.push(u));
  const goodLens = { schema: "sse.lens/0", id: "mine.survey", label: "Survey", cards: [{ subject_kind: "planet", fields: ["a_au", "surface_k"], density: "compact" }], layers: ["orbits", "labels"], tools: [], cost: "light" };
  const replies = [
    fence({ note: "n", make: [
      { kind: "lens", title: "Survey", doc: { ...goodLens, id: "basic", layers: ["orbits", "sonar"], cards: [{ subject_kind: "planet", fields: ["albedo"], density: "compact" }] } },
      { kind: "atom_issue", title: "Tidal heating", doc: { title: "Tidal heating", issue: "i", spec: "s", test: "t", references: [{ text: "Segatz 1988" }] } },
      { kind: "spaceship", doc: {} },
    ] }),
    fence({ note: "n", make: [
      { kind: "lens", title: "Survey", doc: goodLens },
      { kind: "atom_issue", title: "Tidal heating", doc: { title: "Tidal heating", issue: "Heat from tides in eccentric moons.", spec: "P = 21/2 k2/Q ...", test: "Io: about 1e14 W", references: [{ text: "Segatz et al. 1988", doi: "10.1016/0019-1035(88)90054-1" }] } },
    ] }),
  ];
  const p = await mockProvider({ replies });
  try {
    const r = await runOrac({ request: "A survey lens, and Io's tidal heat", targets: [lens, issue], catalogue: lensEntries(SHIPPED_LENSES, fields), ask: asker(linkedTo(p.base), "building") });
    const first = r.rounds[0]!.problems.join("\n");
    for (const want of [/unknown layer sonar/, /new lens id starts with "mine\."/, /already a lens basic/, /no planet has a field albedo/, /reference 1 needs a DOI/, /there is no kind "spaceship"/]) assert.match(first, want);
    assert.equal(r.ok, true, r.message);
    assert.equal(r.passed.length, 2);
    assert.equal(saved.length, 0, "nothing saved before accept");
    for (const c of r.passed) await c.value!.target.accept(c.value!.value);
    assert.equal(saved[0]!.id, "mine.survey");
    assert.match(opened[0]!, /^https:\/\/github\.com\/FrunkQ\/star-system-explorer\/issues\/new\?title=Atom%3A\+Tidal\+heating/);
    assert.match(decodeURIComponent(opened[0]!), /https:\/\/doi\.org\/10\.1016\/0019-1035\(88\)90054-1/);
  } finally {
    await p.close();
  }
});

test("Orac is for the GM only, and the loop stops at its own limit", async () => {
  assert.equal(oracMay({ authority: "gm", audience: "all" }), true);
  assert.equal(oracMay({ authority: "player", audience: "everyone" }), false);
  assert.deepEqual(checkAtomIssue({ title: "x", issue: "x", spec: "x", test: "x", references: [{ text: "r", url: "https://example.org/a" }] }), []);
  assert.ok(atomIssueUrl({ title: "t", issue: "i", spec: "s", test: "t", references: [{ text: "r", url: "https://x.org" }] }).includes("labels=atom"));
  assert.match(oracSystem([], [{ key: "flow:a", line: "l", full: "f" }]), /flow:a: l/);
  // A run limit: no next call once the spend so far plus the last call would pass it.
  let n = 0;
  const r = await draftLoop({ system: "s", request: "r", cap: 0.015, ask: async () => (n++, { ok: true, text: fence({ note: "", make: [{ kind: "k", doc: {} }] }), usd: 0.01, tokens_in: 1, tokens_out: 1 }), check: async () => ({ problems: ["no"] }) });
  assert.equal(n, 1);
  assert.match(r.message, /limit of \$0\.015/);
});
