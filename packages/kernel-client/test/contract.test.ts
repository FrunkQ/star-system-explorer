// Contract test through a real worker thread: every kernel call must return
// exactly the bytes the native kernel returned (tests/golden/contract.jsonl).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { Worker } from "node:worker_threads";
import { KernelClient, runContract, type Port } from "../src/index.ts";

const root = new URL("../../../", import.meta.url);
const read = (p: string) => readFileSync(new URL(p, root), "utf8");

export function nodeClient(): { client: KernelClient; worker: Worker } {
  const worker = new Worker(new URL("../src/worker.ts", import.meta.url));
  const port: Port = { postMessage: (m) => worker.postMessage(m), onMessage: (cb) => worker.on("message", cb) };
  const jsUrl = new URL("packages/kernel-client/wasm/sse_wasm.js", root).href;
  const wasmBytes = readFileSync(new URL("packages/kernel-client/wasm/sse_wasm_bg.wasm", root));
  return { client: new KernelClient(port, { jsUrl, wasmBytes }), worker };
}

test("every request through the worker matches the native golden responses", async () => {
  const { client, worker } = nodeClient();
  const requests = JSON.parse(read("tests/contract/requests.json"));
  const packs: Record<string, unknown> = {};
  for (const f of readdirSync(new URL("tests/fixtures/packs/", root)).sort()) {
    if (f.endsWith(".json")) packs[f.slice(0, -5)] = JSON.parse(read(`tests/fixtures/packs/${f}`));
  }
  const got = await runContract(client, requests, packs as never);
  const golden = read("tests/golden/contract.jsonl").trimEnd().split("\n");
  assert.equal(got.length, golden.length);
  got.forEach((line, i) => assert.equal(line, golden[i], `response ${i} differs`));
  await worker.terminate();
});

test("jobs report progress in work units and can be cancelled", async () => {
  const { client, worker } = nodeClient();
  const seen: number[] = [];
  const done = await client.runJob({ kind: "benchmark", units: 50_000 }, { budget: 10_000, onProgress: (d) => seen.push(d) });
  assert.equal(done.type, "job_done");
  assert.deepEqual(seen, [10_000, 20_000, 30_000, 40_000]);

  const ctl = new AbortController();
  let steps = 0;
  const r = await client.runJob({ kind: "benchmark", units: 10_000_000 }, {
    budget: 1_000,
    signal: ctl.signal,
    onProgress: () => { if (++steps === 3) ctl.abort(); },
  });
  assert.equal(r.type, "job_cancelled");
  assert.ok(steps >= 3 && steps < 10_000, `cancelled after ${steps} steps`);
  await worker.terminate();
});

test("capability tier and visible edges", async () => {
  const { client, worker } = nodeClient();
  const cap = await client.measure(20_000);
  assert.ok(cap.unitsPerSecond > 0);
  assert.ok(["basic", "standard", "capable"].includes(cap.tier));
  assert.deepEqual(client.can("trivial"), { ok: true });
  // Pretend to be a weak device: heavy work must be refused with a reason.
  client.capability = { tier: "basic", unitsPerSecond: 50_000, runsUpTo: "light" };
  const edge = client.can("heavy");
  assert.equal(edge.ok, false);
  assert.match(edge.reason!, /Beyond this device's range: heavy work needs the capable tier .* GM's machine/);
  const info = await client.info();
  assert.ok(info.ops.every((o) => ["trivial", "light", "medium", "heavy"].includes(o.cost)));
  await worker.terminate();
});
