// The performance readout (PF1): the recorder, its chunks and its reports.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { PERF_CHUNKS, PerfRecorder, perf, perfReport, stats, verdict, type DeviceDescription } from "../src/perf.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const device: DeviceDescription = { user_agent: "test", screen: "1×1", pixel_ratio: 1, gpu: { backend: "none" }, reduced_motion: false };

test("stats: nearest-rank median and 95th percentile", () => {
  assert.deepEqual(stats([]), { n: 0, median: 0, p95: 0, max: 0 });
  const xs = Array.from({ length: 100 }, (_, i) => i + 1);
  assert.deepEqual(stats(xs), { n: 100, median: 50, p95: 95, max: 100 });
  assert.deepEqual(stats([3]), { n: 1, median: 3, p95: 3, max: 3 });
  assert.equal(verdict(50, 100), "within");
  assert.equal(verdict(80, 100), "near");
  assert.equal(verdict(101, 100), "over");
});

test("off records nothing; on groups by chunk, with unknown sources under Other by name", () => {
  const r = new PerfRecorder(PERF_CHUNKS, 4);
  r.record("derive", 5);
  assert.deepEqual(r.summary(), [], "off: nothing kept");
  r.setEnabled(true);
  for (const ms of [1, 2, 3, 4, 50]) r.record("derive", ms, { kernel_ms: ms / 2 });
  r.record("weather", 400);
  r.record("brand_new_op", 7);
  const s = r.summary();
  assert.deepEqual(s.map((c) => c.id), ["resolve", "climate", "other"], "the document's order, Other last");
  const resolve = s[0]!;
  assert.equal(resolve.calls, 5, "every call counted");
  assert.equal(resolve.sources[0]!.median_ms, 3, "only the last 4 samples kept: 2, 3, 4, 50");
  assert.equal(resolve.sources[0]!.kernel_median_ms, 1.5);
  assert.equal(s[1]!.verdict, "over", "400 ms is over the climate budget");
  assert.equal(s[2]!.sources[0]!.source, "brand_new_op");
  r.clear();
  assert.deepEqual(r.summary(), []);
});

test("start-up stamps are kept even while off", () => {
  const r = new PerfRecorder();
  r.stamp("startup.kernel", 812.3);
  assert.equal(r.stamps()["startup.kernel"], 812.3);
});

test("every chunk source is listed once, and every kernel op has a chunk", () => {
  const seen = new Map<string, string>();
  for (const c of PERF_CHUNKS.chunks) {
    assert.ok(c.budget_ms > 0 && c.name && c.meaning, c.id);
    for (const s of c.sources) {
      assert.ok(!seen.has(s), `${s} is in ${seen.get(s)} and ${c.id}`);
      seen.set(s, c.id);
    }
  }
  // The kernel's request enum, variant names to snake_case ops. Jobs are timed whole, as job:<kind>.
  const proto = readFileSync(new URL("crates/sse-kernel/src/protocol.rs", root), "utf8");
  const body = /pub enum Request \{([\s\S]*?)\n\}/.exec(proto)![1]!;
  const ops = [...body.matchAll(/^ {4}([A-Z][A-Za-z0-9]*)\s*(?:\{|,|\()/gm)].map((m) => m[1]!.replace(/[A-Z]/g, (c, i) => (i ? "_" : "") + c.toLowerCase()));
  assert.ok(ops.length > 60, `found ${ops.length} ops`);
  const internal = new Set(["start_job", "step_job", "cancel_job"]);
  const missing = ops.filter((o) => !internal.has(o) && !seen.has(o));
  assert.deepEqual(missing, [], "give each new kernel op a chunk in perf-chunks.json");
});

test("a report holds timings and the device, never a universe", () => {
  const r = new PerfRecorder();
  r.setEnabled(true);
  r.record("galaxy_chart", 120, { wasm_bytes: 64e6 });
  const rep = perfReport({ recorder: r, device, label: "test rig", kernel: "0.1.0", today: "2026-11-22" });
  assert.equal(rep.schema, "sse.perf-report/1");
  assert.equal(rep.made_on, "2026-11-22");
  assert.equal(rep.memory.wasm_mb, 64);
  assert.equal(rep.chunks[0]!.id, "galaxy");
  assert.deepEqual(Object.keys(rep).sort(), ["app", "chunks", "device", "label", "long_tasks", "made_on", "memory", "schema", "source", "startup_ms"]);
});

test("through a real kernel worker: ops are named, the kernel's own time and memory come back", async () => {
  const k = nodeKernel();
  perf.clear();
  perf.setEnabled(true);
  try {
    await k.client.info();
    await k.client.call({ op: "open_universe", id: "perf-test", seed: "1" } as never);
    await k.client.measure(2_000);
    const s = perf.summary();
    const startup = s.find((c) => c.id === "startup")!;
    const names = startup.sources.map((x) => x.source).sort();
    assert.deepEqual(names, ["hello", "job:benchmark", "open_universe"]);
    for (const x of startup.sources) assert.ok(x.kernel_median_ms !== undefined && x.kernel_median_ms <= x.median_ms + 0.5, `${x.source}: kernel time within the round trip`);
    assert.ok(perf.wasmBytes > 0, "the kernel's memory is reported");
    perf.setEnabled(false);
    await k.client.call({ op: "state_hash" } as never);
    assert.equal(perf.summary().find((c) => c.id === "resolve"), undefined, "off again: unwatched");
  } finally {
    perf.setEnabled(false);
    await k.close();
  }
});

test("PF2: every engine scenario has one bench chunk, and the bench's sums and comparisons hold", async () => {
  const bench = await import("../../../tests/harness/bench.js");
  const lib = readFileSync(new URL("crates/sse-wasm/src/lib.rs", root), "utf8");
  const names = [...lib.matchAll(/^\s+"([a-z-]+)" => [a-z_:]+\(\),$/gm)].map((m) => m[1]!);
  assert.ok(names.length >= 38, `found ${names.length} scenarios`);
  const listed = PERF_CHUNKS.chunks.flatMap((c) => c.bench ?? []);
  assert.deepEqual([...listed].sort(), [...new Set(listed)].sort(), "no scenario in two chunks");
  assert.deepEqual(names.filter((n) => !listed.includes(n)), [], "give each new engine scenario a chunk's bench list");
  const xs = [5, 1, 9, 3, 7, 2];
  assert.deepEqual(bench.stats(xs), stats(xs), "the bench's stats are the readout's");
  let t = 0;
  const fake = { engine_names: () => JSON.stringify(["resolve", "nav"]), engine_run: (n: string) => ((t += n === "resolve" ? 10 : 4), "{}") };
  const results = await bench.runBench(fake, PERF_CHUNKS, { runs: 3, now: () => t });
  assert.deepEqual(results.map((r: { chunk: string; runs_ms: number[] }) => [r.chunk, r.runs_ms]), [["resolve", [10, 10, 10]], ["voyages", [4, 4, 4]]]);
  const before = bench.benchReport(results, PERF_CHUNKS, { today: "2026-11-22" });
  assert.equal(before.source, "bench");
  const slow = structuredClone(before);
  slow.chunks[0].sources[0].median_ms = 10.6;
  slow.chunks[0].median_ms = 10.6;
  const cmp = bench.compareBench(before, slow);
  assert.deepEqual(cmp.slower.map((r: { what: string }) => r.what), ["bench.resolve", "chunk resolve"], "6 % slower is caught");
  slow.chunks[0].sources[0].median_ms = 10.4;
  slow.chunks[0].median_ms = 10.4;
  assert.deepEqual(bench.compareBench(before, slow).slower, [], "4 % is within");
});

test("a chunk's 95th percentile is its worst op's, so one slow kind of call is never averaged away", () => {
  const r = new PerfRecorder();
  r.setEnabled(true);
  for (let i = 0; i < 40; i++) r.record("set_authored", 0.2);
  r.record("resolved_system", 160);
  const c = r.summary()[0]!;
  assert.equal(c.median_ms, 0.2);
  assert.equal(c.p95_ms, 160);
  assert.equal(c.verdict, "over");
});
