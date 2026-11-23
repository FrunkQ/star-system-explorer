// The bench (PF2, handover/design/SSE4-performance-and-minimum-spec.md): the engine scenarios timed one by one,
// grouped into the big chunks of perf-chunks.json. The same work on every device, so two machines (or one machine
// before and after a change) can be compared. Plain JavaScript, shared by the device check page (harness.js) and the
// bench script (bench.mjs). Times are read here, outside the kernel, and never enter a hash.

/** Nearest-rank median and 95th percentile, as the readout's `stats` (packages/ui-contract/src/perf.ts). */
export function stats(xs) {
  if (!xs.length) return { n: 0, median: 0, p95: 0, max: 0 };
  const s = [...xs].sort((a, b) => a - b);
  const at = (q) => s[Math.min(s.length - 1, Math.max(0, Math.ceil(q * s.length) - 1))];
  return { n: s.length, median: at(0.5), p95: at(0.95), max: s[s.length - 1] };
}

const round = (x) => (x >= 100 ? Math.round(x) : Math.round(x * 10) / 10);

/** The chunk each scenario belongs to ("other" when perf-chunks.json does not name it). */
export function chunkOfScenario(doc, name) {
  return doc.chunks.find((c) => (c.bench ?? []).includes(name)) ?? { id: "other", name: "Other", meaning: "Not yet given a chunk.", budget_ms: 0 };
}

/**
 * Time every scenario `runs` times. `wasm` has engine_names() and engine_run(name); `now` is the clock.
 * `onProgress(done, total, name)` is told before each run; `yieldTo` lets a page paint between runs.
 * Returns { name, chunk, runs_ms[] } per scenario, in the kernel's order.
 */
export async function runBench(wasm, doc, { runs = 3, now = () => performance.now(), onProgress, yieldTo, only } = {}) {
  const names = JSON.parse(wasm.engine_names()).filter((n) => !only || only.includes(n));
  const out = names.map((name) => ({ name, chunk: chunkOfScenario(doc, name).id, runs_ms: [] }));
  const total = names.length * runs;
  let done = 0;
  // Round-robin, so a machine warming up or slowing down spreads over every scenario alike.
  for (let r = 0; r < runs; r++) {
    for (const s of out) {
      onProgress?.(done, total, s.name);
      if (yieldTo) await yieldTo();
      const t0 = now();
      wasm.engine_run(s.name);
      s.runs_ms.push(now() - t0);
      done++;
    }
  }
  onProgress?.(done, total, "");
  return out;
}

/**
 * A bench report (sse.perf-report/1, source "bench"): per chunk, the sum of its scenarios' medians (one figure to
 * compare between machines), and per scenario its median, 95th percentile and slowest run. Bench chunks carry no
 * budget or verdict: budgets are for what a person waits for in the app, not for test workloads.
 */
export function benchReport(results, doc, { device, label = "", kernel, site, today, runs } = {}) {
  const chunks = [];
  for (const c of [...doc.chunks, { id: "other", name: "Other", meaning: "Not yet given a chunk." }]) {
    const mine = results.filter((s) => s.chunk === c.id);
    if (!mine.length) continue;
    const sources = mine.map((s) => {
      const t = stats(s.runs_ms);
      // Unsteady: a run over twice the median (and 50 ms or more over it, so a clock tick on a tiny scenario does not count), the mark of a pause on the device (screen dimmed, app switched),
      // not of the work. The median is still sound; the report says so, so nobody reads the slow run as the speed.
      const unsteady = t.n > 1 && t.max > 2 * t.median && t.max - t.median > 50;
      return { source: `bench.${s.name}`, calls: s.runs_ms.length, last_ms: round(s.runs_ms[s.runs_ms.length - 1]), median_ms: round(t.median), p95_ms: round(t.p95), max_ms: round(t.max), ...(unsteady ? { unsteady: true } : {}) };
    });
    const total = sources.reduce((a, s) => a + s.median_ms, 0);
    chunks.push({ id: c.id, name: c.name, meaning: c.meaning, calls: sources.reduce((a, s) => a + s.calls, 0), median_ms: round(total), p95_ms: round(sources.reduce((a, s) => a + s.p95_ms, 0)), max_ms: round(sources.reduce((a, s) => a + s.max_ms, 0)), sources });
  }
  return {
    schema: "sse.perf-report/1",
    label,
    source: "bench",
    made_on: today ?? new Date().toISOString().slice(0, 10),
    app: { ...(kernel ? { kernel } : {}), ...(site ? { site } : {}) },
    device: device ?? {},
    startup_ms: {},
    chunks,
    memory: {},
    long_tasks: 0,
    ...(runs ? { runs } : {}),
  };
}

/**
 * Before and after on one machine (ND3b's timing condition, SSE4-equation-block-design.md 8B): every scenario whose
 * median is more than `tolerance` slower in `after`, and every chunk likewise. Scenarios under `floor_ms` in both
 * are left out: at a millisecond or two the clock's own grain is bigger than 5 %.
 */
export function compareBench(before, after, { tolerance = 0.05, floor_ms = 2 } = {}) {
  const flat = (rep) => new Map(rep.chunks.flatMap((c) => c.sources.map((s) => [s.source, s.median_ms])));
  const a = flat(before);
  const b = flat(after);
  const rows = [];
  for (const [k, was] of a) {
    const now = b.get(k);
    if (now === undefined || (was < floor_ms && now < floor_ms)) continue;
    rows.push({ what: k, before_ms: was, after_ms: now, change: was > 0 ? now / was - 1 : 0 });
  }
  for (const c of before.chunks) {
    const d = after.chunks.find((x) => x.id === c.id);
    if (d && c.median_ms >= floor_ms) rows.push({ what: `chunk ${c.id}`, before_ms: c.median_ms, after_ms: d.median_ms, change: d.median_ms / c.median_ms - 1 });
  }
  return { slower: rows.filter((r) => r.change > tolerance), rows };
}
