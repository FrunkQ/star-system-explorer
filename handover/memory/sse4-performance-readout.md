---
name: sse4-performance-readout
description: SSE4 track PF (performance readout, bench, minimum spec): state, mechanism and open questions as of 22 Nov 2026
metadata:
  type: project
  modified: 2026-11-22T16:11:22.398Z
---
- Alex asked 22 Nov for switchable performance measurement giving a "minimum spec" per big chunk. Thread "Measure performance for minimum specs".
- PF1 built (PR 193): More › Performance readout, off by default (pref sse4.perf.readout). One recorder `perf` in packages/ui-contract/src/perf.ts; chunks and budgets as data in perf-chunks.json (a test fails when a kernel op has no chunk); kernel client `setCallObserver`; worker returns kernel ms and wasm memory; report schema sse.perf-report/1, copied or saved only, never sent.
- PF1b (players' switch), PF2 (bench on /harness/, tests/harness/bench.mjs --against = ND3b 5% check) and PF3 (scripts/min-spec.mjs → docs/minimum-spec.md from docs/perf-reports/) merged 22 Nov (PR 199). AC-ProArt bench report added 22 Nov (PR 201; about 2x the cloud runner); Pixel 11 Pro bench added 23 Nov (PR 215; about cloud-runner speed, WebGL2 not WebGPU, borderline capable tier). Bench marks "unsteady" runs (a pause on the device). Next useful data: an app readout report from the phone. Chromium CPU throttling does not slow Web Workers, so app readouts under it are no phone stand-in. Opening Voyages is the in-system plan_voyage (PR 203 only fixed between-star): ~630 ms, now ~450 ms after sse_core::math::sqrt became f64::sqrt (same bits, 23 Nov, PR 215); then 259 ms (within 300 ms) after PR 245 replaced pow in kepler::lambert (PR 251 updates the table).
- Alex agreed to all five questions 22 Nov 16:17 UTC: players' page gets the switch (PF1b); the switch moves to the chart room's one settings place when it exists; budgets stand; no CI timing gate (before/after script on one machine); he will run the bench on AC-ProArt and his phone once PF2 lands, so tell him when it is ready. Design: handover/design/SSE4-performance-and-minimum-spec.md.
- tests/ui/perf.e2e.mjs is in npm ui:e2e but not in CI (workflow edits need Alex's yes).
- From 23 Nov (PR 216): status rows go in handover/status/performance.md, never in START-HERE.md (now a one-page index).
