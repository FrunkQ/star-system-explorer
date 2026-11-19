// A1 part 2: the pieces reconciled with Ovid Next's source. Addresses mended
// as Ovid mends them; a negative listed price is "varies", never a price; a
// local model plans with what it is loaded with (4096 when the server does not
// say), never its listed maximum; keys are kept apart from the settings and
// taken out of anything sent; failed calls are kept in the ledger.
import { test } from "node:test";
import assert from "node:assert/strict";
import { chat, emptyAgents, estimate, keyEntry, loadAgents, memoryStore, newProvider, readLoaded, saveAgents, STORE_KEY, type AgentsState } from "../src/agents.ts";
import { clean, fromLmStudioV1, fromOllamaPs, listedPrice, normalBase, readJson, readLookup, UNSAID } from "../src/lifted.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

test("addresses are mended as Ovid mends them", () => {
  assert.equal(normalBase("openrouter.ai/api/v1/"), "https://openrouter.ai/api/v1");
  assert.equal(normalBase("localhost:1234"), "http://localhost:1234/v1");
  assert.equal(normalBase("http:/127.0.0.1:1234/v1/chat/completions"), "http://127.0.0.1:1234/v1");
  assert.equal(normalBase("192.168.1.20:11434/api", true), "http://192.168.1.20:11434/v1");
  assert.equal(normalBase("https://api.example.com\\v1\\models"), "https://api.example.com/v1");
  assert.equal(normalBase("not an address at all"), "not an address at all");
  const s = emptyAgents();
  const p = newProvider(s, "lmstudio", "", "localhost:1234", "");
  assert.ok(!("problem" in p) && p.base === "http://localhost:1234/v1" && p.label === "LM Studio on this computer");
  assert.ok("problem" in newProvider(s, "openrouter", "", "https://openrouter.ai/api/v1", ""), "OpenRouter needs a key");
});

test("a negative listed price is 'varies', never a price", () => {
  assert.deepEqual(listedPrice({ prompt: -1, completion: -1 }), { varies: true });
  assert.deepEqual(listedPrice({ prompt: 1e-6, completion: 2e-6 }), { price: { prompt: 1e-6, completion: 2e-6 } });
  assert.deepEqual(listedPrice(undefined), {});
});

test("a local model plans with what it is loaded with, else 4096", async () => {
  const lm = { models: [{ key: "qwen/qwen3-8b", max_context_length: 131072, loaded_instances: [{ id: "qwen3-8b", config: { context_length: 8192 } }] }] };
  assert.deepEqual(fromLmStudioV1(lm, "qwen3-8b"), { tokens: 8192, most: 131072 });
  assert.equal(fromLmStudioV1({ models: [] }, "x"), null);
  assert.equal(fromOllamaPs({ models: [{ name: "llama3:8b", context_length: 2048 }] }, "llama3:8b"), 2048);
  const p = { id: "lmstudio", kind: "lmstudio" as const, label: "LM", base: "http://127.0.0.1:1234/v1" };
  let asked = "";
  const fetcher = (async (u: string) => ((asked = u), new Response(JSON.stringify(lm)))) as unknown as typeof fetch;
  assert.equal(await readLoaded(p, "qwen3-8b", fetcher), 8192);
  assert.equal(asked, "http://127.0.0.1:1234/api/v1/models");
  assert.equal(await readLoaded({ ...p, kind: "openrouter" }, "x", fetcher), null, "hosted links use their listed context");

  // Linked with a listed context of 131072 but nothing read: plans for 4096, so a long prompt is refused unsent.
  const s: AgentsState = { ...emptyAgents(), providers: [p], links: { building: { provider: "lmstudio", model: "qwen3-8b", context: 131072 } } };
  const long = [{ role: "user" as const, content: "x".repeat(20000) }];
  assert.equal(estimate(s, "building", long, 600).context, UNSAID);
  const r = await chat(s, "building", long, { fetcher: () => assert.fail("must not send") });
  assert.ok(!r.ok && /is loaded with 4096\. Nothing was sent\. Load the model with a longer context/.test(r.message), r.ok ? "" : r.message);
  s.links.building!.loaded = 32768;
  assert.equal(estimate(s, "building", long, 600).fits, true);
});

test("keys live apart from the settings, and never travel inside a prompt", async () => {
  const KEY = "sk-or-v1-abcdefghijklmnop";
  const store = memoryStore();
  // Part 1 kept the key inside the settings; reading moves it to its own entry.
  store.setItem(STORE_KEY, JSON.stringify({ ...emptyAgents(), providers: [{ id: "openrouter", kind: "openrouter", label: "OR", base: "https://openrouter.ai/api/v1", key: KEY }] }));
  const s = loadAgents(store);
  assert.equal(s.providers[0]!.key, KEY, "the key is there for this page");
  assert.ok(!store.getItem(STORE_KEY)!.includes(KEY), "the settings no longer hold it");
  assert.equal(store.getItem(keyEntry("openrouter")), KEY);
  assert.equal(loadAgents(store).providers[0]!.key, KEY, "and it reads back");
  saveAgents(store, { ...s, providers: [] }, ["openrouter"]);
  assert.equal(store.getItem(keyEntry("openrouter")), null, "a removed provider's key goes with it");

  // A key pasted into a question is taken out before sending, by value and by shape.
  assert.equal(clean(`my key is ${KEY} and Bearer abc.def`, []), "my key is [a key] and [a key]");
  assert.equal(clean("hash 3f9a0c4be1d2a7f68c9b0e1d2c3b4a5f69788796a5b4c3d2e1f0a9b8c7d6e5f4 stays"), "hash 3f9a0c4be1d2a7f68c9b0e1d2c3b4a5f69788796a5b4c3d2e1f0a9b8c7d6e5f4 stays", "state hashes are not keys");
  const remote = await mockProvider({ key: "custom-key-123456" });
  try {
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: remote.base, key: "custom-key-123456" }], links: { support: { provider: "x", model: "mock/small" } } };
    const r = await chat(st, "support", [{ role: "user", content: "Is custom-key-123456 my key?" }]);
    assert.ok(r.ok);
    const body = remote.seen.find((x: { path: string }) => x.path.endsWith("/chat/completions")).body as string;
    assert.ok(!body.includes("custom-key-123456"), "the body never carries the key");
    assert.match(body, /Is \[a key\] my key\?/);
  } finally {
    await remote.close();
  }
});

test("a failed call is kept in the ledger, with no cost invented", async () => {
  const s: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: "http://127.0.0.1:9/v1" }], links: { support: { provider: "x", model: "m" } } };
  const r = await chat(s, "support", [{ role: "user", content: "hi" }], { fetcher: (async () => new Response("{}", { status: 500 })) as unknown as typeof fetch });
  assert.ok(!r.ok && r.row && r.row.ok === false && r.row.usd === undefined && r.message === "The provider answered 500.");
});

test("replies: JSON in a fence or bare; lookups apart from answers", () => {
  assert.deepEqual(readJson('Sure.\n```json\n{"a": 1}\n```\nDone'), { a: 1 });
  assert.deepEqual(readJson('noise {"a": [1, 2]} noise'), { a: [1, 2] });
  assert.equal(readJson("no json here"), null);
  assert.deepEqual(readLookup('{"need": ["atom:sse.climate.teq"]}'), { need: ["atom:sse.climate.teq"], find: null });
  assert.equal(readLookup('{"make": [], "need": ["x"]}'), null, "an answer is not a lookup");
});
