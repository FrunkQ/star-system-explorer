// A1 acceptance in Node: works with two providers (a keyed, priced one like
// OpenRouter, and a keyless local one like LM Studio); a key goes only to its
// own provider; every run is estimated first and refused before sending when
// it would not fit or would pass the day's cap; with nothing linked, every
// job says so and nothing is sent. Keys cannot reach a peer: no peer-to-peer,
// share, players' or session module imports the agents.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { chat, emptyAgents, estimate, linked, listModels, readAgents, type AgentsState } from "../src/agents.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

test("works with two providers; each key goes only to its own provider", async () => {
  const remote = await mockProvider({ key: "sk-test-remote", priced: true, context: 8192 });
  const local = await mockProvider();
  try {
    const s: AgentsState = {
      ...emptyAgents(),
      providers: [
        { id: "openrouter", kind: "openrouter", label: "Remote", base: remote.base, key: "sk-test-remote" },
        { id: "lmstudio", kind: "lmstudio", label: "Local", base: local.base },
      ],
    };
    const a = await listModels(s.providers[0]!);
    const b = await listModels(s.providers[1]!);
    assert.ok(a.ok && b.ok);
    assert.deepEqual(a.models[0], { id: "mock/small", context: 8192, price: { prompt: 0.000001, completion: 0.000002 } });
    s.links.support = { provider: "openrouter", model: "mock/small", context: 8192, price: a.models[0]!.price };
    s.links.building = { provider: "lmstudio", model: "mock/large" };
    const r1 = await chat(s, "support", [{ role: "user", content: "What is Tarn?" }]);
    const r2 = await chat(s, "building", [{ role: "user", content: "Draft a flow" }]);
    assert.ok(r1.ok && r2.ok);
    assert.equal(r1.text, "Mock answer to: What is Tarn?");
    assert.equal(r1.row.usd, 120 * 0.000001 + 12 * 0.000002);
    assert.equal(r2.row.usd, undefined, "no price given, none invented");
    assert.ok(remote.seen.every((x: { authorization: string }) => x.authorization === "Bearer sk-test-remote"));
    assert.ok(local.seen.every((x: { authorization: string | null }) => x.authorization === null), "the local provider never sees the remote key");
    assert.ok(!JSON.stringify(local.seen).includes("sk-test-remote"));
    // A wrong key is refused by the provider and said plainly.
    const bad = await listModels({ ...s.providers[0]!, key: "wrong" });
    assert.deepEqual(bad, { ok: false, message: "The provider refused the key." });
  } finally {
    await remote.close();
    await local.close();
  }
});

test("estimate first; refuse before sending when too long or over the cap", async () => {
  const p = await mockProvider({ priced: true, context: 300 });
  try {
    const s: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { support: { provider: "x", model: "mock/small", context: 300, price: { prompt: 0.01, completion: 0.01 } } }, cap_usd: 1 };
    const long = [{ role: "user" as const, content: "x".repeat(2000) }];
    const e = estimate(s, "support", long, 100);
    assert.equal(e.fits, false);
    const r = await chat(s, "support", long, { max_out: 100 });
    assert.ok(!r.ok && /Nothing was sent/.test(r.message));
    const short = [{ role: "user" as const, content: "hello" }];
    const r2 = await chat(s, "support", short, { max_out: 150 });
    assert.ok(!r2.ok && /cap/.test(r2.message), "over the day's cap");
    assert.equal(p.seen.length, 0, "nothing reached the provider");
  } finally {
    await p.close();
  }
});

test("with nothing linked, every job says so and nothing is sent", async () => {
  const s = emptyAgents();
  for (const j of ["support", "building", "describing", "deciding"] as const) {
    assert.equal(linked(s, j), false);
    const r = await chat(s, j, [{ role: "user", content: "hi" }], { fetcher: () => assert.fail("must not fetch") });
    assert.ok(!r.ok && /works without one/.test(r.message));
  }
  assert.deepEqual(readAgents("garbage"), emptyAgents());
});

test("no peer-to-peer, share, players' or session code imports the agents", () => {
  const root = new URL("../../../", import.meta.url);
  const files = [
    // Every file under ui-contract/src, its folders too (the Atlas has ui-contract/src/atlas/).
    ...readdirSync(new URL("packages/ui-contract/src/", root), { recursive: true, withFileTypes: true })
      .filter((d) => d.isFile())
      .map((d) => `${d.parentPath.slice(d.parentPath.indexOf("packages/ui-contract/src"))}/${d.name}`),
    "packages/shell/src/share.svelte.ts",
    "packages/shell/src/Players.svelte",
    "packages/shell/src/players.ts",
    "worker/index.js",
  ];
  for (const f of files) assert.ok(!/agents\//.test(readFileSync(new URL(f, root), "utf8")), `${f} must not import the agents`);
});
