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

// Run a body on a fresh worker and always end the worker. A failed assertion then fails the test at
// once; before, it left a live worker that kept the process (and CI's wasm job) running for hours.
async function withWorker(body: (client: KernelClient) => Promise<void>): Promise<void> {
  const { client, worker } = nodeClient();
  try {
    await body(client);
  } finally {
    await worker.terminate();
  }
}

function fixturePacks(): Record<string, unknown> {
  const packs: Record<string, unknown> = {};
  for (const f of readdirSync(new URL("tests/fixtures/packs/", root)).sort()) {
    if (f.endsWith(".json")) packs[f.slice(0, -5)] = JSON.parse(read(`tests/fixtures/packs/${f}`));
  }
  return packs;
}

// Compare got against golden and name the first line that differs, with both versions in the message.
function sameLines(got: string[], golden: string[], what: string) {
  const first = got.findIndex((line, i) => line !== golden[i]);
  if (first >= 0) {
    assert.fail(`${what}: response ${first} differs\n  worker: ${got[first]}\n  golden: ${golden[first] ?? "(none)"}`);
  }
  assert.equal(got.length, golden.length, `${what}: ${got.length} responses, the golden has ${golden.length}`);
}

test("every request through the worker matches the native golden responses", async () => {
  await withWorker(async (client) => {
    const requests = JSON.parse(read("tests/contract/requests.json"));
    const got = await runContract(client, requests, fixturePacks() as never);
    sameLines(got, read("tests/golden/contract.jsonl").trimEnd().split("\n"), "contract");
  });
});

// Each contract area runs on a fresh kernel, as natively (tests/contract/areas/, tests/golden/areas/).
test("every contract area through a fresh worker matches its native golden", async () => {
  const packs = fixturePacks();
  const areas = readdirSync(new URL("tests/contract/areas/", root)).filter((f) => f.endsWith(".json")).sort();
  assert.ok(areas.length > 0, "at least one contract area");
  for (const f of areas) {
    const name = f.slice(0, -5);
    await withWorker(async (client) => {
      const got = await runContract(client, JSON.parse(read(`tests/contract/areas/${f}`)), packs as never);
      sameLines(got, read(`tests/golden/areas/${name}.jsonl`).trimEnd().split("\n"), `area ${name}`);
    });
  }
});

test("jobs report progress in work units and can be cancelled", async () => {
  await withWorker(async (client) => {
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
  });
});

test("capability tier and visible edges", async () => {
  await withWorker(async (client) => {
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
  });
});

test("number spellings give one hash through the worker (281, 281.0, 2.81e2)", async () => {
  await withWorker(async (client) => {
    const hashes: string[] = [];
    // Raw text, because JSON.stringify would already turn 281.0 into 281.
    for (const [i, text] of ["281", "281.0", "2.81e2", "1.2e27", "1.2e+27"].entries()) {
      await client.callRaw(`{"op":"open_universe","id":"n","seed":"numbers"}`);
      const r = JSON.parse(await client.callRaw(
        `{"op":"set_authored","edit":{"address":"u:n/sys:0","field":"x","value":${text},"occurred_at_s":0}}`));
      assert.equal(r.type, "authored", `${text} (${i})`);
      hashes.push(r.state_hash);
    }
    // The native kernel's hash for 281 (crates/sse-kernel number_spellings_hash_identically).
    assert.equal(hashes[0], "c0b8445be888aeca4bbf63634feec6907958eafb1b4155030629cc71442d3dee");
    assert.equal(hashes[1], hashes[0]);
    assert.equal(hashes[2], hashes[0]);
    assert.equal(hashes[4], hashes[3]);
    assert.notEqual(hashes[3], hashes[0]);
  });
});

// Every shipped pack hashes the same as its file's own text (how the native
// kernel reads it) and as a browser sends it (JSON.parse, then
// JSON.stringify: 4.560563969097498e17 goes out as 456056396909749800).
// The native side of the same rule: crates/sse-pack/tests/web_numbers.rs.
test("every shipped pack gives one stack hash as written and as a browser sends it", async () => {
  const { client, worker } = nodeClient();
  try {
    const dirs = readdirSync(new URL("packs/", root)).sort();
    const text = (d: string) => read(`packs/${d}/pack.json`);
    const physics = text("base-physics");
    let n = 0;
    for (const d of dirs) {
      let own: string;
      try { own = text(d); } catch { continue; }
      const stack = d === "base-physics" ? [physics] : [physics, own];
      const asWritten = JSON.parse(await client.callRaw(`{"op":"load_stack","packs":[${stack.join(",")}]}`));
      const asSent = JSON.parse(await client.callRaw(JSON.stringify({ op: "load_stack", packs: stack.map((t) => JSON.parse(t)) })));
      assert.equal(asWritten.type, "stack_loaded", `${d}: ${asWritten.report ?? ""}`);
      assert.equal(asSent.stack_hash, asWritten.stack_hash, `${d}: the browser's copy hashes differently`);
      n++;
    }
    assert.ok(n > 20, "the shipped packs are found");
  } finally {
    await worker.terminate();
  }
});
