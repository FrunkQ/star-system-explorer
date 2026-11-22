# PF1: the performance readout

Design: `design/SSE4-performance-and-minimum-spec.md` (track PF). Built 22 Nov 2026 by the thread "Measure performance for minimum specs", from Alex's ask for performance measurement that can be switched on in settings and starts a minimum spec for each big chunk.

## What it does
More › "Performance readout" (off by default, remembered per device as `sse4.perf.readout`) opens a card at the foot of the chart room, a sheet along the bottom on a phone. Each big chunk shows its median and 95th percentile with a bar against its budget and a verdict (within, near, over); a row opens to its ops with calls, median, 95th percentile and the kernel's own time. Below: start-up stamps, kernel and page memory, long tasks. "Copy report" and "Save" make an `sse.perf-report/1`; nothing is sent anywhere. × switches it off. It folds to one line (the frame time).

## Files
- `packages/kernel-client/src/messages.ts`, `worker.ts`: a response carries `ms` (the kernel's own time, a job's steps summed) and `mem` (WebAssembly memory bytes). Read in the worker, never in the kernel.
- `packages/kernel-client/src/client.ts`: `setCallObserver` and `CallTiming`; `callRaw` and `runJob` report to it only while set.
- `packages/ui-contract/src/perf.ts`: `PerfRecorder` (ring of 240 samples per source, nearest-rank stats, chunk grouping), the page's `perf`, `watchPage` (frame interval, event timing, long tasks), `describeDevice`, `perfReport`. Exported from the index.
- `packages/ui-contract/src/perf-chunks.json`: the chunks, their sources and budgets (data, `sse.perf-chunks/1`).
- `packages/ui-contract/src/host-browser.ts`: start-up stamps (kernel ready, universe open, device measured).
- `packages/renderer/src/runtime.ts`: one line, the globe's frame work as `render.globe` per tier.
- `packages/shell/src/PerfReadout.svelte`: the card. `App.svelte`: the More entry, the pref, two stamps (packs fetched, first chart).

## Checks
- `packages/ui-contract/test/perf.test.ts` (in `npm run ui:test`): stats; off records nothing; chunk grouping and Other; stamps while off; every kernel op in `protocol.rs` has a chunk (fails when a new op is added without one); report shape; through a real kernel worker, ops named, kernel time within the round trip, memory reported, unwatched again when off.
- `tests/ui/perf.e2e.mjs` (in `npm run ui:e2e`): off by default; on from More; chunks fill as screens open; a row opens to ops; the copied report parses, has the device and tier, and no universe; remembered across reload with start-up timed; a sheet with no sideways scroll on a phone; Night chart; × off and remembered.
- Golden hashes unchanged (no kernel change).

## For other threads
- A new kernel op needs a line in `perf-chunks.json` or `perf.test.ts` fails, naming it.
- Anything else worth timing on the page: `perf.record("<source>", ms, { detail })` from `ui-contract`; it costs nothing while the readout is off. Give the source a chunk in the JSON.

## Next
PF1b players' page switch (question 1), PF2 the bench on `/harness/`, PF3 the minimum spec table.
