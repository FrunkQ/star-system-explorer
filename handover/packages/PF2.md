# PF1b, PF2 and PF3: players' switch, the bench, the minimum spec table

Design: `design/SSE4-performance-and-minimum-spec.md` (accepted by Alex 22 Nov 2026, all five questions yes). Built 22 Nov 2026 by the thread "Measure performance for minimum specs", after PF1 (`PF1.md`, PR 193).

## PF1b: players
`packages/shell/src/Players.svelte`: a "Performance" box beside Theme, Symbols and Screen effects, the same pref (`sse4.perf.readout`) and the same card. Checked in `tests/ui/perf.e2e.mjs` on a phone-sized page.

## PF1 follow-up: a chunk's 95th percentile is its worst op's
The median still pools every call; the 95th percentile (and so the verdict) is the slowest op's own, so one `resolved_system` at 60 ms among forty 0.2 ms edits shows as 60 ms, not 0.8 ms. Test in `perf.test.ts`.

## PF2: the bench
- `crates/sse-wasm/src/lib.rs`: the engine scenarios are one list (`engines!` macro); `engines_run` is unchanged in output, plus `engine_names()` and `engine_run(name)`. Golden hashes unchanged (the Node and browser harness still pass).
- `packages/ui-contract/src/perf-chunks.json`: each chunk has a `bench` list of its scenarios; `perf.test.ts` fails when a scenario in `lib.rs` has none, or has two.
- `tests/harness/bench.js` (plain JavaScript, shared): `runBench` (round-robin runs), `benchReport` (`sse.perf-report/1`, source `bench`, no budgets), `compareBench` (more than 5 % slower; scenarios under 2 ms in both left out).
- `tests/harness/index.html`, `harness.js`: "Run the bench" after the hash check, a table by chunk, Copy report and Save. `?bench` runs it at once; `&runs=N` sets the runs.
- `tests/harness/bench.mjs`: runners `node`, `chromium`, `chromium-cpu4`; `--out <dir>` writes a report per runner; `--against <dir>` exits 1 when anything is more than 5 % slower. This is ND3b's timing condition: run it on one machine before and after a family moves.
- `scripts/build-harness.sh` copies the chunk list to `pkg/`; `scripts/build-site.sh` leaves `bench.mjs` out of the site.

Measured on the cloud runner (4 cores, Chromium 157), one run of every scenario: about 15 s in Node and in Chromium; Chromium with the CPU slowed 4× is in the reports. A Node re-run of five scenarios came back within 5 % of the first.

Found: Chromium's CPU slowdown does not reach Web Workers, so the chart room's kernel (in a worker) is not slowed by it. The bench runs on the page and is slowed. Phone timings in the app need a real phone.

## PF3: the minimum spec table
- `scripts/min-spec.mjs` reads every report under `docs/perf-reports/` and writes `docs/minimum-spec.md` (`--check` says whether it is up to date). Per chunk: the budget, the slowest device within it so far (by the kernel's measured speed), and devices over it; then the app reports' 95th percentiles, the bench by chunk with each device's factor against the fastest, and the devices.
- First reports, `docs/perf-reports/cloud-runner/`: the bench in Node, Chromium and Chromium with CPU slowed 4×, and one app report from Chromium (the chart room opened on every main screen).
- First findings from the app report: `plan_voyage` took about 0.7 s on opening Voyages (budget 300 ms; the Voyages chunk is over), `galaxy_chart` about 0.5 s (within 1 s), `resolved_system` about 60 ms.

## Adding a device
Run the bench on `/harness/` (or open the chart room with the readout on and use it a while), Save, put the file in `docs/perf-reports/<machine>/`, run `node scripts/min-spec.mjs`, commit both.

## First real machine (22 Nov, evening)
Alex's PC, AC-ProArt (Windows, 24 cores, 32 GB, WebGPU on AMD RDNA 3, Chrome 157), ran the bench on the live `/harness/`: `docs/perf-reports/ac-proart/bench-chrome.json`. It is about twice as fast as the cloud runner in every chunk (1.9× to 2.4×) and about nine times as fast as the cloud runner with CPU slowed 4×.
- A bench report has no start-up stamps or memory by design: those come from the app readout.
- Fixed after it: the bench report named the harness version as `kernel`; it now gives the real kernel version under `kernel` and the harness version under `harness` (Alex's report was corrected by hand to match). `bench.mjs --only` now sums the chunks from the kept scenarios.

## First phone (23 Nov)
Alex's Pixel 11 Pro (Chrome 157, 7 cores reported, 8 GB, PowerVR GPU through WebGL2): `docs/perf-reports/pixel-11-pro/bench-chrome.json`. By medians it runs at about the cloud runner's speed (0.9× to 1.4× its times) and about half AC-ProArt's. Its kernel speed, 2.2 million units a second, is just over the "capable" line of 2 million, so the tier could flip between visits.
- The third run of `time` and `time-generated` took 5.7× their medians (13 s and 18 s) while every scenario after them ran at normal speed. A heat throttle would not lift that fast; it reads as the phone pausing the page (screen dimmed or another app). The bench now marks a source "unsteady" when a run is over twice its median and 50 ms or more above it; the page says so and suggests keeping the screen on; the table marks it ‡. Medians are used throughout.
- WebGL2, not WebGPU: Chrome on this phone does not offer WebGPU (the user agent's "Android 10; K" is Chrome's reduced string, not the real version). The renderer picks its WebGL2 tier here.

## Opening Voyages, measured again (23 Nov)
PR 203 (V8) made a voyage between stars fast (633 to 86 ms), but the chart room's Voyages screen opens on a voyage inside a system (the `nav` form of `plan_voyage`: Tarn to Vault, fusion torch). That one was still about 630 ms in the browser and in wasm under Node, on every call, not only the first. A profile put nearly all of it in the in-system finder's Lambert solves (`sse_nav::courses::candidates` → `kepler::lambert`): libm's `pow` about 29 % of the time, `sin` 18 %, libm's software `sqrt` 15 %.
- `sse_core::math::sqrt` now uses the hardware square root (`f64::sqrt`, one `f64.sqrt` instruction in wasm). IEEE 754 requires square root correctly rounded, so it gives the same bits as libm's; every golden is unchanged and `scripts/check.sh` passes. The module's own header already said `sqrt` is used directly; the function did not.
- After it: `plan_voyage` on opening Voyages 427 to 480 ms in the chart room (cloud runner, Chromium), about 410 ms in wasm under Node. Still over the 300 ms budget. The app report in `docs/perf-reports/cloud-runner/app-chromium.json` is the new one. The bench reports were recorded before this change, so they are slightly slow now in every chunk that takes square roots.
- Left for the voyages owner: the Lambert time-of-flight function's `pow(y/c, 1.5)` as `q * sqrt(q)` takes most of what remains, but it moves the contract golden (the answers shift in the last bits), so it is a golden-changing change for that track, not this one.
- After PR 245 (the voyages track's Lambert time of flight without `pow`, 23 Nov): `plan_voyage` on opening Voyages takes 259 ms in the chart room (cloud runner, Chromium) and 285 to 305 ms in wasm under Node. The Voyages chunk is now **within** its 300 ms budget, though near it (over 75 %). The app report has been replaced again.
