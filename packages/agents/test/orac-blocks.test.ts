// A2 part 4 acceptance: Orac drafts boxes (J3) and game tables as class
// atlas schemes. The kernel checks each draft by trying the stack it would
// make: a box is compiled and its worked examples run; a table's classes are
// loaded into the atlas. Refusals go back in the kernel's words; nothing is
// stacked until accept.
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
import { BOXES_PACK, TABLES_PACK, blockEntries, boxTarget, tableTarget } from "../src/orac-blocks.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const atlas: PackDoc = json("packs/base-atlas/pack.json");
const scene = json("packages/shell/src/documents/kestrel.scene.json");
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const made = { kind: "", doc: null };

const heft = (g: number, wired = true) => ({
  id: "mine.heft",
  inputs: [
    { name: "mass", kind: "body.mass" },
    { name: "radius", kind: "body.radius" },
  ],
  outputs: [
    { name: "g", from: "g.g" },
    { name: "v", from: "esc.v_esc" },
  ],
  nodes: [
    { id: "g", atom: "sse.planet.surface_gravity" },
    { id: "esc", atom: "sse.planet.escape_velocity" },
  ],
  wires: [
    { from: "in.mass", to: "g.mass" },
    { from: "in.radius", to: "g.radius" },
    { from: "in.mass", to: "esc.mass" },
    ...(wired ? [{ from: "in.radius", to: "esc.radius" }] : []),
  ],
  explain: { curious: "How hard a world pulls, and how fast you must go to leave it.", science: "Newtonian gravity at the surface of a sphere.", formula: "g = GM/R^2; v = sqrt(2GM/R)" },
  tests: [{ name: "Earth (NASA fact sheet)", inputs: { mass: { value: 5.9722e24, unit: "kg" }, radius: { value: 6371, unit: "km" } }, expect: { g: { value: g, unit: "m_s2", tol: 0.03 }, v: { value: 11.186, unit: "km_s", tol: 0.02 } } }],
});

const pull = (axis = "gravity") => ({
  scheme: { id: "mine.pull", question: "How heavy does it feel to stand there?" },
  classes: [
    { id: "mine.pull/light", scheme: "mine.pull", kind: "base", region: [{ axis, core: [0, 0.5], fringe: [0, 0.5] }], threshold: 1, names: { physics: "L", plain: "light" }, source: "house rules" },
    { id: "mine.pull/standard", scheme: "mine.pull", kind: "base", region: [{ axis, core: [0.5, 1.5], fringe: [0.5, 1.5] }], threshold: 1, names: { physics: "S", plain: "standard" }, source: "house rules" },
    { id: "mine.pull/heavy", scheme: "mine.pull", kind: "base", region: [{ axis, core: [1.5, 10], fringe: [1.5, 10] }], threshold: 1, names: { physics: "H", plain: "heavy" }, source: "house rules" },
  ],
});

test("boxes: the kernel compiles the box and runs its worked examples; accept stacks the campaign's boxes", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base], k.scratch);
    const t = boxTarget(s, plainWorkspace(scene.now_yr));
    const stack0 = s.stack().hash;
    const unwired = await t.check(heft(9.82, false), made);
    assert.ok(unwired.problems.some((p) => /radius/.test(p)), unwired.problems.join("; "));
    const wrong = await t.check(heft(20), made);
    assert.ok(wrong.problems.some((p) => /Earth/.test(p) && /test/.test(p)), wrong.problems.join("; "));
    const good = await t.check(heft(9.82), made);
    assert.deepEqual(good.problems, []);
    assert.equal(s.stack().hash, stack0, "checking stacks nothing");
    assert.match(t.summary(good.value!), /^Box mine\.heft@1\.0\.0: mass, radius → g, v, from 2 blocks; 1 worked example passes/);
    assert.equal(await t.accept(good.value!), null);
    assert.ok(s.stack().added.some((p) => p.id === BOXES_PACK));
    assert.notEqual(s.stack().hash, stack0);
    assert.deepEqual((await t.check(heft(9.82), made)).problems, ["there is already a box or flow mine.heft"]);
    assert.ok(blockEntries(s).some((e) => e.key === "box:mine.heft" && /mass, radius → g, v/.test(e.line)));
  } finally {
    await k.close();
  }
});

test("tables: a game table becomes a class scheme; the atlas refuses an unknown axis; accept stacks it", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, atlas], k.scratch);
    const t = tableTarget(s, plainWorkspace(scene.now_yr));
    const unknown = await t.check(pull("heft"), made);
    assert.ok(unknown.problems.some((p) => /unknown axis heft/.test(p)), unknown.problems.join("; "));
    const misfiled = await t.check({ ...pull(), classes: [{ ...pull().classes[0]!, scheme: "physical" }] }, made);
    assert.match(misfiled.problems[0]!, /filed under scheme physical/);
    const good = await t.check(pull(), made);
    assert.deepEqual(good.problems, []);
    assert.match(t.summary(good.value!), /^How heavy does it feel to stand there\?: 3 rows \(L, S, H\) over gravity\./);
    assert.ok(!s.stack().added.some((p) => p.id === TABLES_PACK), "nothing stacked before accept");
    assert.equal(await t.accept(good.value!), null);
    assert.ok(s.stack().added.some((p) => p.id === TABLES_PACK));
    const again = await t.check(pull(), made);
    assert.ok(again.problems.some((p) => /mine\.pull.*already defined/.test(p)), again.problems.join("; "));
    const entries = blockEntries(s);
    assert.match(entries.find((e) => e.key === "axes")!.full, /^gravity: reads body\.surface_gravity, linear, in g/m);
    assert.match(entries.find((e) => e.key === "scheme:mine.pull")!.full, /mine\.pull\/heavy \(base\): gravity 1\.5 to 10 "heavy"/);
  } finally {
    await k.close();
  }
});

test("Orac drafts a game table: the unknown axis goes back, the fix is offered", async () => {
  const k = nodeKernel();
  const p = await mockProvider({ replies: (m: { content: string }[]) => fence({ note: "A gravity table.", make: [{ kind: "table", title: "Gravity bands", doc: pull(/unknown axis/.test(m.at(-1)!.content) ? "gravity" : "heft") }] }) });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, atlas], k.scratch);
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { building: { provider: "x", model: "m" } } };
    const r = await runOrac({ request: "Light, standard and heavy gravity as a table.", targets: [tableTarget(s, plainWorkspace(scene.now_yr))], catalogue: blockEntries(s), ask: asker(st, "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /unknown axis/);
    assert.equal(r.passed.length, 1);
    assert.ok(!s.stack().added.some((x) => x.id === TABLES_PACK), "nothing stacked before accept");
  } finally {
    await p.close();
    await k.close();
  }
});
