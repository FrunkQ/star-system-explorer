// A3 acceptance (build plan: no number in output that is not in the supplied
// facts, an automated check; will not reveal unscanned data). Against the real
// kernel's scenes for the Kestrel campaign: the table ("everyone") has not
// scanned Tarn, and the GM's scene holds a secret note.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { enterFacts, type AuthoredFacts, type PackDoc, type Vocab } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { checkNarrative, factsBundle, hiddenFrom, hitsFor, leaks, numbersIn, runMinerva, strayNumbers } from "../src/minerva.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const vocab: Vocab = json("packages/shell/src/documents/vocab.plain.json");
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const narrative = (text: string) => fence({ note: "n", make: [{ kind: "narrative", title: "Tarn", doc: { text } }] });

test("numbers: digits and words, all must be in the facts", () => {
  assert.deepEqual(numbersIn("1,200 km, 4.61 Gyr, three moons and 6.5e24 kg").map((n) => n.value), [1200, 4.61, 6.5e24, 3]);
  const b = { audience: "everyone", text: "- Tarn: Moons: 3; Orbit: 1.2 AU" };
  assert.deepEqual(strayNumbers("Tarn has three moons at 1.2 AU.", b), []);
  assert.deepEqual(strayNumbers("Tarn has four moons, 2 of them icy, at 1.2 AU.", b), ["2", "four"]);
  assert.deepEqual(checkNarrative("", b), ["write the narrative as text"]);
});

test("Minerva writes only from the audience's scene; stray numbers go back; secrets never sent", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), facts.universe, [base, playerData], k.scratch);
    assert.equal(await enterFacts(s, facts), null);
    const table = await s.scene(facts.scope, { audience: "everyone" });
    const gm = await s.scene(facts.scope);
    assert.ok(table.ok && gm.ok);
    const bundle = factsBundle(table.scene, vocab, { title: "Kestrel Reach", events: [{ when: "4.61 Gyr", text: "Oceans form on Tarn" }] });
    const hidden = hiddenFrom(gm.scene, table.scene);
    assert.ok(hidden.includes("Pirate cache under the north pole."), "the GM's secret is known to the check");
    assert.ok(!bundle.text.includes("Pirate"), "and never in what Minerva is given");
    assert.match(bundle.text, /Not known to them: .*not yet scanned/, "unscanned facts are listed as unknown, not filled");

    // A first draft invents a number; a second gives away the cache; the third is clean.
    const p = await mockProvider({
      replies: [
        narrative("Tarn hangs 7 light-minutes from its star, wrapped in cloud."),
        narrative("Rumour says a pirate cache waits under the north pole of Tarn."),
        narrative(`Tarn turns beneath its clouds; oceans formed here 4.61 Gyr ago, and little more is known. What lies below is not yet scanned.`),
      ],
    });
    try {
      const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { describing: { provider: "x", model: "m" } } };
      const r = await runMinerva({ bundle, hidden, notes: { seed: "the approach to Tarn", style: "ship's log" }, ask: asker(st, "describing") });
      assert.equal(r.ok, true, r.message);
      assert.match(r.rounds[0]!.problems[0]!, /not in the facts you were given: 7/);
      assert.match(r.rounds[1]!.problems[0]!, /something they have not been shown/);
      assert.equal(r.passed.length, 1);
      assert.deepEqual(r.passed[0]!.value!.label, "narrative");
      assert.equal(r.passed[0]!.value!.audience, "everyone");
      // SSE never sent the secret, in any call: not in the instructions, the facts or the feedback.
      // (The scripted model "guessed" it in its second answer, which comes back to it as its own turn.)
      const bodies = p.seen.filter((x: { path: string }) => x.path.endsWith("/chat/completions")).map((x: { body: string }) => JSON.parse(x.body).messages as { role: string; content: string }[]);
      assert.equal(bodies.length, 3);
      for (const msgs of bodies) for (const m of msgs) if (m.role !== "assistant") assert.ok(!/pirate|north pole/i.test(m.content), `SSE sent a secret in a ${m.role} turn`);
      // The guard catches a leak in other words too.
      assert.deepEqual(leaks("They say the CACHE is under the pole, north side.", ["Pirate cache under the north pole."], bundle), ["Pirate cache under the north pole."]);
    } finally {
      await p.close();
    }
  } finally {
    await k.close();
  }
});

test("a draft that keeps inventing numbers is never shown", async () => {
  const p = await mockProvider({ replies: () => narrative("It has 99 moons.") });
  try {
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { describing: { provider: "x", model: "m" } } };
    const r = await runMinerva({ bundle: { audience: "everyone", text: "- Tarn: nothing known." }, ask: asker(st, "describing") });
    assert.equal(r.ok, false);
    assert.equal(r.passed.length, 0);
    assert.equal(r.failed[0]!.value, undefined);
  } finally {
    await p.close();
  }
});

test("sifter hits come from what the audience is shown, never the GM's", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), facts.universe, [base, playerData], k.scratch);
    assert.equal(await enterFacts(s, facts), null);
    const starters = json("packages/views/story-finder/src/starters.json").sifters;
    const table = await s.scene(facts.scope, { audience: "everyone" });
    assert.ok(table.ok);
    const theirs = await s.sift({ scope: facts.scope, sifters: starters, audience: "everyone" });
    const gms = await s.sift({ scope: facts.scope, sifters: starters });
    assert.ok(theirs.ok && gms.ok);
    assert.ok(gms.result.hits.some((h) => h.sifter === "story.secret"), "the GM's sift finds the secret");
    // The GM's hits are refused for an audience (as an older kernel that ignored the audience would send).
    assert.deepEqual(hitsFor(gms.result, "everyone", table.scene), []);
    const hits = hitsFor(theirs.result, "everyone", table.scene);
    assert.deepEqual(hits.map((h) => h.title), ["A rumour to chase, at " + hits[0]!.title.split(", at ")[1]]);
    assert.match(hits[0]!.evidence[0]!, /third moon sings/);
    const bundle = factsBundle(table.scene, vocab, { title: "Kestrel Reach", hits });
    assert.match(bundle.text, /Worth a story:\n- A rumour to chase/);
    assert.ok(!/Pirate|ruins|garden/i.test(bundle.text), "nothing the table has not been shown");
  } finally {
    await k.close();
  }
});

test("Describe hands her one subject, what it orbits, what orbits it and the star (V3 gap 23)", async () => {
  const { aboutOne } = await import("../src/minerva.ts");
  const s = (address: string) => ({ address, view: "survey", facts: [], tags: [] });
  const scene = { audience: "everyone", scope: "u:k/sys:1/body:0", subjects: ["u:k/sys:1/body:0", "u:k/sys:1/body:3", "u:k/sys:1/body:3/body:0", "u:k/sys:1/body:4", "u:k/sys:1/body:4/body:0"].map(s) };
  assert.deepEqual(aboutOne(scene, "u:k/sys:1/body:3")!.subjects.map((x) => x.address), ["u:k/sys:1/body:0", "u:k/sys:1/body:3", "u:k/sys:1/body:3/body:0"]);
  assert.deepEqual(aboutOne(scene, "u:k/sys:1/body:4/body:0")!.subjects.map((x) => x.address), ["u:k/sys:1/body:0", "u:k/sys:1/body:4", "u:k/sys:1/body:4/body:0"]);
  assert.equal(aboutOne(scene, "u:k/sys:1/body:9"), null, "a subject they do not know is not described");
});

test("a Guide's note (AT11): one line, at most 200 characters, numbers only from the audience's facts; a long one goes back", async () => {
  const long = "Tarn is wet. ".repeat(20).trim();
  const p = await mockProvider({ replies: (m: { content: string }[]) => narrative(/this one is \d+/.test(m.at(-1)!.content) ? "Tarn: bring a towel, and then a second towel." : long) });
  try {
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { describing: { provider: "x", model: "m" } } };
    const r = await runMinerva({ bundle: { audience: "everyone", text: "- Tarn: a world; most of it is sea." }, notes: { length: "note", style: "The Guide" }, ask: asker(st, "describing") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /a Guide's note is at most 200 characters on one line; this one is \d+/);
    assert.equal(r.passed[0]!.value!.text, "Tarn: bring a towel, and then a second towel.");
    assert.match(JSON.parse(p.seen[0].body).messages[1].content, /one or two sentences for the margin of the Guide, at most 200 characters on one line: opinion and wit, never a new fact/);
  } finally {
    await p.close();
  }
});
